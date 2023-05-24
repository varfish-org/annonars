//! Import of UCSC 100 vertebrate conservation data

use std::sync::Arc;

use clap::Parser;
use prost::Message;

use crate::{
    common::{self, keys},
    cons,
};

/// Command line arguments for `tsv import` sub command.
#[derive(Parser, Debug, Clone)]
#[command(about = "import conservation data into RocksDB", long_about = None)]
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
    #[arg(long, default_value = "ucsc_conservation")]
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

    // Read through all records.  Collect all at the same position into a `RecordList` and
    // insert these into the database.
    let mut record_list = cons::pbs::RecordList::default();
    let mut last_pos = keys::Pos::default();
    for result in csv_reader.deserialize() {
        let record: cons::pbs::Record = result?;
        let pos = keys::Pos::from(&record.chrom, record.start);

        if pos != last_pos {
            if !record_list.records.is_empty() {
                let key: Vec<u8> = last_pos.into();
                let buf = record_list.encode_to_vec();

                db.put_cf(&cf_data, &key, &buf)?;
            }

            record_list = cons::pbs::RecordList::default();
            last_pos = pos;
        }

        record_list.records.push(record);
    }

    // Handle last record list.
    if !record_list.records.is_empty() {
        let key: Vec<u8> = last_pos.into();
        let buf = record_list.encode_to_vec();

        db.put_cf(&cf_data, &key, &buf)?;
    }

    Ok(())
}

/// Implementation of `cons import` sub command.
pub fn run(common: &common::cli::Args, args: &Args) -> Result<(), anyhow::Error> {
    tracing::info!("Starting 'cons import' command");
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
    db.put_cf(&cf_meta, "db-name", "ucsc-conservation")?;
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
            path_in_tsv: String::from("tests/cons/example/tgds.tsv"),
            path_out_rocksdb: format!("{}", tmp_dir.join("out-rocksdb").display()),
            cf_name: String::from("ucsc_conservation"),
            path_wal_dir: None,
        };

        run(&common, &args).unwrap();
    }
}
