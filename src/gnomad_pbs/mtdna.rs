//! Protocolbuffers for gnomAD nuclear data structures.

use std::str::FromStr;

use noodles_vcf::record::info::field;

use super::vep_gnomad3::Vep;
use crate::common;

include!(concat!(env!("OUT_DIR"), "/annonars.gnomad.v1.mtdna.rs"));

/// Options struct that allows to specify which details fields are to be extracted from
/// gnomAD-mtDNA VCF records.
///
/// The only field that has `true` as its default is `vep`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DetailsOptions {
    /// Enable extraction of `Vep` records.
    pub vep: bool,
    /// Enable creation of `QualityInfo`.
    pub quality: bool,
    /// Enable creation of `HeteroplasmyInfo`.
    pub heteroplasmy: bool,
    /// Enable creation of `FilterHistograms`.
    pub filter_hists: bool,
    /// Enable creation of `PopulationInfo`.
    pub pop_details: bool,
    /// Enable creation of `HaplogroupInfo`.
    pub haplogroups_details: bool,
    /// Enable creation of `AgeInfo`.
    pub age_hists: bool,
    /// Enable creation of `DepthInfo`.
    pub depth_details: bool,
}

impl Default for DetailsOptions {
    fn default() -> Self {
        Self {
            vep: true,
            quality: false,
            heteroplasmy: false,
            filter_hists: false,
            pop_details: false,
            haplogroups_details: false,
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
            quality: true,
            heteroplasmy: true,
            filter_hists: true,
            pop_details: true,
            haplogroups_details: true,
            age_hists: true,
            depth_details: true,
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
        let variant_collapsed = common::noodles::get_string(record, "variant_collapsed")?;
        let excluded_ac = common::noodles::get_i32(record, "excluded_AC")?;
        let an = common::noodles::get_i32(record, "AN")?;
        let ac_hom = common::noodles::get_i32(record, "AC_hom")?;
        let ac_het = common::noodles::get_i32(record, "AC_het")?;
        let af_hom = common::noodles::get_f32(record, "AF_hom")?;
        let af_het = common::noodles::get_f32(record, "AF_het")?;
        let filters = Self::extract_filters(record)?;
        let mitotip_score = common::noodles::get_f32(record, "mitotip_score").ok();
        let mitotip_trna_prediction =
            common::noodles::get_string(record, "mitotip_trna_prediction").ok();
        let pon_mt_trna_prediction =
            common::noodles::get_string(record, "pon_mt_trna_prediction").ok();
        let pon_ml_probability_of_pathogenicity =
            common::noodles::get_string(record, "pon_ml_probability_of_pathogenicity").ok();

        // Extract optional fields.
        let vep = options
            .vep
            .then(|| Self::extract_vep(record))
            .transpose()?
            .unwrap_or_default();
        let quality_info = options
            .quality
            .then(|| Self::extract_quality(record))
            .transpose()?;
        let heteroplasmy_info = options
            .heteroplasmy
            .then(|| Self::extract_heteroplasmy(record))
            .transpose()?;
        let filter_histograms = options
            .filter_hists
            .then(|| Self::extract_filter_histograms(record))
            .transpose()?;
        let population_info = options
            .pop_details
            .then(|| Self::extract_population(record))
            .transpose()?;
        let haplogroup_info = options
            .haplogroups_details
            .then(|| Self::extract_haplogroup(record))
            .transpose()?;
        let age_info = options
            .age_hists
            .then(|| Self::extract_age(record))
            .transpose()?;
        let depth_info = options
            .depth_details
            .then(|| Self::extract_depth(record))
            .transpose()?;

        Ok(Record {
            chrom,
            pos,
            ref_allele,
            alt_allele,
            variant_collapsed,
            excluded_ac,
            an,
            ac_hom,
            ac_het,
            af_hom,
            af_het,
            filters,
            mitotip_score,
            mitotip_trna_prediction,
            pon_mt_trna_prediction,
            pon_ml_probability_of_pathogenicity,
            vep,
            quality_info,
            heteroplasmy_info,
            filter_histograms,
            population_info,
            haplogroup_info,
            age_info,
            depth_info,
        })
    }

    /// Extract the "vep" field.
    fn extract_vep(record: &noodles_vcf::Record) -> Result<Vec<Vep>, anyhow::Error> {
        if let Some(Some(field::Value::Array(field::value::Array::String(v)))) =
            record.info().get(&field::Key::from_str("vep")?)
        {
            v.iter()
                .flat_map(|v| v.as_ref().map(|s| Vep::from_str(s)))
                .collect::<Result<Vec<_>, _>>()
        } else {
            anyhow::bail!("missing INFO/vep in gnomAD-mtDNA record")
        }
    }

    /// Extract the heteroplasmy-related fields from the VCF record.
    fn extract_heteroplasmy(
        record: &noodles_vcf::record::Record,
    ) -> Result<HeteroplasmyInfo, anyhow::Error> {
        Ok(HeteroplasmyInfo {
            heteroplasmy_below_min_het_threshold_hist: common::noodles::get_vec::<i32>(
                record,
                "heteroplasmy_below_min_het_threshold_hist",
            )?,
            hl_hist: common::noodles::get_vec::<i32>(record, "hl_hist")?,
            common_low_heteroplasmy: common::noodles::get_flag(record, "common_low_heteroplasmy")?,
            max_hl: common::noodles::get_f32(record, "max_hl")?,
        })
    }

    /// Extract the filter histogram related fields form the VCF record.
    fn extract_filter_histograms(
        record: &noodles_vcf::record::Record,
    ) -> Result<FilterHistograms, anyhow::Error> {
        Ok(FilterHistograms {
            base_qual_hist: common::noodles::get_vec::<i32>(record, "base_qual_hist")
                .unwrap_or_default(),
            position_hist: common::noodles::get_vec::<i32>(record, "position_hist")
                .unwrap_or_default(),
            strand_bias_hist: common::noodles::get_vec::<i32>(record, "strand_bias_hist")
                .unwrap_or_default(),
            weak_evidence_hist: common::noodles::get_vec::<i32>(record, "weak_evidence_hist")
                .unwrap_or_default(),
            contamination_hist: common::noodles::get_vec::<i32>(record, "contamination_hist")
                .unwrap_or_default(),
        })
    }

    /// Extract the population related fields from the VCF record.
    fn extract_population(
        record: &noodles_vcf::record::Record,
    ) -> Result<PopulationInfo, anyhow::Error> {
        Ok(PopulationInfo {
            pop_an: common::noodles::get_vec::<i32>(record, "pop_AN")?,
            pop_ac_het: common::noodles::get_vec::<i32>(record, "pop_AC_het").unwrap_or_default(),
            pop_ac_hom: common::noodles::get_vec::<i32>(record, "pop_AC_hom").unwrap_or_default(),
            pop_af_hom: common::noodles::get_vec::<f32>(record, "pop_AF_hom").unwrap_or_default(),
            pop_af_het: common::noodles::get_vec::<f32>(record, "pop_AF_het").unwrap_or_default(),
            pop_hl_hist: common::noodles::get_vec_vec::<i32>(record, "pop_hl_hist")
                .unwrap_or_default(),
        })
    }

    /// Extract the haplogroup related fields from the VCF record.
    fn extract_haplogroup(
        record: &noodles_vcf::record::Record,
    ) -> Result<HaplogroupInfo, anyhow::Error> {
        Ok(HaplogroupInfo {
            hap_defining_variant: common::noodles::get_flag(record, "hap_defining_variant")?,
            hap_an: common::noodles::get_vec::<i32>(record, "hap_AN").unwrap_or_default(),
            hap_ac_het: common::noodles::get_vec::<i32>(record, "hap_AC_het").unwrap_or_default(),
            hap_ac_hom: common::noodles::get_vec::<i32>(record, "hap_AC_hom").unwrap_or_default(),
            hap_af_het: common::noodles::get_vec::<f32>(record, "hap_AF_het").unwrap_or_default(),
            hap_af_hom: common::noodles::get_vec::<f32>(record, "hap_AF_hom").unwrap_or_default(),
            hap_hl_hist: common::noodles::get_vec_vec::<i32>(record, "hap_hl_hist")
                .unwrap_or_default(),
            hap_faf_hom: common::noodles::get_vec::<f32>(record, "hap_faf_hom").unwrap_or_default(),
            hapmax_af_hom: common::noodles::get_string(record, "hapmax_AF_hom").ok(),
            hapmax_af_het: common::noodles::get_string(record, "hapmax_AF_het").ok(),
            faf_hapmax_hom: common::noodles::get_f32(record, "faf_hapmax_hom").ok(),
        })
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
            dp_mean: common::noodles::get_f32(record, "dp_mean").ok(),
            mq_mean: common::noodles::get_f32(record, "mq_mean").ok(),
            tlod_mean: common::noodles::get_f32(record, "tlod_mean").ok(),
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
                        Some("artifact_prone_site") => Ok(Filter::ArtifactProneSite as i32),
                        Some("indel_stack") => Ok(Filter::IndelStack as i32),
                        Some("npg") => Ok(Filter::NoPassGenotype as i32),
                        Some(val) => anyhow::bail!("invalid filter value {}", val),
                        None => anyhow::bail!("missing filter value"),
                    })
                    .collect::<Result<Vec<_>, _>>()?
            } else {
                Vec::new()
            },
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_record_from_vcf_allele() -> Result<(), anyhow::Error> {
        let path_vcf = "tests/gnomad-mtdna/example/gnomad-mtdna.vcf";
        let mut reader_vcf =
            noodles_util::variant::reader::Builder::default().build_from_path(path_vcf)?;
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
