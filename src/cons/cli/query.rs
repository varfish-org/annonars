//! Query of UCSC 100 vertebrate conservation data.

use std::sync::Arc;

use prost::Message;

use crate::{
    common::{self, cli::extract_chrom, keys, spdi},
    cons,
};

/// Command line arguments for `cons query` sub command.
#[derive(clap::Parser, Debug, Clone)]
#[command(about = "query conservation data from RocksDB", long_about = None)]
pub struct Args {
    /// Path to RocksDB directory with data.
    #[arg(long)]
    pub path_rocksdb: String,
    /// Name of the column family to import into.
    #[arg(long, default_value = "ucsc_conservation")]
    pub cf_name: String,
    /// Output file (default is stdout == "-").
    #[arg(long, default_value = "-")]
    pub out_file: String,
    /// Output format.
    #[arg(long, default_value = "jsonl")]
    pub out_format: common::cli::OutputFormat,

    /// Range to query for (or all).
    #[command(flatten)]
    pub query: ArgsQuery,
    /// Optional HGNC gene identifier to limit query to.
    #[arg(long)]
    pub hgnc_id: Option<String>,
}

/// Argument group for specifying one of range or all.
#[derive(clap::Args, Debug, Clone, Default)]
#[group(required = true, multiple = false)]
pub struct ArgsQuery {
    /// Specify range to query for.
    #[arg(long, group = "query")]
    pub range: Option<spdi::Range>,
    /// Query for all variants.
    #[arg(long, group = "query")]
    pub all: bool,
}

/// Meta information as read from database.
#[derive(Debug)]
struct Meta {
    /// Genome release of data in database.
    pub genome_release: String,
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
        let meta_genome_release = String::from_utf8(
            db.get_cf(&cf_meta, "genome-release")?
                .ok_or_else(|| anyhow::anyhow!("missing value meta:genome-release"))?,
        )?;
        Meta {
            genome_release: meta_genome_release,
        }
    };

    tracing::info!("  meta:genome-release = {}", &meta.genome_release);
    tracing::info!(
        "... opening RocksDB database took {:?}",
        before_open.elapsed()
    );

    Ok((db, meta))
}

/// Print values to `out_writer`.
fn print_values(
    out_writer: &mut Box<dyn std::io::Write>,
    output_format: common::cli::OutputFormat,
    record: &cons::pbs::Record,
) -> Result<(), anyhow::Error> {
    match output_format {
        common::cli::OutputFormat::Jsonl => {
            writeln!(out_writer, "{}", serde_json::to_string(&record)?)?;
        }
    }

    Ok(())
}

/// Implementation of `cons query` sub command.
pub fn run(common: &common::cli::Args, args: &Args) -> Result<(), anyhow::Error> {
    tracing::info!("Starting 'cons query' command");
    tracing::info!("common = {:#?}", &common);
    tracing::info!("args = {:#?}", &args);

    // Open the RocksDB database.
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
    let (start, stop) = if let Some(range) = args.query.range.as_ref() {
        let range = spdi::Range {
            sequence: extract_chrom::from_range(range, Some(&meta.genome_release))?,
            ..range.clone()
        };

        let (start, stop) = range.into();
        (Some(start), Some(stop))
    } else {
        (None, None)
    };
    tracing::debug!("  start = {:?}, stop = {:?}", &start, &stop);

    // Obtain iterator and seek to start (actually 2 bp before as each alignment column spans
    // one codon).
    let mut iter = db.raw_iterator_cf(&cf_data);
    if let Some(start) = start.as_ref() {
        let tmp = keys::Pos {
            chrom: start.sequence.clone(),
            pos: start.position - 2,
        };
        let pos: keys::Pos = tmp;
        let key: Vec<u8> = pos.into();
        tracing::debug!("  seeking to key {:?}", &key);
        iter.seek(&key);
    } else {
        iter.seek(b"")
    }

    // Cast stop to `keys::Pos`.
    let stop = stop.map(|stop| -> keys::Pos { stop.into() });
    if let Some(stop) = stop.as_ref() {
        let stop: Vec<u8> = stop.clone().into();
        tracing::debug!("  stop = {:?}", &stop);
    }

    // Iterate over all variants until we are behind stop.
    while iter.valid() {
        if let Some(value) = iter.value() {
            tracing::trace!("  iterator at {:?} => {:?}", &iter.key(), &value);

            // Stop if we are behind the range end already.
            if let Some(stop) = stop.as_ref() {
                let iter_key = iter.key().unwrap();
                let iter_pos: keys::Pos = iter_key.into();

                if &iter_pos > stop {
                    break;
                }
            }

            // Decode the record list and iterate it.
            let record_list = cons::pbs::RecordList::decode(value)?;
            dbg!(&record_list);
            for record in &record_list.records {
                // Skip record if end of iterator is before start of range.  This can happen as we
                // jump two base pairs before the start position as alignment columns span one codon.
                if let Some(start) = start.as_ref() {
                    if record.stop < start.position {
                        iter.next();
                        continue;
                    }
                }

                // If the user provided a HGNC gene ID then skip all records that do not match.
                if let Some(hgnc_id) = args.hgnc_id.as_ref() {
                    if &record.hgnc_id != hgnc_id {
                        tracing::debug!("  skipping record {:?}", &record);
                        iter.next();
                        continue;
                    }
                }

                // If we reach here then we have a record that matches the query range and HGNC gene
                // ID (if given).
                print_values(&mut out_writer, args.out_format, &record)?;
            }

            // Proceed to the next database row.
            iter.next();
        } else {
            break;
        }
    }
    tracing::info!("... done querying in {:?}", before_query.elapsed());

    tracing::info!("All done. Have a nice day!");
    Ok(())
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;

    use temp_testdir::TempDir;

    fn args(query: ArgsQuery, hgnc_id: Option<String>) -> (common::cli::Args, Args, TempDir) {
        let temp = TempDir::default();
        let common = common::cli::Args {
            verbose: clap_verbosity_flag::Verbosity::new(1, 0),
        };
        let args = Args {
            path_rocksdb: String::from("tests/cons/example/tgds.tsv.db"),
            cf_name: String::from("ucsc_conservation"),
            out_file: temp.join("out").to_string_lossy().to_string(),
            out_format: common::cli::OutputFormat::Jsonl,
            hgnc_id,
            query,
        };

        (common, args, temp)
    }

    #[test]
    fn smoke_query_range_without_hgnc_id() -> Result<(), anyhow::Error> {
        let (common, args, _temp) = args(
            ArgsQuery {
                range: Some(spdi::Range::from_str("GRCh37:13:95248336:95248351")?),
                all: false,
            },
            None,
        );
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[test]
    fn smoke_query_all_without_hgnc_id_result() -> Result<(), anyhow::Error> {
        let (common, args, _temp) = args(
            ArgsQuery {
                range: None,
                all: true,
            },
            None,
        );
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[test]
    fn smoke_query_range_with_hgnc_id_result() -> Result<(), anyhow::Error> {
        let (common, args, _temp) = args(
            ArgsQuery {
                range: Some(spdi::Range::from_str("GRCh37:13:95248336:95248351")?),
                all: false,
            },
            Some(String::from("HGNC:20324")),
        );
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[test]
    fn smoke_query_range_with_hgnc_id_no_result() -> Result<(), anyhow::Error> {
        let (common, args, _temp) = args(
            ArgsQuery {
                range: Some(spdi::Range::from_str("GRCh37:13:95248334:95248351")?),
                all: false,
            },
            Some(String::from("nonexisting")),
        );
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[test]
    fn smoke_query_all_with_hgnc_id_no_result() -> Result<(), anyhow::Error> {
        let (common, args, _temp) = args(
            ArgsQuery {
                range: None,
                all: true,
            },
            Some(String::from("nonexisting")),
        );
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }
}
