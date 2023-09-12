//! Querying for gene annotation data.

use std::sync::Arc;

use prost::Message;

use crate::{clinvar_genes, common};

/// Command line arguments for `clinvar-gene query` sub command.
#[derive(clap::Parser, Debug, Clone)]
#[command(about = "query gene information data from RocksDB", long_about = None)]
pub struct Args {
    /// Path to RocksDB directory with data.
    #[arg(long)]
    pub path_rocksdb: String,
    /// Name of the column family to import into.
    #[arg(long, default_value = "clinvar-genes")]
    pub cf_name: String,
    /// Output file (default is stdout == "-").
    #[arg(long, default_value = "-")]
    pub out_file: String,
    /// Output format.
    #[arg(long, default_value = "jsonl")]
    pub out_format: common::cli::OutputFormat,

    /// HGNC gene identifier to query for.
    #[arg(long)]
    pub hgnc_id: String,
}

/// Open RocksDB database.
fn open_rocksdb(
    args: &Args,
) -> Result<Arc<rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>>, anyhow::Error> {
    tracing::info!("Opening RocksDB database ...");
    let before_open = std::time::Instant::now();
    let cf_names = &["meta", &args.cf_name];
    let db = Arc::new(rocksdb::DB::open_cf_for_read_only(
        &rocksdb::Options::default(),
        &args.path_rocksdb,
        cf_names,
        true,
    )?);

    tracing::info!(
        "... opening RocksDB database took {:?}",
        before_open.elapsed()
    );

    Ok(db)
}

/// Print values to `out_writer`.
fn print_record(
    out_writer: &mut Box<dyn std::io::Write>,
    output_format: common::cli::OutputFormat,
    value: &clinvar_genes::pbs::ClinvarPerGeneRecord,
) -> Result<(), anyhow::Error> {
    match output_format {
        common::cli::OutputFormat::Jsonl => {
            writeln!(out_writer, "{}", serde_json::to_string(value)?)?;
        }
    }

    Ok(())
}

/// Implementation of `gene query` sub command.
pub fn run(common: &common::cli::Args, args: &Args) -> Result<(), anyhow::Error> {
    tracing::info!("Starting 'gene query' command");
    tracing::info!("common = {:#?}", &common);
    tracing::info!("args = {:#?}", &args);

    // Open the RocksDB database.
    let db = open_rocksdb(args)?;
    let cf_data = db.cf_handle(&args.cf_name).unwrap();

    // Obtain writer to output.
    let mut out_writer = match args.out_file.as_ref() {
        "-" => Box::new(std::io::stdout()) as Box<dyn std::io::Write>,
        out_file => {
            let path = std::path::Path::new(out_file);
            Box::new(std::fs::File::create(path).unwrap()) as Box<dyn std::io::Write>
        }
    };

    tracing::info!("Running query...");
    let raw_value = db.get_cf(&cf_data, args.hgnc_id.as_bytes())?;
    if let Some(raw_value) = raw_value {
        print_record(
            &mut out_writer,
            args.out_format,
            &clinvar_genes::pbs::ClinvarPerGeneRecord::decode(&mut std::io::Cursor::new(
                &raw_value,
            ))?,
        )?;
    } else {
        tracing::info!("No data found for HGNC ID {}", args.hgnc_id);
    }

    tracing::info!("All done. Have a nice day!");
    Ok(())
}
