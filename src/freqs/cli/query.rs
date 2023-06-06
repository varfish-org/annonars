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

/// Open RocksDB database.
fn open_rocksdb(
    args: &Args,
) -> Result<(Arc<rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>>, Meta), anyhow::Error> {
    tracing::info!("Opening RocksDB database ...");
    let before_open = std::time::Instant::now();
    let cf_names = &["meta", "autosomal", "gonosomal", "mitochondrial"];
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

fn query_for_variant(
    variant: &spdi::Var,
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    out_writer: &mut dyn std::io::Write,
    _out_format: common::cli::OutputFormat,
) -> Result<(), anyhow::Error> {
    let seq = variant.sequence.to_lowercase();
    let var: keys::Var = variant.clone().into();
    let key: Vec<u8> = var.into();
    if seq.contains('m') {
        let cf_mtdna: Arc<rocksdb::BoundColumnFamily> = db.cf_handle("mitochondrial").unwrap();
        let raw_value = db.get_cf(&cf_mtdna, &key)?;
        if let Some(raw_value) = raw_value {
            let value = freqs::serialized::mt::Record::from_buf(&raw_value);
            let json_value = serde_json::to_value(value)?;
            let json = serde_json::to_string(&json_value)?;
            writeln!(out_writer, "{}", &json)?;
        }
    } else if seq.contains('x') || seq.contains('y') {
        let cf_xy: Arc<rocksdb::BoundColumnFamily> = db.cf_handle("gonosomal").unwrap();
        let raw_value = db.get_cf(&cf_xy, &key)?;
        if let Some(raw_value) = raw_value {
            let value = freqs::serialized::mt::Record::from_buf(&raw_value);
            let json_value = serde_json::to_value(value)?;
            let json = serde_json::to_string(&json_value)?;
            writeln!(out_writer, "{}", &json)?;
        }
    } else {
        let cf_auto: Arc<rocksdb::BoundColumnFamily> = db.cf_handle("autosomal").unwrap();
        let raw_value = db.get_cf(&cf_auto, &key)?;
        if let Some(raw_value) = raw_value {
            let value = freqs::serialized::mt::Record::from_buf(&raw_value);
            let json_value = serde_json::to_value(value)?;
            let json = serde_json::to_string(&json_value)?;
            writeln!(out_writer, "{}", &json)?;
        }
    }

    Ok(())
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

    let (db, _meta) = open_rocksdb(args)?;

    tracing::info!("Running query...");
    let before_query = std::time::Instant::now();
    query_for_variant(&args.variant, &db, &mut out_writer, args.out_format)?;
    tracing::info!("... done querying in {:?}", before_query.elapsed());

    tracing::info!("All done. Have a nice day!");
    Ok(())
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;

    use temp_testdir::TempDir;

    fn args_exomes(variant: spdi::Var) -> (common::cli::Args, Args, TempDir) {
        let temp = TempDir::default();
        let common = common::cli::Args {
            verbose: clap_verbosity_flag::Verbosity::new(1, 0),
        };
        let args = Args {
            path_rocksdb: String::from("tests/freqs/example/freqs.db"),
            out_format: common::cli::OutputFormat::Jsonl,
            path_output: temp.join("out").to_string_lossy().to_string(),
            variant,
        };

        (common, args, temp)
    }

    #[test]
    fn smoke_query_exomes_var_single_match_chr_1() -> Result<(), anyhow::Error> {
        let (common, args, _temp) = args_exomes(spdi::Var::from_str("1:55516885:G:A")?);
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.path_output)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[test]
    fn smoke_query_exomes_var_single_match_chr_x() -> Result<(), anyhow::Error> {
        let (common, args, _temp) = args_exomes(spdi::Var::from_str("X:69902557:G:T")?);
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.path_output)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[test]
    fn smoke_query_exomes_var_single_match_chr_y() -> Result<(), anyhow::Error> {
        let (common, args, _temp) = args_exomes(spdi::Var::from_str("Y:4967199:G:T")?);
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.path_output)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }
    #[test]
    fn smoke_query_exomes_var_single_match_chr_mt() -> Result<(), anyhow::Error> {
        let (common, args, _temp) = args_exomes(spdi::Var::from_str("M:11:C:T")?);
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.path_output)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[test]
    fn smoke_query_exomes_var_single_nomatch() -> Result<(), anyhow::Error> {
        let (common, args, _temp) = args_exomes(spdi::Var::from_str("1:55516885:G:TT")?);
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.path_output)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }
}
