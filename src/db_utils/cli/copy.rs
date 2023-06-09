//! Implementation of `db-utils copy` sub command.

use std::{fs::File, io::BufReader, path::PathBuf};

use clap::Parser;
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;

use crate::common::{self, cli::extract_chrom, keys, spdi};

/// Command line arguments for `db copy` sub command.
#[derive(Parser, Debug, Clone)]
#[command(about = "Copy rocksdb databases", long_about = None)]
pub struct Args {
    /// Path to input directory.
    #[arg(long)]
    pub path_in: String,
    /// Path to output directory.
    #[arg(long)]
    pub path_out: String,

    /// Range to query for (or all).
    #[command(flatten)]
    pub query: ArgsQuery,

    /// Optional path to RocksDB WAL directory.
    #[arg(long)]
    pub path_wal_dir: Option<String>,
}

/// Argument group for specifying one of variant, position, or range.
#[derive(clap::Args, Debug, Clone, Default)]
#[group(required = true, multiple = false)]
pub struct ArgsQuery {
    /// Specify position to query for.
    #[arg(long, group = "query")]
    pub position: Option<spdi::Pos>,
    /// Specify range to query for.
    #[arg(long, group = "query")]
    pub range: Option<spdi::Range>,
    /// Specify path(s) to BED files to read from.
    #[arg(long, group = "query")]
    pub path_beds: Vec<PathBuf>,
    /// Query for all variants.
    #[arg(long, group = "query")]
    pub all: bool,
}

/// Copy data from `db_read` to `db_write` for column family `cf_name` in the intervals in `path_bed`.
fn copy_cf_bed(
    db_read: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    db_write: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    cf_name: &str,
    path_bed: &PathBuf,
) -> Result<(), anyhow::Error> {
    let mut reader = File::open(path_bed)
        .map(BufReader::new)
        .map(noodles_bed::Reader::new)?;

    tracing::info!("  reading BED records...");
    let bed_records = reader
        .records::<3>()
        .collect::<Result<Vec<noodles_bed::Record<3>>, _>>()?;
    tracing::info!(
        "  will process {} BED records in parallel...",
        bed_records.len()
    );

    bed_records
        .par_iter()
        .progress_with(common::cli::progress_bar(bed_records.len()))
        .map(|record| {
            let chrom = record.reference_sequence_name();
            let start: usize = record.start_position().into();
            let start = start + 1;
            let stop: usize = record.end_position().into();

            let start = spdi::Pos {
                sequence: chrom.to_string(),
                position: start as i32,
            };
            let stop = spdi::Pos {
                sequence: chrom.to_string(),
                position: stop as i32,
            };

            copy_cf(db_read, db_write, cf_name, Some(start), Some(stop))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(())
}

/// Copy data from `db_read` to `db_write` for column family `cf_name` between `start` and `stop`.
fn copy_cf(
    db_read: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    db_write: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    cf_name: &str,
    start: Option<spdi::Pos>,
    stop: Option<spdi::Pos>,
) -> Result<(), anyhow::Error> {
    // Obtain
    let cf_read = db_read.cf_handle(cf_name).unwrap();
    let cf_write = db_write.cf_handle(cf_name).unwrap();

    tracing::debug!("start = {:?}, stop = {:?}", &start, &stop);

    // Obtain iterator and seek to start.
    let mut iter = db_read.raw_iterator_cf(&cf_read);
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
        if let Some(iter_value) = iter.value() {
            tracing::trace!("iterator at {:?} => {:?}", &iter.key(), &iter_value);
            let iter_key = iter.key().unwrap();
            if let Some(stop) = stop.as_ref() {
                let iter_pos: keys::Pos = iter_key.into();

                if &iter_pos > stop {
                    break;
                }
            }

            db_write.put_cf(&cf_write, iter_key, iter_value)?;

            iter.next();
        } else {
            break;
        }
    }
    Ok(())
}

/// Main entry point for `db copy` sub command.
pub fn run(common: &common::cli::Args, args: &Args) -> Result<(), anyhow::Error> {
    tracing::info!("Starting 'db-utils copy' command");
    tracing::info!("common = {:#?}", &common);
    tracing::info!("args = {:#?}", &args);

    tracing::info!("Opening input database");
    // List all column families in database.
    let cf_names = rocksdb::DB::list_cf(&rocksdb::Options::default(), &args.path_in)?;
    let db_read = rocksdb::DB::open_cf_for_read_only(
        &rocksdb::Options::default(),
        &args.path_in,
        &cf_names,
        false,
    )?;

    // Obtain genome release from "meta" column family if exists.
    let genome_release = if cf_names.iter().any(|s| s == "meta") {
        let cf_meta = db_read.cf_handle("meta").unwrap();
        db_read
            .get_cf(&cf_meta, "genome-release")?
            .map(|bytes| String::from_utf8(bytes.to_vec()))
            .transpose()
            .ok()
            .flatten()
    } else {
        None
    };

    tracing::info!("Opening output database");
    let options = rocksdb_utils_lookup::tune_options(
        rocksdb::Options::default(),
        args.path_wal_dir.as_ref().map(|s| s.as_ref()),
    );
    let db_write = rocksdb::DB::open_cf_with_opts(
        &options,
        &args.path_out,
        cf_names
            .iter()
            .map(|name| (name.to_string(), options.clone()))
            .collect::<Vec<_>>(),
    )?;

    // Perform the main work of copying over data.
    tracing::info!("Copying data");
    for cf_name in &cf_names {
        tracing::info!("  copying data from column family {}", cf_name);
        if cf_name == "meta" {
            tracing::info!("  ignoring query for column family meta");

            copy_cf(&db_read, &db_write, cf_name, None, None)?;
        } else if !args.query.path_beds.is_empty() {
            // If BED files were given then use each to query for ranges.
            for path_bed in &args.query.path_beds {
                copy_cf_bed(&db_read, &db_write, cf_name, path_bed)?;
            }
        } else {
            // Otherwise, get single range from command line arguments.
            let (start, stop) = if let Some(position) = args.query.position.as_ref() {
                let position = spdi::Pos {
                    sequence: extract_chrom::from_pos(position, genome_release.as_deref())?,
                    ..position.clone()
                };
                (Some(position.clone()), Some(position))
            } else if let Some(range) = args.query.range.as_ref() {
                let range = spdi::Range {
                    sequence: extract_chrom::from_range(range, genome_release.as_deref())?,
                    ..range.clone()
                };
                let (start, stop) = range.into();
                (Some(start), Some(stop))
            } else {
                (None, None)
            };

            copy_cf(&db_read, &db_write, cf_name, start, stop)?;
        }
    }

    // Finally, compact manually.
    tracing::info!("Running RocksDB compaction ...");
    let before_compaction = std::time::Instant::now();
    rocksdb_utils_lookup::force_compaction_cf(&db_write, cf_names, Some("  "), true)?;
    tracing::info!(
        "... done compacting RocksDB in {:?}",
        before_compaction.elapsed()
    );

    Ok(())
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;

    use clap_verbosity_flag::Verbosity;
    use temp_testdir::TempDir;

    #[test]
    fn smoke_test_copy_all() -> Result<(), anyhow::Error> {
        let tmp_dir = TempDir::default();
        let common = common::cli::Args {
            verbose: Verbosity::new(1, 0),
        };
        let args = Args {
            path_in: String::from("tests/dbsnp/example/dbsnp.brca1.vcf.bgz.db"),
            path_out: format!("{}", tmp_dir.join("out-rocksdb").display()),
            query: ArgsQuery {
                position: None,
                range: None,
                path_beds: Vec::new(),
                all: true,
            },
            path_wal_dir: None,
        };

        run(&common, &args)
    }

    #[test]
    fn smoke_test_copy_position() -> Result<(), anyhow::Error> {
        let tmp_dir = TempDir::default();
        let common = common::cli::Args {
            verbose: Verbosity::new(1, 0),
        };
        let args = Args {
            path_in: String::from("tests/dbsnp/example/dbsnp.brca1.vcf.bgz.db"),
            path_out: format!("{}", tmp_dir.join("out-rocksdb").display()),
            query: ArgsQuery {
                position: Some(spdi::Pos::from_str("17:41267752")?),
                range: None,
                path_beds: Vec::new(),
                all: false,
            },
            path_wal_dir: None,
        };

        run(&common, &args)
    }

    #[test]
    fn smoke_test_copy_range() -> Result<(), anyhow::Error> {
        let tmp_dir = TempDir::default();
        let common = common::cli::Args {
            verbose: Verbosity::new(1, 0),
        };
        let args = Args {
            path_in: String::from("tests/dbsnp/example/dbsnp.brca1.vcf.bgz.db"),
            path_out: format!("{}", tmp_dir.join("out-rocksdb").display()),
            query: ArgsQuery {
                position: None,
                range: Some(spdi::Range::from_str("17:41267752:41267774")?),
                path_beds: Vec::new(),
                all: false,
            },
            path_wal_dir: None,
        };

        run(&common, &args)
    }
}
