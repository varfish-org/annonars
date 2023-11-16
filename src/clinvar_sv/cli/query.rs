//! Query of ClinVar SV annonars data.

use std::{io::Write, sync::Arc};

use bio::data_structures::interval_tree::ArrayBackedIntervalTree;
use prost::Message;

use crate::common::{self, cli::extract_chrom, spdi};

/// Argument group for specifying accession or range.
#[derive(clap::Args, Debug, Clone, Default)]
#[group(required = true, multiple = false)]
pub struct ArgsQuery {
    /// Specify range to query for.
    #[arg(long, group = "query")]
    pub accession: Option<String>,
    /// Query for all variants.
    #[arg(long, group = "query")]
    pub all: bool,
    /// Specify range to query for.
    #[arg(long, group = "query")]
    pub range: Option<spdi::Range>,
}

/// Command line arguments for `tsv query` sub command.
#[derive(clap::Parser, Debug, Clone)]
#[command(about = "query gnomAD-mtDNA data stored in RocksDB", long_about = None)]
pub struct Args {
    /// Path to RocksDB directory with data.
    #[arg(long)]
    pub path_rocksdb: String,
    /// Name of the column family to import into.
    #[arg(long, default_value = "clinvar_sv")]
    pub cf_name: String,
    /// Mapping from ClinVar RCV to ClinVar VCV.
    #[arg(long, default_value = "clinvar_sv_by_rcv")]
    pub cf_name_by_rcv: String,
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
pub struct Meta {
    /// Genome release of data in database.
    pub genome_release: String,
}

/// Open RocksDb given path and column family name for data and metadata.
pub fn open_rocksdb<P: AsRef<std::path::Path>>(
    path_rocksdb: P,
    cf_data: &str,
    cf_meta: &str,
    cf_by_rcv: &str,
) -> Result<(Arc<rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>>, Meta), anyhow::Error> {
    tracing::info!("Opening RocksDB database ...");
    let before_open = std::time::Instant::now();
    let cf_names = &[cf_meta, cf_data, cf_by_rcv];
    let db = Arc::new(rocksdb::DB::open_cf_for_read_only(
        &rocksdb::Options::default(),
        common::readlink_f(&path_rocksdb)?,
        cf_names,
        true,
    )?);
    tracing::info!("  reading meta information");
    let meta = {
        let cf_meta = db.cf_handle(cf_meta).unwrap();
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

/// Open RocksDB database from command line arguments.
pub fn open_rocksdb_from_args(
    args: &Args,
) -> Result<(Arc<rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>>, Meta), anyhow::Error> {
    open_rocksdb(
        &args.path_rocksdb,
        &args.cf_name,
        "meta",
        &args.cf_name_by_rcv,
    )
}

fn print_record(
    out_writer: &mut Box<dyn std::io::Write>,
    output_format: common::cli::OutputFormat,
    value: &crate::pbs::annonars::clinvar::v1::sv::Record,
) -> Result<(), anyhow::Error> {
    match output_format {
        common::cli::OutputFormat::Jsonl => {
            writeln!(out_writer, "{}", serde_json::to_string(value)?)?;
        }
    }

    Ok(())
}

/// Query by accession.
pub fn query_for_accession(
    accession: &str,
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    cf_data: &Arc<rocksdb::BoundColumnFamily>,
    cf_by_rcv: &Arc<rocksdb::BoundColumnFamily>,
) -> Result<Option<crate::pbs::annonars::clinvar::v1::sv::Record>, anyhow::Error> {
    // Execute query.
    tracing::debug!("accession = {:?}", &accession);
    let vcv = if accession.starts_with("VCV") {
        accession.as_bytes().into()
    } else if accession.starts_with("RCV") {
        db.get_cf(cf_by_rcv, accession.as_bytes())?
            .ok_or_else(|| anyhow::anyhow!("no VCV found for RCV {}", accession))?
    } else {
        anyhow::bail!("Not a valid VCV/RCV accession: {:?}", &accession);
    };
    tracing::debug!("vcv = {:?}", &vcv);

    let raw_value = db
        .get_cf(cf_data, vcv.clone())
        .map_err(|e| anyhow::anyhow!("error while querying for vcv {:?}: {}", vcv, e))?;
    raw_value
        .map(|raw_value| {
            // Decode via prost.
            crate::pbs::annonars::clinvar::v1::sv::Record::decode(&mut std::io::Cursor::new(
                &raw_value,
            ))
            .map_err(|e| anyhow::anyhow!("failed to decode record: {}", e))
        })
        .transpose()
}

/// Query all variants and print to `out_writer`.
fn print_all(
    out_writer: &mut Box<dyn std::io::Write>,
    out_format: common::cli::OutputFormat,
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    cf_data: &Arc<rocksdb::BoundColumnFamily>,
) -> Result<(), anyhow::Error> {
    tracing::info!("dumping all records...");

    let mut iter = db.raw_iterator_cf(cf_data);
    iter.seek(b"");
    while iter.valid() {
        if let Some(raw_value) = iter.value() {
            let record = crate::pbs::annonars::clinvar::v1::sv::Record::decode(
                &mut std::io::Cursor::new(&raw_value),
            )
            .map_err(|e| anyhow::anyhow!("failed to decode record: {}", e))?;
            print_record(out_writer, out_format, &record)?;
            iter.next();
        } else {
            break;
        }
    }

    tracing::info!("... done dumping all records");
    Ok(())
}

/// Helper data structure that provides per-chromosome interval trees for querying.
pub struct IntervalTrees {
    /// Per-chromosome interval trees.
    trees: rustc_hash::FxHashMap<String, ArrayBackedIntervalTree<u64, String>>,
    /// Backing RocksDB.
    db: Arc<rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>>,
    /// Name of column family with data.
    cf_data_name: String,
    /// Meta information from database.
    meta: Meta,
}

impl IntervalTrees {
    /// Construct new per-contig interval trees.
    ///
    /// This will read all records from the database and build the interval trees.
    ///
    /// # Arguments
    ///
    /// * `db` - Database to read from.
    /// * `cf_data_name` - Name of column family with data.
    /// * `meta` - Meta information from database.
    ///
    /// # Returns
    ///
    /// * `Self` - New instance.
    ///
    /// # Errors
    ///
    /// * If reading from the database fails.
    pub fn with_db(
        db: Arc<rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>>,
        cf_data_name: &str,
        meta: Meta,
    ) -> Result<Self, anyhow::Error> {
        let cf_data = db.cf_handle(cf_data_name).ok_or_else(|| {
            anyhow::anyhow!("no column family with name {:?} found", cf_data_name)
        })?;
        Ok(Self {
            trees: Self::build_trees(db.clone(), cf_data.clone())?,
            db: db.clone(),
            cf_data_name: cf_data_name.to_string(),
            meta: meta,
        })
    }

    /// Build the interval trees.
    fn build_trees(
        db: Arc<rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>>,
        cf_data: Arc<rocksdb::BoundColumnFamily>,
    ) -> Result<rustc_hash::FxHashMap<String, ArrayBackedIntervalTree<u64, String>>, anyhow::Error>
    {
        let mut result: rustc_hash::FxHashMap<String, ArrayBackedIntervalTree<u64, String>> =
            rustc_hash::FxHashMap::default();

        // Obtain iterator and seek to start.
        let mut iter = db.raw_iterator_cf(&cf_data);
        iter.seek(b"");
        while iter.valid() {
            if let Some(raw_value) = iter.value() {
                let record = crate::pbs::annonars::clinvar::v1::sv::Record::decode(
                    &mut std::io::Cursor::new(&raw_value),
                )
                .map_err(|e| anyhow::anyhow!("failed to decode record: {}", e))?;
                tracing::trace!("iterator at {:?} => {:?}", &iter.key(), &record);

                let crate::pbs::annonars::clinvar::v1::sv::Record {
                    chromosome,
                    start,
                    stop,
                    vcv,
                    ..
                } = record;

                let interval = (start as u64 - 1)..(stop as u64);
                tracing::trace!("contig = {} / {:?} / {}", &chromosome, &interval, &vcv);
                result
                    .entry(chromosome.clone())
                    .or_default()
                    .insert(interval, vcv);
                assert!(result.contains_key(&chromosome));

                iter.next();
            } else {
                break;
            }
        }

        result.values_mut().for_each(|tree| tree.index());

        Ok(result)
    }

    /// Query for a range.
    pub fn query(
        &self,
        range: &spdi::Range,
    ) -> Result<Vec<crate::pbs::annonars::clinvar::v1::sv::Record>, anyhow::Error> {
        let contig = extract_chrom::from_range(&range, Some(&self.meta.genome_release))?;
        let cf_data = self.db.cf_handle(&self.cf_data_name).ok_or_else(|| {
            anyhow::anyhow!("no column family with name {:?} found", &self.cf_data_name)
        })?;
        let interval = (range.start as u64 - 1)..(range.end as u64);
        let mut result = Vec::new();
        if let Some(tree) = self.trees.get(&contig) {
            for entry in tree.find(&interval) {
                if let Some(raw_value) = self.db.get_cf(&cf_data, entry.data().as_bytes())? {
                    let record = crate::pbs::annonars::clinvar::v1::sv::Record::decode(
                        &mut std::io::Cursor::new(&raw_value),
                    )
                    .map_err(|e| anyhow::anyhow!("failed to decode record: {}", e))?;
                    result.push(record);
                }
            }
        } else {
            tracing::warn!("unknown contig: {:?}", &contig);
        }

        Ok(result)
    }
}

/// Implementation of `tsv query` sub command.
pub fn run(common: &common::cli::Args, args: &Args) -> Result<(), anyhow::Error> {
    tracing::info!("Starting 'clinvar-sv query' command");
    tracing::info!("common = {:#?}", &common);
    tracing::info!("args = {:#?}", &args);

    let (db, meta) = open_rocksdb_from_args(args)?;
    let cf_data = db.cf_handle(&args.cf_name).unwrap();
    let cf_by_rcv = db.cf_handle(&args.cf_name_by_rcv).unwrap();

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
    if let Some(accession) = args.query.accession.as_ref() {
        tracing::info!("for accession {}", &accession);
        if let Some(record) = query_for_accession(accession, &db, &cf_data, &cf_by_rcv)? {
            print_record(&mut out_writer, args.out_format, &record)?;
        } else {
            tracing::info!("no record found for accession {:?}", &accession);
        }
    } else if let Some(range) = args.query.range.as_ref() {
        tracing::info!("for range {:?}", &range);
        tracing::info!("Building interval trees...");
        let trees = IntervalTrees::with_db(db.clone(), &args.cf_name, meta)
            .map_err(|e| anyhow::anyhow!("failed to build interval trees: {}", e))?;
        tracing::info!("... done building interval trees");
        tracing::info!("Running query...");
        let records = trees
            .query(&range)
            .map_err(|e| anyhow::anyhow!("failed to query interval trees: {}", e))?;
        for record in &records {
            print_record(&mut out_writer, args.out_format, &record)?;
        }
        tracing::info!("... done running query");
    } else if args.query.all {
        tracing::info!("for all");
        print_all(&mut out_writer, args.out_format, &db, &cf_data)?;
    } else {
        unreachable!();
    }
    tracing::info!("... done querying in {:?}", before_query.elapsed());

    tracing::info!("All done. Have a nice day!");
    Ok(())
}

#[cfg(test)]
mod test {
    use std::str::FromStr as _;

    use super::*;

    use temp_testdir::TempDir;

    fn args(query: ArgsQuery) -> (common::cli::Args, Args, TempDir) {
        let temp = TempDir::default();
        let common = common::cli::Args {
            verbose: clap_verbosity_flag::Verbosity::new(1, 0),
        };
        let args = Args {
            path_rocksdb: String::from("tests/clinvar-sv/clinvar-sv-grch37.tsv.db"),
            cf_name: String::from("clinvar_sv"),
            cf_name_by_rcv: String::from("clinvar_sv_by_rcv"),
            out_file: temp.join("out").to_string_lossy().to_string(),
            out_format: common::cli::OutputFormat::Jsonl,
            query,
        };

        (common, args, temp)
    }

    #[test]
    fn smoke_query_var_vcv() -> Result<(), anyhow::Error> {
        let (common, args, _temp) = args(ArgsQuery {
            accession: Some("VCV000057688".into()),
            ..Default::default()
        });
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[test]
    fn smoke_query_var_rcv() -> Result<(), anyhow::Error> {
        let (common, args, _temp) = args(ArgsQuery {
            accession: Some("RCV000051426".into()),
            ..Default::default()
        });
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[test]
    fn smoke_query_var_all() -> Result<(), anyhow::Error> {
        let (common, args, _temp) = args(ArgsQuery {
            all: true,
            ..Default::default()
        });
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[test]
    fn smoke_query_var_range_exact() -> Result<(), anyhow::Error> {
        let (common, args, _temp) = args(ArgsQuery {
            range: Some(spdi::Range::from_str("GRCh37:22:34150132:34182300")?),
            ..Default::default()
        });
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[test]
    fn smoke_query_var_range_overlap() -> Result<(), anyhow::Error> {
        let (common, args, _temp) = args(ArgsQuery {
            range: Some(spdi::Range::from_str("GRCh37:22:34150132:34150200")?),
            ..Default::default()
        });
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[test]
    fn smoke_query_var_range_no_overlap() -> Result<(), anyhow::Error> {
        let (common, args, _temp) = args(ArgsQuery {
            range: Some(spdi::Range::from_str("GRCh37:22:34182000:34182300")?),
            ..Default::default()
        });
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }
}
