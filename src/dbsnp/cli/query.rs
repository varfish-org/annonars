//! Query of dbSNP annotation data.

use std::{io::Write, sync::Arc};

use crate::{
    common::{self, keys, spdi},
    cons::cli::args::vars::ArgsQuery,
    dbsnp,
};

/// Command line arguments for `tsv query` sub command.
#[derive(clap::Parser, Debug, Clone)]
#[command(about = "query dbSNP data stored in RocksDB", long_about = None)]
pub struct Args {
    /// Path to RocksDB directory with data.
    #[arg(long)]
    pub path_rocksdb: String,
    /// Name of the column family to import into.
    #[arg(long, default_value = "dbsnp_data")]
    pub cf_name: String,
    /// Output file (default is stdout == "-").
    #[arg(long, default_value = "-")]
    pub out_file: String,
    /// Output format.
    #[arg(long, default_value = "jsonl")]
    pub out_format: common::cli::OutputFormat,

    /// Variant or position to query for.
    #[command(flatten)]
    pub query: ArgsQuery,
}

/// Meta information as read from database.
#[derive(Debug)]
struct Meta {
    /// Genome release of data in database.
    pub genome_release: String,
    /// Name of the database.
    pub db_name: String,
    /// Version of the database.
    pub db_version: String,
}

/// Open RocksDB database.
fn open_rocksdb(
    args: &Args,
) -> Result<(Arc<rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>>, Meta), anyhow::Error> {
    tracing::info!("Opening RocksDB database ...");
    let before_open = std::time::Instant::now();
    let cf_names = &["meta", &args.cf_name];
    let db = Arc::new(rocksdb::DB::open_cf_for_read_only(
        &rocksdb::Options::default(),
        &args.path_rocksdb,
        cf_names,
        true,
    )?);
    tracing::info!("  reading meta information");
    let meta = {
        let cf_meta = db.cf_handle("meta").unwrap();
        let meta_db_name = String::from_utf8(
            db.get_cf(&cf_meta, "db-name")?
                .ok_or_else(|| anyhow::anyhow!("missing value meta:db-schema"))?,
        )?;
        let meta_genome_release = String::from_utf8(
            db.get_cf(&cf_meta, "genome-release")?
                .ok_or_else(|| anyhow::anyhow!("missing value meta:genome-release"))?,
        )?;
        let meta_db_version = String::from_utf8(
            db.get_cf(&cf_meta, "db-version")?
                .ok_or_else(|| anyhow::anyhow!("missing value meta:db-schema"))?,
        )?;
        Meta {
            genome_release: meta_genome_release,
            db_name: meta_db_name,
            db_version: meta_db_version,
        }
    };

    tracing::info!("  meta:db-name = {}", &meta.db_name);
    tracing::info!("  meta:genome-release = {}", &meta.genome_release);
    tracing::info!("  meta:db-version = {}", &meta.db_version);
    tracing::info!(
        "... opening RocksDB database took {:?}",
        before_open.elapsed()
    );

    Ok((db, meta))
}

fn print_values(
    out_writer: &dyn Write,
    out_format: common::cli::OutputFormat,
    meta: &Meta,
    value: &dbsnp::pbs::Record,
) -> Result<(), anyhow::Error> {
    todo!()
}

fn query_for_variant(
    variant: &common::spdi::Var,
    meta: &Meta,
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    cf_data: &Arc<rocksdb::BoundColumnFamily>,
) -> Result<dbsnp::pbs::Record, anyhow::Error> {
    todo!()
}

/// Get chromosome from the SPDI variant.
///
/// If the optional genome release was given then it is compared to the one specified
/// in `meta` and stripped (comparision is case insensitive).
fn extract_chrom_var(variant: &spdi::Var, meta: &Meta) -> Result<String, anyhow::Error> {
    if variant.sequence.contains(':') {
        let mut iter = variant.sequence.rsplitn(2, ':');
        let chromosome = iter.next().unwrap();
        if let Some(genome_release) = iter.next() {
            if genome_release.to_lowercase() != meta.genome_release.to_lowercase() {
                return Err(anyhow::anyhow!(
                    "genome release mismatch (lowercase): expected {}, got {}",
                    meta.genome_release,
                    genome_release
                ));
            }
        }
        Ok(chromosome.to_owned())
    } else {
        Ok(variant.sequence.clone())
    }
}

/// Get chromosome from the SPDI position.
///
/// See `extract_chrom_var` for details.
fn extract_chrom_pos(pos: &spdi::Pos, meta: &Meta) -> Result<String, anyhow::Error> {
    if pos.sequence.contains(':') {
        let mut iter = pos.sequence.rsplitn(2, ':');
        let chromosome = iter.next().unwrap();
        if let Some(genome_release) = iter.next() {
            if genome_release.to_lowercase() != meta.genome_release.to_lowercase() {
                return Err(anyhow::anyhow!(
                    "genome release mismatch (lowercase): expected {}, got {}",
                    meta.genome_release,
                    genome_release
                ));
            }
        }
        Ok(chromosome.to_owned())
    } else {
        Ok(pos.sequence.clone())
    }
}

/// Get chromosome from the SPDI range.
///
/// See `extract_chrom_var` for details.
fn extract_chrom_range(range: &spdi::Range, meta: &Meta) -> Result<String, anyhow::Error> {
    if range.sequence.contains(':') {
        let mut iter = range.sequence.rsplitn(2, ':');
        let chromosome = iter.next().unwrap();
        if let Some(genome_release) = iter.next() {
            if genome_release.to_lowercase() != meta.genome_release.to_lowercase() {
                return Err(anyhow::anyhow!(
                    "genome release mismatch (lowercase): expected {}, got {}",
                    meta.genome_release,
                    genome_release
                ));
            }
        }
        Ok(chromosome.to_owned())
    } else {
        Ok(range.sequence.clone())
    }
}

/// Implementation of `tsv query` sub command.
pub fn run(common: &common::cli::Args, args: &Args) -> Result<(), anyhow::Error> {
    tracing::info!("Starting 'dbsnp query' command");
    tracing::info!("common = {:#?}", &common);
    tracing::info!("args = {:#?}", &args);

    let (db, meta) = open_rocksdb(args)?;
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
    let before_query = std::time::Instant::now();
    if let Some(variant) = args.query.variant.as_ref() {
        print_values(
            &mut out_writer,
            args.out_format,
            &meta,
            &query_for_variant(variant, &meta, &db, &cf_data)?,
        )?;
    } else {
        let (start, stop) = if let Some(position) = args.query.position.as_ref() {
            let position = spdi::Pos {
                sequence: extract_chrom_pos(position, &meta)?,
                ..position.clone()
            };
            (Some(position.clone()), Some(position))
        } else if let Some(range) = args.query.range.as_ref() {
            let range = spdi::Range {
                sequence: extract_chrom_range(range, &meta)?,
                ..range.clone()
            };
            let (start, stop) = range.into();
            (Some(start), Some(stop))
        } else if args.query.all {
            (None, None)
        } else {
            unreachable!()
        };

        tracing::debug!("start = {:?}, stop = {:?}", &start, &stop);

        // Obtain iterator and seek to start.
        let mut iter = db.raw_iterator_cf(&cf_data);
        if let Some(start) = start {
            let pos: keys::Pos = start.into();
            let key: Vec<u8> = pos.into();
            tracing::debug!("seeking to key {:?}", &key);
            iter.seek(&key);
        } else {
            iter.seek(b"")
        }

        // Cast stop to `keys::Pos`.
        let stop = stop.map(|stop| -> keys::Pos { stop.into() });
        if let Some(stop) = stop.as_ref() {
            let stop: Vec<u8> = stop.clone().into();
            tracing::debug!("stop = {:?}", &stop);
        }

        // Iterate over all variants until we are behind stop.
        while iter.valid() {
            if let Some(value) = iter.value() {
                tracing::trace!("iterator at {:?} => {:?}", &iter.key(), &value);
                if let Some(stop) = stop.as_ref() {
                    let iter_key = iter.key().unwrap();
                    let iter_pos: keys::Pos = iter_key.into();

                    if &iter_pos > stop {
                        break;
                    }
                }

                // let values = ctx.decode_values(value)?;
                // print_values(&mut out_writer, args.out_format, &meta, values)?;
                iter.next();
            } else {
                break;
            }
        }
    }
    tracing::info!("... done querying in {:?}", before_query.elapsed());

    tracing::info!("All done. Have a nice day!");
    Ok(())
}
