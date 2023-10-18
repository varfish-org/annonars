//! Implementation of `db-utils dump-meta` sub command.

use clap::Parser;

use crate::common;

/// Command line arguments for `db-utils dump-meta` sub command.
#[derive(Parser, Debug, Clone)]
#[command(about = "Dump the metadata columns", long_about = None)]
pub struct Args {
    /// Path to input directory.
    #[arg(long)]
    pub path_in: String,
}

/// Main entry point for `db-utils dump-meta` sub command.
pub fn run(common: &common::cli::Args, args: &Args) -> Result<(), anyhow::Error> {
    tracing::info!("Starting 'db-utils dump-meta' command");
    tracing::info!("common = {:#?}", &common);
    tracing::info!("args = {:#?}", &args);

    tracing::info!("Opening input database");
    // List all column families in database and check that the meta column exists.
    let cf_names = rocksdb::DB::list_cf(&rocksdb::Options::default(), &args.path_in)?;
    if !cf_names.iter().any(|s| s == "meta") {
        anyhow::bail!("input database does not contain a column family named 'meta'");
    }
    // Open database for reading.
    let db_read = rocksdb::DB::open_cf_for_read_only(
        &rocksdb::Options::default(),
        common::readlink_f(&args.path_in)?,
        ["meta"],
        false,
    )?;

    // Iterate over all values in the "meta" column family.
    println!("#key\tvalue");
    let mut count = 0;
    let cf_read = db_read.cf_handle("meta").unwrap();
    let mut iter = db_read.raw_iterator_cf(&cf_read);
    iter.seek(b"");
    while iter.valid() {
        if let Some(iter_value) = iter.value() {
            let iter_key = iter.key().unwrap();
            println!(
                "{}\t{}",
                String::from_utf8(iter_key.to_vec())?,
                String::from_utf8(iter_value.to_vec())?
            );
            iter.next();
            count += 1;
        } else {
            break;
        }
    }
    println!("#rows\t{}", count);

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    use clap_verbosity_flag::Verbosity;

    #[test]
    fn smoke_test_dump() -> Result<(), anyhow::Error> {
        let common = common::cli::Args {
            verbose: Verbosity::new(1, 0),
        };
        let args = Args {
            path_in: String::from("tests/dbsnp/example/dbsnp.brca1.vcf.bgz.db"),
        };

        run(&common, &args)
    }
}
