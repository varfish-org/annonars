//! Query gnomAD-SV RocksDB

use std::{io::Write, sync::Arc};

use bio::data_structures::interval_tree::ArrayBackedIntervalTree;
use prost::Message;

use crate::common::{self, cli::extract_chrom, spdi};

/// Argument group for specifying accession or range.
#[derive(clap::Args, Debug, Clone, Default)]
#[group(required = true, multiple = false)]
pub struct ArgsQuery {
    /// Query for all variants.
    #[arg(long, group = "query")]
    pub all: bool,
    /// Query for variant with a specicic accession.
    #[arg(long, group = "query")]
    pub accession: Option<String>,
    /// Specify range to query for.
    #[arg(long, group = "query")]
    pub range: Option<spdi::Range>,
}

/// Command line arguments for `gnomad-sv query` sub command.
#[derive(clap::Parser, Debug, Clone, Default)]
#[command(about = "query gnomAD SV data stored in RocksDB", long_about = None)]
pub struct Args {
    /// Path to RocksDB directory with data.
    #[arg(long)]
    pub path_rocksdb: String,
    /// Name of the column family to import into.
    #[arg(long, default_value = "gnomad_sv")]
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
pub struct Meta {
    /// Genome release of data in database.
    pub genome_release: String,
    /// gnomAD kind
    pub gnomad_kind: String,
    /// gnomAD version
    pub gnomad_version: String,
}

/// Open RocksDb given path and column family name for data and metadata.
pub fn open_rocksdb<P: AsRef<std::path::Path>>(
    path_rocksdb: P,
    cf_data: &str,
    cf_meta: &str,
) -> Result<(Arc<rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>>, Meta), anyhow::Error> {
    tracing::info!("Opening RocksDB database ...");
    let before_open = std::time::Instant::now();
    let cf_names = &[cf_meta, cf_data];
    let db = Arc::new(rocksdb::DB::open_cf_for_read_only(
        &rocksdb::Options::default(),
        common::readlink_f(&path_rocksdb)?,
        cf_names,
        true,
    )?);
    tracing::info!("  reading meta information");
    let meta = {
        let cf_meta = db.cf_handle(cf_meta).unwrap();
        let genome_release = String::from_utf8(
            db.get_cf(&cf_meta, "genome-release")?
                .ok_or_else(|| anyhow::anyhow!("missing value meta:genome-release"))?,
        )?;
        let gnomad_kind = String::from_utf8(
            db.get_cf(&cf_meta, "gnomad-kind")?
                .ok_or_else(|| anyhow::anyhow!("missing value meta:gnomad-kind"))?,
        )?;
        let gnomad_version = String::from_utf8(
            db.get_cf(&cf_meta, "gnomad-version")?
                .ok_or_else(|| anyhow::anyhow!("missing value meta:gnomad-version"))?,
        )?;
        Meta {
            genome_release,
            gnomad_kind,
            gnomad_version,
        }
    };

    tracing::info!("  meta:genome-release = {}", &meta.genome_release);
    tracing::info!("  meta:genome-kind = {}", &meta.gnomad_kind);
    tracing::info!("  meta:genome-version = {}", &meta.gnomad_version);
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
    open_rocksdb(&args.path_rocksdb, &args.cf_name, "meta")
}

/// Enumeration for the different record types that we have.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Record {
    /// ExAC SV record.
    ExacCnv(crate::gnomad_pbs::exac_cnv::Record),
    /// gnomAD-SV v2 record.
    GnomadSv2(crate::gnomad_pbs::gnomad_sv2::Record),
    /// gnomAD-CNV v4 record.
    GnomadCnv4(crate::gnomad_pbs::gnomad_cnv4::Record),
    /// gnomAD-SV v4 record.
    GnomadSv4(crate::gnomad_pbs::gnomad_sv4::Record),
}

/// The necessary data for the tree construction.
#[derive(Debug)]
pub struct TreeData {
    /// The chromosome.
    pub chromosome: String,
    /// The start position.
    pub start: u32,
    /// The stop position.
    pub stop: u32,
}

impl Record {
    fn tree_data(&self) -> TreeData {
        match self {
            Record::ExacCnv(record) => TreeData {
                chromosome: record.chrom.clone(),
                start: record.start as u32,
                stop: record.stop as u32,
            },
            Record::GnomadSv2(record) => TreeData {
                chromosome: record.chrom.clone(),
                start: record.pos as u32,
                stop: record
                    .end
                    .map(|end| end as u32)
                    .unwrap_or(record.pos as u32 + 1),
            },
            Record::GnomadCnv4(record) => TreeData {
                chromosome: record.chrom.clone(),
                start: record.start as u32,
                stop: record.stop as u32,
            },
            Record::GnomadSv4(record) => TreeData {
                chromosome: record.chrom.clone(),
                start: record.pos as u32,
                stop: record
                    .end
                    .map(|end| end as u32)
                    .unwrap_or(record.pos as u32 + 1),
            },
        }
    }
}

/// Write a single record to `out_writer`.
fn print_record(
    out_writer: &mut Box<dyn std::io::Write>,
    output_format: common::cli::OutputFormat,
    value: &Record,
) -> Result<(), anyhow::Error> {
    match (output_format, value) {
        (common::cli::OutputFormat::Jsonl, Record::ExacCnv(record)) => {
            writeln!(out_writer, "{}", serde_json::to_string(record)?)?
        }
        (common::cli::OutputFormat::Jsonl, Record::GnomadSv2(record)) => {
            writeln!(out_writer, "{}", serde_json::to_string(record)?)?
        }
        (common::cli::OutputFormat::Jsonl, Record::GnomadCnv4(record)) => {
            writeln!(out_writer, "{}", serde_json::to_string(record)?)?
        }
        (common::cli::OutputFormat::Jsonl, Record::GnomadSv4(record)) => {
            writeln!(out_writer, "{}", serde_json::to_string(record)?)?
        }
    }

    Ok(())
}

/// Decode a record with `Meta` data as help.
fn decode_record(data: &[u8], meta: &Meta) -> Result<Record, anyhow::Error> {
    Ok(
        match (
            meta.genome_release.as_str(),
            meta.gnomad_kind.as_str(),
            meta.gnomad_version.as_str(),
        ) {
            ("grch37", "exomes", "1.0") => Record::ExacCnv(
                crate::gnomad_pbs::exac_cnv::Record::decode(&mut std::io::Cursor::new(&data))?,
            ),
            ("grch37", "genomes", "2.1") => Record::GnomadSv2(
                crate::gnomad_pbs::gnomad_sv2::Record::decode(&mut std::io::Cursor::new(&data))?,
            ),
            ("grch38", "exomes", "4.0") => Record::GnomadCnv4(
                crate::gnomad_pbs::gnomad_cnv4::Record::decode(&mut std::io::Cursor::new(&data))?,
            ),
            ("grch38", "genomes", "4.0") => Record::GnomadSv4(
                crate::gnomad_pbs::gnomad_sv4::Record::decode(&mut std::io::Cursor::new(&data))?,
            ),
            _ => {
                anyhow::bail!(
                    "cannot handle combination: {} / {} / {}",
                    &meta.genome_release,
                    &meta.gnomad_kind,
                    &meta.gnomad_version
                );
            }
        },
    )
}

/// Query all variants and print to `out_writer`.
fn print_all(
    out_writer: &mut Box<dyn std::io::Write>,
    out_format: common::cli::OutputFormat,
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    cf_data: &Arc<rocksdb::BoundColumnFamily>,
    meta: &Meta,
) -> Result<(), anyhow::Error> {
    tracing::info!("dumping all records...");

    let mut iter = db.raw_iterator_cf(cf_data);
    iter.seek(b"");
    while iter.valid() {
        if let Some(raw_value) = iter.value() {
            print_record(out_writer, out_format, &decode_record(raw_value, meta)?)?;
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
    trees: rustc_hash::FxHashMap<String, ArrayBackedIntervalTree<u64, Vec<u8>>>,
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
            trees: Self::build_trees(db.clone(), cf_data.clone(), &meta)?,
            db: db.clone(),
            cf_data_name: cf_data_name.to_string(),
            meta,
        })
    }

    /// Build the interval trees.
    fn build_trees(
        db: Arc<rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>>,
        cf_data: Arc<rocksdb::BoundColumnFamily>,
        meta: &Meta,
    ) -> Result<rustc_hash::FxHashMap<String, ArrayBackedIntervalTree<u64, Vec<u8>>>, anyhow::Error>
    {
        let mut result: rustc_hash::FxHashMap<String, ArrayBackedIntervalTree<u64, Vec<u8>>> =
            rustc_hash::FxHashMap::default();

        // Obtain iterator and seek to start.
        let mut iter = db.raw_iterator_cf(&cf_data);
        iter.seek(b"");
        while iter.valid() {
            if let Some(raw_value) = iter.value() {
                let record = decode_record(raw_value, meta)
                    .map_err(|e| anyhow::anyhow!("failed to decode record: {}", e))?;
                let key = iter.key().unwrap().to_vec();
                tracing::trace!("iterator at {:?} => {:?}", &key, &record);

                let TreeData {
                    chromosome,
                    start,
                    stop,
                } = record.tree_data();

                let interval = (start as u64 - 1)..(stop as u64);
                let chrom = chromosome.strip_prefix("chr").unwrap_or(&chromosome);
                tracing::trace!("contig = {} / {:?} / {:?}", &chrom, &interval, &key);
                result
                    .entry(chrom.to_string())
                    .or_default()
                    .insert(interval, key);
                assert!(result.contains_key(chrom));

                iter.next();
            } else {
                break;
            }
        }

        result.values_mut().for_each(|tree| tree.index());

        Ok(result)
    }

    /// Query for a range.
    pub fn query(&self, range: &spdi::Range) -> Result<Vec<Record>, anyhow::Error> {
        tracing::trace!("query for {:?}", &range);
        let contig = extract_chrom::from_range(range, Some(&self.meta.genome_release))?;
        let cf_data = self.db.cf_handle(&self.cf_data_name).ok_or_else(|| {
            anyhow::anyhow!("no column family with name {:?} found", &self.cf_data_name)
        })?;
        let interval = (range.start as u64 - 1)..(range.end as u64);
        let mut result = Vec::new();
        if let Some(tree) = self.trees.get(&contig) {
            for entry in tree.find(&interval) {
                tracing::info!("found entry: {:?}", &entry);
                if let Some(raw_value) = self.db.get_cf(&cf_data, entry.data())? {
                    result.push(decode_record(&raw_value, &self.meta)?);
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
    if let Some(range) = args.query.range.as_ref() {
        tracing::info!("for range {:?}", &range);
        tracing::info!("Building interval trees...");
        let trees = IntervalTrees::with_db(db.clone(), &args.cf_name, meta)
            .map_err(|e| anyhow::anyhow!("failed to build interval trees: {}", e))?;
        tracing::info!("... done building interval trees");
        tracing::info!("Running query...");
        let records = trees
            .query(range)
            .map_err(|e| anyhow::anyhow!("failed to query interval trees: {}", e))?;
        for record in &records {
            print_record(&mut out_writer, args.out_format, record)?;
        }
        tracing::info!("... done running query");
    } else if let Some(accession) = args.query.accession.as_ref() {
        tracing::info!("for accession {}", accession);
        let buf = db
            .get_cf(&cf_data, accession.as_bytes())
            .map_err(|e| anyhow::anyhow!("failed to query RocksDB: {}", e))?;
        if let Some(buf) = buf {
            let record = decode_record(&buf, &meta)?;
            print_record(&mut out_writer, args.out_format, &record)?;
        } else {
            tracing::warn!("no record found for accession {}", accession);
        }
        tracing::info!("... done running query");
    } else if args.query.all {
        tracing::info!("for all");
        print_all(&mut out_writer, args.out_format, &db, &cf_data, &meta)?;
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
    use temp_testdir::TempDir;

    use crate::common;

    #[rstest::fixture]
    fn args_args_temp() -> (common::cli::Args, super::Args, TempDir) {
        let temp = TempDir::default();
        let common = common::cli::Args {
            verbose: clap_verbosity_flag::Verbosity::new(1, 0),
        };
        let args = super::Args {
            cf_name: String::from("gnomad_sv"),
            out_file: temp.join("out").to_string_lossy().to_string(),
            out_format: common::cli::OutputFormat::Jsonl,
            ..Default::default()
        };

        (common, args, temp)
    }

    #[tracing_test::traced_test]
    #[rstest::rstest]
    #[case("tests/gnomad-sv/exac-cnv/rocksdb", "exac-cnv")]
    #[case("tests/gnomad-sv/gnomad-sv2/rocksdb", "gnomad-sv2")]
    #[case("tests/gnomad-sv/gnomad-cnv4/rocksdb", "gnomad-cnv4")]
    #[case("tests/gnomad-sv/gnomad-sv4/rocksdb", "gnomad-sv4")]
    fn smoke_query_var_all(
        #[case] path_rocksdb: &str,
        #[case] label: &str,
        args_args_temp: (common::cli::Args, super::Args, TempDir),
    ) -> Result<(), anyhow::Error> {
        crate::common::set_snapshot_suffix!("{}", &label);

        let (common, args, _temp) = args_args_temp;
        let args = super::Args {
            path_rocksdb: path_rocksdb.to_string(),
            query: super::ArgsQuery {
                all: true,
                ..Default::default()
            },
            ..args
        };
        super::run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[tracing_test::traced_test]
    #[rstest::rstest]
    #[case(
        "tests/gnomad-sv/gnomad-sv2/rocksdb",
        "gnomAD-SV_v2.1_BND_1_1",
        "gnomad-sv2"
    )]
    #[case(
        "tests/gnomad-sv/gnomad-cnv4/rocksdb",
        "variant_is_80_2__DEL",
        "gnomad-cnv4"
    )]
    #[case(
        "tests/gnomad-sv/gnomad-sv4/rocksdb",
        "gnomAD-SV_v3_DUP_chr1_01c2781c",
        "gnomad-sv4"
    )]
    fn smoke_query_var_accession(
        #[case] path_rocksdb: &str,
        #[case] accession: &str,
        #[case] label: &str,
        args_args_temp: (common::cli::Args, super::Args, TempDir),
    ) -> Result<(), anyhow::Error> {
        crate::common::set_snapshot_suffix!("{}", &label);

        let (common, args, _temp) = args_args_temp;
        let args = super::Args {
            path_rocksdb: path_rocksdb.to_string(),
            query: super::ArgsQuery {
                accession: Some(accession.to_string()),
                ..Default::default()
            },
            ..args
        };
        super::run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[tracing_test::traced_test]
    #[rstest::rstest]
    #[case(
        "tests/gnomad-sv/exac-cnv/rocksdb",
        "10:51620321:51748701",
        "exac-cnv-some"
    )]
    #[case(
        "tests/gnomad-sv/exac-cnv/rocksdb",
        "1:51620321:51748701",
        "exac-cnv-none"
    )]
    #[case(
        "tests/gnomad-sv/gnomad-sv2/rocksdb",
        "1:94000:94010",
        "gnomad-sv2-some"
    )]
    #[case(
        "tests/gnomad-sv/gnomad-sv2/rocksdb",
        "10:94000:94010",
        "gnomad-sv2-none"
    )]
    #[case(
        "tests/gnomad-sv/gnomad-cnv4/rocksdb",
        "chr1:925634:925635",
        "gnomad-cnv4-some"
    )]
    #[case(
        "tests/gnomad-sv/gnomad-cnv4/rocksdb",
        "chr10:925634:925635",
        "gnomad-cnv4-none"
    )]
    #[case(
        "tests/gnomad-sv/gnomad-sv4/rocksdb",
        "chr1:120937:120938",
        "gnomad-sv4-some"
    )]
    #[case(
        "tests/gnomad-sv/gnomad-sv4/rocksdb",
        "chr10:120937:120938",
        "gnomad-sv4-none"
    )]
    fn smoke_query_range(
        #[case] path_rocksdb: &str,
        #[case] range: &str,
        #[case] label: &str,
        args_args_temp: (common::cli::Args, super::Args, TempDir),
    ) -> Result<(), anyhow::Error> {
        crate::common::set_snapshot_suffix!("{}", &label);

        let (common, args, _temp) = args_args_temp;
        let args = super::Args {
            path_rocksdb: path_rocksdb.to_string(),
            query: super::ArgsQuery {
                range: Some(crate::common::spdi::Range::from_str(range)?),
                ..Default::default()
            },
            ..args
        };
        super::run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }
}
