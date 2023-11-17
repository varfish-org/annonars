//! gnomAD CNV v4 import.

use std::{str::FromStr, sync::Arc};

use crate::{
    common::noodles::{get_f32, get_i32, get_string, get_vec_str},
    gnomad_pbs::exac_cnv::CnvType,
    gnomad_pbs::gnomad_cnv4::{
        CarrierCounts, CarrierCountsBySex, Population, PopulationAlleleCounts, Record,
    },
};

use prost::Message;

impl FromStr for CnvType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "DEL" => CnvType::Del,
            "DUP" => CnvType::Dup,
            _ => anyhow::bail!("unknown CNV type: {}", s),
        })
    }
}

impl FromStr for Population {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "AFR" => Population::Afr,
            "AMR" => Population::Amr,
            "ASJ" => Population::Asj,
            "EAS" => Population::Eas,
            "FIN" => Population::Fin,
            "MID" => Population::Mid,
            "NFE" => Population::Nfe,
            "SAS" => Population::Sas,
            _ => anyhow::bail!("unknown population: {}", s),
        })
    }
}

impl ToString for Population {
    fn to_string(&self) -> String {
        match self {
            Population::Afr => "AFR".to_string(),
            Population::Amr => "AMR".to_string(),
            Population::Asj => "ASJ".to_string(),
            Population::Eas => "EAS".to_string(),
            Population::Fin => "FIN".to_string(),
            Population::Mid => "MID".to_string(),
            Population::Nfe => "NFE".to_string(),
            Population::Sas => "SAS".to_string(),
        }
    }
}

impl Record {
    /// Create new Record from VCF record.
    ///
    /// # Arguments
    ///
    /// * `record` - VCF record to create new record from.
    ///
    /// # Returns
    ///
    /// * `Self` - New record.
    ///
    /// # Errors
    ///
    /// * Any error encountered during the creation.
    pub fn from_vcf_record(record: &noodles_vcf::Record) -> Result<Self, anyhow::Error> {
        let chrom = record.chromosome().to_string();
        let start: usize = record.position().into();
        let stop = get_i32(record, "END").expect("no END?");
        let inner_start = get_i32(record, "POSMAX").expect("no POSMAX?");
        let outer_start = get_i32(record, "POSMIN").expect("no POSMIN?");
        let inner_stop = get_i32(record, "ENDMIN").expect("no ENDMIN?");
        let outer_stop = get_i32(record, "ENDMAX").expect("no ENDMAX?");
        let id = record
            .ids()
            .iter()
            .next()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("no ID found in VCF record"))?;
        let sv_len = get_i32(record, "SVLEN").expect("no SVLEN?");
        let sv_type = get_string(record, "SVTYPE")?
            .parse::<CnvType>()
            .map(|x| x as i32)?;
        let n_exn_var = get_i32(record, "N_EXN_VAR,").unwrap_or_default();
        let n_int_var = get_i32(record, "N_INT_VAR").unwrap_or_default();
        let genes = get_vec_str(record, "GENES").unwrap_or_default();
        let counts = Some(Self::carrier_counts_by_sex_from_vcf_record(record, None)?);
        let mut by_population = Vec::new();
        for population in [
            Population::Afr,
            Population::Amr,
            Population::Asj,
            Population::Eas,
            Population::Fin,
            Population::Mid,
            Population::Nfe,
            Population::Sas,
        ] {
            by_population.push(PopulationAlleleCounts {
                population: population as i32,
                counts: Some(Self::carrier_counts_by_sex_from_vcf_record(
                    record,
                    Some(population),
                )?),
            })
        }
        Ok(Self {
            chrom,
            start: start as i32,
            stop,
            inner_start,
            inner_stop,
            outer_start,
            outer_stop,
            id,
            sv_len,
            sv_type,
            n_exn_var,
            n_int_var,
            genes,
            counts,
            by_population,
        })
    }

    /// Extract allele counts from VCF record.
    fn carrier_counts_by_sex_from_vcf_record(
        record: &noodles_vcf::Record,
        population: Option<Population>,
    ) -> Result<CarrierCountsBySex, anyhow::Error> {
        let pop_prefix = population
            .map(|p| format!("{}_", p.to_string().to_lowercase()))
            .unwrap_or_default();

        Ok(CarrierCountsBySex {
            overall: Self::extract_carrier_counts(record, &pop_prefix).ok(),
            xx: Self::extract_carrier_counts(record, &format!("{}FEMALE_", &pop_prefix)).ok(),
            xy: Self::extract_carrier_counts(record, &format!("{}MALE_", &pop_prefix)).ok(),
        })
    }

    /// Extract allele counts for a given population from VCF record.
    fn extract_carrier_counts(
        record: &noodles_vcf::Record,
        prefix: &str,
    ) -> Result<CarrierCounts, anyhow::Error> {
        let sc = get_f32(record, &format!("{}SC", prefix)).unwrap_or_default() as i32;
        let sf = get_f32(record, &format!("{}SF", prefix)).unwrap_or_default();
        let sn = get_f32(record, &format!("{}SN", prefix)).unwrap_or_default() as i32;

        Ok(CarrierCounts { sc, sf, sn })
    }
}

/// Perform import of gnomAD-CNV v4 data.
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
    path_in_vcf: &str,
) -> Result<(), anyhow::Error> {
    let mut reader = noodles_vcf::reader::Builder::default().build_from_path(path_in_vcf)?;
    let header = reader.read_header()?;

    for result in reader.records(&header) {
        let vcf_record = result?;
        let key = format!("{}", vcf_record.ids()).into_bytes();

        // Build record for VCF record and write it to database.
        let record = Record::from_vcf_record(&vcf_record)
            .map_err(|e| anyhow::anyhow!("problem building record from VCF: {}", e))?;
        db.put_cf(cf_data, key, record.encode_to_vec())?;
    }

    Ok(())
}
