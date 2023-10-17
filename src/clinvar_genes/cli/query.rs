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

/// Open RocksDb given path and column family name for data and metadata.
pub fn open_rocksdb<P: AsRef<std::path::Path>>(
    path_rocksdb: P,
    cf_data: &str,
    cf_meta: &str,
) -> Result<Arc<rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>>, anyhow::Error> {
    tracing::info!("Opening RocksDB database ...");
    let before_open = std::time::Instant::now();
    let cf_names = &[cf_meta, cf_data];
    let db = Arc::new(rocksdb::DB::open_cf_for_read_only(
        &rocksdb::Options::default(),
        &path_rocksdb,
        cf_names,
        true,
    )?);

    tracing::info!(
        "... opening RocksDB database took {:?}",
        before_open.elapsed()
    );

    Ok(db)
}

/// Open RocksDB database from command line arguments.
pub fn open_rocksdb_from_args(
    args: &Args,
) -> Result<Arc<rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>>, anyhow::Error> {
    open_rocksdb(&args.path_rocksdb, &args.cf_name, "meta")
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

/// Query for one gene annotation record.
pub fn query_for_gene(
    hgnc_id: &str,
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    cf_data: &Arc<rocksdb::BoundColumnFamily>,
) -> Result<Option<clinvar_genes::pbs::ClinvarPerGeneRecord>, anyhow::Error> {
    let raw_value = db
        .get_cf(cf_data, hgnc_id.as_bytes())
        .map_err(|e| anyhow::anyhow!("error while querying for HGNC ID {}: {}", hgnc_id, e))?;
    raw_value
        .map(|raw_value| {
            clinvar_genes::pbs::ClinvarPerGeneRecord::decode(&mut std::io::Cursor::new(&raw_value))
                .map_err(|e| {
                    anyhow::anyhow!(
                        "error while decoding clinvar per gene record for HGNC ID {}: {}",
                        hgnc_id,
                        e
                    )
                })
        })
        .transpose()
}

/// Implementation of `gene query` sub command.
pub fn run(common: &common::cli::Args, args: &Args) -> Result<(), anyhow::Error> {
    tracing::info!("Starting 'gene query' command");
    tracing::info!("common = {:#?}", &common);
    tracing::info!("args = {:#?}", &args);

    // Open the RocksDB database.
    let db = open_rocksdb_from_args(args)?;
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
    if let Some(record) = query_for_gene(&args.hgnc_id, &db, &cf_data)? {
        print_record(&mut out_writer, args.out_format, &record)?;
    } else {
        tracing::info!("No data found for HGNC ID {}", args.hgnc_id);
    }

    tracing::info!("All done. Have a nice day!");
    Ok(())
}
