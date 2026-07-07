//! Merge per-track databases (CADD/SpliceAI/dbSNP) into one unified RocksDB.
//!
//! Each input track was built with compact contig-ID keys against its own
//! contig dictionary. The merge re-keys everything onto a single canonical
//! dictionary (derived from the reference `.fai`) and consolidates all tracks
//! for a coordinate into one [`IntegratedVariantRecord`], so the unified
//! database contains each coordinate exactly once (no key duplication).

use clap::Parser;
use prost::Message;
use rocksdb::{Direction, IteratorMode};

use crate::common;
use crate::common::contig::ContigDict;
use crate::common::keys::Var;
use crate::pbs::seqvars::base::{CaddRecord, DbsnpRecord, IntegratedVariantRecord, SpliceAiRecord};

/// Command line arguments for `seqvars unified`.
#[derive(Parser, Debug, Clone)]
#[command(about = "Merge track databases into a unified RocksDB", long_about = None)]
pub struct Args {
    /// Path to a CADD track RocksDB to include.
    #[arg(long)]
    pub cadd: Option<String>,
    /// Path to a SpliceAI track RocksDB to include.
    #[arg(long)]
    pub spliceai: Option<String>,
    /// Path to a dbSNP track RocksDB to include.
    #[arg(long)]
    pub dbsnp: Option<String>,
    /// Path to the reference FASTA index (`.fai`) defining the canonical contig dictionary.
    #[arg(long)]
    pub path_reference_fai: String,
    /// Assembly / genome-release label to record in the meta CF.
    #[arg(long)]
    pub assembly: String,
    /// Path to the output (unified) RocksDB directory.
    #[arg(long)]
    pub path_out_rocksdb: String,
    /// Name of the unified data column family.
    #[arg(long, default_value = "unified")]
    pub cf_name: String,
}

/// The kind of source track.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrackKind {
    Cadd,
    Spliceai,
    Dbsnp,
}

impl TrackKind {
    fn cf(self) -> &'static str {
        match self {
            TrackKind::Cadd => "cadd",
            TrackKind::Spliceai => "spliceai",
            TrackKind::Dbsnp => "dbsnp",
        }
    }
}

/// An opened source track database plus its contig dictionary.
struct Source {
    kind: TrackKind,
    db: rocksdb::DB,
    dict: ContigDict,
}

/// A coordinate on the current contig, used to order the k-way merge.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Coord {
    pos: i32,
    reference: String,
    alternative: String,
}

fn open_source(kind: TrackKind, path: &str) -> Result<Source, anyhow::Error> {
    let db = rocksdb::DB::open_cf_for_read_only(
        &rocksdb::Options::default(),
        common::readlink_f(path)?,
        ["meta", kind.cf()],
        false,
    )?;
    let dict = crate::seqvars::read_contig_dict(&db)?;
    Ok(Source { kind, db, dict })
}

/// Main entry point for `seqvars unified`.
pub fn run(_common: &common::cli::Args, args: &Args) -> Result<(), anyhow::Error> {
    tracing::info!("Merging track databases into {}", &args.path_out_rocksdb);
    let canonical = ContigDict::from_fai(&args.path_reference_fai)?;

    let mut sources = Vec::new();
    for (kind, path) in [
        (TrackKind::Cadd, &args.cadd),
        (TrackKind::Spliceai, &args.spliceai),
        (TrackKind::Dbsnp, &args.dbsnp),
    ] {
        if let Some(path) = path {
            tracing::info!("  loading {} track from {}", kind.cf(), path);
            sources.push(open_source(kind, path)?);
        }
    }
    if sources.is_empty() {
        anyhow::bail!("no input tracks given (provide at least one of --cadd/--spliceai/--dbsnp)");
    }

    let (out_db, cf_names) =
        crate::seqvars::open_track_db_for_write(&args.path_out_rocksdb, &args.cf_name, None)?;
    crate::seqvars::write_track_meta(&out_db, "unified", "1.0", &args.assembly, &canonical)?;
    let cf_out = out_db
        .cf_handle(&args.cf_name)
        .ok_or_else(|| anyhow::anyhow!("output column family {} missing", &args.cf_name))?;

    let mut count: u64 = 0;
    // Process contig by contig, in canonical (output) ID order.
    for out_id in 0..canonical.len() as u32 {
        let contig_name = canonical
            .name_of(out_id)
            .expect("canonical id in range")
            .to_string();

        // One forward prefix iterator per source that has this contig.
        struct Front<'a> {
            kind: TrackKind,
            var: Var,
            value: Vec<u8>,
            iter: rocksdb::DBIteratorWithThreadMode<'a, rocksdb::DB>,
            prefix: Vec<u8>,
            dict: &'a ContigDict,
        }

        let mut fronts: Vec<Front> = Vec::new();
        for src in &sources {
            let Some(local_id) = src.dict.id_of(&contig_name) else {
                continue;
            };
            let id_bytes = local_id.to_be_bytes();
            let prefix = id_bytes[id_bytes.len() - crate::common::keys::CONTIG_ID_LEN..].to_vec();
            let cf = src
                .db
                .cf_handle(src.kind.cf())
                .expect("source data cf exists");
            let mut iter = src.db.iterator_cf_opt(
                &cf,
                rocksdb::ReadOptions::default(),
                IteratorMode::From(&prefix, Direction::Forward),
            );
            if let Some(item) = iter.next() {
                let (k, v) = item?;
                if k.starts_with(&prefix) {
                    let var = Var::decode_with_ctx(&k, src.dict.id_to_name());
                    fronts.push(Front {
                        kind: src.kind,
                        var,
                        value: v.to_vec(),
                        iter,
                        prefix,
                        dict: &src.dict,
                    });
                }
            }
        }

        while !fronts.is_empty() {
            // Lowest coordinate across all fronts.
            let min = fronts
                .iter()
                .map(|f| Coord {
                    pos: f.var.pos,
                    reference: f.var.reference.clone(),
                    alternative: f.var.alternative.clone(),
                })
                .min()
                .expect("fronts non-empty");

            let mut record = IntegratedVariantRecord::default();
            let mut i = 0;
            while i < fronts.len() {
                let is_match = fronts[i].var.pos == min.pos
                    && fronts[i].var.reference == min.reference
                    && fronts[i].var.alternative == min.alternative;
                if !is_match {
                    i += 1;
                    continue;
                }
                match fronts[i].kind {
                    TrackKind::Cadd => {
                        record.cadd = Some(CaddRecord::decode(&fronts[i].value[..])?)
                    }
                    TrackKind::Spliceai => {
                        record.splice_ai = Some(SpliceAiRecord::decode(&fronts[i].value[..])?)
                    }
                    TrackKind::Dbsnp => {
                        record.dbsnp = Some(DbsnpRecord::decode(&fronts[i].value[..])?)
                    }
                }
                // Advance this front; drop it if exhausted / past the prefix.
                match fronts[i].iter.next() {
                    Some(item) => {
                        let (k, v) = item?;
                        if k.starts_with(&fronts[i].prefix) {
                            fronts[i].var = Var::decode_with_ctx(&k, fronts[i].dict.id_to_name());
                            fronts[i].value = v.to_vec();
                            i += 1;
                        } else {
                            fronts.remove(i);
                        }
                    }
                    None => {
                        fronts.remove(i);
                    }
                }
            }

            let out_var = Var::new(contig_name.clone(), min.pos, min.reference, min.alternative);
            let out_key = out_var.encode_with_id(out_id);
            out_db.put_cf(&cf_out, out_key, record.encode_to_vec())?;
            count += 1;
        }
    }
    tracing::info!("  wrote {} unified records", count);

    let cf_refs = cf_names.iter().map(String::as_str).collect::<Vec<_>>();
    rocksdb_utils_lookup::force_compaction_cf(&out_db, &cf_refs, Some("  "), true)?;

    tracing::info!("All done. Have a nice day!");
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use clap_verbosity_flag::Verbosity;
    use temp_testdir::TempDir;

    fn common() -> common::cli::Args {
        common::cli::Args {
            verbose: Verbosity::new(0, 0),
        }
    }

    #[test]
    fn merge_cadd_and_dbsnp() {
        let tmp = TempDir::default();
        let fai = tmp.join("ref.fa.fai");
        std::fs::write(&fai, "1\t249250621\n").unwrap();

        // CADD: shared 1:100 A>T and cadd-only 1:200 C>G.
        let cadd_tsv = tmp.join("cadd.tsv");
        std::fs::write(
            &cadd_tsv,
            "1\t100\tA\tT\t0.5\t10.2\n1\t200\tC\tG\t-0.3\t3.1\n",
        )
        .unwrap();
        let cadd_db = tmp.join("cadd-db");
        super::super::cadd::run(
            &common(),
            &super::super::cadd::Args {
                path_in_tsv: format!("{}", cadd_tsv.display()),
                path_reference_fai: format!("{}", fai.display()),
                assembly: "GRCh37".into(),
                path_out_rocksdb: format!("{}", cadd_db.display()),
                cf_name: "cadd".into(),
                path_wal_dir: None,
            },
        )
        .unwrap();

        // dbSNP: shared 1:100 A>T and dbsnp-only 1:300 G>A.
        let dbsnp_vcf = tmp.join("dbsnp.vcf");
        std::fs::write(
            &dbsnp_vcf,
            "##fileformat=VCFv4.2\n##contig=<ID=1>\n\
             #CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\n\
             1\t100\trs1\tA\tT\t.\t.\t.\n1\t300\trs2\tG\tA\t.\t.\t.\n",
        )
        .unwrap();
        let dbsnp_db = tmp.join("dbsnp-db");
        super::super::dbsnp::run(
            &common(),
            &super::super::dbsnp::Args {
                path_in_vcf: format!("{}", dbsnp_vcf.display()),
                path_reference_fai: format!("{}", fai.display()),
                assembly: "GRCh37".into(),
                path_out_rocksdb: format!("{}", dbsnp_db.display()),
                cf_name: "dbsnp".into(),
                path_wal_dir: None,
            },
        )
        .unwrap();

        // Merge.
        let out = tmp.join("unified-db");
        run(
            &common(),
            &Args {
                cadd: Some(format!("{}", cadd_db.display())),
                spliceai: None,
                dbsnp: Some(format!("{}", dbsnp_db.display())),
                path_reference_fai: format!("{}", fai.display()),
                assembly: "GRCh37".into(),
                path_out_rocksdb: format!("{}", out.display()),
                cf_name: "unified".into(),
            },
        )
        .unwrap();

        let db = rocksdb::DB::open_cf_for_read_only(
            &rocksdb::Options::default(),
            &out,
            ["meta", "unified"],
            false,
        )
        .unwrap();
        let dict = crate::seqvars::read_contig_dict(&db).unwrap();
        let cf = db.cf_handle("unified").unwrap();
        let id = dict.id_of("1").unwrap();
        let get = |pos, r: &str, a: &str| {
            db.get_cf(&cf, Var::from("1", pos, r, a).encode_with_id(id))
                .unwrap()
                .map(|raw| IntegratedVariantRecord::decode(&raw[..]).unwrap())
        };

        // Shared coordinate: both tracks present, single unified key.
        let shared = get(100, "A", "T").expect("shared present");
        assert!(shared.cadd.is_some() && shared.dbsnp.is_some());
        // CADD-only.
        let c = get(200, "C", "G").expect("cadd-only present");
        assert!(c.cadd.is_some() && c.dbsnp.is_none());
        // dbSNP-only.
        let d = get(300, "G", "A").expect("dbsnp-only present");
        assert!(d.dbsnp.is_some() && d.cadd.is_none());

        // Exactly three unified records.
        let n = db.iterator_cf(&cf, IteratorMode::Start).count();
        assert_eq!(n, 3);
    }

    #[test]
    fn merge_handles_chr_prefix_mismatch() {
        // Tracks built against different naming conventions ("chr1" vs "1") must
        // still merge onto the canonical dictionary via alias canonicalization.
        let tmp = TempDir::default();
        let canonical_fai = tmp.join("canonical.fai");
        std::fs::write(&canonical_fai, "1\t249250621\n").unwrap();

        // CADD track uses "chr1".
        let cadd_fai = tmp.join("cadd.fai");
        std::fs::write(&cadd_fai, "chr1\t249250621\n").unwrap();
        let cadd_tsv = tmp.join("cadd.tsv");
        std::fs::write(&cadd_tsv, "chr1\t100\tA\tT\t0.5\t10.2\n").unwrap();
        let cadd_db = tmp.join("cadd-db");
        super::super::cadd::run(
            &common(),
            &super::super::cadd::Args {
                path_in_tsv: format!("{}", cadd_tsv.display()),
                path_reference_fai: format!("{}", cadd_fai.display()),
                assembly: "GRCh37".into(),
                path_out_rocksdb: format!("{}", cadd_db.display()),
                cf_name: "cadd".into(),
                path_wal_dir: None,
            },
        )
        .unwrap();

        // dbSNP track uses "1" (no prefix) for the same variant.
        let dbsnp_vcf = tmp.join("dbsnp.vcf");
        std::fs::write(
            &dbsnp_vcf,
            "##fileformat=VCFv4.2\n##contig=<ID=1>\n\
             #CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\n\
             1\t100\trs1\tA\tT\t.\t.\t.\n",
        )
        .unwrap();
        let dbsnp_db = tmp.join("dbsnp-db");
        super::super::dbsnp::run(
            &common(),
            &super::super::dbsnp::Args {
                path_in_vcf: format!("{}", dbsnp_vcf.display()),
                path_reference_fai: format!("{}", canonical_fai.display()),
                assembly: "GRCh37".into(),
                path_out_rocksdb: format!("{}", dbsnp_db.display()),
                cf_name: "dbsnp".into(),
                path_wal_dir: None,
            },
        )
        .unwrap();

        let out = tmp.join("unified-db");
        run(
            &common(),
            &Args {
                cadd: Some(format!("{}", cadd_db.display())),
                spliceai: None,
                dbsnp: Some(format!("{}", dbsnp_db.display())),
                path_reference_fai: format!("{}", canonical_fai.display()),
                assembly: "GRCh37".into(),
                path_out_rocksdb: format!("{}", out.display()),
                cf_name: "unified".into(),
            },
        )
        .unwrap();

        let db = rocksdb::DB::open_cf_for_read_only(
            &rocksdb::Options::default(),
            &out,
            ["meta", "unified"],
            false,
        )
        .unwrap();
        let dict = crate::seqvars::read_contig_dict(&db).unwrap();
        let cf = db.cf_handle("unified").unwrap();

        // Despite the "chr1"/"1" mismatch, both tracks land on the same key.
        let key = Var::from("1", 100, "A", "T").encode_with_id(dict.id_of("1").unwrap());
        let rec = db
            .get_cf(&cf, key)
            .unwrap()
            .map(|raw| IntegratedVariantRecord::decode(&raw[..]).unwrap())
            .expect("merged record present");
        assert!(rec.cadd.is_some() && rec.dbsnp.is_some());
        assert_eq!(db.iterator_cf(&cf, IteratorMode::Start).count(), 1);
    }
}
