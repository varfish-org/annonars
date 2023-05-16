//! Code for importing TSV without tabix.

use std::io::{BufRead, BufReader};

use super::Args;

use crate::tsv;

/// Perform the import of a single TSV file sequentially.
pub fn tsv_import(
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    args: &Args,
    config: &tsv::schema::infer::Config,
    schema: &tsv::schema::FileSchema,
    path_in_tsv: &str,
) -> Result<(), anyhow::Error> {
    let cf_data = db.cf_handle(&args.cf_name).unwrap();

    // Open the file with a buffered reader.  If the extension indicates gzip-ed or bgziped
    // data then first try to open as bgzip.  If this fails then open with deflate.
    tracing::debug!("opening file '{}'", path_in_tsv);
    let reader: Box<dyn BufRead> = if path_in_tsv.ends_with(".gz") || path_in_tsv.ends_with(".bgz")
    {
        if let Ok(reader) = bgzip::BGZFReader::new(std::fs::File::open(path_in_tsv)?) {
            Box::new(reader)
        } else {
            Box::new(BufReader::new(flate2::read::GzDecoder::new(
                std::fs::File::open(path_in_tsv)?,
            )))
        }
    } else {
        Box::new(BufReader::new(std::fs::File::open(path_in_tsv)?))
    };

    let ctx = tsv::coding::Context::new(config.clone(), schema.clone());

    // Read the file line by line, decode the values, extract position, and insert into RocksDB
    // instance.
    for (i, line) in reader.lines().enumerate() {
        if i <= args.skip_row_count {
            // skip lines (also: skip header)
            continue;
        }

        super::process_tsv_line(
            &line.map_err(|e| anyhow::anyhow!("failed to read line {}:  {}", i, e))?,
            &ctx,
            db,
            &cf_data,
        )?;
    }

    Ok(())
}
