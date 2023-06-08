//! Import of minimal ClinVar data.

use std::sync::Arc;

use clap::Parser;
use prost::Message;

use crate::{
    clinvar_minimal,
    common::{self, keys},
};

/// Command line arguments for `tsv import` sub command.
#[derive(Parser, Debug, Clone)]
#[command(about = "import minimal ClinVar data into RocksDB", long_about = None)]
pub struct Args {
    /// Genome build to use in the build.
    #[arg(long, value_enum)]
    pub genome_release: common::cli::GenomeRelease,
    /// Path to input TSV file(s).
    #[arg(long, required = true)]
    pub path_in_tsv: String,
    /// Path to output RocksDB directory.
    #[arg(long)]
    pub path_out_rocksdb: String,

    /// Name of the column family to import into.
    #[arg(long, default_value = "clinvar")]
    pub cf_name: String,
    /// Optional path to RocksDB WAL directory.
    #[arg(long)]
    pub path_wal_dir: Option<String>,
}

/// Perform import of the TSV file.
fn tsv_import(
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    args: &Args,
) -> Result<(), anyhow::Error> {
    let cf_data = db.cf_handle(&args.cf_name).unwrap();

    // Open reader, possibly decompressing gziped files.
    let reader: Box<dyn std::io::Read> = if args.path_in_tsv.ends_with(".gz") {
        Box::new(flate2::read::GzDecoder::new(std::fs::File::open(
            &args.path_in_tsv,
        )?))
    } else {
        Box::new(std::fs::File::open(&args.path_in_tsv)?)
    };

    // Construct CSV reader.
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(true)
        .from_reader(reader);

    // Read through all records and insert each into the database.
    for result in csv_reader.deserialize() {
        let record: clinvar_minimal::cli::reading::Record = result?;
        let clinvar_minimal::cli::reading::Record {
            release,
            chromosome,
            start,
            end,
            reference,
            alternative,
            vcv,
            summary_clinvar_pathogenicity,
            summary_clinvar_gold_stars,
            summary_paranoid_pathogenicity,
            summary_paranoid_gold_stars,
        } = record;
        let summary_clinvar_pathogenicity = summary_clinvar_pathogenicity
            .into_iter()
            .map(|p| {
                let p: clinvar_minimal::pbs::Pathogenicity = p.into();
                p as i32
            })
            .collect();
        let summary_paranoid_pathogenicity = summary_paranoid_pathogenicity
            .into_iter()
            .map(|p| {
                let p: clinvar_minimal::pbs::Pathogenicity = p.into();
                p as i32
            })
            .collect();
        let record = clinvar_minimal::pbs::Record {
            release,
            chromosome,
            start,
            end,
            reference,
            alternative,
            vcv,
            summary_clinvar_pathogenicity,
            summary_clinvar_gold_stars,
            summary_paranoid_pathogenicity,
            summary_paranoid_gold_stars,
        };
        let buf = record.encode_to_vec();

        let var = keys::Var::from(&record.chromosome, record.start as i32, &record.reference, &record.alternative);
        let key: Vec<u8> = var.into();

        db.put_cf(&cf_data, key, buf)?;
    }

    Ok(())
}

/// Implementation of `clinvar-minimal import` sub command.
pub fn run(common: &common::cli::Args, args: &Args) -> Result<(), anyhow::Error> {
    tracing::info!("Starting 'clinvar-minimal import' command");
    tracing::info!("common = {:#?}", &common);
    tracing::info!("args = {:#?}", &args);

    // Open the RocksDB for writing.
    tracing::info!("Opening RocksDB for writing ...");
    let before_opening_rocksdb = std::time::Instant::now();
    let options = common::rocks_utils::tune_options(
        rocksdb::Options::default(),
        args.path_wal_dir.as_ref().map(|s| s.as_ref()),
    );
    let cf_names = &["meta", &args.cf_name];
    let db = Arc::new(rocksdb::DB::open_cf_with_opts(
        &options,
        &args.path_out_rocksdb,
        cf_names
            .iter()
            .map(|name| (name.to_string(), options.clone()))
            .collect::<Vec<_>>(),
    )?);
    tracing::info!("  writing meta information");
    let cf_meta = db.cf_handle("meta").unwrap();
    db.put_cf(&cf_meta, "annonars-version", crate::VERSION)?;
    db.put_cf(
        &cf_meta,
        "genome-release",
        format!("{}", args.genome_release),
    )?;
    db.put_cf(&cf_meta, "db-name", "clinvar-minimal")?;
    tracing::info!(
        "... done opening RocksDB for writing in {:?}",
        before_opening_rocksdb.elapsed()
    );

    tracing::info!("Importing TSV files ...");
    let before_import = std::time::Instant::now();
    tsv_import(&db, args)?;
    tracing::info!(
        "... done importing TSV files in {:?}",
        before_import.elapsed()
    );

    tracing::info!("Running RocksDB compaction ...");
    let before_compaction = std::time::Instant::now();
    common::rocks_utils::force_compaction_cf(&db, cf_names, Some("  "), true)?;
    tracing::info!(
        "... done compacting RocksDB in {:?}",
        before_compaction.elapsed()
    );

    tracing::info!("All done. Have a nice day!");
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    use clap_verbosity_flag::Verbosity;
    use temp_testdir::TempDir;

    #[test]
    fn smoke_test_import_tsv() {
        let tmp_dir = TempDir::default();
        let common = common::cli::Args {
            verbose: Verbosity::new(1, 0),
        };
        let args = Args {
            genome_release: common::cli::GenomeRelease::Grch37,
            path_in_tsv: String::from("tests/clinvar-minimal/clinvar-seqvars-grch37-tgds.tsv"),
            path_out_rocksdb: format!("{}", tmp_dir.join("out-rocksdb").display()),
            cf_name: String::from("clinvar"),
            path_wal_dir: None,
        };

        run(&common, &args).unwrap();
    }
}
