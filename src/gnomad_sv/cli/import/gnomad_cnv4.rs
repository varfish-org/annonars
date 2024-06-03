//! gnomAD CNV v4 import.

use itertools::Itertools;
use noodles::vcf::variant::record::Ids;
use std::{str::FromStr, sync::Arc};

use crate::{
    common::noodles::{get_f32, get_i32, get_string, get_vec_str},
    pbs::gnomad::exac_cnv::CnvType,
    pbs::gnomad::gnomad_cnv4::{
        CarrierCounts, CarrierCountsBySex, CohortCarrierCounts, Population,
        PopulationCarrierCounts, Record,
    },
};

use prost::Message as _;

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
            _ => unreachable!("unknown population: {:?}", self),
        }
    }
}

impl Record {
    /// Create new Record from VCF record.
    ///
    /// # Arguments
    ///
    /// * `record` - VCF record to create new record from.
    /// * `cohort_name` - Name of the cohort.
    ///
    /// # Returns
    ///
    /// * `Self` - New record.
    ///
    /// # Errors
    ///
    /// * Any error encountered during the creation.
    pub fn from_vcf_record(
        record: &noodles::vcf::variant::RecordBuf,
        cohort_name: &str,
    ) -> Result<Self, anyhow::Error> {
        let chrom = record.reference_sequence_name().to_string();
        let start: usize = record
            .variant_start()
            .expect("Telomeric breakends not supported")
            .get();
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
        let by_sex = Some(Self::carrier_counts_by_sex_from_vcf_record(record, None)?);
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
            by_population.push(PopulationCarrierCounts {
                population: population as i32,
                counts: Some(Self::carrier_counts_by_sex_from_vcf_record(
                    record,
                    Some(population),
                )?),
            })
        }
        let carrier_counts = vec![CohortCarrierCounts {
            cohort: if cohort_name.is_empty() {
                None
            } else {
                Some(cohort_name.to_string())
            },
            by_sex,
            by_population,
        }];
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
            carrier_counts,
        })
    }

    /// Extract allele counts from VCF record.
    fn carrier_counts_by_sex_from_vcf_record(
        record: &noodles::vcf::variant::RecordBuf,
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
        record: &noodles::vcf::variant::RecordBuf,
        prefix: &str,
    ) -> Result<CarrierCounts, anyhow::Error> {
        let sc = get_f32(record, &format!("{}SC", prefix)).unwrap_or_default() as i32;
        let sf = get_f32(record, &format!("{}SF", prefix)).unwrap_or_default();
        let sn = get_f32(record, &format!("{}SN", prefix)).unwrap_or_default() as i32;

        Ok(CarrierCounts { sc, sf, sn })
    }

    /// Merge with another record.
    ///
    /// We assume that the record IDs are the same and just concatenate the allele counts.
    ///
    /// # Arguments
    ///
    /// * `other` - Other record to merge with.
    ///
    /// # Returns
    ///
    /// * `self` - Merged record.
    pub fn merge_with(mut self, other: Self) -> Self {
        let mut other = other;
        self.carrier_counts.append(&mut other.carrier_counts);
        self.carrier_counts.sort_by(|a, b| a.cohort.cmp(&b.cohort));
        self
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
    let cohort_name = if path_in_vcf.contains("non_neuro_controls") {
        "non_neuro_controls"
    } else if path_in_vcf.contains("non_neuro") {
        "non_neuro"
    } else {
        "all"
    };
    tracing::info!("importing gnomAD-CNV v4 {} cohort", cohort_name);

    let mut reader = noodles::vcf::io::reader::Builder::default().build_from_path(path_in_vcf)?;
    let header = reader.read_header()?;

    for result in reader.record_bufs(&header) {
        let vcf_record = result?;
        // TODO make sure this doesn't change anything
        let key = vcf_record.ids().as_ref().iter().join(",").into_bytes();

        // Build record for VCF record.
        let record = Record::from_vcf_record(&vcf_record, cohort_name)
            .map_err(|e| anyhow::anyhow!("problem building record from VCF: {}", e))?;

        // Attempt to read existing record from the database.
        let data = db
            .get_cf(cf_data, key.clone())
            .map_err(|e| anyhow::anyhow!("problem querying database: {}", e))?;
        let record = if let Some(data) = data {
            let db_record = Record::decode(&data[..])?;
            db_record.merge_with(record)
        } else {
            record
        };

        // Write back new or merged records.
        db.put_cf(cf_data, key, record.encode_to_vec())?;
    }

    Ok(())
}
