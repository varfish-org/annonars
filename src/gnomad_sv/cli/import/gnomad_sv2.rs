//! gnomAD SV v2 import.
//!
//! Note that gnomAD v2 did not have distinction between different cohorts within
//! one file.  Rather, there is one file for each cohort (all, controls, non-neuro).

use std::{str::FromStr, sync::Arc};

use crate::{
    common::noodles::{get_f32, get_i32, get_string},
    pbs::gnomad::gnomad_sv2::{
        AlleleCounts, AlleleCountsBySex, CohortAlleleCounts, CpxType, Filter, Population,
        PopulationAlleleCounts, Record, SvType,
    },
};

use prost::Message;

impl FromStr for Filter {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "LOW_CALL_RATE" => Filter::LowCallRate,
            "MULTIALLELIC" => Filter::LowCallRate,
            "PASS" => Filter::Pass,
            "PCRPLUS_ENRICHED" => Filter::PcrplusEnriched,
            "UNRESOLVED" => Filter::Unresolved,
            "UNSTABLE_AF_PCRMINUS" => Filter::UnstableAfPcrminus,
            _ => anyhow::bail!("unknown FILTER: {}", s),
        })
    }
}

impl FromStr for Population {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "AFR" => Population::Afr,
            "AMR" => Population::Amr,
            "EAS" => Population::Eas,
            "EUR" => Population::Eur,
            "OTH" => Population::Other,
            _ => anyhow::bail!("unknown population: {}", s),
        })
    }
}

impl ToString for Population {
    fn to_string(&self) -> String {
        match self {
            Population::Afr => "AFR",
            Population::Amr => "AMR",
            Population::Eas => "EAS",
            Population::Eur => "EUR",
            Population::Other => "OTH",
            _ => unreachable!("unknown population: {:?}", self),
        }
        .to_string()
    }
}

impl FromStr for SvType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "BND" => SvType::Bnd,
            "CPX" => SvType::Cpx,
            "CTX" => SvType::Ctx,
            "DEL" => SvType::Del,
            "DUP" => SvType::Dup,
            "INS" => SvType::Ins,
            "INV" => SvType::Inv,
            "MCNV" => SvType::Mcnv,
            _ => anyhow::bail!("unknown SVTYPE: {}", s),
        })
    }
}

impl FromStr for CpxType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "CCR" => CpxType::Ccr,
            "dDUP" => CpxType::Ddup,
            "dDUP_iDEL" => CpxType::DdupIdel,
            "delINV" => CpxType::DelInv,
            "delINVdel" => CpxType::DelInvDel,
            "delINVdup" => CpxType::DelInvDup,
            "dupINV" => CpxType::DupInv,
            "dupINVdel" => CpxType::DupInvDel,
            "dupINVdup" => CpxType::DupInvDup,
            "INS_iDEL" => CpxType::InsIdel,
            "INVdel" => CpxType::InvDel,
            "INVdup" => CpxType::InvDup,
            _ => anyhow::bail!("unknown CPX_TYPE: {}", s),
        })
    }
}

impl Record {
    /// Create new Record from VCF record.
    ///
    /// Note that we do not handle the MCNV fields very sensibly as we count all
    /// alternate alleles as alternates.  This means that 100% of the samples
    /// are considered variant at this site but in the context of identifying
    /// benign variants, this is not a problem as these sites are copy number
    /// variable.
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
        record: &noodles_vcf::Record,
        cohort_name: &str,
    ) -> Result<Self, anyhow::Error> {
        let chrom = record.chromosome().to_string();
        let pos: usize = record.position().into();
        let pos = pos as i32;
        let end = get_i32(record, "END").ok();
        let chrom2 = get_string(record, "CHROM2").ok();
        let end2 = get_i32(record, "END2").ok();
        let id = record
            .ids()
            .iter()
            .next()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("no ID found in VCF record"))?;
        let filters = record
            .filters()
            .map(|f| -> Result<_, anyhow::Error> {
                use noodles_vcf::record::Filters::*;
                Ok(match f {
                    Pass => vec![Filter::Pass as i32],
                    Fail(f) => {
                        let mut result = f
                            .iter()
                            .map(|s| s.parse::<Filter>().map(|f| f as i32))
                            .collect::<Result<Vec<_>, _>>()
                            .map_err(|e| anyhow::anyhow!("problem parsing FILTER: {}", e))?;
                        result.sort();
                        result
                    }
                })
            })
            .transpose()?
            .unwrap_or_else(|| vec![Filter::Pass as i32]);
        let sv_type = get_string(record, "SVTYPE")?
            .parse::<SvType>()
            .map(|x| x as i32)?;
        let cpx_type = get_string(record, "CPX_TYPE")
            .ok()
            .map(|s| s.parse::<CpxType>().map(|x| x as i32))
            .transpose()?;
        let allele_counts = vec![Self::allele_counts_from_vcf_record(record, cohort_name)?];

        Ok(Self {
            chrom,
            pos,
            end,
            chrom2,
            end2,
            id,
            filters,
            sv_type,
            cpx_type,
            allele_counts,
        })
    }

    /// Extract allele counts from VCF record.
    fn allele_counts_from_vcf_record(
        record: &noodles_vcf::Record,
        cohort_name: &str,
    ) -> Result<CohortAlleleCounts, anyhow::Error> {
        let cohort = if cohort_name == "all" {
            None
        } else {
            Some(cohort_name.to_string())
        };

        let by_sex = AlleleCountsBySex {
            overall: Self::extract_allele_counts(record, "", "").ok(),
            xx: Self::extract_allele_counts(record, "FEMALE_", "").ok(),
            xy: Self::extract_allele_counts(record, "MALE_", "").ok(),
        };
        let by_sex = if by_sex.overall.is_some() || by_sex.xx.is_some() || by_sex.xy.is_some() {
            Some(by_sex)
        } else {
            None
        };

        let mut by_population = Vec::new();
        for pop in [
            Population::Afr,
            Population::Amr,
            Population::Eas,
            Population::Eur,
            Population::Other,
        ] {
            by_population.push(Self::extract_population_allele_counts(record, pop)?);
        }

        Ok(CohortAlleleCounts {
            cohort,
            by_sex,
            by_population,
        })
    }

    /// Extract poulation allele counts.
    fn extract_population_allele_counts(
        record: &noodles_vcf::Record,
        population: Population,
    ) -> Result<PopulationAlleleCounts, anyhow::Error> {
        let pop_str = population.to_string();
        let counts = AlleleCountsBySex {
            overall: Self::extract_allele_counts(record, "", &pop_str).ok(),
            xx: Self::extract_allele_counts(record, "FEMALE_", &pop_str).ok(),
            xy: Self::extract_allele_counts(record, "MALE_", &pop_str).ok(),
        };
        let counts = if counts.overall.is_some() && counts.xx.is_some() && counts.xy.is_some() {
            Some(counts)
        } else {
            None
        };
        Ok(PopulationAlleleCounts {
            population: population as i32,
            counts,
        })
    }

    /// Extract allele counts for a given population from VCF record.
    fn extract_allele_counts(
        record: &noodles_vcf::Record,
        prefix: &str,
        population: &str,
    ) -> Result<AlleleCounts, anyhow::Error> {
        let key = |name| -> String {
            if !population.is_empty() {
                format!("{}_{}{}", population, prefix, name)
            } else {
                format!("{}{}", prefix, name)
            }
        };

        let ac = get_i32(record, &key("AC")).unwrap_or_default();
        let an = get_i32(record, &key("AN")).unwrap_or_default();
        let n_bi_genos = get_i32(record, &key("N_BI_GENOS")).unwrap_or_default();
        let n_homref = get_i32(record, &key("N_HOMREF")).unwrap_or_default();
        let n_het = get_i32(record, &key("N_HET")).unwrap_or_default();
        let n_homalt = get_i32(record, &key("N_HOMALT")).unwrap_or_default();
        let af = get_f32(record, &key("AF")).unwrap_or_default();
        let freq_homref = get_f32(record, &key("FREQ_HOMREF")).unwrap_or_default();
        let freq_het = get_f32(record, &key("FREQ_HET")).unwrap_or_default();
        let freq_homalt = get_f32(record, &key("FREQ_HOMALT")).unwrap_or_default();

        Ok(AlleleCounts {
            ac,
            an,
            af,
            n_bi_genos,
            n_homref,
            n_het,
            n_homalt,
            freq_homref,
            freq_het,
            freq_homalt,
        })
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
        self.allele_counts.append(&mut other.allele_counts);
        self.allele_counts.sort_by(|a, b| a.cohort.cmp(&b.cohort));
        self
    }
}

/// Perform import of gnomAD-SV CNV data.
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
    let cohort_name = if path_in_vcf.contains("controls") {
        "controls"
    } else if path_in_vcf.contains("nonneuro") {
        "non_neuro"
    } else {
        "all"
    };
    tracing::info!("importing gnomAD-SV v2 {} cohort", cohort_name);

    let mut reader = noodles_vcf::reader::Builder::default().build_from_path(path_in_vcf)?;
    let header = reader.read_header()?;

    for result in reader.records(&header) {
        let vcf_record = result?;
        let key = format!("{}", vcf_record.ids()).into_bytes();

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
