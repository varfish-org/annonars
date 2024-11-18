//! Implementation of endpoint `/api/v1/seqvars/annos`.
//!
//! Also includes the implementation of the `/annos/variant` endpoint (deprecated).

use actix_web::{
    get,
    web::{self, Data, Json, Path},
};
use strum::IntoEnumIterator;

use crate::{
    common::{keys, version},
    server::run::{fetch::fetch_pos_protobuf, AnnoDb},
};

use super::error::CustomError;
use super::fetch::{
    fetch_pos_protobuf_json, fetch_var_protobuf, fetch_var_protobuf_json, fetch_var_tsv_json,
};

/// Parameters for `variant_annos::handle`.
///
/// Defines a variant in VCF-style format with a genome release specification.
#[serde_with::skip_serializing_none]
#[serde_with::serde_as]
#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema, utoipa::IntoParams,
)]
pub struct SeqvarsAnnosQuery {
    /// Genome release specification.
    pub genome_release: String,
    /// Chromosome name.
    pub chromosome: String,
    /// 1-based position for VCF-style variant.
    pub pos: u32,
    /// Reference allele bases.
    pub reference: String,
    /// Alterantive allele bases.
    pub alternative: String,
}

impl From<SeqvarsAnnosQuery> for keys::Var {
    fn from(value: SeqvarsAnnosQuery) -> Self {
        keys::Var {
            chrom: value.chromosome,
            pos: value.pos as i32,
            reference: value.reference,
            alternative: value.alternative,
        }
    }
}

impl From<SeqvarsAnnosQuery> for keys::Pos {
    fn from(value: SeqvarsAnnosQuery) -> Self {
        keys::Pos {
            chrom: value.chromosome,
            pos: value.pos as i32,
        }
    }
}

/// Result for `handle`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde_with::skip_serializing_none]
struct Container {
    /// Version of the server code.
    pub server_version: String,
    /// The query parameters.
    pub query: SeqvarsAnnosQuery,
    /// Annotations for the variant from each database.
    pub result: std::collections::BTreeMap<AnnoDb, Option<serde_json::Value>>,
}

/// Query for annotations for one variant.
#[get("/annos/variant")]
async fn handle(
    data: Data<crate::server::run::WebServerData>,
    _path: Path<()>,
    query: web::Query<SeqvarsAnnosQuery>,
) -> actix_web::Result<Json<Container>, CustomError> {
    let genome_release =
        query
            .clone()
            .into_inner()
            .genome_release
            .parse()
            .map_err(|e: strum::ParseError| {
                CustomError::new(anyhow::anyhow!("problem getting genome release: {}", e))
            })?;

    let mut annotations = std::collections::BTreeMap::default();
    for anno_db in AnnoDb::iter() {
        match anno_db {
            AnnoDb::Other => (),
            AnnoDb::Clinvar => {
                data.annos[genome_release][anno_db]
                    .as_ref()
                    .map(|db| {
                        fetch_var_protobuf_json::<
                            crate::pbs::clinvar::minimal::ExtractedVcvRecordList,
                        >(
                            &db.data,
                            anno_db.cf_name(),
                            query.clone().into_inner().into(),
                        )
                    })
                    .transpose()?
                    .map(|v| annotations.insert(anno_db, v));
            }
            AnnoDb::Cadd | AnnoDb::Dbnsfp | AnnoDb::Dbscsnv => {
                data.annos[genome_release][anno_db]
                    .as_ref()
                    .map(|db| {
                        fetch_var_tsv_json(
                            &db.data,
                            anno_db.cf_name(),
                            query.clone().into_inner().into(),
                        )
                    })
                    .transpose()?
                    .map(|v| annotations.insert(anno_db, v));
            }
            AnnoDb::Dbsnp => {
                data.annos[genome_release][anno_db]
                    .as_ref()
                    .map(|db| {
                        fetch_var_protobuf_json::<crate::dbsnp::pbs::Record>(
                            &db.data,
                            anno_db.cf_name(),
                            query.clone().into_inner().into(),
                        )
                    })
                    .transpose()?
                    .map(|v| annotations.insert(anno_db, v));
            }
            AnnoDb::Helixmtdb => {
                data.annos[genome_release][anno_db]
                    .as_ref()
                    .map(|db| {
                        fetch_var_protobuf_json::<crate::helixmtdb::pbs::Record>(
                            &db.data,
                            anno_db.cf_name(),
                            query.clone().into_inner().into(),
                        )
                    })
                    .transpose()?
                    .map(|v| annotations.insert(anno_db, v));
            }
            AnnoDb::GnomadMtdna => {
                data.annos[genome_release][anno_db]
                    .as_ref()
                    .map(|db| {
                        fetch_var_protobuf_json::<crate::pbs::gnomad::mtdna::Record>(
                            &db.data,
                            anno_db.cf_name(),
                            query.clone().into_inner().into(),
                        )
                    })
                    .transpose()?
                    .map(|v| annotations.insert(anno_db, v));
            }
            AnnoDb::GnomadExomes => {
                data.annos[genome_release][anno_db]
                    .as_ref()
                    .map(|db| {
                        let db_version = data.db_infos[genome_release][anno_db]
                            .as_ref()
                            .expect("must have db info here")
                            .db_version
                            .as_ref()
                            .expect("gnomAD must have db version");

                        if db_version.starts_with("2.") {
                            fetch_var_protobuf_json::<crate::pbs::gnomad::gnomad2::Record>(
                                &db.data,
                                anno_db.cf_name(),
                                query.clone().into_inner().into(),
                            )
                        } else if db_version.starts_with("4.") {
                            fetch_var_protobuf_json::<crate::pbs::gnomad::gnomad4::Record>(
                                &db.data,
                                anno_db.cf_name(),
                                query.clone().into_inner().into(),
                            )
                        } else {
                            Err(CustomError::new(anyhow::anyhow!(
                                "don't know how to handle gnomAD version {}",
                                db_version
                            )))
                        }
                    })
                    .transpose()?
                    .map(|v| annotations.insert(anno_db, v));
            }
            AnnoDb::GnomadGenomes => {
                data.annos[genome_release][anno_db]
                    .as_ref()
                    .map(|db| {
                        let db_version = data.db_infos[genome_release][anno_db]
                            .as_ref()
                            .expect("must have db info here")
                            .db_version
                            .as_ref()
                            .expect("gnomAD must have db version");
                        if db_version.starts_with("2.") {
                            fetch_var_protobuf_json::<crate::pbs::gnomad::gnomad2::Record>(
                                &db.data,
                                anno_db.cf_name(),
                                query.clone().into_inner().into(),
                            )
                        } else if db_version.starts_with("3.") {
                            fetch_var_protobuf_json::<crate::pbs::gnomad::gnomad3::Record>(
                                &db.data,
                                anno_db.cf_name(),
                                query.clone().into_inner().into(),
                            )
                        } else if db_version.starts_with("4.") {
                            fetch_var_protobuf_json::<crate::pbs::gnomad::gnomad4::Record>(
                                &db.data,
                                anno_db.cf_name(),
                                query.clone().into_inner().into(),
                            )
                        } else {
                            Err(CustomError::new(anyhow::anyhow!(
                                "don't know how to handle gnomAD version {}",
                                db_version
                            )))
                        }
                    })
                    .transpose()?
                    .map(|v| annotations.insert(anno_db, v));
            }
            AnnoDb::UcscConservation => {
                data.annos[genome_release][anno_db]
                    .as_ref()
                    .map(|db| {
                        let start: keys::Pos = query.clone().into_inner().into();
                        let start = keys::Pos {
                            chrom: start.chrom,
                            pos: start.pos - 2,
                        };
                        let stop = query.clone().into_inner().into();
                        fetch_pos_protobuf_json::<crate::pbs::cons::RecordList>(
                            &db.data,
                            anno_db.cf_name(),
                            start,
                            stop,
                        )
                    })
                    .transpose()?
                    .map(|v| annotations.insert(anno_db, v));
            }
        }
    }

    let result = Container {
        server_version: version().to_string(),
        query: query.into_inner(),
        result: annotations,
    };

    Ok(Json(result))
}

/// `SeqvarsAnnosResponse` and related types.
pub mod response {
    use crate::{pbs, server::run::clinvar_data::ClinvarExtractedVcvRecord};

    /// Protocol buffer for `Vep.domains`
    #[derive(
        Debug,
        Default,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        utoipa::ToSchema,
        utoipa::ToResponse,
    )]
    pub struct VepCommonDomain {
        /// Domain ID.
        pub id: String,
        /// Domain source.
        pub source: String,
    }

    impl From<pbs::gnomad::vep_common::Domain> for VepCommonDomain {
        fn from(value: pbs::gnomad::vep_common::Domain) -> Self {
            VepCommonDomain {
                id: value.id,
                source: value.source,
            }
        }
    }

    /// Store the scoring of a prediction.
    #[derive(
        Debug,
        Default,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        utoipa::ToSchema,
        utoipa::ToResponse,
    )]
    pub struct VepCommonPrediction {
        /// Prediction.
        pub prediction: String,
        /// Score.
        pub score: f32,
    }

    impl From<pbs::gnomad::vep_common::Prediction> for VepCommonPrediction {
        fn from(value: pbs::gnomad::vep_common::Prediction) -> Self {
            VepCommonPrediction {
                prediction: value.prediction,
                score: value.score,
            }
        }
    }

    // This file is @generated by prost-build.
    /// Protocol buffer for the gnomAD-nuclear VEP predictions.
    #[derive(
        Debug,
        Default,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        utoipa::ToSchema,
        utoipa::ToResponse,
    )]
    pub struct Gnomad3Vep {
        /// Allele of record.
        pub allele: String,
        /// Consequence, e.g., `"missense_variant"`.
        pub consequence: String,
        /// Impact, e.g., `"MODERATE"`.
        pub impact: String,
        /// Gene symbol, e.g., `"PCSK9"`.
        pub symbol: String,
        /// Gene ID, `e.g., "ENSG00000169174"`.
        pub gene: String,
        /// Feature type, e.g., `"Transcript"`.
        pub feature_type: String,
        /// Feature ID, e.g., `"ENST00000302118"`.
        pub feature: String,
        /// Feature biotype, e.g., `"protein_coding"`.
        pub feature_biotype: String,
        /// Ranked exon number, e.g., `"1/4"`.
        pub exon: Option<String>,
        /// Ranked intron number, e.g., `"1/4"`.
        pub intron: Option<String>,
        /// cDNA position, e.g., `"ENST00000302118.5:c.89C>G"`.
        pub hgvsc: Option<String>,
        /// Protein position, e.g., `"ENSP00000302118.5:p.Thr30Arg"`.
        pub hgvsp: Option<String>,
        /// cDNA position, e.g., `"89/1863"`.
        pub cdna_position: Option<String>,
        /// CDS position, e.g., `"89/1863"`.
        pub cds_position: Option<String>,
        /// Protein position, e.g., `"30/620"`.
        pub protein_position: Option<String>,
        /// Amino acids, e.g., `"T/R"`.
        pub amino_acids: Option<String>,
        /// Codons, e.g., `"gCg/gGg"`.
        pub codons: Option<String>,
        /// TODO: actually is optional int32 allele_num = 18;
        /// dbSNP ID, e.g., `"rs28942080"`.
        pub dbsnp_id: Option<String>,
        /// Distance output of VEP.
        pub distance: Option<String>,
        /// Strand, e.g., `"1"`.
        pub strand: Option<String>,
        /// Variant class, e.g., `"SNV"`.
        pub variant_class: Option<String>,
        /// Minimised output of VEP.
        pub minimised: Option<String>,
        /// Symbol source, e.g., `"HGNC"`.
        pub symbol_source: Option<String>,
        /// HGNC ID, e.g., `"HGNC:8706"`.
        pub hgnc_id: Option<String>,
        /// Whether this is the canonical transcript.
        pub canonical: Option<bool>,
        /// Transcript support level, e.g., `"1"`.
        pub tsl: Option<i32>,
        /// APPRIS annotation, e.g. `"P1"`.
        pub appris: Option<String>,
        /// CCDS ID, e.g., `"CCDS30547.1"`.
        pub ccds: Option<String>,
        /// Ensembl protein ID, e.g., `"ENSP00000302118"`.
        pub ensp: Option<String>,
        /// SwissProt ID, e.g., `"P04114"`.
        pub swissprot: Option<String>,
        /// TREMBL ID, e.g., `"Q5T4W7"`.
        pub trembl: Option<String>,
        /// UniParc ID, e.g., `"UPI000002D4B2"`.
        pub uniparc: Option<String>,
        /// Gene phenotype from VEP.
        pub gene_pheno: Option<String>,
        /// SIFT prediction, e.g., `"tolerated(0.06)"`.
        pub sift: Option<VepCommonPrediction>,
        /// PolyPhen prediction, e.g., `"benign(0.001)"`.
        pub polyphen: Option<VepCommonPrediction>,
        /// Protein domains, e.g., `\[["2p4e", "ENSP_mappings"\], \["2qtw", "ENSP_mappings"]\]`.
        pub domains: Vec<VepCommonDomain>,
        /// HGVS offset.
        pub hgvs_offset: Option<String>,
        /// Motif name.
        pub motif_name: Option<String>,
        /// Motif name.
        pub motif_pos: Option<String>,
        /// "high inf pos" from VEP.
        pub high_inf_pos: Option<String>,
        /// Motif score change.
        pub motif_score_change: Option<String>,
        /// Loss of function prediction.
        pub lof: Option<String>,
        /// Loss of function filter.
        pub lof_filter: Option<String>,
        /// Loss of function flags.
        pub lof_flags: Option<String>,
        /// Loss of function info.
        pub lof_info: Option<String>,
    }

    impl From<pbs::gnomad::vep_gnomad3::Vep> for Gnomad3Vep {
        fn from(value: pbs::gnomad::vep_gnomad3::Vep) -> Self {
            Gnomad3Vep {
                allele: value.allele,
                consequence: value.consequence,
                impact: value.impact,
                symbol: value.symbol,
                gene: value.gene,
                feature_type: value.feature_type,
                feature: value.feature,
                feature_biotype: value.feature_biotype,
                exon: value.exon,
                intron: value.intron,
                hgvsc: value.hgvsc,
                hgvsp: value.hgvsp,
                cdna_position: value.cdna_position,
                cds_position: value.cds_position,
                protein_position: value.protein_position,
                amino_acids: value.amino_acids,
                codons: value.codons,
                dbsnp_id: value.dbsnp_id,
                distance: value.distance,
                strand: value.strand,
                variant_class: value.variant_class,
                minimised: value.minimised,
                symbol_source: value.symbol_source,
                hgnc_id: value.hgnc_id,
                canonical: value.canonical,
                tsl: value.tsl,
                appris: value.appris,
                ccds: value.ccds,
                ensp: value.ensp,
                swissprot: value.swissprot,
                trembl: value.trembl,
                uniparc: value.uniparc,
                gene_pheno: value.gene_pheno,
                sift: value.sift.map(Into::into),
                polyphen: value.polyphen.map(Into::into),
                domains: value.domains.into_iter().map(Into::into).collect(),
                hgvs_offset: value.hgvs_offset,
                motif_name: value.motif_name,
                motif_pos: value.motif_pos,
                high_inf_pos: value.high_inf_pos,
                motif_score_change: value.motif_score_change,
                lof: value.lof,
                lof_filter: value.lof_filter,
                lof_flags: value.lof_flags,
                lof_info: value.lof_info,
            }
        }
    }

    /// Encapsulate VCF INFO fields related to quality.
    #[derive(
        Debug,
        Default,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        utoipa::ToSchema,
        utoipa::ToResponse,
    )]
    pub struct GnomadMtdnaQualityInfo {
        /// Mean depth across all individuals for the site.
        pub dp_mean: Option<f32>,
        /// Mean MMQ (median mapping quality) across individuals with a variant for the site.
        pub mq_mean: Option<f32>,
        /// Mean TLOD (Log 10 likelihood ratio score of variant existing versus not existing) across
        /// individuals with a variant for the site.
        pub tlod_mean: Option<f32>,
    }

    impl From<pbs::gnomad::mtdna::QualityInfo> for GnomadMtdnaQualityInfo {
        fn from(value: pbs::gnomad::mtdna::QualityInfo) -> Self {
            GnomadMtdnaQualityInfo {
                dp_mean: value.dp_mean,
                mq_mean: value.mq_mean,
                tlod_mean: value.tlod_mean,
            }
        }
    }

    /// Encapsulate VCF INFO fields related to heteroplasmy levels.
    #[derive(
        Debug,
        Default,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        utoipa::ToSchema,
        utoipa::ToResponse,
    )]
    pub struct GnomadMtdnaHeteroplasmyInfo {
        /// Histogram of number of individuals with a heteroplasmy level below 0.1, bin edges are: [0.0,
        /// 0.1, 0.2, 0.30000000000000004, 0.4, 0.5, 0.6000000000000001, 0.7000000000000001, 0.8, 0.9,
        /// 1.0]
        pub heteroplasmy_below_min_het_threshold_hist: Vec<i32>,
        /// Histogram of heteroplasmy levels; bin edges are: [0.0, 0.1, 0.2, 0.30000000000000004, 0.4,
        /// 0.5, 0.6000000000000001, 0.7000000000000001, 0.8, 0.9, 1.0].
        pub hl_hist: Vec<i32>,
        /// Present if variant is found at an overall frequency of .001 across all samples with a
        /// heteroplasmy level > 0 and < 0.50 (includes variants <0.01 heteroplasmy which are
        /// subsequently filtered)
        pub common_low_heteroplasmy: bool,
        /// Maximum heteroplasmy level observed among all samples for that variant.
        pub max_hl: f32,
    }

    impl From<pbs::gnomad::mtdna::HeteroplasmyInfo> for GnomadMtdnaHeteroplasmyInfo {
        fn from(value: pbs::gnomad::mtdna::HeteroplasmyInfo) -> Self {
            GnomadMtdnaHeteroplasmyInfo {
                heteroplasmy_below_min_het_threshold_hist: value
                    .heteroplasmy_below_min_het_threshold_hist,
                hl_hist: value.hl_hist,
                common_low_heteroplasmy: value.common_low_heteroplasmy,
                max_hl: value.max_hl,
            }
        }
    }

    /// Encapsulate VCF INFO fields related to filter failure histograms.
    #[derive(
        Debug,
        Default,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        utoipa::ToSchema,
        utoipa::ToResponse,
    )]
    pub struct GnomadMtdnaFilterHistograms {
        /// Histogram of number of individuals failing the base_qual filter (alternate allele median base
        /// quality) across heteroplasmy levels, bin edges are: [0.0, 0.1, 0.2, 0.30000000000000004, 0.4,
        /// 0.5, 0.6000000000000001, 0.7000000000000001, 0.8, 0.9, 1.0]
        pub base_qual_hist: Vec<i32>,
        /// Histogram of number of individuals failing the position filter (median distance of alternate
        /// variants from end of reads) across heteroplasmy levels, bin edges are: [0.0, 0.1, 0.2, 0.
        /// 30000000000000004, 0.4, 0.5, 0.6000000000000001, 0.7000000000000001, 0.8, 0.9, 1.0]
        pub position_hist: Vec<i32>,
        /// Histogram of number of individuals failing the strand_bias filter (evidence for alternate
        /// allele comes from one read direction only) across heteroplasmy levels, bin edges are: [0.0,
        /// 0.1, 0.2, 0.30000000000000004, 0.4, 0.5, 0.6000000000000001, 0.7000000000000001, 0.8, 0.9,
        /// 1.0]
        pub strand_bias_hist: Vec<i32>,
        /// Histogram of number of individuals failing the weak_evidence filter (mutation does not meet
        /// likelihood threshold) across heteroplasmy levels, bin edges are: [0.0, 0.1, 0.2,
        /// 0.30000000000000004, 0.4, 0.5, 0.6000000000000001, 0.7000000000000001, 0.8, 0.9, 1.0]
        pub weak_evidence_hist: Vec<i32>,
        /// Histogram of number of individuals failing the contamination filter across heteroplasmy
        /// levels, bin edges are: [0.0, 0.1, 0.2, 0.30000000000000004, 0.4, 0.5, 0.6000000000000001,
        /// 0.7000000000000001, 0.8, 0.9, 1.0]
        pub contamination_hist: Vec<i32>,
    }

    impl From<pbs::gnomad::mtdna::FilterHistograms> for GnomadMtdnaFilterHistograms {
        fn from(value: pbs::gnomad::mtdna::FilterHistograms) -> Self {
            GnomadMtdnaFilterHistograms {
                base_qual_hist: value.base_qual_hist,
                position_hist: value.position_hist,
                strand_bias_hist: value.strand_bias_hist,
                weak_evidence_hist: value.weak_evidence_hist,
                contamination_hist: value.contamination_hist,
            }
        }
    }

    /// Encapsulate VCF INFO fields related to populations.
    #[derive(
        Debug,
        Default,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        utoipa::ToSchema,
        utoipa::ToResponse,
    )]
    pub struct GnomadMtdnaPopulationInfo {
        /// List of overall allele number for each population, population order: ['afr', 'ami', 'amr',
        /// 'asj', 'eas', 'fin', 'nfe', 'oth', 'sas', 'mid']
        pub pop_an: Vec<i32>,
        /// List of AC_het for each population, population order: ['afr', 'ami', 'amr', 'asj', 'eas',
        /// 'fin', 'nfe', 'oth', 'sas', 'mid']
        pub pop_ac_het: Vec<i32>,
        /// List of AC_hom for each population, population order: ['afr', 'ami', 'amr', 'asj', 'eas',
        /// 'fin', 'nfe', 'oth', 'sas', 'mid']
        pub pop_ac_hom: Vec<i32>,
        /// List of AF_hom for each population, population order: ['afr', 'ami', 'amr', 'asj', 'eas',
        /// 'fin', 'nfe', 'oth', 'sas', 'mid']
        pub pop_af_hom: Vec<f32>,
        /// List of AF_het for each population, population order: ['afr', 'ami', 'amr', 'asj', 'eas',
        /// 'fin', 'nfe', 'oth', 'sas', 'mid']
        pub pop_af_het: Vec<f32>,
        /// Histogram of heteroplasmy levels for each population; bin edges are: [0.0, 0.1, 0.2,
        /// 0.30000000000000004, 0.4, 0.5, 0.6000000000000001, 0.7000000000000001, 0.8, 0.9, 1.0],
        /// population order: \['afr', 'ami', 'amr', 'asj', 'eas', 'fin', 'nfe', 'oth', 'sas', 'mid'\]
        ///
        /// Note that we encode this by concatenating all lists here because of limitations in
        /// protocolbuffers (no native nested repeated fields).
        pub pop_hl_hist: Vec<i32>,
    }

    impl From<pbs::gnomad::mtdna::PopulationInfo> for GnomadMtdnaPopulationInfo {
        fn from(value: pbs::gnomad::mtdna::PopulationInfo) -> Self {
            GnomadMtdnaPopulationInfo {
                pop_an: value.pop_an,
                pop_ac_het: value.pop_ac_het,
                pop_ac_hom: value.pop_ac_hom,
                pop_af_hom: value.pop_af_hom,
                pop_af_het: value.pop_af_het,
                pop_hl_hist: value.pop_hl_hist,
            }
        }
    }
    /// Encapsulate VCF INFO fields related to haplogroups.
    #[derive(
        Debug,
        Default,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        utoipa::ToSchema,
        utoipa::ToResponse,
    )]
    pub struct GnomadMtdnaHaplogroupInfo {
        /// Present if variant is present as a haplogroup defining variant in PhyloTree build 17.
        pub hap_defining_variant: bool,
        /// List of overall allele number for each haplogroup, haplogroup order: ['A', 'B', 'C', 'D',
        /// 'E', 'F', 'G', 'H', 'HV', 'I', 'J', 'K', 'L0', 'L1', 'L2', 'L3', 'L4', 'L5', 'M', 'N', 'P',
        /// 'R', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z']
        pub hap_an: Vec<i32>,
        /// List of AC_het for each haplogroup, haplogroup order: ['A', 'B', 'C', 'D', 'E', 'F', 'G',
        /// 'H', 'HV', 'I', 'J', 'K', 'L0', 'L1', 'L2', 'L3', 'L4', 'L5', 'M', 'N', 'P', 'R', 'T', 'U',
        /// 'V', 'W', 'X', 'Y', 'Z']
        pub hap_ac_het: Vec<i32>,
        /// List of AC_hom for each haplogroup, haplogroup order: ['A', 'B', 'C', 'D', 'E', 'F', 'G',
        /// 'H', 'HV', 'I', 'J', 'K', 'L0', 'L1', 'L2', 'L3', 'L4', 'L5', 'M', 'N', 'P', 'R', 'T', 'U',
        /// 'V', 'W', 'X', 'Y', 'Z']
        pub hap_ac_hom: Vec<i32>,
        /// List of AF_het for each haplogroup, haplogroup order: ['A', 'B', 'C', 'D', 'E', 'F', 'G',
        /// 'H', 'HV', 'I', 'J', 'K', 'L0', 'L1', 'L2', 'L3', 'L4', 'L5', 'M', 'N', 'P', 'R', 'T', 'U',
        /// 'V', 'W', 'X', 'Y', 'Z']
        pub hap_af_het: Vec<f32>,
        /// List of AF_hom for each haplogroup, haplogroup order: ['A', 'B', 'C', 'D', 'E', 'F', 'G',
        /// 'H', 'HV', 'I', 'J', 'K', 'L0', 'L1', 'L2', 'L3', 'L4', 'L5', 'M', 'N', 'P', 'R', 'T', 'U',
        /// 'V', 'W', 'X', 'Y', 'Z']
        pub hap_af_hom: Vec<f32>,
        /// Histogram of heteroplasmy levels for each haplogroup; bin edges are: [0.0, 0.1, 0.2,
        /// 0.30000000000000004, 0.4, 0.5, 0.6000000000000001, 0.7000000000000001, 0.8, 0.9, 1.0],
        /// haplogroup order: ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'HV', 'I', 'J', 'K', 'L0', 'L1',
        /// 'L2', 'L3', 'L4', 'L5', 'M', 'N', 'P', 'R', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z']
        ///
        /// Note that we encode this by concatenating all lists here because of limitations in
        /// protocolbuffers (no native nested repeated fields).
        pub hap_hl_hist: Vec<i32>,
        /// List of filtering allele frequency for each haplogroup restricted to homoplasmic variants,
        /// haplogroup order: ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'HV', 'I', 'J', 'K', 'L0', 'L1',
        /// 'L2', 'L3', 'L4', 'L5', 'M', 'N', 'P', 'R', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z']
        pub hap_faf_hom: Vec<f32>,
        /// Haplogroup with maximum AF_hom.
        pub hapmax_af_hom: Option<String>,
        /// Haplogroup with maximum AF_het.
        pub hapmax_af_het: Option<String>,
        /// Maximum filtering allele frequency across haplogroups restricted to homoplasmic variants.
        pub faf_hapmax_hom: Option<f32>,
    }

    impl From<pbs::gnomad::mtdna::HaplogroupInfo> for GnomadMtdnaHaplogroupInfo {
        fn from(value: pbs::gnomad::mtdna::HaplogroupInfo) -> Self {
            GnomadMtdnaHaplogroupInfo {
                hap_defining_variant: value.hap_defining_variant,
                hap_an: value.hap_an,
                hap_ac_het: value.hap_ac_het,
                hap_ac_hom: value.hap_ac_hom,
                hap_af_het: value.hap_af_het,
                hap_af_hom: value.hap_af_hom,
                hap_hl_hist: value.hap_hl_hist,
                hap_faf_hom: value.hap_faf_hom,
                hapmax_af_hom: value.hapmax_af_hom,
                hapmax_af_het: value.hapmax_af_het,
                faf_hapmax_hom: value.faf_hapmax_hom,
            }
        }
    }

    /// Encapsulate VCF INFO fields related to age.
    #[derive(
        Debug,
        Default,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        utoipa::ToSchema,
        utoipa::ToResponse,
    )]
    pub struct GnomadMtdnaAgeInfo {
        /// Histogram of ages of individuals with a homoplasmic variant; bin edges are: [30.0, 35.0,
        /// 40.0, 45.0, 50.0, 55.0, 60.0, 65.0, 70.0, 75.0, 80.0].
        pub age_hist_hom_bin_freq: Vec<i32>,
        /// Count of age values falling below lowest histogram bin edge for individuals with a
        /// homoplasmic variant.
        pub age_hist_hom_n_smaller: Option<i32>,
        /// Count of age values falling above highest histogram bin edge for individuals with a
        /// homoplasmic variant.
        pub age_hist_hom_n_larger: Option<i32>,
        /// Histogram of ages of individuals with a heteroplasmic variant; bin edges are: [30.0, 35.0,
        /// 40.0, 45.0, 50.0, 55.0, 60.0, 65.0, 70.0, 75.0, 80.0]
        pub age_hist_het_bin_freq: Vec<i32>,
        /// Count of age values falling below lowest histogram bin edge for individuals with a
        /// heteroplasmic variant.
        pub age_hist_het_n_smaller: Option<i32>,
        /// Count of age values falling above highest histogram bin edge for individuals with a
        /// heteroplasmic variant.
        pub age_hist_het_n_larger: Option<i32>,
    }

    impl From<pbs::gnomad::mtdna::AgeInfo> for GnomadMtdnaAgeInfo {
        fn from(value: pbs::gnomad::mtdna::AgeInfo) -> Self {
            GnomadMtdnaAgeInfo {
                age_hist_hom_bin_freq: value.age_hist_hom_bin_freq,
                age_hist_hom_n_smaller: value.age_hist_hom_n_smaller,
                age_hist_hom_n_larger: value.age_hist_hom_n_larger,
                age_hist_het_bin_freq: value.age_hist_het_bin_freq,
                age_hist_het_n_smaller: value.age_hist_het_n_smaller,
                age_hist_het_n_larger: value.age_hist_het_n_larger,
            }
        }
    }

    /// Encapsulate VCF INFO fields related to depth.
    #[derive(
        Debug,
        Default,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        utoipa::ToSchema,
        utoipa::ToResponse,
    )]
    pub struct GnomadMtdnaDepthInfo {
        /// Count of dp values falling above highest histogram bin edge for all individuals.
        pub dp_hist_all_n_larger: Option<i32>,
        /// Count of dp values falling above highest histogram bin edge for individuals with the
        /// alternative allele
        pub dp_hist_alt_n_larger: Option<i32>,
        /// Histogram of dp values for all individuals; bin edges are: [0.0, 200.0, 400.0, 600.0, 800.0,
        /// 1000.0, 1200.0, 1400.0, 1600.0, 1800.0, 2000.0]
        pub dp_hist_all_bin_freq: Vec<i32>,
        /// Histogram of dp values for individuals with the alternative allele; bin edges are: [0.0,
        /// 200.0, 400.0, 600.0, 800.0, 1000.0, 1200.0, 1400.0, 1600.0, 1800.0, 2000.0]
        pub dp_hist_alt_bin_freq: Vec<i32>,
    }

    impl From<pbs::gnomad::mtdna::DepthInfo> for GnomadMtdnaDepthInfo {
        fn from(value: pbs::gnomad::mtdna::DepthInfo) -> Self {
            GnomadMtdnaDepthInfo {
                dp_hist_all_n_larger: value.dp_hist_all_n_larger,
                dp_hist_alt_n_larger: value.dp_hist_alt_n_larger,
                dp_hist_all_bin_freq: value.dp_hist_all_bin_freq,
                dp_hist_alt_bin_freq: value.dp_hist_alt_bin_freq,
            }
        }
    }

    /// Protocol buffer for the gnomAD-mtDNA VCF record.
    ///
    /// The more specialized fields from the INFO column are stored in separate, optional fields such
    /// that we don't end up with a humongous message.
    #[derive(
        Debug,
        Default,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        utoipa::ToSchema,
        utoipa::ToResponse,
    )]
    pub struct GnomadMtdnaRecord {
        /// Chromosome name.
        pub chrom: String,
        /// 1-based start position.
        pub pos: i32,
        /// Reference allele.
        pub ref_allele: String,
        /// Alternate allele.
        pub alt_allele: String,
        /// Variant in format of RefPosAlt
        pub variant_collapsed: String,
        /// Excluded allele count (number of individuals in which the variant was filtered out).
        pub excluded_ac: i32,
        /// Overall allele number (number of samples with non-missing genotype).
        pub an: i32,
        /// Allele count restricted to variants with a heteroplasmy level >= 0.95.
        pub ac_hom: i32,
        /// Allele count restricted to variants with a heteroplasmy level >= 0.10 and < 0.95.
        pub ac_het: i32,
        /// Allele frequency restricted to variants with a heteroplasmy level >= 0.95.
        pub af_hom: f32,
        /// Allele frequency restricted to variants with a heteroplasmy level >= 0.10 and < 0.95.
        pub af_het: f32,
        /// Site-level filters.
        pub filters: Vec<GnomadMtdnaFilter>,
        /// MitoTip raw score
        pub mitotip_score: Option<f32>,
        /// MitoTip score interpretation
        pub mitotip_trna_prediction: Option<String>,
        /// tRNA pathogenicity classification from PON-mt-tRNA
        pub pon_mt_trna_prediction: Option<String>,
        /// tRNA ML_probability_of_pathogenicity from PON-mt-tRNA
        pub pon_ml_probability_of_pathogenicity: Option<String>,
        /// VEP v3 annotation records.
        pub vep: Vec<Gnomad3Vep>,
        /// Summary information for variant quality interpretation.
        pub quality_info: Option<GnomadMtdnaQualityInfo>,
        /// Information related to heteroplasmy levels.
        pub heteroplasmy_info: Option<GnomadMtdnaHeteroplasmyInfo>,
        /// Histograms related to variant quality filters.
        pub filter_histograms: Option<GnomadMtdnaFilterHistograms>,
        /// Population-related information.
        pub population_info: Option<GnomadMtdnaPopulationInfo>,
        /// Haplogroup-related information.
        pub haplogroup_info: Option<GnomadMtdnaHaplogroupInfo>,
        /// Age-related information.
        pub age_info: Option<GnomadMtdnaAgeInfo>,
        /// Depth of coverage-related information.
        pub depth_info: Option<GnomadMtdnaDepthInfo>,
    }

    impl TryFrom<pbs::gnomad::mtdna::Record> for GnomadMtdnaRecord {
        type Error = anyhow::Error;

        fn try_from(value: pbs::gnomad::mtdna::Record) -> Result<Self, Self::Error> {
            Ok(GnomadMtdnaRecord {
                chrom: value.chrom,
                pos: value.pos,
                ref_allele: value.ref_allele,
                alt_allele: value.alt_allele,
                variant_collapsed: value.variant_collapsed,
                excluded_ac: value.excluded_ac,
                an: value.an,
                ac_hom: value.ac_hom,
                ac_het: value.ac_het,
                af_hom: value.af_hom,
                af_het: value.af_het,
                filters: value
                    .filters
                    .into_iter()
                    .map(|filter| {
                        GnomadMtdnaFilter::try_from(
                            pbs::gnomad::mtdna::Filter::try_from(filter)
                                .map_err(anyhow::Error::from)?,
                        )
                    })
                    .collect::<Result<_, _>>()?,
                mitotip_score: value.mitotip_score,
                mitotip_trna_prediction: value.mitotip_trna_prediction,
                pon_mt_trna_prediction: value.pon_mt_trna_prediction,
                pon_ml_probability_of_pathogenicity: value.pon_ml_probability_of_pathogenicity,
                vep: value.vep.into_iter().map(Into::into).collect(),
                quality_info: value.quality_info.map(Into::into),
                heteroplasmy_info: value.heteroplasmy_info.map(Into::into),
                filter_histograms: value.filter_histograms.map(Into::into),
                population_info: value.population_info.map(Into::into),
                haplogroup_info: value.haplogroup_info.map(Into::into),
                age_info: value.age_info.map(Into::into),
                depth_info: value.depth_info.map(Into::into),
            })
        }
    }

    /// Protocol buffer enum for site-level filters.
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        serde::Serialize,
        serde::Deserialize,
        strum::Display,
        strum::EnumString,
        utoipa::ToSchema,
    )]
    #[serde(rename_all = "snake_case")]
    #[strum(serialize_all = "snake_case")]
    pub enum GnomadMtdnaFilter {
        /// Variant overlaps site that is commonly reported in literature to be artifact prone.
        ArtifactProneSite,
        /// Allele where all samples with the variant call had at least 2 different heteroplasmic indels
        /// called at the position.
        IndelStack,
        /// No-pass-genotypes site (no individuals were PASS for the variant).
        NoPassGenotype,
    }

    impl TryFrom<pbs::gnomad::mtdna::Filter> for GnomadMtdnaFilter {
        type Error = anyhow::Error;

        fn try_from(value: pbs::gnomad::mtdna::Filter) -> Result<Self, Self::Error> {
            Ok(match value {
                pbs::gnomad::mtdna::Filter::ArtifactProneSite => {
                    GnomadMtdnaFilter::ArtifactProneSite
                }
                pbs::gnomad::mtdna::Filter::IndelStack => GnomadMtdnaFilter::IndelStack,
                pbs::gnomad::mtdna::Filter::NoPassGenotype => GnomadMtdnaFilter::NoPassGenotype,
                _ => anyhow::bail!("unknown gnomad::mtdna::Filter: {:?}", value),
            })
        }
    }

    /// A record corresponding to dbSNP VCF.
    #[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct DbsnpRecord {
        /// Chromosome name.
        pub chrom: String,
        /// 1-based start position.
        pub pos: i32,
        /// Reference allele.
        pub ref_allele: String,
        /// Alternate allele.
        pub alt_allele: String,
        /// The rs ID.
        pub rs_id: i32,
    }

    impl From<crate::pbs::dbsnp::Record> for DbsnpRecord {
        fn from(value: crate::pbs::dbsnp::Record) -> Self {
            DbsnpRecord {
                chrom: value.chrom,
                pos: value.pos,
                ref_allele: value.ref_allele,
                alt_allele: value.alt_allele,
                rs_id: value.rs_id,
            }
        }
    }

    /// A HelixMtDb record.
    #[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct HelixMtDbRecord {
        /// Chromosome name.
        pub chrom: String,
        /// 1-based start position.
        pub pos: i32,
        /// Reference allele.
        pub ref_allele: String,
        /// / Alternate allele.
        pub alt_allele: String,
        /// Total number of individuals.
        pub num_total: i32,
        /// Number of homoplasmic carriers.
        pub num_het: i32,
        /// Number of heteroplasmic carriers.
        pub num_hom: i32,
        /// Feature type.
        pub feature_type: String,
        /// Gene name.
        pub gene_name: String,
    }

    impl Into<HelixMtDbRecord> for crate::pbs::helixmtdb::Record {
        fn into(self) -> HelixMtDbRecord {
            HelixMtDbRecord {
                chrom: self.chrom,
                pos: self.pos,
                ref_allele: self.ref_allele,
                alt_allele: self.alt_allele,
                num_total: self.num_total,
                num_het: self.num_het,
                num_hom: self.num_hom,
                feature_type: self.feature_type,
                gene_name: self.gene_name,
            }
        }
    }

    /// A UCSC conservation record.
    #[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct UcscConservationRecord {
        /// Chromosome name.
        pub chrom: String,
        /// 1-based, inclusive start position.
        pub start: i32,
        /// 1-based, inclusive stop position.
        pub stop: i32,
        /// HGNC identifier.
        pub hgnc_id: String,
        /// ENST identifier.
        pub enst_id: String,
        /// Exon number (1-based).
        pub exon_num: i32,
        /// Exon count.
        pub exon_count: i32,
        /// Alignment.
        pub alignment: String,
    }

    impl From<crate::pbs::cons::Record> for UcscConservationRecord {
        fn from(value: crate::pbs::cons::Record) -> Self {
            UcscConservationRecord {
                chrom: value.chrom,
                start: value.start,
                stop: value.stop,
                hgnc_id: value.hgnc_id,
                enst_id: value.enst_id,
                exon_num: value.exon_num,
                exon_count: value.exon_count,
                alignment: value.alignment,
            }
        }
    }

    /// List of `Record`s.
    #[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct UcscConservationRecordList {
        /// The records in the list.
        pub records: Vec<UcscConservationRecord>,
    }

    impl From<crate::pbs::cons::RecordList> for UcscConservationRecordList {
        fn from(value: crate::pbs::cons::RecordList) -> Self {
            UcscConservationRecordList {
                records: value.records.into_iter().map(Into::into).collect(),
            }
        }
    }
    /// List of `ClinvarExtractedVcvRecord`s.
    #[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct ExtractedVcvRecordList {
        /// The list of VCV records that may share a global variant.
        pub records: Vec<ClinvarExtractedVcvRecord>,
    }

    impl TryFrom<crate::pbs::clinvar::minimal::ExtractedVcvRecordList> for ExtractedVcvRecordList {
        type Error = anyhow::Error;

        fn try_from(
            value: crate::pbs::clinvar::minimal::ExtractedVcvRecordList,
        ) -> Result<Self, Self::Error> {
            Ok(ExtractedVcvRecordList {
                records: value
                    .records
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?,
            })
        }
    }

    /// Annotation for a sinngle variant.
    #[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct SeqvarsAnnoResponseRecord {
        /// Annotations from CADD (TSV annotation file).
        pub cadd: Option<indexmap::IndexMap<String, serde_json::Value>>,
        /// Annotations from dbSNP.
        pub dbsnp: Option<DbsnpRecord>,
        /// Annotations from dbNSFP (TSV annotation file).
        pub dbnsfp: Option<indexmap::IndexMap<String, serde_json::Value>>,
        /// Annotations from dbscSNV.
        pub dbscsnv: Option<indexmap::IndexMap<String, serde_json::Value>>,
        /// Annotations from gnomAD-mtDNA.
        pub gnomad_mtdna: Option<GnomadMtdnaRecord>,
        /// Annotations from gnomAD-exomes.
        pub gnomad_exomes: Option<bool>,
        /// Annotations from gnomAD-genomes.
        pub gnomad_genomes: Option<bool>,
        /// Annotations from HelixMTdb.
        pub helixmtdb: Option<HelixMtDbRecord>,
        /// Annotations from UCSC conservation.
        pub ucsc_conservation: Option<UcscConservationRecordList>,
        /// Minimal extracted data from ClinVar.
        pub clinvar: Option<ExtractedVcvRecordList>,
    }

    /// Query response for `handle_with_openapi()`.
    #[derive(
        Debug,
        Default,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        utoipa::ToSchema,
        utoipa::ToResponse,
    )]
    pub struct SeqvarsAnnosResponse {
        /// The result records.
        pub result: SeqvarsAnnoResponseRecord,
    }
}

use response::*;

/// Query for annotations for a single variant.
#[utoipa::path(
    get,
    operation_id = "seqvarsAnosQuery",
    params(SeqvarsAnnosQuery),
    responses(
        (status = 200, description = "Annotation for a single variant.", body = SeqvarsAnnosResponse),
        (status = 500, description = "Internal server error.", body = CustomError)
    )
)]
#[get("/api/v1/genes/info")]
pub async fn handle_with_openapi(
    data: Data<crate::server::run::WebServerData>,
    _path: Path<()>,
    query: web::Query<SeqvarsAnnosQuery>,
) -> actix_web::Result<Json<SeqvarsAnnosResponse>, CustomError> {
    let genome_release = query
        .genome_release
        .parse()
        .map_err(|e: strum::ParseError| {
            CustomError::new(anyhow::anyhow!("problem getting genome release: {}", e))
        })?;

    fn json_value_to_indexmap(
        value: serde_json::Value,
    ) -> Result<indexmap::IndexMap<String, serde_json::Value>, CustomError> {
        value
            .as_object()
            .map(|v| {
                Ok(v.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect::<indexmap::IndexMap<_, _>>())
            })
            .unwrap_or_else(|| Err(CustomError::new(anyhow::anyhow!("expected object"))))
    }

    let result = SeqvarsAnnoResponseRecord {
        cadd: data.annos[genome_release][AnnoDb::Cadd]
            .as_ref()
            .map(|db| {
                fetch_var_tsv_json(
                    &db.data,
                    AnnoDb::Cadd.cf_name(),
                    query.clone().into_inner().into(),
                )
            })
            .transpose()?
            .flatten()
            .map(json_value_to_indexmap)
            .transpose()?,
        dbsnp: data.annos[genome_release][AnnoDb::Dbsnp]
            .as_ref()
            .map(|db| {
                fetch_var_protobuf::<crate::dbsnp::pbs::Record>(
                    &db.data,
                    AnnoDb::Dbsnp.cf_name(),
                    query.clone().into_inner().into(),
                )
            })
            .transpose()?
            .flatten()
            .map(Into::into),
        dbnsfp: data.annos[genome_release][AnnoDb::Dbnsfp]
            .as_ref()
            .map(|db| {
                fetch_var_tsv_json(
                    &db.data,
                    AnnoDb::Dbnsfp.cf_name(),
                    query.clone().into_inner().into(),
                )
            })
            .transpose()?
            .flatten()
            .map(json_value_to_indexmap)
            .transpose()?,
        dbscsnv: data.annos[genome_release][AnnoDb::Dbscsnv]
            .as_ref()
            .map(|db| {
                fetch_var_tsv_json(
                    &db.data,
                    AnnoDb::Dbscsnv.cf_name(),
                    query.clone().into_inner().into(),
                )
            })
            .transpose()?
            .flatten()
            .map(json_value_to_indexmap)
            .transpose()?,
        gnomad_mtdna: data.annos[genome_release][AnnoDb::GnomadMtdna]
            .as_ref()
            .map(|db| {
                fetch_var_protobuf::<crate::pbs::gnomad::mtdna::Record>(
                    &db.data,
                    AnnoDb::GnomadMtdna.cf_name(),
                    query.clone().into_inner().into(),
                )?
                .map(TryInto::<GnomadMtdnaRecord>::try_into)
                .transpose()
                .map_err(|e| CustomError::new(e))
            })
            .transpose()?
            .flatten()
            .map(Into::into),
        // gnomad_exomes: Option<bool>,
        // gnomad_genomes: Option<bool>,
        helixmtdb: data.annos[genome_release][AnnoDb::Helixmtdb]
            .as_ref()
            .map(|db| {
                Ok(fetch_var_protobuf::<crate::pbs::helixmtdb::Record>(
                    &db.data,
                    AnnoDb::Helixmtdb.cf_name(),
                    query.clone().into_inner().into(),
                )?
                .map(Into::into))
            })
            .transpose()?
            .flatten(),
        ucsc_conservation: data.annos[genome_release][AnnoDb::UcscConservation]
            .as_ref()
            .map(|db| {
                let start: keys::Pos = query.clone().into_inner().into();
                let start = keys::Pos {
                    chrom: start.chrom,
                    pos: start.pos - 2,
                };
                let stop = query.clone().into_inner().into();
                Ok(fetch_pos_protobuf::<crate::pbs::cons::RecordList>(
                    &db.data,
                    AnnoDb::UcscConservation.cf_name(),
                    start,
                    stop,
                )?
                .into_iter()
                .next()
                .map(Into::into))
            })
            .transpose()?
            .flatten(),
        clinvar: data.annos[genome_release][AnnoDb::Clinvar]
            .as_ref()
            .map(|db| {
                fetch_var_protobuf::<crate::pbs::clinvar::minimal::ExtractedVcvRecordList>(
                    &db.data,
                    AnnoDb::Clinvar.cf_name(),
                    query.clone().into_inner().into(),
                )?
                .map(TryInto::<ExtractedVcvRecordList>::try_into)
                .transpose()
                .map_err(|e| CustomError::new(e))
            })
            .transpose()?
            .flatten(),
        ..Default::default()
    };

    Ok(Json(SeqvarsAnnosResponse { result }))
}
