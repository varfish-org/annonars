//! Implementation of `tsv import`.

use std::sync::Arc;

use clap::Parser;
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;

use crate::{common, tsv};

pub mod no_tbi;
pub mod par_tbi;

/// Command line arguments for `tsv import` sub command.
#[derive(Parser, Debug, Clone)]
#[command(about = "import tsv data into rocksdb", long_about = None)]
pub struct Args {
    /// Genome build to use in the build.
    #[arg(long, value_enum)]
    pub genome_release: common::cli::GenomeRelease,
    /// Path to input TSV file(s).
    #[arg(long, required = true)]
    pub path_in_tsv: Vec<String>,
    /// Path to output RocksDB directory.
    #[arg(long)]
    pub path_out_rocksdb: String,
    /// Optional path to schema dump in JSON format to start schema inference with.
    #[arg(long)]
    pub path_schema_json: Option<String>,

    /// Name of database to write to metadata.
    #[arg(long)]
    pub db_name: String,
    /// Version of database to write to metadata.
    #[arg(long)]
    pub db_version: String,

    /// Number of rows for schema inference.
    #[arg(long, default_value = "1000")]
    pub inference_row_count: usize,
    /// Number of rows to skip.
    #[arg(long, default_value = "0")]
    pub skip_row_count: usize,
    /// Windows size for TBI-based parallel import.
    #[arg(long, default_value = "1000000")]
    pub tbi_window_size: usize,
    /// Name of the column family to import into.
    #[arg(long, default_value = "tsv_data")]
    pub cf_name: String,
    /// Optional path to RocksDB WAL directory.
    #[arg(long)]
    pub path_wal_dir: Option<String>,

    /// Name of colum containing the chromosome.
    #[arg(long)]
    pub col_chrom: String,
    /// Name of colum containing the 1-based start position.
    #[arg(long)]
    pub col_start: String,
    /// Name of colum containing the reference allele.
    #[arg(long)]
    pub col_ref: String,
    /// Name of colum containing the alternate allele.
    #[arg(long)]
    pub col_alt: String,

    /// Values to be interpreted as null.
    #[arg(long)]
    pub null_values: Vec<String>,
    /// Whether to add the default set of NULL values (NA, ., -).
    #[arg(long)]
    pub add_default_null_values: bool,
}

/// Process a single TSV line.
pub fn process_tsv_line(
    line: &str,
    ctx: &tsv::coding::Context,
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    cf_data: &std::sync::Arc<rocksdb::BoundColumnFamily>,
) -> Result<(), anyhow::Error> {
    let line = line;
    let values = ctx.line_to_values(line)?;
    let values = values.iter().collect::<Vec<_>>();
    let var = ctx.values_to_var(&values)?;

    if let Some(var) = var.as_ref() {
        let key: Vec<u8> = var.clone().into();
        let value = ctx.encode_values(&values)?;

        tracing::trace!(
            "putting for var = {:?}, key = {:?}, value = {:?}",
            &var,
            &key,
            &value
        );

        db.put_cf(cf_data, key, value)?;
    } else {
        tracing::trace!("skipping line: {:?}", &line);
    }

    Ok(())
}

/// Default null values.
const DEFAULT_NULL_VALUES: &[&str] = &["NA", ".", "-"];

/// Implementation of `tsv import` sub command.
pub fn run(common: &common::cli::Args, args: &Args) -> Result<(), anyhow::Error> {
    tracing::info!("Starting 'tsv import' command");
    tracing::info!("common = {:#?}", &common);
    tracing::info!("args = {:#?}", &args);

    // Infer the schema from the input TSV file.
    tracing::info!("Inferring schema from TSV ...");
    let before_inference = std::time::Instant::now();
    let mut null_values = Vec::new();
    if args.add_default_null_values {
        null_values.extend_from_slice(DEFAULT_NULL_VALUES);
    }
    args.null_values.iter().for_each(|s| null_values.push(s));
    let infer_config = tsv::schema::infer::Config {
        null_values: null_values
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
        skip_rows: args.skip_row_count,
        num_rows: args.inference_row_count,
        col_chrom: args.col_chrom.clone(),
        col_start: args.col_start.clone(),
        col_ref: args.col_ref.clone(),
        col_alt: args.col_alt.clone(),
        ..Default::default()
    };
    tracing::info!("  using infer config: {:#?}", &infer_config);
    let infer_ctx = tsv::schema::infer::Context::new(&infer_config);
    let mut schema: Option<tsv::schema::FileSchema> =
        if let Some(path_json_schema) = &args.path_schema_json {
            tracing::info!("  loading initial schema from JSON: {}", path_json_schema);
            let json_string = std::fs::read_to_string(path_json_schema)
                .map_err(|e| anyhow::anyhow!("failed to read schema JSON: {}", e))?;
            serde_json::from_str(&json_string)
                .map_err(|e| anyhow::anyhow!("failed to parse schema JSON: {}", e))?
        } else {
            None
        };
    for path_in_tsv in &args.path_in_tsv {
        tracing::info!("  infer schema from TSV: {}", path_in_tsv);
        let other = infer_ctx.infer_from_path(path_in_tsv)?;
        schema = if let Some(schema) = &schema {
            Some(schema.merge(&other)?)
        } else {
            Some(other)
        }
    }
    let schema = schema.ok_or_else(|| anyhow::anyhow!("failed to infer schema"))?;
    tracing::info!(
        "... done inferring schema from TSV in {:?}",
        before_inference.elapsed()
    );

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
        &format!("{}", args.genome_release),
    )?;
    db.put_cf(&cf_meta, "db-name", &args.db_name)?;
    db.put_cf(&cf_meta, "db-version", &args.db_version)?;
    db.put_cf(&cf_meta, "db-schema", serde_json::to_string(&schema)?)?;
    db.put_cf(
        &cf_meta,
        "db-infer-config",
        serde_json::to_string(&infer_config)?,
    )?;
    tracing::info!(
        "  putting infer config: {}",
        serde_json::to_string(&infer_config)?
    );
    tracing::info!("  putting schema: {}", serde_json::to_string(&schema)?);
    tracing::info!(
        "... done opening RocksDB for writing in {:?}",
        before_opening_rocksdb.elapsed()
    );

    // Check whether a TBI index file exists for all input files.
    tracing::info!("Checking whether TBI index files exist ...");
    let before_checking_tbi = std::time::Instant::now();
    let have_tbi = args
        .path_in_tsv
        .iter()
        .all(|p| std::path::Path::new(&format!("{}.tbi", &p)).exists());
    if have_tbi {
        tracing::info!(
            "  have TBI files, will import one after the other with parallel processing"
        );
    } else {
        tracing::info!("  no TBI files, will import all at once (but each sequentially)");
    }
    tracing::info!(
        "... done checking whether TBI index files exist in {:?}",
        before_checking_tbi.elapsed()
    );

    tracing::info!("Importing TSV files ...");
    let before_import = std::time::Instant::now();
    if have_tbi {
        // If we have TBI files then we can import the files them using window-based
        // parallelism.  We should import them one after another, though.
        for path_in_tsv in &args.path_in_tsv {
            par_tbi::tsv_import(&db, args, &infer_config, &schema, path_in_tsv)?;
        }
    } else {
        // If we don't have TBI files then we have to import them sequentially but
        // can process the list of files in parallel.
        args.path_in_tsv
            .par_iter()
            .progress_with_style(common::cli::indicatif_style())
            .map(|path_in_tsv| no_tbi::tsv_import(&db, args, &infer_config, &schema, path_in_tsv))
            .collect::<Result<Vec<_>, _>>()?;
    }
    tracing::info!(
        "... done importing TSV files in {:?}",
        before_import.elapsed()
    );

    tracing::info!("Running RocksDB compaction ...");
    let before_compaction = std::time::Instant::now();
    common::rocks_utils::force_compaction_cf(&db, cf_names, Some("  "))?;
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

    /// Smoke test for running without a TBI file.
    #[test]
    fn smoke_test_import_tsv_no_tbi() {
        let tmp_dir = TempDir::default();
        let common = common::cli::Args {
            verbose: Verbosity::new(1, 0),
        };
        let args = Args {
            path_in_tsv: vec![String::from("tests/tsv/example/data.tsv")],
            path_out_rocksdb: format!("{}", tmp_dir.join("out-rocksdb").display()),
            path_wal_dir: None,
            genome_release: common::cli::GenomeRelease::Grch37,
            db_name: String::from("test"),
            db_version: String::from("0.0.0"),
            cf_name: String::from("data"),
            skip_row_count: 0,
            path_schema_json: None,
            inference_row_count: 100,
            tbi_window_size: 1000000,
            col_chrom: String::from("CHROM"),
            col_start: String::from("POS"),
            col_ref: String::from("REF"),
            col_alt: String::from("ALT"),
            null_values: Vec::new(),
            add_default_null_values: true,
        };

        run(&common, &args).unwrap();
    }

    /// Smoke test for running with a TBI file.
    #[test]
    fn smoke_test_import_tsv_with_tbi() {
        let tmp_dir = TempDir::default();
        let common = common::cli::Args {
            verbose: Verbosity::new(1, 0),
        };
        let args = Args {
            path_in_tsv: vec![String::from("tests/tsv/example/data.tsv.bgz")],
            path_out_rocksdb: format!("{}", tmp_dir.join("out-rocksdb").display()),
            path_wal_dir: None,
            genome_release: common::cli::GenomeRelease::Grch37,
            db_name: String::from("test"),
            db_version: String::from("0.0.0"),
            cf_name: String::from("data"),
            skip_row_count: 0,
            path_schema_json: None,
            inference_row_count: 100,
            tbi_window_size: 1000000,
            col_chrom: String::from("CHROM"),
            col_start: String::from("POS"),
            col_ref: String::from("REF"),
            col_alt: String::from("ALT"),
            null_values: Vec::new(),
            add_default_null_values: true,
        };

        run(&common, &args).unwrap();
    }
}
