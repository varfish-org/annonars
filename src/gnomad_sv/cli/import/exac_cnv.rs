//! ExAC CNV import.

use std::{
    fs::File,
    io::{BufRead as _, BufReader},
    str::FromStr,
    sync::Arc,
};

use byteorder::ByteOrder as _;
use prost::Message as _;

pub use crate::pbs::gnomad::exac_cnv::CnvType;
use crate::pbs::gnomad::exac_cnv::{Population, Record};

/// The to be imported section.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum CurrentSection {
    /// Deletions
    #[default]
    Dels,
    /// Duplications
    Dups,
}

impl FromStr for Population {
    type Err = anyhow::Error;

    /// Parse from value from BED file.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "ExAC-AFR" => Population::Afr,
            "ExAC-AMR" => Population::Amr,
            "ExAC-EAS" => Population::Eas,
            "ExAC-FIN" => Population::Fin,
            "ExAC-NFE" => Population::Nfe,
            "ExAC-SAS" => Population::Sas,
            "ExAC-OTH" => Population::Other,
            _ => anyhow::bail!("unknown population: {}", s),
        })
    }
}

/// Perform import of ExAC CNV data.
///
/// # Arguments
///
/// * `db` - Database connection.
/// * `cf_data` - Column family for data.
/// * `path_in_tsv` - Path to input TSV file.
///
/// # Errors
///
/// * Any error encountered during the import.
pub fn import(
    db: &Arc<rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>>,
    cf_data: &Arc<rocksdb::BoundColumnFamily>,
    path_in_tsv: &str,
) -> Result<(), anyhow::Error> {
    tracing::info!("- selected ExAC CNV import for GRCh37");

    let reader = File::open(path_in_tsv)
        .map(BufReader::new)
        .map_err(|e| anyhow::anyhow!("could not open file: {}", e))?;

    let mut section = CurrentSection::Dels;
    let mut idx: u32 = 0;

    for line in reader.lines() {
        let line = line.map_err(|e| anyhow::anyhow!("could not read line: {}", e))?;
        if line.starts_with("track name=delControls") {
            section = CurrentSection::Dels;
        } else if line.starts_with("track name=dupControls") {
            section = CurrentSection::Dups;
        } else {
            let arr = line.trim().split(' ').collect::<Vec<_>>();

            let record = Record {
                chrom: arr[0]
                    .strip_prefix("chr")
                    .ok_or_else(|| anyhow::anyhow!("invalid chromosome: {}", arr[0]))?
                    .to_string(),
                start: arr[1]
                    .parse()
                    .map_err(|e| anyhow::anyhow!("invalid start: {}", e))?,
                stop: arr[2]
                    .parse()
                    .map_err(|e| anyhow::anyhow!("invalid stop: {}", e))?,
                sv_type: match section {
                    CurrentSection::Dels => CnvType::Del,
                    CurrentSection::Dups => CnvType::Dup,
                } as i32,
                population: arr[3].parse::<Population>()? as i32,
            };
            let buf = record.encode_to_vec();

            let mut key = [0; 4];
            byteorder::LittleEndian::write_u32(&mut key[0..4], idx);
            db.put_cf(cf_data, key, &buf)?;

            idx += 1;
        }
    }

    tracing::info!("  - imported {} records", idx);

    Ok(())
}
