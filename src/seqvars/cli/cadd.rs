//! Import of CADD scores (TSV) into a RocksDB track database.

use std::io::Read;

use clap::Parser;
use prost::Message;

use crate::common;
use crate::common::contig::ContigDict;
use crate::common::keys::Var;
use crate::pbs::seqvars::base::CaddRecord;

/// Command line arguments for `seqvars cadd` (build a CADD track database).
#[derive(Parser, Debug, Clone)]
#[command(about = "Construct CADD score RocksDB database", long_about = None)]
pub struct Args {
    /// Path to the input CADD TSV file (optionally gzip/bgzip-compressed).
    #[arg(long)]
    pub path_in_tsv: String,
    /// Path to the reference FASTA index (`.fai`) defining the contig dictionary.
    #[arg(long)]
    pub path_reference_fai: String,
    /// Assembly / genome-release label to record in the meta CF.
    #[arg(long)]
    pub assembly: String,
    /// Path to the output RocksDB directory.
    #[arg(long)]
    pub path_out_rocksdb: String,
    /// Name of the data column family.
    #[arg(long, default_value = "cadd")]
    pub cf_name: String,
    /// Optional path to a RocksDB WAL directory.
    #[arg(long)]
    pub path_wal_dir: Option<String>,
}

/// One row of the CADD TSV: `chrom, pos, ref, alt, raw_score, phred` (no header).
#[derive(serde::Deserialize, Debug, Clone)]
struct CaddRow {
    chrom: String,
    pos: i32,
    r#ref: String,
    alt: String,
    raw_score: f32,
    phred: f32,
}

/// Open the (possibly compressed) TSV as a byte reader.
fn open_tsv(path: &str) -> Result<Box<dyn Read>, anyhow::Error> {
    let file =
        std::fs::File::open(path).map_err(|e| anyhow::anyhow!("failed to open {}: {}", path, e))?;
    if path.ends_with(".gz") || path.ends_with(".bgz") {
        Ok(Box::new(flate2::read::MultiGzDecoder::new(file)))
    } else {
        Ok(Box::new(file))
    }
}

/// Main entry point for `seqvars cadd`.
pub fn run(_common: &common::cli::Args, args: &Args) -> Result<(), anyhow::Error> {
    tracing::info!("Building CADD track database");
    tracing::info!(
        "  loading contig dictionary from {}",
        &args.path_reference_fai
    );
    let dict = ContigDict::from_fai(&args.path_reference_fai)?;

    tracing::info!("  opening output RocksDB at {}", &args.path_out_rocksdb);
    let (db, cf_names) = crate::seqvars::open_track_db_for_write(
        &args.path_out_rocksdb,
        &args.cf_name,
        args.path_wal_dir.as_deref(),
    )?;
    crate::seqvars::write_track_meta(&db, "cadd", "1.0", &args.assembly, &dict)?;
    let cf_data = db
        .cf_handle(&args.cf_name)
        .ok_or_else(|| anyhow::anyhow!("data column family {} missing", &args.cf_name))?;

    tracing::info!("  importing CADD scores from {}", &args.path_in_tsv);
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .comment(Some(b'#'))
        .from_reader(open_tsv(&args.path_in_tsv)?);

    let mut count: u64 = 0;
    for result in reader.deserialize() {
        let row: CaddRow = result?;
        let chrom_id = dict.id_of(&row.chrom).ok_or_else(|| {
            anyhow::anyhow!(
                "contig {:?} from input is not in the reference dictionary",
                row.chrom
            )
        })?;
        let var = Var::new(
            row.chrom.clone(),
            row.pos,
            row.r#ref.clone(),
            row.alt.clone(),
        );
        let key = var.encode_with_id(chrom_id);

        let record = CaddRecord {
            raw_score: row.raw_score,
            phred: row.phred,
        };
        let mut value = Vec::new();
        record.encode(&mut value)?;
        db.put_cf(&cf_data, key, value)?;
        count += 1;
    }
    tracing::info!("  wrote {} CADD records", count);

    tracing::info!("  compacting");
    let cf_refs = cf_names.iter().map(String::as_str).collect::<Vec<_>>();
    rocksdb_utils_lookup::force_compaction_cf(&db, &cf_refs, Some("  "), true)?;

    tracing::info!("All done. Have a nice day!");
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use clap_verbosity_flag::Verbosity;
    use temp_testdir::TempDir;

    #[test]
    fn smoke_import_cadd_and_read_back() {
        let tmp = TempDir::default();
        let fai = tmp.join("ref.fa.fai");
        let tsv = tmp.join("cadd.tsv");
        let out = tmp.join("out-rocksdb");
        std::fs::write(&fai, "1\t249250621\n2\t243199373\nMT\t16569\n").unwrap();
        std::fs::write(
            &tsv,
            "# comment line\n\
             1\t100\tA\tT\t0.5\t10.2\n\
             1\t200\tC\tG\t-0.3\t3.1\n\
             MT\t50\tG\tA\t1.25\t20.0\n",
        )
        .unwrap();

        let common = common::cli::Args {
            verbose: Verbosity::new(0, 0),
        };
        let args = Args {
            path_in_tsv: format!("{}", tsv.display()),
            path_reference_fai: format!("{}", fai.display()),
            assembly: "GRCh37".to_string(),
            path_out_rocksdb: format!("{}", out.display()),
            cf_name: "cadd".to_string(),
            path_wal_dir: None,
        };
        run(&common, &args).unwrap();

        // Reopen read-only and verify a record round-trips (key + value + dict).
        let db = rocksdb::DB::open_cf_for_read_only(
            &rocksdb::Options::default(),
            &out,
            ["meta", "cadd"],
            false,
        )
        .unwrap();
        let dict = crate::seqvars::read_contig_dict(&db).unwrap();
        assert_eq!(dict.id_of("chr1"), Some(0));

        let cf_data = db.cf_handle("cadd").unwrap();
        let var = Var::from("1", 200, "C", "G");
        let key = var.encode_with_id(dict.id_of("1").unwrap());
        let raw = db.get_cf(&cf_data, key).unwrap().expect("record present");
        let rec = CaddRecord::decode(&raw[..]).unwrap();
        assert!((rec.raw_score - (-0.3)).abs() < 1e-6);
        assert!((rec.phred - 3.1).abs() < 1e-6);

        // The MT record must be retrievable via the "chrM" alias too.
        let key_mt = Var::from("MT", 50, "G", "A").encode_with_id(dict.id_of("chrM").unwrap());
        assert!(db.get_cf(&cf_data, key_mt).unwrap().is_some());
    }
}
