//! Code generate for protobufs by `prost-build`.

use std::str::FromStr;

use noodles_vcf::record::info::field;

use crate::common;

include!(concat!(env!("OUT_DIR"), "/annonars.gnomad.gnomad2.rs"));
include!(concat!(
    env!("OUT_DIR"),
    "/annonars.gnomad.gnomad2.serde.rs"
));

/// The cohorts that are available in the gnomAD-exomes/genomes VCFs.
pub static COHORTS: &[&str] = &["controls", "non_cancer", "non_neuro", "non_topmed"];

/// The populations that are available in the gnomAD-exomes/genomes VCFs.
///
/// This includes the "global" population represented by an empty string.
pub static POPS: &[&str] = &[
    "afr", "amr", "asj", "eas", "eas_jpn", "eas_oea", "eas_kor", "fin", "nfe", "nfe_bgr",
    "nfe_est", "nfe_nwe", "nfe_onf", "nfe_seu", "nfe_swe", "oth", "sas",
];

/// Options struct that allows to specify which details fields are to be extracted from
/// gnomAD-exomes/genomes VCF records.
///
/// The fields that have `true` as its default are `vep`, `var_info`, and `pop_global_cohort`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DetailsOptions {
    /// Enable extraction of `Vep` records.
    pub vep: bool,
    /// Enable variant details info.
    pub var_info: bool,
    /// Enable extraction of sub populations in the "global" cohort.
    pub global_cohort_pops: bool,
    /// Enable extraction of all sub cohorts (requires `pop_global_cohorts`).
    pub all_cohorts: bool,
    /// Enable extraction of detailed random forest info.
    pub rf_info: bool,
    /// Enable extraction of detailed quality info.
    pub quality: bool,
    /// Enable extraction of detailed age info.
    pub age_hists: bool,
    /// Enable extraction of detailed depth of coverage info.
    pub depth_details: bool,
    /// Enable extraction of liftover info.
    pub liftover: bool,
}

impl Default for DetailsOptions {
    fn default() -> Self {
        Self {
            vep: true,
            var_info: true,
            global_cohort_pops: true,
            all_cohorts: false,
            rf_info: false,
            quality: false,
            age_hists: false,
            depth_details: false,
            liftover: false,
        }
    }
}

impl DetailsOptions {
    /// Create a new `DetailsOptions` with all fields enabled.
    pub fn with_all_enabled() -> Self {
        Self {
            vep: true,
            var_info: true,
            global_cohort_pops: true,
            all_cohorts: true,
            rf_info: true,
            quality: true,
            age_hists: true,
            depth_details: true,
            liftover: true,
        }
    }
}

impl Record {
    /// Creates a new `Record` from a VCF record and allele number.
    pub fn from_vcf_allele(
        record: &noodles_vcf::record::Record,
        allele_no: usize,
        options: &DetailsOptions,
    ) -> Result<Self, anyhow::Error> {
        assert!(allele_no == 0, "only allele 0 is supported");

        assert!(allele_no == 0, "only allele 0 is supported");

        // Extract mandatory fields.
        let chrom = record.chromosome().to_string();
        let pos: usize = record.position().into();
        let pos = pos as i32;
        let ref_allele = record.reference_bases().to_string();
        let alt_allele = record
            .alternate_bases()
            .get(allele_no)
            .ok_or_else(|| anyhow::anyhow!("no such allele: {}", allele_no))?
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
        let rf_info = options
            .rf_info
            .then(|| Self::extract_rf_info(record))
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
        let liftover_info = options
            .liftover
            .then(|| Self::extract_liftover(record))
            .transpose()?
            .flatten();

        Ok(Self {
            chrom,
            pos,
            ref_allele,
            alt_allele,
            filters,
            vep,
            allele_counts,
            liftover_info,
            nonpar,
            rf_info,
            variant_info,
            quality_info,
            age_info,
            depth_info,
        })
    }

    /// Extract the "vep" field into gnomAD v2 `Vep` records.
    fn extract_vep(
        record: &noodles_vcf::Record,
    ) -> Result<Vec<super::vep_gnomad2::Vep>, anyhow::Error> {
        if let Some(Some(field::Value::Array(field::value::Array::String(v)))) =
            record.info().get(&field::Key::from_str("vep")?)
        {
            v.iter()
                .flat_map(|v| {
                    if let Some(s) = v.as_ref() {
                        if s.matches('|').count() + 1 == super::vep_gnomad2::Vep::num_fields() {
                            Some(super::vep_gnomad2::Vep::from_str(s))
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

    /// Extract the liftover related fields into gnomAD v2 `Vep` records.
    fn extract_liftover(
        record: &noodles_vcf::Record,
    ) -> Result<Option<LiftoverInfo>, anyhow::Error> {
        let tmp = LiftoverInfo {
            reverse_complemented_alleles: common::noodles::get_flag(
                record,
                "ReverseComplementedAlleles",
            )?,
            swapped_alleles: common::noodles::get_flag(record, "SwappedAlleles")?,
            original_alleles: common::noodles::get_vec_str(record, "OriginalAlleles")
                .unwrap_or_default(),
            original_contig: common::noodles::get_string(record, "OriginalContig").ok(),
            original_start: common::noodles::get_string(record, "OriginalStart").ok(),
        };
        if tmp.reverse_complemented_alleles
            || tmp.swapped_alleles
            || !tmp.original_alleles.is_empty()
            || tmp.original_contig.is_some()
            || tmp.original_start.is_some()
        {
            Ok(Some(tmp))
        } else {
            Ok(None)
        }
    }

    /// Extract the details on the random forest.
    fn extract_rf_info(record: &noodles_vcf::Record) -> Result<RandomForestInfo, anyhow::Error> {
        Ok(RandomForestInfo {
            rf_tp_probability: common::noodles::get_f32(record, "rf_tp_probability")?,
            rf_positive_label: common::noodles::get_flag(record, "rf_positive_label")?,
            rf_negative_label: common::noodles::get_flag(record, "rf_negative_label")?,
            rf_label: common::noodles::get_string(record, "rf_label").ok(),
            rf_train: common::noodles::get_flag(record, "rf_train")?,
        })
    }

    /// Extract the details on the variant.
    fn extract_variant_info(record: &noodles_vcf::Record) -> Result<VariantInfo, anyhow::Error> {
        Ok(VariantInfo {
            variant_type: common::noodles::get_string(record, "variant_type")?,
            allele_type: common::noodles::get_string(record, "allele_type")?,
            n_alt_alleles: common::noodles::get_i32(record, "n_alt_alleles")?,
            was_mixed: common::noodles::get_flag(record, "was_mixed")?,
            has_star: common::noodles::get_flag(record, "has_star")?,
        })
    }

    /// Extract the filters fields.
    fn extract_filters(record: &noodles_vcf::Record) -> Result<Vec<i32>, anyhow::Error> {
        Ok(
            if let Some(Some(field::Value::Array(field::value::Array::String(value)))) =
                record.info().get(&field::Key::from_str("filters")?)
            {
                value
                    .iter()
                    .map(|v| match v.as_ref().map(|s| s.as_str()) {
                        Some("AC0") => Ok(Filter::AlleleCountIsZero as i32),
                        Some("InbreedingCoeff") => Ok(Filter::InbreedingCoeff as i32),
                        Some("PASS") => Ok(Filter::Pass as i32),
                        Some("RF") => Ok(Filter::RandomForest as i32),
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
    fn extract_age(record: &noodles_vcf::record::Record) -> Result<AgeInfo, anyhow::Error> {
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
    fn extract_depth(record: &noodles_vcf::record::Record) -> Result<DepthInfo, anyhow::Error> {
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
    fn extract_quality(record: &noodles_vcf::record::Record) -> Result<QualityInfo, anyhow::Error> {
        Ok(QualityInfo {
            fs: common::noodles::get_f32(record, "FS").ok(),
            inbreeding_coeff: common::noodles::get_f32(record, "InbreedingCoeff").ok(),
            mq: common::noodles::get_f32(record, "MQ").ok(),
            mq_rank_sum: common::noodles::get_f32(record, "MQRankSum").ok(),
            qd: common::noodles::get_f32(record, "QD").ok(),
            read_pos_rank_sum: common::noodles::get_f32(record, "ReadPosRankSum").ok(),
            sor: common::noodles::get_f32(record, "SOR").ok(),
            vqsr_positive_train_site: common::noodles::get_flag(
                record,
                "VQSR_POSITIVE_TRAIN_SITE",
            )?,
            vqsr_negative_train_site: common::noodles::get_flag(
                record,
                "VQSR_NEGATIVE_TRAIN_SITE",
            )?,
            base_q_rank_sum: common::noodles::get_f32(record, "BaseQRankSum").ok(),
            clipping_rank_sum: common::noodles::get_f32(record, "ClippingRankSum").ok(),
            dp: common::noodles::get_i32(record, "DP").ok(),
            vqslod: common::noodles::get_f32(record, "VQSLOD").ok(),
            vqsr_culprit: common::noodles::get_string(record, "VQSR_culprit").ok(),
            segdup: common::noodles::get_flag(record, "segdup")?,
            lcr: common::noodles::get_flag(record, "lcr")?,
            decoy: common::noodles::get_flag(record, "decoy")?,
            transmitted_singleton: common::noodles::get_flag(record, "transmitted_singleton")?,
            pab_max: common::noodles::get_f32(record, "pab_max").ok(),
        })
    }

    /// Extract the allele counts from the `record` as configured in `options`.
    fn extract_cohorts_allele_counts(
        record: &noodles_vcf::Record,
        options: &DetailsOptions,
    ) -> Result<Vec<CohortAlleleCounts>, anyhow::Error> {
        // Initialize global cohort.  We will always extract the non-population specific
        // counts for them.
        let mut global_counts = CohortAlleleCounts {
            cohort: None,
            by_sex: Some(AlleleCountsBySex {
                overall: Self::extract_allele_counts(record, "", "")?,
                xx: Self::extract_allele_counts(record, "", "_female")?,
                xy: Self::extract_allele_counts(record, "", "_male")?,
            }),
            raw: Self::extract_allele_counts(record, "", "_raw")?,
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
                let prefix = format!("{}_", cohort);
                let mut cohort_counts = CohortAlleleCounts {
                    cohort: Some(cohort.to_string()),
                    by_sex: Some(AlleleCountsBySex {
                        overall: Self::extract_allele_counts(record, &prefix, "")?,
                        xx: Self::extract_allele_counts(record, &prefix, "_female")?,
                        xy: Self::extract_allele_counts(record, &prefix, "_male")?,
                    }),
                    raw: Self::extract_allele_counts(record, &prefix, "_raw")?,
                    popmax: common::noodles::get_string(record, &format!("{}_popmax", cohort)).ok(),
                    af_popmax: common::noodles::get_f32(record, &format!("{}_AF_popmax", cohort))
                        .ok(),
                    ac_popmax: common::noodles::get_i32(record, &format!("{}_AC_popmax", cohort))
                        .ok(),
                    an_popmax: common::noodles::get_i32(record, &format!("{}_AN_popmax", cohort))
                        .ok(),
                    nhomalt_popmax: common::noodles::get_i32(
                        record,
                        &format!("{}_nhomalt_popmax", cohort),
                    )
                    .ok(),
                    by_population: Vec::new(), // to be filled below
                };

                for pop in POPS {
                    cohort_counts
                        .by_population
                        .push(Self::extract_population_allele_counts(
                            record, &prefix, pop,
                        )?);
                }

                result.push(cohort_counts);
            }
        }

        Ok(result)
    }

    /// Extrac the population allele counts from the `record`.
    fn extract_population_allele_counts(
        record: &noodles_vcf::Record,
        prefix: &str,
        pop: &str,
    ) -> Result<PopulationAlleleCounts, anyhow::Error> {
        Ok(PopulationAlleleCounts {
            population: pop.to_string(),
            counts: Some(AlleleCountsBySex {
                overall: Self::extract_allele_counts(record, prefix, &format!("_{}", pop))?,
                xx: Self::extract_allele_counts(record, prefix, &format!("_{}{}", pop, "_female"))?,
                xy: Self::extract_allele_counts(record, prefix, &format!("_{}{}", pop, "_male"))?,
            }),
            // The faf95 and faf99 value is not present for all populations.  We use a blanket
            // "ok()" here so things don't blow up randomly.
            faf95: common::noodles::get_f32(record, &format!("faf95_{}", pop)).ok(),
            faf99: common::noodles::get_f32(record, &format!("faf99_{}", pop)).ok(),
        })
    }

    /// Extract the allele counts from the `record` with the given prefix and suffix.
    fn extract_allele_counts(
        record: &noodles_vcf::Record,
        prefix: &str,
        suffix: &str,
    ) -> Result<Option<AlleleCounts>, anyhow::Error> {
        if common::noodles::get_i32(record, &format!("{}AN{}", prefix, suffix)).unwrap_or_default()
            == 0
        {
            Ok(None)
        } else {
            Ok(Some(AlleleCounts {
                ac: common::noodles::get_i32(record, &format!("{}AC{}", prefix, suffix))
                    .unwrap_or_default(),
                an: common::noodles::get_i32(record, &format!("{}AN{}", prefix, suffix))
                    .unwrap_or_default(),
                nhomalt: common::noodles::get_i32(record, &format!("{}nhomalt{}", prefix, suffix))
                    .unwrap_or_default(),
                af: common::noodles::get_f32(record, &format!("{}AF{}", prefix, suffix))
                    .unwrap_or_default(),
            }))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_record_from_vcf_allele_gnomad_exomes_grch37() -> Result<(), anyhow::Error> {
        let path_vcf = "tests/gnomad-nuclear/example-exomes-grch37/v2.1/gnomad-exomes.vcf";
        let mut reader_vcf = noodles_vcf::reader::Builder::default().build_from_path(path_vcf)?;
        let header = reader_vcf.read_header()?;

        let mut records = Vec::new();
        for row in reader_vcf.records(&header) {
            let vcf_record = row?;
            let record =
                Record::from_vcf_allele(&vcf_record, 0, &DetailsOptions::with_all_enabled())?;
            records.push(record);
        }

        insta::assert_yaml_snapshot!(records);

        Ok(())
    }

    #[test]
    fn test_record_from_vcf_allele_gnomad_genomes_grch37() -> Result<(), anyhow::Error> {
        let path_vcf = "tests/gnomad-nuclear/example-genomes-grch37/v2.1/gnomad-genomes.vcf";
        let mut reader_vcf = noodles_vcf::reader::Builder::default().build_from_path(path_vcf)?;
        let header = reader_vcf.read_header()?;

        let mut records = Vec::new();
        for row in reader_vcf.records(&header) {
            let vcf_record = row?;
            let record =
                Record::from_vcf_allele(&vcf_record, 0, &DetailsOptions::with_all_enabled())?;
            records.push(record);
        }

        insta::assert_yaml_snapshot!(records);

        Ok(())
    }

    #[test]
    fn test_record_from_vcf_allele_gnomad_exomes_grch38() -> Result<(), anyhow::Error> {
        let path_vcf = "tests/gnomad-nuclear/example-exomes-grch38/v2.1/gnomad-exomes.vcf";
        let mut reader_vcf = noodles_vcf::reader::Builder::default().build_from_path(path_vcf)?;
        let header = reader_vcf.read_header()?;

        let mut records = Vec::new();
        for row in reader_vcf.records(&header) {
            let vcf_record = row?;
            let record =
                Record::from_vcf_allele(&vcf_record, 0, &DetailsOptions::with_all_enabled())?;
            records.push(record);
        }

        insta::assert_yaml_snapshot!(records);

        Ok(())
    }
}
