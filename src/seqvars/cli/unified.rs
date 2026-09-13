//! Merge per-track databases into one unified RocksDB.
//!
//! The native tracks ([`crate::cadd`], [`crate::spliceai`]) were built with
//! compact contig-ID keys against their own contig dictionary, while dbSNP is
//! read from an existing `annonars dbsnp import` database with the classic
//! two-byte chromosome keys. The merge re-keys everything onto a single
//! canonical dictionary (derived from the reference `.fai`) and consolidates
//! all tracks for a coordinate into one [`IntegratedVariantRecord`], so the
//! unified database contains each coordinate exactly once (no key duplication).

use clap::Parser;
use prost::Message;
use rocksdb::{Direction, IteratorMode};

use crate::common;
use crate::common::contig::ContigDict;
use crate::common::keys::{self, Var};
use crate::pbs::seqvars::base::{CaddRecord, IntegratedVariantRecord, SpliceAiRecord};

/// Column family holding the data of an `annonars dbsnp import` database.
const DBSNP_CF_DATA: &str = "dbsnp_data";
/// Column family with the RS ID lookup of an `annonars dbsnp import` database.
const DBSNP_CF_BY_RSID: &str = "dbsnp_by_rsid";

/// Command line arguments for `seqvars unified`.
#[derive(Parser, Debug, Clone)]
#[command(about = "Merge track databases into a unified RocksDB", long_about = None)]
pub struct Args {
    /// Path to a CADD RocksDB (from `cadd import`) to include.
    #[arg(long)]
    pub cadd: Option<String>,
    /// Path to a SpliceAI RocksDB (from `spliceai import`) to include.
    #[arg(long)]
    pub spliceai: Option<String>,
    /// Path to a dbSNP RocksDB (from `dbsnp import`) to include.
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

/// A coordinate on the current contig, used to order the k-way merge.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Coord {
    pos: i32,
    reference: String,
    alternative: String,
}

impl From<Var> for Coord {
    fn from(var: Var) -> Self {
        Self {
            pos: var.pos,
            reference: var.reference,
            alternative: var.alternative,
        }
    }
}

/// One track's value at a coordinate.
enum TrackValue {
    Cadd(CaddRecord),
    Spliceai(SpliceAiRecord),
    DbsnpRsId(i32),
}

/// An opened source database and how to read its keys and values.
enum Source {
    /// A native track with compact contig-ID keys plus its contig dictionary.
    Cadd(ContigDict),
    /// A native track with compact contig-ID keys plus its contig dictionary.
    Spliceai(ContigDict),
    /// An `annonars dbsnp` database with classic two-byte chromosome keys.
    Dbsnp,
}

impl Source {
    /// Name of the column family holding the data.
    fn cf(&self) -> &'static str {
        match self {
            Source::Cadd(_) => "cadd",
            Source::Spliceai(_) => "spliceai",
            Source::Dbsnp => DBSNP_CF_DATA,
        }
    }

    /// Key prefix selecting `contig` in this source, or `None` if the source
    /// cannot represent the contig at all.
    fn contig_prefix(&self, contig: &str) -> Option<Vec<u8>> {
        match self {
            Source::Cadd(dict) | Source::Spliceai(dict) => {
                let id_bytes = dict.id_of(contig)?.to_be_bytes();
                Some(id_bytes[id_bytes.len() - keys::CONTIG_ID_LEN..].to_vec())
            }
            // Classic keys only cover the canonical chromosomes.
            Source::Dbsnp => common::cli::is_canonical(contig)
                .then(|| keys::chrom_name_to_key(contig).into_bytes()),
        }
    }

    /// Decode one entry into its coordinate and value.
    fn decode(&self, key: &[u8], value: &[u8]) -> Result<(Coord, TrackValue), anyhow::Error> {
        Ok(match self {
            Source::Cadd(dict) => (
                Var::decode_with_ctx(key, dict.id_to_name()).into(),
                TrackValue::Cadd(CaddRecord::decode(value)?),
            ),
            Source::Spliceai(dict) => (
                Var::decode_with_ctx(key, dict.id_to_name()).into(),
                TrackValue::Spliceai(SpliceAiRecord::decode(value)?),
            ),
            Source::Dbsnp => {
                // The dbSNP record carries its own coordinate, so the classic
                // key does not have to be decoded.
                let record = crate::dbsnp::pbs::Record::decode(value)?;
                let coord = Coord {
                    pos: record.pos,
                    reference: record.ref_allele,
                    alternative: record.alt_allele,
                };
                (coord, TrackValue::DbsnpRsId(record.rs_id))
            }
        })
    }
}

/// An opened source database together with its reader.
struct Input {
    source: Source,
    db: rocksdb::DB,
}

/// The current entry of one input within the contig being merged.
struct Front<'a> {
    source: &'a Source,
    iter: rocksdb::DBIteratorWithThreadMode<'a, rocksdb::DB>,
    prefix: Vec<u8>,
    /// Current entry; `None` once the input is past the contig.
    head: Option<(Coord, TrackValue)>,
}

impl Front<'_> {
    /// Read the next entry within the contig prefix into `head`.
    fn advance(&mut self) -> Result<(), anyhow::Error> {
        self.head = match self.iter.next().transpose()? {
            Some((k, v)) if k.starts_with(&self.prefix) => Some(self.source.decode(&k, &v)?),
            _ => None,
        };
        Ok(())
    }
}

/// Open a native track database (compact keys plus contig dictionary) read-only.
fn open_native(
    path: &str,
    cf: &str,
    make: impl FnOnce(ContigDict) -> Source,
) -> Result<Input, anyhow::Error> {
    let db = rocksdb::DB::open_cf_for_read_only(
        &rocksdb::Options::default(),
        common::readlink_f(path)?,
        ["meta", cf],
        false,
    )?;
    let dict = crate::seqvars::read_contig_dict(&db)?;
    Ok(Input {
        source: make(dict),
        db,
    })
}

/// Open an `annonars dbsnp import` database read-only.
fn open_dbsnp(path: &str) -> Result<Input, anyhow::Error> {
    let db = rocksdb::DB::open_cf_for_read_only(
        &rocksdb::Options::default(),
        common::readlink_f(path)?,
        ["meta", DBSNP_CF_DATA, DBSNP_CF_BY_RSID],
        false,
    )?;
    Ok(Input {
        source: Source::Dbsnp,
        db,
    })
}

/// Main entry point for `seqvars unified`.
pub fn run(_common: &common::cli::Args, args: &Args) -> Result<(), anyhow::Error> {
    tracing::info!("Merging track databases into {}", &args.path_out_rocksdb);
    let canonical = ContigDict::from_fai(&args.path_reference_fai)?;

    let mut inputs = Vec::new();
    if let Some(path) = &args.cadd {
        tracing::info!("  loading CADD track from {}", path);
        inputs.push(open_native(path, "cadd", Source::Cadd)?);
    }
    if let Some(path) = &args.spliceai {
        tracing::info!("  loading SpliceAI track from {}", path);
        inputs.push(open_native(path, "spliceai", Source::Spliceai)?);
    }
    if let Some(path) = &args.dbsnp {
        tracing::info!("  loading dbSNP database from {}", path);
        inputs.push(open_dbsnp(path)?);
    }
    if inputs.is_empty() {
        anyhow::bail!("no input tracks given (provide at least one of --cadd/--spliceai/--dbsnp)");
    }

    let (out_db, cf_names) =
        crate::seqvars::open_track_db_for_write(&args.path_out_rocksdb, &args.cf_name, None)?;
    crate::seqvars::write_track_meta(&out_db, "unified", "1.0", &args.assembly, &canonical)?;
    let cf_out = out_db
        .cf_handle(&args.cf_name)
        .ok_or_else(|| anyhow::anyhow!("output column family {} missing", &args.cf_name))?;

    let mut count: u64 = 0;
    let mut batch = rocksdb::WriteBatch::default();
    // Process contig by contig, in canonical (output) ID order.
    for out_id in 0..canonical.len() as u32 {
        let contig = canonical
            .name_of(out_id)
            .ok_or_else(|| anyhow::anyhow!("canonical contig id {} out of range", out_id))?
            .to_string();

        // One forward prefix iterator per input that has this contig.
        let mut fronts: Vec<Front> = Vec::new();
        for input in &inputs {
            let Some(prefix) = input.source.contig_prefix(&contig) else {
                continue;
            };
            let cf = input
                .db
                .cf_handle(input.source.cf())
                .ok_or_else(|| anyhow::anyhow!("column family {} missing", input.source.cf()))?;
            let iter = input.db.iterator_cf_opt(
                &cf,
                rocksdb::ReadOptions::default(),
                IteratorMode::From(&prefix, Direction::Forward),
            );
            let mut front = Front {
                source: &input.source,
                iter,
                prefix,
                head: None,
            };
            front.advance()?;
            if front.head.is_some() {
                fronts.push(front);
            }
        }

        // k-way merge: repeatedly consume all fronts at the lowest coordinate.
        while let Some(min) = fronts
            .iter()
            .filter_map(|f| f.head.as_ref().map(|(coord, _)| coord))
            .min()
            .cloned()
        {
            let mut record = IntegratedVariantRecord::default();
            for front in fronts.iter_mut() {
                if front.head.as_ref().map(|(coord, _)| coord) != Some(&min) {
                    continue;
                }
                if let Some((_, value)) = front.head.take() {
                    match value {
                        TrackValue::Cadd(r) => record.cadd = Some(r),
                        TrackValue::Spliceai(r) => record.splice_ai = Some(r),
                        TrackValue::DbsnpRsId(rs_id) => record.dbsnp_rs_id = Some(rs_id),
                    }
                }
                front.advance()?;
            }

            let out_var = Var::new(contig.clone(), min.pos, min.reference, min.alternative);
            batch.put_cf(
                &cf_out,
                out_var.encode_with_id(out_id),
                record.encode_to_vec(),
            );
            count += 1;
            if count % crate::seqvars::WRITE_BATCH_SIZE == 0 {
                out_db.write(std::mem::take(&mut batch))?;
            }
        }
    }
    out_db.write(batch)?;
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

    /// A dbSNP database built by `annonars dbsnp import` (BRCA1 excerpt, GRCh37).
    const DBSNP_DB: &str = "tests/dbsnp/example/dbsnp.brca1.vcf.bgz.db";

    fn common() -> common::cli::Args {
        common::cli::Args {
            verbose: Verbosity::new(0, 0),
        }
    }

    /// Build a CADD track database from the given `.fai` and TSV content.
    fn build_cadd(dir: &std::path::Path, fai: &str, tsv: &str) -> String {
        let path_fai = dir.join("cadd-ref.fa.fai");
        let path_tsv = dir.join("cadd.tsv");
        let path_db = dir.join("cadd-db");
        std::fs::write(&path_fai, fai).unwrap();
        std::fs::write(&path_tsv, tsv).unwrap();
        crate::cadd::cli::import::run(
            &common(),
            &crate::cadd::cli::import::Args {
                path_in_tsv: format!("{}", path_tsv.display()),
                path_reference_fai: format!("{}", path_fai.display()),
                assembly: "GRCh37".into(),
                path_out_rocksdb: format!("{}", path_db.display()),
                cf_name: "cadd".into(),
                path_wal_dir: None,
            },
        )
        .unwrap();
        format!("{}", path_db.display())
    }

    /// Number of records in the dbSNP fixture database.
    fn dbsnp_record_count() -> usize {
        let input = open_dbsnp(DBSNP_DB).unwrap();
        let cf = input.db.cf_handle(DBSNP_CF_DATA).unwrap();
        input.db.iterator_cf(&cf, IteratorMode::Start).count()
    }

    /// Merge the given CADD database with the dbSNP fixture and open the result.
    fn merge_with_dbsnp(
        dir: &std::path::Path,
        cadd_db: String,
        fai: &str,
    ) -> (rocksdb::DB, ContigDict) {
        let path_fai = dir.join("canonical.fa.fai");
        let path_out = dir.join("unified-db");
        std::fs::write(&path_fai, fai).unwrap();
        run(
            &common(),
            &Args {
                cadd: Some(cadd_db),
                spliceai: None,
                dbsnp: Some(DBSNP_DB.into()),
                path_reference_fai: format!("{}", path_fai.display()),
                assembly: "GRCh37".into(),
                path_out_rocksdb: format!("{}", path_out.display()),
                cf_name: "unified".into(),
            },
        )
        .unwrap();

        let db = rocksdb::DB::open_cf_for_read_only(
            &rocksdb::Options::default(),
            &path_out,
            ["meta", "unified"],
            false,
        )
        .unwrap();
        let dict = crate::seqvars::read_contig_dict(&db).unwrap();
        (db, dict)
    }

    #[test]
    fn merge_cadd_and_dbsnp() {
        let tmp = TempDir::default();
        // 17:41267746 C>A is also in dbSNP (rs80357446); 17:41267748 T>G is not.
        let cadd_db = build_cadd(
            &tmp,
            "17\t81195210\n",
            "17\t41267746\tC\tA\t0.5\t10.2\n17\t41267748\tT\tG\t-0.3\t3.1\n",
        );
        let (db, dict) = merge_with_dbsnp(&tmp, cadd_db, "17\t81195210\n");

        let cf = db.cf_handle("unified").unwrap();
        let id = dict.id_of("17").unwrap();
        let get = |pos, r: &str, a: &str| {
            db.get_cf(&cf, Var::from("17", pos, r, a).encode_with_id(id))
                .unwrap()
                .map(|raw| IntegratedVariantRecord::decode(&raw[..]).unwrap())
        };

        // Shared coordinate: both tracks in a single unified record.
        let shared = get(41_267_746, "C", "A").expect("shared present");
        assert!(shared.cadd.is_some());
        assert_eq!(shared.dbsnp_rs_id, Some(80_357_446));
        // CADD-only.
        let cadd_only = get(41_267_748, "T", "G").expect("cadd-only present");
        assert!(cadd_only.cadd.is_some() && cadd_only.dbsnp_rs_id.is_none());
        // dbSNP-only.
        let dbsnp_only = get(41_267_747, "A", "C").expect("dbsnp-only present");
        assert_eq!(dbsnp_only.dbsnp_rs_id, Some(80_357_327));
        assert!(dbsnp_only.cadd.is_none());

        // Every dbSNP record plus the single CADD-only one, each exactly once.
        assert_eq!(
            db.iterator_cf(&cf, IteratorMode::Start).count(),
            dbsnp_record_count() + 1
        );
    }

    #[test]
    fn merge_handles_chr_prefix_mismatch() {
        // The CADD track uses "chr17" while dbSNP uses "17"; canonicalization must
        // still land both on the same unified key.
        let tmp = TempDir::default();
        let cadd_db = build_cadd(
            &tmp,
            "chr17\t81195210\n",
            "chr17\t41267746\tC\tA\t0.5\t10.2\n",
        );
        let (db, dict) = merge_with_dbsnp(&tmp, cadd_db, "17\t81195210\n");

        let cf = db.cf_handle("unified").unwrap();
        let key = Var::from("17", 41_267_746, "C", "A").encode_with_id(dict.id_of("17").unwrap());
        let rec = db
            .get_cf(&cf, key)
            .unwrap()
            .map(|raw| IntegratedVariantRecord::decode(&raw[..]).unwrap())
            .expect("merged record present");
        assert!(rec.cadd.is_some());
        assert_eq!(rec.dbsnp_rs_id, Some(80_357_446));
        assert_eq!(
            db.iterator_cf(&cf, IteratorMode::Start).count(),
            dbsnp_record_count()
        );
    }
}
