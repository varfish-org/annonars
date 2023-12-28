//! Code generate for protobufs by `prost-build`.

use std::str::FromStr;

use noodles_vcf::record::info::field;

use super::gnomad3;
use crate::common;

include!(concat!(env!("OUT_DIR"), "/annonars.gnomad.gnomad4.rs"));
include!(concat!(
    env!("OUT_DIR"),
    "/annonars.gnomad.gnomad4.serde.rs"
));

/// The cohorts that are available in the gnomAD-genomes v4.0 VCFs.
pub static COHORTS: &[&str] = &["joint"];

/// The ancestry groups that are available in the gnomAD-genomes v4.0 VCFs.
///
/// Here, this excludes the "global" group represented by an empty string.
pub static GRPS: &[&str] = &[
    "afr",
    "ami",
    "amr",
    "asj",
    "eas",
    "fin",
    "mid",
    "nfe",
    "remaining",
    "sas",
];

impl Record {
    /// Creates a new `Record` from a VCF record and allele number.
    pub fn from_vcf_allele(
        record: &noodles_vcf::record::Record,
        allele_no: usize,
        options: &gnomad3::DetailsOptions,
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
        let filters = gnomad3::Record::extract_filters(record)?;
        let allele_counts = Self::extract_cohorts_allele_counts(record)?;
        let nonpar = common::noodles::get_flag(record, "non_par")?;

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
            .then(|| gnomad3::Record::extract_variant_info(record))
            .transpose()?;
        let quality_info = options
            .quality
            .then(|| gnomad3::Record::extract_quality(record))
            .transpose()?;
        let age_info = options
            .age_hists
            .then(|| gnomad3::Record::extract_age(record))
            .transpose()?;
        let depth_info = options
            .depth_details
            .then(|| gnomad3::Record::extract_depth(record))
            .transpose()?;
        let vrs_info = options
            .var_info
            .then(|| Self::extract_vrs_info(record))
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
            vrs_info,
        })
    }

    /// Extract the "vep" field into gnomAD v3 `Vep` records.
    fn extract_vep(
        record: &noodles_vcf::Record,
    ) -> Result<Vec<super::vep_gnomad4::Vep>, anyhow::Error> {
        if let Some(Some(field::Value::Array(field::value::Array::String(v)))) =
            record.info().get(&field::Key::from_str("vep")?)
        {
            v.iter()
                .flat_map(|v| {
                    if let Some(s) = v.as_ref() {
                        if s.matches('|').count() + 1 == super::vep_gnomad4::Vep::num_fields() {
                            Some(super::vep_gnomad4::Vep::from_str(s))
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

    /// Extract the VRS infos.
    fn extract_vrs_info(record: &noodles_vcf::Record) -> Result<VrsInfo, anyhow::Error> {
        Ok(VrsInfo {
            allele_ids: common::noodles::get_vec_str(record, "VRS_Allele_IDs").unwrap_or_default(),
            ends: common::noodles::get_vec_i32(record, "VRS_Ends").unwrap_or_default(),
            starts: common::noodles::get_vec_i32(record, "VRS_Starts").unwrap_or_default(),
            states: common::noodles::get_vec_str(record, "VRS_States").unwrap_or_default(),
        })
    }

    /// Extract details on the variant effects.
    fn extract_effect_info(record: &noodles_vcf::Record) -> Result<EffectInfo, anyhow::Error> {
        Ok(EffectInfo {
            pangolin_largest_ds: common::noodles::get_f32(record, "pangolin_largest_ds").ok(),
            phylop: common::noodles::get_f32(record, "phylop").ok(),
            polyphen_max: common::noodles::get_f32(record, "polyphen_max").ok(),
            revel_max: common::noodles::get_f32(record, "revel_max").ok(),
            sift_max: common::noodles::get_f32(record, "sift_max").ok(),
            spliceai_ds_max: common::noodles::get_f32(record, "spliceai_ds_max").ok(),
            cadd_raw: common::noodles::get_f32(record, "cadd_raw").ok(),
            cadd_phred: common::noodles::get_f32(record, "cadd_phred").ok(),
        })
    }

    /// Extract the allele counts from the `record` as configured in `options`.
    fn extract_cohorts_allele_counts(
        record: &noodles_vcf::Record,
    ) -> Result<Vec<CohortAlleleCounts>, anyhow::Error> {
        // Initialize global cohort.
        let mut global_counts = CohortAlleleCounts {
            cohort: None,
            by_sex: Some(gnomad3::AlleleCountsBySex {
                overall: Some(Self::extract_allele_counts(record, "", "")?),
                xx: Some(Self::extract_allele_counts(record, "", "_XX")?),
                xy: Some(Self::extract_allele_counts(record, "", "_XY")?),
            }),
            raw: Some(Self::extract_allele_counts(record, "", "_raw")?),
            grpmax: common::noodles::get_string(record, "grpmax").ok(),
            af_grpmax: common::noodles::get_f32(record, "AF_grpmax").ok(),
            ac_grpmax: common::noodles::get_i32(record, "AC_grpmax").ok(),
            an_grpmax: common::noodles::get_i32(record, "AN_grpmax").ok(),
            nhomalt_grpmax: common::noodles::get_i32(record, "nhomalt_grpmax").ok(),
            by_ancestry_group: Vec::new(), // to be filled below
        };

        // Always extract the ancestry group specific counts for v4.
        for pop in GRPS {
            global_counts
                .by_ancestry_group
                .push(Self::extract_ancestry_group_allele_counts(record, "", pop)?);
        }

        // Always extract all ancestry groups in all cohorts for v4.
        let mut result = Vec::new();
        for cohort in COHORTS {
            let infix = format!("_{}", cohort);
            let mut cohort_counts = CohortAlleleCounts {
                cohort: Some(cohort.to_string()),
                by_sex: Some(gnomad3::AlleleCountsBySex {
                    overall: Some(Self::extract_allele_counts(record, &infix, "")?),
                    xx: Some(Self::extract_allele_counts(record, &infix, "_XX")?),
                    xy: Some(Self::extract_allele_counts(record, &infix, "_XY")?),
                }),
                raw: Some(Self::extract_allele_counts(record, &infix, "_raw")?),
                grpmax: common::noodles::get_string(record, &format!("grpmax_{}", cohort)).ok(),
                af_grpmax: common::noodles::get_f32(record, &format!("AF_grpmax_{}", cohort)).ok(),
                ac_grpmax: common::noodles::get_i32(record, &format!("AC_grpmax_{}", cohort)).ok(),
                an_grpmax: common::noodles::get_i32(record, &format!("AN_grpmax_{}", cohort)).ok(),
                nhomalt_grpmax: common::noodles::get_i32(
                    record,
                    &format!("nhomalt_grpmax_{}", cohort),
                )
                .ok(),
                by_ancestry_group: Vec::new(), // to be filled below
            };

            for pop in GRPS {
                cohort_counts
                    .by_ancestry_group
                    .push(Self::extract_ancestry_group_allele_counts(
                        record, &infix, pop,
                    )?);
            }

            result.push(cohort_counts);
        }
        // For gnomAD v4, the "joint" cohort comes first and the global/empty-string cohort second.
        result.insert(1, global_counts);

        Ok(result)
    }

    /// Extrac the ancestry group allele counts from the `record`.
    fn extract_ancestry_group_allele_counts(
        record: &noodles_vcf::Record,
        infix: &str,
        grp: &str,
    ) -> Result<AncestryGroupAlleleCounts, anyhow::Error> {
        Ok(AncestryGroupAlleleCounts {
            ancestry_group: grp.to_string(),
            counts: Some(gnomad3::AlleleCountsBySex {
                overall: Some(Self::extract_allele_counts(
                    record,
                    infix,
                    &format!("_{}", grp),
                )?),
                xx: Some(Self::extract_allele_counts(
                    record,
                    infix,
                    &format!("_{}_XX", grp),
                )?),
                xy: Some(Self::extract_allele_counts(
                    record,
                    infix,
                    &format!("_{}_XY", grp),
                )?),
            }),
            // The faf95 and faf99 value is not present for all ancestry groups.  We use a blanket
            // "ok()" here so things don't blow up randomly.
            faf95: common::noodles::get_f32(record, &format!("faf95_{}", grp)).ok(),
            faf99: common::noodles::get_f32(record, &format!("faf99_{}", grp)).ok(),
            faf95_xx: common::noodles::get_f32(record, &format!("faf95_{}_XX", grp)).ok(),
            faf99_xx: common::noodles::get_f32(record, &format!("faf99_{}_XX", grp)).ok(),
            faf95_xy: common::noodles::get_f32(record, &format!("faf95_{}_XY", grp)).ok(),
            faf99_xy: common::noodles::get_f32(record, &format!("faf99_{}_XY", grp)).ok(),
        })
    }

    /// Extract the allele counts from the `record` with the given infix and suffix.
    fn extract_allele_counts(
        record: &noodles_vcf::Record,
        infix: &str,
        suffix: &str,
    ) -> Result<gnomad3::AlleleCounts, anyhow::Error> {
        Ok(gnomad3::AlleleCounts {
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
        let path_vcf = "tests/gnomad-nuclear/example-genomes-grch38/v4.0/gnomad-genomes.vcf";
        let mut reader_vcf = noodles_vcf::reader::Builder::default().build_from_path(path_vcf)?;
        let header = reader_vcf.read_header()?;

        let mut records = Vec::new();
        for row in reader_vcf.records(&header) {
            let vcf_record = row?;
            let record = Record::from_vcf_allele(
                &vcf_record,
                0,
                &gnomad3::DetailsOptions::with_all_enabled(),
            )?;
            records.push(record);
        }

        insta::assert_yaml_snapshot!(records);

        Ok(())
    }
}
