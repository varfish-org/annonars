//! Code generate for protobufs by `prost-build`.

use noodles::vcf::variant::record::AlternateBases;
use std::str::FromStr;

use noodles::vcf::variant::record_buf::info::field;

use crate::common;

include!(concat!(env!("OUT_DIR"), "/annonars.gnomad.gnomad3.rs"));
include!(concat!(
    env!("OUT_DIR"),
    "/annonars.gnomad.gnomad3.serde.rs"
));

/// The cohorts that are available in the gnomAD-exomes/genomes VCFs.
pub static COHORTS: &[&str] = &[
    "controls_and_biobanks",
    "non_cancer",
    "non_neuro",
    "non_topmed",
    "non_v2",
];

/// The populations that are available in the gnomAD-exomes/genomes VCFs.
///
/// Here, this excludes the "global" population represented by an empty string.
pub static POPS: &[&str] = &[
    "afr", "ami", "amr", "asj", "eas", "fin", "mid", "nfe", "oth", "sas",
];

/// Options struct that allows to specify which details fields are to be extracted from
/// gnomAD-exomes/genomes VCF records.
///
/// The fields that have `true` as its default are `vep`, `var_info`, and `pop_global_cohort`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DetailsOptions {
    /// Enable extraction of `Vep` records.
    pub vep: bool,
    /// Enable variant details info (v4: VRS_* fields).
    pub var_info: bool,
    /// Enable variant effetcs info.
    pub effect_info: bool,
    /// Enable extraction of sub populations in the "global" cohort (always done for v4).
    pub global_cohort_pops: bool,
    /// Enable extraction of all sub cohorts (requires `global_cohort_pops`; always done for v4).
    pub all_cohorts: bool,
    /// Enable extraction of detailed quality info.
    pub quality: bool,
    /// Enable extraction of detailed age info.
    pub age_hists: bool,
    /// Enable extraction of detailed depth of coverage info.
    pub depth_details: bool,
}

impl Default for DetailsOptions {
    fn default() -> Self {
        Self {
            vep: true,
            var_info: true,
            effect_info: true,
            global_cohort_pops: true,
            all_cohorts: false,
            quality: false,
            age_hists: false,
            depth_details: false,
        }
    }
}

impl DetailsOptions {
    /// Create a new `DetailsOptions` with all fields enabled.
    pub fn with_all_enabled() -> Self {
        Self {
            vep: true,
            var_info: true,
            effect_info: true,
            global_cohort_pops: true,
            all_cohorts: true,
            quality: true,
            age_hists: true,
            depth_details: true,
        }
    }
}

impl Record {
    /// Creates a new `Record` from a VCF record and allele number.
    pub fn from_vcf_allele(
        record: &noodles::vcf::variant::RecordBuf,
        allele_no: usize,
        options: &DetailsOptions,
    ) -> Result<Self, anyhow::Error> {
        assert!(allele_no == 0, "only allele 0 is supported");

        // Extract mandatory fields.
        let chrom = record.reference_sequence_name().to_string();
        let pos: usize = record
            .variant_start()
            .expect("Telomeric breakends not supported")
            .get();
        let pos = pos as i32;
        let ref_allele = record.reference_bases().to_string();
        let alt_allele = record
            .alternate_bases()
            .iter()
            .nth(allele_no)
            .ok_or_else(|| anyhow::anyhow!("no such allele: {}", allele_no))??
            .to_string();
        let filters = Self::extract_filters(record)?;
        let allele_counts = Self::extract_cohorts_allele_counts(record, options)?;
        let nonpar = common::noodles::get_flag(record, "nonpar")?;

        // Extract optional fields.
        let vep = options
            .vep
            .then(|| Self::extract_vep(record))
            .transpose()?
            .unwrap_or_default();
        let effect_info = options
            .effect_info
            .then(|| Self::extract_effect_info(record))
            .transpose()?;
        let variant_info = options
            .var_info
            .then(|| Self::extract_variant_info(record))
            .transpose()?;
        let quality_info = options
            .quality
            .then(|| Self::extract_quality(record))
            .transpose()?;
        let age_info = options
            .age_hists
            .then(|| Self::extract_age(record))
            .transpose()?;
        let depth_info = options
            .depth_details
            .then(|| Self::extract_depth(record))
            .transpose()?;

        Ok(Self {
            chrom,
            pos,
            ref_allele,
            alt_allele,
            filters,
            vep,
            allele_counts,
            effect_info,
            nonpar,
            variant_info,
            quality_info,
            age_info,
            depth_info,
        })
    }

    /// Extract the "vep" field into gnomAD v3 `Vep` records.
    pub(crate) fn extract_vep(
        record: &noodles::vcf::variant::RecordBuf,
    ) -> Result<Vec<super::vep_gnomad3::Vep>, anyhow::Error> {
        if let Some(Some(field::Value::Array(field::value::Array::String(v)))) =
            record.info().get("vep")
        {
            v.iter()
                .flat_map(|v| {
                    if let Some(s) = v.as_ref() {
                        if s.matches('|').count() + 1 == super::vep_gnomad3::Vep::num_fields() {
                            Some(super::vep_gnomad3::Vep::from_str(s))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect::<Result<Vec<_>, _>>()
        } else {
            anyhow::bail!("missing INFO/vep in gnomAD-nuclear record")
        }
    }

    /// Extract the details on the variant.
    pub(crate) fn extract_variant_info(
        record: &noodles::vcf::variant::RecordBuf,
    ) -> Result<VariantInfo, anyhow::Error> {
        Ok(VariantInfo {
            variant_type: common::noodles::get_string(record, "variant_type")?,
            allele_type: common::noodles::get_string(record, "allele_type")?,
            n_alt_alleles: common::noodles::get_i32(record, "n_alt_alleles")?,
            was_mixed: common::noodles::get_flag(record, "was_mixed")?,
            monoallelic: common::noodles::get_flag(record, "was_mixed")?,
            var_dp: common::noodles::get_i32(record, "n_alt_alleles")?,
            as_vardp: common::noodles::get_i32(record, "AS_VarDP").ok(),
        })
    }

    /// Extract details on the variant effects.
    pub(crate) fn extract_effect_info(
        record: &noodles::vcf::variant::RecordBuf,
    ) -> Result<EffectInfo, anyhow::Error> {
        Ok(EffectInfo {
            primate_ai_score: common::noodles::get_f32(record, "primate_ai_score").ok(),
            revel_score: common::noodles::get_f32(record, "revel_score").ok(),
            splice_ai_max_ds: common::noodles::get_f32(record, "splice_ai_max_ds").ok(),
            splice_ai_consequence: common::noodles::get_string(record, "splice_ai_consequence")
                .ok(),
            cadd_raw: common::noodles::get_f32(record, "cadd_raw").ok(),
            cadd_phred: common::noodles::get_f32(record, "cadd_phred").ok(),
        })
    }

    /// Extract the filters fields.
    pub(crate) fn extract_filters(
        record: &noodles::vcf::variant::RecordBuf,
    ) -> Result<Vec<i32>, anyhow::Error> {
        Ok(
            if let Some(Some(field::Value::Array(field::value::Array::String(value)))) =
                record.info().get("filters")
            {
                value
                    .iter()
                    .map(|v| match v.as_ref().map(|s| s.as_str()) {
                        Some("AC0") => Ok(Filter::AlleleCountIsZero as i32),
                        Some("InbreedingCoeff") | Some("inbreeding_coeff") => {
                            Ok(Filter::InbreedingCoeff as i32)
                        }
                        Some("PASS") => Ok(Filter::Pass as i32),
                        Some("AS_VQSR") => Ok(Filter::AsVsqr as i32),
                        Some(val) => anyhow::bail!("invalid filter value {}", val),
                        None => anyhow::bail!("missing filter value"),
                    })
                    .collect::<Result<Vec<_>, _>>()?
            } else {
                Vec::new()
            },
        )
    }

    /// Extract the age related fields from the VCF record.
    pub(crate) fn extract_age(
        record: &noodles::vcf::variant::RecordBuf,
    ) -> Result<AgeInfo, anyhow::Error> {
        Ok(AgeInfo {
            age_hist_hom_bin_freq: common::noodles::get_vec::<i32>(record, "age_hist_hom_bin_freq")
                .unwrap_or_default(),
            age_hist_hom_n_smaller: common::noodles::get_i32(record, "age_hist_hom_n_smaller").ok(),
            age_hist_hom_n_larger: common::noodles::get_i32(record, "age_hist_hom_n_larger").ok(),
            age_hist_het_bin_freq: common::noodles::get_vec::<i32>(record, "age_hist_het_bin_freq")
                .unwrap_or_default(),
            age_hist_het_n_smaller: common::noodles::get_i32(record, "age_hist_het_n_smaller").ok(),
            age_hist_het_n_larger: common::noodles::get_i32(record, "age_hist_het_n_larger").ok(),
        })
    }

    /// Extract the depth related fields from the VCF record.
    pub(crate) fn extract_depth(
        record: &noodles::vcf::variant::RecordBuf,
    ) -> Result<DepthInfo, anyhow::Error> {
        Ok(DepthInfo {
            dp_hist_all_n_larger: common::noodles::get_i32(record, "dp_hist_all_n_larger").ok(),
            dp_hist_alt_n_larger: common::noodles::get_i32(record, "dp_hist_alt_n_larger").ok(),
            dp_hist_all_bin_freq: common::noodles::get_vec::<i32>(record, "dp_hist_all_bin_freq")
                .unwrap_or_default(),
            dp_hist_alt_bin_freq: common::noodles::get_vec::<i32>(record, "dp_hist_alt_bin_freq")
                .unwrap_or_default(),
        })
    }

    /// Extract the quality-related fields from the VCF record.
    pub(crate) fn extract_quality(
        record: &noodles::vcf::variant::RecordBuf,
    ) -> Result<QualityInfo, anyhow::Error> {
        Ok(QualityInfo {
            as_fs: common::noodles::get_f32(record, "AS_FS").ok(),
            inbreeding_coeff: common::noodles::get_f32(record, "InbreedingCoeff").ok(),
            as_mq: common::noodles::get_f32(record, "AS_MQ").ok(),
            mq_rank_sum: common::noodles::get_f32(record, "MQRankSum").ok(),
            as_mq_rank_sum: common::noodles::get_f32(record, "AS_MQRankSum").ok(),
            as_qd: common::noodles::get_f32(record, "AS_QD").ok(),
            read_pos_rank_sum: common::noodles::get_f32(record, "ReadPosRankSum").ok(),
            as_read_pos_rank_sum: common::noodles::get_f32(record, "AS_ReadPosRankSum").ok(),
            as_sor: common::noodles::get_f32(record, "AS_SOR").ok(),
            positive_train_site: common::noodles::get_flag(record, "POSITIVE_TRAIN_SITE")? // < v4.0
            || common::noodles::get_flag(record, "positive_train_site")?, // >= v4.0
            negative_train_site: common::noodles::get_flag(record, "NEGATIVE_TRAIN_SITE")? // < v4.0
            || common::noodles::get_flag(record, "negative_train_site")?, // >= v4.0
            as_vqslod: common::noodles::get_f32(record, "AS_VQSLOD").ok(),
            as_culprit: common::noodles::get_string(record, "AS_culprit").ok(),
            segdup: common::noodles::get_flag(record, "seqdup")?,
            lcr: common::noodles::get_flag(record, "lcr")?,
            transmitted_singleton: common::noodles::get_flag(record, "transmitted_singleton")?,
            as_pab_max: common::noodles::get_f32(record, "AS_pab_max").ok(),
            as_qual_approx: common::noodles::get_i32(record, "AS_QUALapprox").ok(),
            as_sb_table: common::noodles::get_string(record, "AS_SB_TABLE").ok(),
            sor: common::noodles::get_f32(record, "SOR").ok(),
        })
    }

    /// Extract the allele counts from the `record` as configured in `options`.
    pub(crate) fn extract_cohorts_allele_counts(
        record: &noodles::vcf::variant::RecordBuf,
        options: &DetailsOptions,
    ) -> Result<Vec<CohortAlleleCounts>, anyhow::Error> {
        // Initialize global cohort.  We will always extract the non-population specific
        // counts for them.
        let mut global_counts = CohortAlleleCounts {
            cohort: None,
            by_sex: Some(AlleleCountsBySex {
                overall: Some(Self::extract_allele_counts(record, "", "")?),
                xx: Some(Self::extract_allele_counts(record, "", "_XX")?),
                xy: Some(Self::extract_allele_counts(record, "", "_XY")?),
            }),
            raw: Some(Self::extract_allele_counts(record, "", "_raw")?),
            popmax: common::noodles::get_string(record, "popmax").ok(),
            af_popmax: common::noodles::get_f32(record, "AF_popmax").ok(),
            ac_popmax: common::noodles::get_i32(record, "AC_popmax").ok(),
            an_popmax: common::noodles::get_i32(record, "AN_popmax").ok(),
            nhomalt_popmax: common::noodles::get_i32(record, "nhomalt_popmax").ok(),
            by_population: Vec::new(), // maybe filled below
        };

        // If configured to do so, extract the population specific counts.
        if options.global_cohort_pops {
            for pop in POPS {
                global_counts
                    .by_population
                    .push(Self::extract_population_allele_counts(record, "", pop)?);
            }
        }

        // If configured, extract all populations in all cohorts.
        let mut result = vec![global_counts];
        if options.all_cohorts {
            for cohort in COHORTS {
                let infix = format!("_{}", cohort);
                let mut cohort_counts = CohortAlleleCounts {
                    cohort: Some(cohort.to_string()),
                    by_sex: Some(AlleleCountsBySex {
                        overall: Some(Self::extract_allele_counts(record, &infix, "")?),
                        xx: Some(Self::extract_allele_counts(record, &infix, "_XX")?),
                        xy: Some(Self::extract_allele_counts(record, &infix, "_XY")?),
                    }),
                    raw: Some(Self::extract_allele_counts(record, &infix, "_raw")?),
                    popmax: common::noodles::get_string(record, &format!("{}_popmax", cohort)).ok(),
                    af_popmax: common::noodles::get_f32(record, &format!("AF_{}_popmax", cohort))
                        .ok(),
                    ac_popmax: common::noodles::get_i32(record, &format!("AC_{}_popmax", cohort))
                        .ok(),
                    an_popmax: common::noodles::get_i32(record, &format!("AN_{}_popmax", cohort))
                        .ok(),
                    nhomalt_popmax: common::noodles::get_i32(
                        record,
                        &format!("nhomalt_{}_popmax", cohort),
                    )
                    .ok(),
                    by_population: Vec::new(), // to be filled below
                };

                for pop in POPS {
                    cohort_counts
                        .by_population
                        .push(Self::extract_population_allele_counts(record, &infix, pop)?);
                }

                result.push(cohort_counts);
            }
        }

        Ok(result)
    }

    /// Extrac the population allele counts from the `record`.
    pub(crate) fn extract_population_allele_counts(
        record: &noodles::vcf::variant::RecordBuf,
        infix: &str,
        pop: &str,
    ) -> Result<PopulationAlleleCounts, anyhow::Error> {
        Ok(PopulationAlleleCounts {
            population: pop.to_string(),
            counts: Some(AlleleCountsBySex {
                overall: Some(Self::extract_allele_counts(
                    record,
                    infix,
                    &format!("_{}", pop),
                )?),
                xx: Some(Self::extract_allele_counts(
                    record,
                    infix,
                    &format!("_{}_XX", pop),
                )?),
                xy: Some(Self::extract_allele_counts(
                    record,
                    infix,
                    &format!("_{}_XY", pop),
                )?),
            }),
            // The faf95 and faf99 value is not present for all populations.  We use a blanket
            // "ok()" here so things don't blow up randomly.
            faf95: common::noodles::get_f32(record, &format!("faf95_{}", pop)).ok(),
            faf99: common::noodles::get_f32(record, &format!("faf99_{}", pop)).ok(),
            faf95_xx: common::noodles::get_f32(record, &format!("faf95_{}_XX", pop)).ok(),
            faf99_xx: common::noodles::get_f32(record, &format!("faf99_{}_XX", pop)).ok(),
            faf95_xy: common::noodles::get_f32(record, &format!("faf95_{}_XY", pop)).ok(),
            faf99_xy: common::noodles::get_f32(record, &format!("faf99_{}_XY", pop)).ok(),
        })
    }

    /// Extract the allele counts from the `record` with the given infix and suffix.
    pub(crate) fn extract_allele_counts(
        record: &noodles::vcf::variant::RecordBuf,
        infix: &str,
        suffix: &str,
    ) -> Result<AlleleCounts, anyhow::Error> {
        Ok(AlleleCounts {
            ac: common::noodles::get_i32(record, &format!("AC{}{}", infix, suffix))
                .unwrap_or_default(),
            an: common::noodles::get_i32(record, &format!("AN{}{}", infix, suffix))
                .unwrap_or_default(),
            nhomalt: common::noodles::get_i32(record, &format!("nhomalt{}{}", infix, suffix))
                .unwrap_or_default(),
            af: common::noodles::get_f32(record, &format!("AF{}{}", infix, suffix))
                .unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_record_from_vcf_allele_gnomad_genomes_grch38() -> Result<(), anyhow::Error> {
        let path_vcf = "tests/gnomad-nuclear/example-genomes-grch38/v3.1/gnomad-genomes.vcf";
        let mut reader_vcf =
            noodles::vcf::io::reader::Builder::default().build_from_path(path_vcf)?;
        let header = reader_vcf.read_header()?;

        let mut records = Vec::new();
        for row in reader_vcf.record_bufs(&header) {
            let vcf_record = row?;
            let record =
                Record::from_vcf_allele(&vcf_record, 0, &DetailsOptions::with_all_enabled())?;
            records.push(record);
        }

        insta::assert_yaml_snapshot!(records);

        Ok(())
    }
}
