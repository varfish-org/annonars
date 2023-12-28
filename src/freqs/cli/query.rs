//! Query of sequence variant frequency information.

use std::sync::Arc;

use crate::{
    common::{self, keys, spdi},
    freqs,
};

/// Command line arguments for `freq query` sub command.
#[derive(clap::Parser, Debug, Clone)]
#[command(about = "query frequency count stored in RocksDB", long_about = None)]
pub struct Args {
    /// Path to RocksDB directory with data.
    #[arg(long)]
    pub path_rocksdb: String,

    /// Path to output file, use "-" for stdout.
    #[arg(long, default_value = "-")]
    pub path_output: String,
    /// Output format.
    #[arg(long, default_value = "jsonl")]
    pub out_format: common::cli::OutputFormat,

    /// Variant to query for.
    #[arg(long)]
    pub variant: spdi::Var,
}

/// Meta information as read from database.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Meta {
    /// Genome release of data in database.
    pub genome_release: String,
}

/// Open RocksDb given path and column family name for data and metadata.
pub fn open_rocksdb<P: AsRef<std::path::Path>>(
    path_rocksdb: P,
    cf_auto: &str,
    cf_gono: &str,
    cf_mtdna: &str,
    cf_meta: &str,
) -> Result<(Arc<rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>>, Meta), anyhow::Error> {
    tracing::info!("Opening RocksDB database ...");
    let before_open = std::time::Instant::now();
    let cf_names = &[cf_meta, cf_auto, cf_gono, cf_mtdna];
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
        "autosomal",
        "gonosomal",
        "mitochondrial",
        "meta",
    )
}

/// Enumeration of possible result records.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Record {
    /// Record for autosomal variant.
    Autosomal(freqs::serialized::auto::Record),
    /// Record for gonosomal variant.
    Gonosomal(freqs::serialized::xy::Record),
    /// Record for mitochondrial variant.
    Mitochondrial(freqs::serialized::mt::Record),
}

/// Query for a single variant in the RocksDB database.
pub fn query_for_variant(
    variant: &spdi::Var,
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    _out_format: common::cli::OutputFormat,
) -> Result<Option<Record>, anyhow::Error> {
    let seq = variant.sequence.to_lowercase();
    let var: keys::Var = variant.clone().into();
    let key: Vec<u8> = var.into();
    if seq.contains('m') {
        let cf_mtdna: Arc<rocksdb::BoundColumnFamily> = db.cf_handle("mitochondrial").unwrap();
        let raw_value = db
            .get_cf(&cf_mtdna, &key)
            .map_err(|e| anyhow::anyhow!("error reading from RocksDB: {}", e))?;
        if let Some(raw_value) = raw_value {
            return Ok(Some(Record::Mitochondrial(
                freqs::serialized::mt::Record::from_buf(&raw_value),
            )));
        }
    } else if seq.contains('x') || seq.contains('y') {
        let cf_xy: Arc<rocksdb::BoundColumnFamily> = db.cf_handle("gonosomal").unwrap();
        let raw_value = db
            .get_cf(&cf_xy, &key)
            .map_err(|e| anyhow::anyhow!("error reading from RocksDB: {}", e))?;
        if let Some(raw_value) = raw_value {
            return Ok(Some(Record::Gonosomal(
                freqs::serialized::xy::Record::from_buf(&raw_value),
            )));
        }
    } else {
        let cf_auto: Arc<rocksdb::BoundColumnFamily> = db.cf_handle("autosomal").unwrap();
        let raw_value = db
            .get_cf(&cf_auto, &key)
            .map_err(|e| anyhow::anyhow!("error reading from RocksDB: {}", e))?;
        if let Some(raw_value) = raw_value {
            return Ok(Some(Record::Autosomal(
                freqs::serialized::auto::Record::from_buf(&raw_value),
            )));
        }
    }

    Ok(None)
}

/// Implementation of `tsv query` sub command.
pub fn run(common: &common::cli::Args, args: &Args) -> Result<(), anyhow::Error> {
    tracing::info!("Starting 'freqs query' command");
    tracing::info!("common = {:#?}", &common);
    tracing::info!("args = {:#?}", &args);

    // Obtain writer to output.
    let mut out_writer = match args.path_output.as_ref() {
        "-" => Box::new(std::io::stdout()) as Box<dyn std::io::Write>,
        out_file => {
            let path = std::path::Path::new(out_file);
            Box::new(std::fs::File::create(path).unwrap()) as Box<dyn std::io::Write>
        }
    };

    let (db, _meta) = open_rocksdb_from_args(args)?;

    tracing::info!("Running query...");
    let before_query = std::time::Instant::now();
    if let Some(variant) = query_for_variant(&args.variant, &db, args.out_format)? {
        match variant {
            Record::Autosomal(record) => {
                let json_value = serde_json::to_value(record)?;
                let json = serde_json::to_string(&json_value)?;
                writeln!(out_writer, "{}", &json)?;
            }
            Record::Gonosomal(record) => {
                let json_value = serde_json::to_value(record)?;
                let json = serde_json::to_string(&json_value)?;
                writeln!(out_writer, "{}", &json)?;
            }
            Record::Mitochondrial(record) => {
                let json_value = serde_json::to_value(record)?;
                let json = serde_json::to_string(&json_value)?;
                writeln!(out_writer, "{}", &json)?;
            }
        }
    } else {
        tracing::info!("no record found for variant {:?}", &args.variant);
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

    /// Helper struct for `args_freqs`.
    struct ArgsFreqs {
        /// Common command line arguments.
        common_args: common::cli::Args,
        /// Command line arguments for `freqs query` sub command.
        args: Args,
        /// Path to temporary directory (for RAAI).
        #[allow(dead_code)]
        temp: TempDir,
        /// Genome release.
        genome: String,
        /// Version.
        version: String,
    }

    #[rstest::fixture]
    fn args_freqs(
        #[default("")] variant_str: &str,
        #[default("grch37")] genome: &str,
        #[default("2.1")] version: &str,
    ) -> ArgsFreqs {
        let temp = TempDir::default();
        let common_args = common::cli::Args {
            verbose: clap_verbosity_flag::Verbosity::new(1, 0),
        };
        let args = Args {
            path_rocksdb: format!("tests/freqs/{genome}/v{version}/example/freqs.db"),
            out_format: common::cli::OutputFormat::Jsonl,
            path_output: temp.join("out").to_string_lossy().to_string(),
            variant: spdi::Var::from_str(variant_str).expect("invalid SPDI"),
        };

        ArgsFreqs {
            common_args,
            args,
            temp,
            genome: genome.to_string(),
            version: version.to_string(),
        }
    }

    #[rstest::rstest]
    fn smoke_query_exomes_var_single_match_chr_1(
        #[with("1:55516885:G:A", "grch37", "2.1")] args_freqs: ArgsFreqs,
    ) -> Result<(), anyhow::Error> {
        let ArgsFreqs {
            common_args,
            args,
            temp: _,
            genome,
            version,
        } = args_freqs;
        crate::common::set_snapshot_suffix!(
            "{}-{}-{}",
            &genome,
            &version,
            &args.variant.to_string().replace(':', "_")
        );
        run(&common_args, &args)?;
        let out_data = std::fs::read_to_string(&args.path_output)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[rstest::rstest]
    fn smoke_query_exomes_var_single_match_chr_x(
        #[with("X:69902557:G:T", "grch37", "2.1")] args_freqs: ArgsFreqs,
    ) -> Result<(), anyhow::Error> {
        let ArgsFreqs {
            common_args,
            args,
            temp: _,
            genome,
            version,
        } = args_freqs;
        crate::common::set_snapshot_suffix!(
            "{}-{}-{}",
            &genome,
            &version,
            &args.variant.to_string().replace(':', "_")
        );
        run(&common_args, &args)?;
        let out_data = std::fs::read_to_string(&args.path_output)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[rstest::rstest]
    fn smoke_query_exomes_var_single_match_chr_y(
        #[with("Y:4967199:G:T", "grch37", "2.1")] args_freqs: ArgsFreqs,
    ) -> Result<(), anyhow::Error> {
        let ArgsFreqs {
            common_args,
            args,
            temp: _,
            genome,
            version,
        } = args_freqs;
        crate::common::set_snapshot_suffix!(
            "{}-{}-{}",
            &genome,
            &version,
            &args.variant.to_string().replace(':', "_")
        );
        run(&common_args, &args)?;
        let out_data = std::fs::read_to_string(&args.path_output)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[rstest::rstest]
    fn smoke_query_exomes_var_single_match_chr_mt(
        #[with("M:11:C:T", "grch37", "2.1")] args_freqs: ArgsFreqs,
    ) -> Result<(), anyhow::Error> {
        let ArgsFreqs {
            common_args,
            args,
            temp: _,
            genome,
            version,
        } = args_freqs;
        crate::common::set_snapshot_suffix!(
            "{}-{}-{}",
            &genome,
            &version,
            &args.variant.to_string().replace(':', "_")
        );
        run(&common_args, &args)?;
        let out_data = std::fs::read_to_string(&args.path_output)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[rstest::rstest]
    #[case("1:55516885:G:TT", "grch37", "2.1")]
    fn smoke_query_exomes_var_single_nomatch(
        #[case] _variant: &str,
        #[case] _genome: &str,
        #[case] _version: &str,
        #[with(_variant, _genome, _version)] args_freqs: ArgsFreqs,
    ) -> Result<(), anyhow::Error> {
        let ArgsFreqs {
            common_args,
            args,
            temp: _,
            genome,
            version,
        } = args_freqs;
        crate::common::set_snapshot_suffix!(
            "{}-{}-{}",
            &genome,
            &version,
            &args.variant.to_string().replace(':', "_")
        );
        run(&common_args, &args)?;
        let out_data = std::fs::read_to_string(&args.path_output)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }
}
