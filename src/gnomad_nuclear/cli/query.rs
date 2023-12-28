//! Query of gnomAD-exomes and genomes annotation data.

use erased_serde::serialize_trait_object;
use std::{io::Write, sync::Arc};

use prost::Message as _;

/// Helper trait for type erased serialization.
pub trait SerializeRecordTrait: erased_serde::Serialize {}
impl SerializeRecordTrait for pbs::gnomad::gnomad2::Record {}
impl SerializeRecordTrait for pbs::gnomad::gnomad3::Record {}
impl SerializeRecordTrait for pbs::gnomad::gnomad4::Record {}

serialize_trait_object!(SerializeRecordTrait);

use crate::{
    common::{self, cli::extract_chrom, keys, spdi},
    cons::cli::args::vars::ArgsQuery,
    pbs,
};

/// Command line arguments for `tsv query` sub command.
#[derive(clap::Parser, Debug, Clone)]
#[command(about = "query gnomAD-nuclear data stored in RocksDB", long_about = None)]
pub struct Args {
    /// Path to RocksDB directory with data.
    #[arg(long)]
    pub path_rocksdb: String,
    /// Name of the column family to import into.
    #[arg(long, default_value = "gnomad_nuclear_data")]
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
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Meta {
    /// Genome release of data in database.
    pub genome_release: String,
    /// gnomAD version.
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
    let cf_names: &[&str; 2] = &[cf_meta, cf_data];
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
        let meta_gnomad_version = String::from_utf8(
            db.get_cf(&cf_meta, "gnomad-version")?
                .ok_or_else(|| anyhow::anyhow!("missing value meta:gnomad-version"))?,
        )?;
        Meta {
            genome_release: meta_genome_release,
            gnomad_version: meta_gnomad_version,
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
    open_rocksdb(&args.path_rocksdb, &args.cf_name, "meta")
}

fn print_record(
    out_writer: &mut Box<dyn std::io::Write>,
    output_format: common::cli::OutputFormat,
    value: &Box<dyn SerializeRecordTrait>,
) -> Result<(), anyhow::Error> {
    match output_format {
        common::cli::OutputFormat::Jsonl => {
            writeln!(out_writer, "{}", serde_json::to_string(value)?)?;
        }
    }

    Ok(())
}

/// Query for a single variant in the RocksDB database.
pub fn query_for_variant<T>(
    variant: &common::spdi::Var,
    meta: &Meta,
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    cf_data: &Arc<rocksdb::BoundColumnFamily>,
) -> Result<Option<Box<dyn SerializeRecordTrait>>, anyhow::Error>
where
    T: SerializeRecordTrait + prost::Message + std::default::Default + 'static,
{
    // Split off the genome release (checked) and convert to key as used in database.
    let query = spdi::Var {
        sequence: extract_chrom::from_var(variant, Some(&meta.genome_release))?,
        ..variant.clone()
    };
    // Execute query.
    tracing::debug!("query = {:?}", &query);
    let var: keys::Var = query.into();
    let key: Vec<u8> = var.into();
    let raw_value = db
        .get_cf(cf_data, key)
        .map_err(|e| anyhow::anyhow!("problem querying RocksDB: {}", e))?;
    raw_value
        .map(|raw_value| {
            // Decode via prost, box object, and map errors properly.
            match T::decode(&mut std::io::Cursor::new(&raw_value)) {
                Ok(record) => Ok(Box::new(record) as Box<dyn SerializeRecordTrait>),
                Err(e) => Err(anyhow::anyhow!("failed to decode record: {}", e)),
            }
        })
        .transpose()
}

/// Implementation of `tsv query` sub command.
pub fn run(common: &common::cli::Args, args: &Args) -> Result<(), anyhow::Error> {
    tracing::info!("Starting 'gnomad-nuclear query' command");
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
    if let Some(variant) = args.query.variant.as_ref() {
        let query_result = match meta.gnomad_version[0..1].parse::<char>()? {
            '2' => {
                query_for_variant::<pbs::gnomad::gnomad2::Record>(variant, &meta, &db, &cf_data)?
            }
            '3' => {
                query_for_variant::<pbs::gnomad::gnomad3::Record>(variant, &meta, &db, &cf_data)?
            }
            '4' => {
                query_for_variant::<pbs::gnomad::gnomad4::Record>(variant, &meta, &db, &cf_data)?
            }
            _ => unreachable!("unhandled gnomAD version: {}", &meta.gnomad_version),
        };
        if let Some(record) = query_result {
            print_record(&mut out_writer, args.out_format, &record)?
        } else {
            tracing::info!("no record found for variant {:?}", &variant);
        }
    } else {
        let (start, stop) = if let Some(position) = args.query.position.as_ref() {
            let position = spdi::Pos {
                sequence: extract_chrom::from_pos(position, Some(&meta.genome_release))?,
                ..position.clone()
            };
            (Some(position.clone()), Some(position))
        } else if let Some(range) = args.query.range.as_ref() {
            let range = spdi::Range {
                sequence: extract_chrom::from_range(range, Some(&meta.genome_release))?,
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
            if let Some(raw_value) = iter.value() {
                tracing::trace!("iterator at {:?} => {:?}", &iter.key(), &raw_value);
                if let Some(stop) = stop.as_ref() {
                    let iter_key = iter.key().unwrap();
                    let iter_pos: keys::Pos = iter_key.into();

                    if iter_pos.chrom != stop.chrom || iter_pos.pos > stop.pos {
                        break;
                    }
                }

                let mut cursor = std::io::Cursor::new(&raw_value);
                let record: Box<dyn SerializeRecordTrait> =
                    match meta.gnomad_version[0..1].parse::<char>()? {
                        '2' => Box::new(pbs::gnomad::gnomad2::Record::decode(&mut cursor)?),
                        '3' => Box::new(pbs::gnomad::gnomad3::Record::decode(&mut cursor)?),
                        '4' => Box::new(pbs::gnomad::gnomad4::Record::decode(&mut cursor)?),
                        _ => unreachable!("unhandled gnomAD version: {}", &meta.gnomad_version),
                    };
                print_record(&mut out_writer, args.out_format, &record)?;
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

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;

    use temp_testdir::TempDir;

    fn build_args(
        query: ArgsQuery,
        kind: &str,
        genome_release: &str,
        version: &str,
    ) -> (common::cli::Args, Args, TempDir) {
        let temp = TempDir::default();
        let common = common::cli::Args {
            verbose: clap_verbosity_flag::Verbosity::new(1, 0),
        };
        let args = Args {
            path_rocksdb: format!(
                "tests/gnomad-nuclear/example-{}-{}/v{}/gnomad-{}.vcf.bgz.db",
                kind, genome_release, version, kind
            ),
            cf_name: String::from("gnomad_nuclear_data"),
            out_file: temp.join("out").to_string_lossy().to_string(),
            out_format: common::cli::OutputFormat::Jsonl,
            query,
        };

        (common, args, temp)
    }

    #[rstest::rstest]
    #[case("exomes", "grch37", "2.1")]
    #[case("exomes", "grch38", "4.0")]
    #[case("genomes", "grch38", "4.0")]
    fn smoke_query_all(
        #[case] kind: &str,
        #[case] genome_release: &str,
        #[case] version: &str,
    ) -> Result<(), anyhow::Error> {
        crate::common::set_snapshot_suffix!("{}-{}-{}", kind, genome_release, version);
        let (common, args, _temp) = build_args(
            ArgsQuery {
                all: true,
                ..Default::default()
            },
            kind,
            genome_release,
            version,
        );
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[rstest::rstest]
    #[case("exomes", "grch37", "2.1")]
    #[case("exomes", "grch38", "4.0")]
    #[case("genomes", "grch38", "4.0")]
    fn smoke_query_var_single(
        #[case] kind: &str,
        #[case] genome_release: &str,
        #[case] version: &str,
    ) -> Result<(), anyhow::Error> {
        crate::common::set_snapshot_suffix!("{}-{}-{}", kind, genome_release, version);
        let (common, args, _temp) = build_args(
            ArgsQuery {
                variant: Some(spdi::Var::from_str(&format!(
                    "{}:1:55516888:G:GA",
                    if genome_release == "grch37" {
                        "GRCh37"
                    } else {
                        "GRCh38"
                    }
                ))?),
                ..Default::default()
            },
            kind,
            genome_release,
            version,
        );
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[rstest::rstest]
    #[case("exomes", "grch37", "2.1")]
    #[case("exomes", "grch38", "4.0")]
    #[case("genomes", "grch38", "4.0")]
    fn smoke_query_pos_single(
        #[case] kind: &str,
        #[case] genome_release: &str,
        #[case] version: &str,
    ) -> Result<(), anyhow::Error> {
        crate::common::set_snapshot_suffix!("{}-{}-{}", kind, genome_release, version);
        let (common, args, _temp) = build_args(
            ArgsQuery {
                position: Some(spdi::Pos::from_str(&format!(
                    "{}:1:55516888",
                    if genome_release == "grch37" {
                        "GRCh37"
                    } else {
                        "GRCh38"
                    }
                ))?),
                ..Default::default()
            },
            kind,
            genome_release,
            version,
        );
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[rstest::rstest]
    #[case("exomes", "grch37", "2.1")]
    #[case("exomes", "grch38", "4.0")]
    #[case("genomes", "grch38", "4.0")]
    fn smoke_query_range_find_all_chr1(
        #[case] kind: &str,
        #[case] genome_release: &str,
        #[case] version: &str,
    ) -> Result<(), anyhow::Error> {
        crate::common::set_snapshot_suffix!("{}-{}-{}", kind, genome_release, version);
        let (common, args, _temp) = build_args(
            ArgsQuery {
                range: Some(spdi::Range::from_str(&format!(
                    "{}:1:1:249250621",
                    if genome_release == "grch37" {
                        "GRCh37"
                    } else {
                        "GRCh38"
                    }
                ))?),
                ..Default::default()
            },
            kind,
            genome_release,
            version,
        );
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[rstest::rstest]
    #[case("exomes", "grch37", "2.1")]
    #[case("exomes", "grch38", "4.0")]
    #[case("genomes", "grch38", "4.0")]
    fn smoke_query_range_find_first(
        #[case] kind: &str,
        #[case] genome_release: &str,
        #[case] version: &str,
    ) -> Result<(), anyhow::Error> {
        crate::common::set_snapshot_suffix!("{}-{}-{}", kind, genome_release, version);
        let (common, args, _temp) = build_args(
            ArgsQuery {
                range: Some(spdi::Range::from_str(&format!(
                    "{}:1:55505599:55505599",
                    if genome_release == "grch37" {
                        "GRCh37"
                    } else {
                        "GRCh38"
                    }
                ))?),
                ..Default::default()
            },
            kind,
            genome_release,
            version,
        );
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[rstest::rstest]
    #[case("exomes", "grch37", "2.1")]
    #[case("exomes", "grch38", "4.0")]
    #[case("genomes", "grch38", "4.0")]
    fn smoke_query_range_find_second(
        #[case] kind: &str,
        #[case] genome_release: &str,
        #[case] version: &str,
    ) -> Result<(), anyhow::Error> {
        crate::common::set_snapshot_suffix!("{}-{}-{}", kind, genome_release, version);
        let (common, args, _temp) = build_args(
            ArgsQuery {
                range: Some(spdi::Range::from_str(&format!(
                    "{}:1:55505615:55505615",
                    if genome_release == "grch37" {
                        "GRCh37"
                    } else {
                        "GRCh38"
                    }
                ))?),
                ..Default::default()
            },
            kind,
            genome_release,
            version,
        );
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[rstest::rstest]
    #[case("exomes", "grch37", "2.1")]
    #[case("exomes", "grch38", "4.0")]
    #[case("genomes", "grch38", "4.0")]
    fn smoke_query_range_find_none_smaller(
        #[case] kind: &str,
        #[case] genome_release: &str,
        #[case] version: &str,
    ) -> Result<(), anyhow::Error> {
        crate::common::set_snapshot_suffix!("{}-{}-{}", kind, genome_release, version);
        let (common, args, _temp) = build_args(
            ArgsQuery {
                range: Some(spdi::Range::from_str(&format!(
                    "{}:1:1:55505598",
                    if genome_release == "grch37" {
                        "GRCh37"
                    } else {
                        "GRCh38"
                    }
                ))?),
                ..Default::default()
            },
            kind,
            genome_release,
            version,
        );
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[rstest::rstest]
    #[case("exomes", "grch37", "2.1")]
    #[case("exomes", "grch38", "4.0")]
    #[case("genomes", "grch38", "4.0")]
    fn smoke_query_range_find_none_larger(
        #[case] kind: &str,
        #[case] genome_release: &str,
        #[case] version: &str,
    ) -> Result<(), anyhow::Error> {
        crate::common::set_snapshot_suffix!("{}-{}-{}", kind, genome_release, version);
        let (common, args, _temp) = build_args(
            ArgsQuery {
                range: Some(spdi::Range::from_str(&format!(
                    "{}:1:55516889:249250621",
                    if genome_release == "grch37" {
                        "GRCh37"
                    } else {
                        "GRCh38"
                    }
                ))?),
                ..Default::default()
            },
            kind,
            genome_release,
            version,
        );
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }
}
