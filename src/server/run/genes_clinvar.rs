//! Code for `/genes/clinvar`.

use actix_web::{
    get,
    web::{self, Data, Json, Path},
};
use prost::Message;

use crate::pbs::clinvar::per_gene::ClinvarPerGeneRecord;

use super::error::CustomError;
use serde_with::{formats::CommaSeparator, StringWithSeparator};

/// Parameters for `handle`.
#[serde_with::skip_serializing_none]
#[serde_with::serde_as]
#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema, utoipa::IntoParams,
)]
#[serde(rename_all = "snake_case")]
struct GenesClinvarQuery {
    /// The HGNC IDs to search for.
    #[serde_as(as = "Option<StringWithSeparator::<CommaSeparator, String>>")]
    pub hgnc_id: Option<Vec<String>>,
}

/// Result for `handle`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde_with::skip_serializing_none]
struct Container {
    // TODO: add data version
    /// The resulting per-gene ClinVar information.
    pub genes: indexmap::IndexMap<String, ClinvarPerGeneRecord>,
}

/// Implementation of both endpoints.
async fn handle_impl(
    data: Data<crate::server::run::WebServerData>,
    _path: Path<()>,
    query: web::Query<GenesClinvarQuery>,
) -> actix_web::Result<Container, CustomError> {
    let genes_db = data.genes.as_ref().ok_or(CustomError::new(anyhow::anyhow!(
        "genes database not available"
    )))?;
    let db_clinvar = genes_db
        .data
        .db_clinvar
        .as_ref()
        .ok_or(CustomError::new(anyhow::anyhow!(
            "clinvar-genes database not available"
        )))?;
    let cf_genes = db_clinvar
        .cf_handle("clinvar-genes")
        .expect("no 'clinvar-genes' column family");
    let mut genes = indexmap::IndexMap::new();
    if let Some(hgnc_id) = query.hgnc_id.as_ref() {
        for hgnc_id in hgnc_id {
            if let Some(raw_buf) = db_clinvar.get_cf(&cf_genes, hgnc_id).map_err(|e| {
                CustomError::new(anyhow::anyhow!("problem querying database: {}", e))
            })? {
                let record = crate::pbs::clinvar::per_gene::ClinvarPerGeneRecord::decode(
                    std::io::Cursor::new(raw_buf),
                )
                .map_err(|e| CustomError::new(anyhow::anyhow!("problem decoding value: {}", e)))?;
                genes.insert(hgnc_id.to_string(), record);
            } else {
                tracing::debug!("no such gene: {}", hgnc_id);
            }
        }
    }

    let cf_meta = db_clinvar
        .cf_handle("meta")
        .expect("no 'meta' column family");
    let raw_builder_version = &db_clinvar
        .get_cf(&cf_meta, "annonars-version")
        .map_err(|e| CustomError::new(anyhow::anyhow!("problem querying database: {}", e)))?
        .expect("database missing 'annonars-version' key?");
    let _builder_version = std::str::from_utf8(raw_builder_version)
        .map_err(|e| CustomError::new(anyhow::anyhow!("problem decoding value: {}", e)))?
        .to_string();

    Ok(Container { genes })
}

/// Query for ClinVar information for one or more genes.
#[get("/genes/clinvar")]
async fn handle(
    data: Data<crate::server::run::WebServerData>,
    _path: Path<()>,
    query: web::Query<GenesClinvarQuery>,
) -> actix_web::Result<Json<Container>, CustomError> {
    Ok(Json(handle_impl(data, _path, query).await?))
}

/// Types used in the response.
pub(crate) mod response {
    use crate::pbs;

    /// Accession with version.
    pub struct VersionedAccession {
        /// The accession.
        pub accession: String,
        /// The version.
        pub version: i32,
    }

    impl From<pbs::clinvar_data::extracted_vars::VersionedAccession> for VersionedAccession {
        fn from(value: pbs::clinvar_data::extracted_vars::VersionedAccession) -> Self {
            Self {
                accession: value.accession,
                version: value.version,
            }
        }
    }

    /// Local type for ClassifiedConditionList.
    ///
    /// nested elements
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct ClassifiedConditionList {
        /// List of interpreted conditions.
        pub classified_conditions: ::prost::alloc::vec::Vec<super::ClassifiedCondition>,
        /// Trait set ID.
        pub trait_set_id: Option<i64>,
    }

    /// Local type for GermlineClassification.
    ///
    /// The aggregate review status based on
    /// all germline submissions for this record.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GermlineClassification {
        /// The aggregate review status based on
        /// all somatic clinical impact submissions for this
        /// record.
        pub review_status: i32,
        /// The oncogenicity description.
        pub description: Option<germline_classification::Description>,
    }

    /// Nested message and enum types in `GermlineClassification`.
    pub mod germline_classification {
        /// Local type for Description.
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
        pub struct Description {
            /// The description.
            pub value: String,
            /// The date of the description.
            pub date_last_evaluated: Option<::pbjson_types::Timestamp>,
            /// The number of submissions.
            pub submission_count: Option<u32>,
        }
    }
    /// Local type for SomaticClinicalImpact.
    ///
    /// The aggregate review status based on
    /// all somatic clinical impact submissions for this
    /// record.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct SomaticClinicalImpact {
        /// The aggregate review status based on
        /// all somatic clinical impact submissions for this
        /// record.
        pub review_status: i32,
        /// The oncogenicity description.
        pub descriptions: ::prost::alloc::vec::Vec<somatic_clinical_impact::Description>,
    }

    /// Local type for Description.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct Description {
        /// The description.
        pub value: String,
        /// Clinical impact assertion type.
        pub clinical_impact_assertion_type: Option<String>,
        /// Clinical impact significance
        pub clinical_impact_clinical_significance: Option<String>,
        /// The date of the description.
        pub date_last_evaluated: Option<::pbjson_types::Timestamp>,
        /// The number of submissions.
        pub submission_count: Option<u32>,
    }

    /// Local type for OncogenicityClassification.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct OncogenicityClassification {
        /// The aggregate review status based on
        /// all oncogenic submissions for this record.
        pub review_status: i32,
        /// The oncogenicity description.
        pub description: Option<oncogenicity_classification::Description>,
    }

    /// Local type for Description.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct Description {
        /// The description.
        pub value: String,
        /// The date of the description.
        pub date_last_evaluated: Option<::pbjson_types::Timestamp>,
        /// The number of submissions.
        pub submission_count: Option<u32>,
    }
    /// Local type for RCV classifications.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct RcvClassifications {
        /// Germline classification.
        pub germline_classification: Option<GermlineClassification>,
        /// Somatic clinical impact.
        pub somatic_clinical_impact: Option<SomaticClinicalImpact>,
        /// Oncogenicity classification.
        pub oncogenicity_classification: Option<OncogenicityClassification>,
    }

    /// Protocol buffer for storing essential information of one RCV.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct ExtractedRcvRecord {
        /// The accession.
        pub accession: Option<VersionedAccession>,
        /// Title of RCV.
        pub title: String,
        /// Classifications (thinned out).
        pub classifications: Option<RcvClassifications>,
    }

    /// Protocol buffer for storing essential information of one VCV.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct ExtractedVcvRecord {
        /// The accession.
        pub accession: Option<VersionedAccession>,
        /// List of aggregated RCVs.
        pub rcvs: Vec<ExtractedRcvRecord>,
        /// Name of VCV.
        pub name: String,
        /// The type of the variant.
        pub variation_type: i32,
        /// Classifications (thinned out).
        pub classifications: Option<super::clinvar_public::AggregateClassificationSet>,
        /// Clinical assertions (thinned out),
        pub clinical_assertions: Vec<super::clinvar_public::ClinicalAssertion>,
        /// The sequence location on one reference.
        pub sequence_location: Option<super::clinvar_public::location::SequenceLocation>,
        /// List of HGNC IDs.
        pub hgnc_ids: Vec<String>,
    }

    /// Enumeration for the type of the variant.
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
        utoipa::ToSchema,
    )]
    pub enum VariationType {
        /// Corresponds to "insertion".
        Insertion,
        /// Corresponds to "deletion".
        Deletion,
        /// Corresponds to "single nucleotide variant".
        Snv,
        /// Corresponds to "indel".
        Indel,
        /// Corresponds to "duplication".
        Duplication,
        /// Corresponds to "tandem duplication".
        TandemDuplication,
        /// Corresponds to "structural variant".
        StructuralVariant,
        /// Corresponds to "copy number gain".
        CopyNumberGain,
        /// Corresponds to "copy number loss".
        CopyNumberLoss,
        /// Corresponds to "protein only".
        ProteinOnly,
        /// Corresponds to "microsatellite".
        Microsatellite,
        /// Corresponds to "inversion".
        Inversion,
        /// Corresponds to "other".
        Other,
    }

    impl TryFrom<pbs::clinvar_data::extracted_vars::VariationType> for VariationType {
        type Error = anyhow::Error;

        fn try_from(
            value: pbs::clinvar_data::extracted_vars::VariationType,
        ) -> Result<Self, Self::Error> {
            match value {
                pbs::clinvar_data::extracted_vars::VariationType::Insertion => {
                    Ok(VariationType::Insertion)
                }
                pbs::clinvar_data::extracted_vars::VariationType::Deletion => {
                    Ok(VariationType::Deletion)
                }
                pbs::clinvar_data::extracted_vars::VariationType::Snv => Ok(VariationType::Snv),
                pbs::clinvar_data::extracted_vars::VariationType::Indel => Ok(VariationType::Indel),
                pbs::clinvar_data::extracted_vars::VariationType::Duplication => {
                    Ok(VariationType::Duplication)
                }
                pbs::clinvar_data::extracted_vars::VariationType::TandemDuplication => {
                    Ok(VariationType::TandemDuplication)
                }
                pbs::clinvar_data::extracted_vars::VariationType::StructuralVariant => {
                    Ok(VariationType::StructuralVariant)
                }
                pbs::clinvar_data::extracted_vars::VariationType::CopyNumberGain => {
                    Ok(VariationType::CopyNumberGain)
                }
                pbs::clinvar_data::extracted_vars::VariationType::CopyNumberLoss => {
                    Ok(VariationType::CopyNumberLoss)
                }
                pbs::clinvar_data::extracted_vars::VariationType::ProteinOnly => {
                    Ok(VariationType::ProteinOnly)
                }
                pbs::clinvar_data::extracted_vars::VariationType::Microsatellite => {
                    Ok(VariationType::Microsatellite)
                }
                pbs::clinvar_data::extracted_vars::VariationType::Inversion => {
                    Ok(VariationType::Inversion)
                }
                pbs::clinvar_data::extracted_vars::VariationType::Other => Ok(VariationType::Other),
                _ => Err(anyhow::anyhow!("unknown value: {:?}", value)),
            }
        }
    }

    /// Extracted variants per release.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct ExtractedVariantsPerRelease {
        /// Release version.
        pub release: Option<String>,
        /// Variants per gene.
        pub variants: Vec<ExtractedVcvRecord>,
    }

    impl TryFrom<pbs::clinvar::per_gene::ExtractedVariantsPerRelease>
        for ExtractedVariantsPerRelease
    {
        type Error = anyhow::Error;

        fn try_from(
            value: pbs::clinvar::per_gene::ExtractedVariantsPerRelease,
        ) -> Result<Self, Self::Error> {
            let variants = value
                .variants
                .into_iter()
                .map(ExtractedVcvRecord::try_from)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ExtractedVariantsPerRelease {
                release: value.release,
                variants,
            })
        }
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GeneCoarseClinsigFrequencyCounts {
        /// The gene HGNC ID.
        pub hgnc_id: String,
        /// The counts for (likely) pathogenic.
        pub pathogenic_counts: Vec<u32>,
        /// The counts for uncertain significance.
        pub uncertain_counts: Vec<u32>,
        /// The counts for (likely) benign.
        pub benign_counts: Vec<u32>,
    }

    impl From<pbs::clinvar_data::class_by_freq::GeneCoarseClinsigFrequencyCounts>
        for GeneCoarseClinsigFrequencyCounts
    {
        fn from(value: pbs::clinvar_data::class_by_freq::GeneCoarseClinsigFrequencyCounts) -> Self {
            Self {
                hgnc_id: value.hgnc_id,
                pathogenic_counts: value.pathogenic_counts,
                uncertain_counts: value.uncertain_counts,
                benign_counts: value.benign_counts,
            }
        }
    }

    /// Enumeration for coarse-grain classification.
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
        utoipa::ToSchema,
    )]
    pub enum CoarseClinicalSignificance {
        /// Corresponds to "benign".
        Benign,
        /// Corresponds to "uncertain".
        Uncertain,
        /// Corresponds to "pathogenic".
        Pathogenic,
    }

    impl TryFrom<pbs::clinvar_data::class_by_freq::CoarseClinicalSignificance>
        for CoarseClinicalSignificance
    {
        type Error = anyhow::Error;

        fn try_from(
            value: pbs::clinvar_data::class_by_freq::CoarseClinicalSignificance,
        ) -> Result<Self, Self::Error> {
            match value {
                pbs::clinvar_data::class_by_freq::CoarseClinicalSignificance::Benign => {
                    Ok(CoarseClinicalSignificance::Benign)
                }
                pbs::clinvar_data::class_by_freq::CoarseClinicalSignificance::Uncertain => {
                    Ok(CoarseClinicalSignificance::Uncertain)
                }
                pbs::clinvar_data::class_by_freq::CoarseClinicalSignificance::Pathogenic => {
                    Ok(CoarseClinicalSignificance::Pathogenic)
                }
                _ => Err(anyhow::anyhow!("unknown value: {:?}", value)),
            }
        }
    }

    /// Enumeration with the variant consequence.
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
        utoipa::ToSchema,
    )]
    pub enum GeneImpact {
        /// unspecified impact
        Unspecified,
        /// Corresponds to "3_prime_UTR_variant"
        ThreePrimeUtrVariant,
        /// Corresponds to "5_prime_UTR_variant"
        FivePrimeUtrVariant,
        /// Corresponds to "downstream_gene_variant"
        DownstreamTranscriptVariant,
        /// Corresponds to "frameshift_variant"
        FrameshiftVariant,
        /// Corresponds to "inframe_indel"
        InframeIndel,
        /// Corresponds to "start_lost"
        StartLost,
        /// Corresponds to "intron_variant"
        IntronVariant,
        /// Corresponds to "missense_variant"
        MissenseVariant,
        /// Corresponds to "non_codnig_transcript_variant"
        NonCodingTranscriptVariant,
        /// Corresponds to "stop_gained"
        StopGained,
        /// Corresponds to "no_sequence_alteration"
        NoSequenceAlteration,
        /// Corresponds to "splice_acceptor_variant"
        SpliceAcceptorVariant,
        /// Corresponds to "splice_donor_variant"
        SpliceDonorVariant,
        /// Corresponds to "stop_lost"
        StopLost,
        /// Corresponds to "synonymous_variant"
        SynonymousVariant,
        /// Corresponds to "upstream_gene_variant"
        UpstreamTranscriptVariant,
    }

    impl TryFrom<pbs::clinvar_data::gene_impact::GeneImpact> for GeneImpact {
        type Error = anyhow::Error;

        fn try_from(
            value: pbs::clinvar_data::gene_impact::GeneImpact,
        ) -> Result<Self, Self::Error> {
            match value {
                pbs::clinvar_data::gene_impact::GeneImpact::Unspecified => {
                    Ok(GeneImpact::Unspecified)
                }
                pbs::clinvar_data::gene_impact::GeneImpact::ThreePrimeUtrVariant => {
                    Ok(GeneImpact::ThreePrimeUtrVariant)
                }
                pbs::clinvar_data::gene_impact::GeneImpact::FivePrimeUtrVariant => {
                    Ok(GeneImpact::FivePrimeUtrVariant)
                }
                pbs::clinvar_data::gene_impact::GeneImpact::DownstreamTranscriptVariant => {
                    Ok(GeneImpact::DownstreamTranscriptVariant)
                }
                pbs::clinvar_data::gene_impact::GeneImpact::FrameshiftVariant => {
                    Ok(GeneImpact::FrameshiftVariant)
                }
                pbs::clinvar_data::gene_impact::GeneImpact::InframeIndel => {
                    Ok(GeneImpact::InframeIndel)
                }
                pbs::clinvar_data::gene_impact::GeneImpact::StartLost => Ok(GeneImpact::StartLost),
                pbs::clinvar_data::gene_impact::GeneImpact::IntronVariant => {
                    Ok(GeneImpact::IntronVariant)
                }
                pbs::clinvar_data::gene_impact::GeneImpact::MissenseVariant => {
                    Ok(GeneImpact::MissenseVariant)
                }
                pbs::clinvar_data::gene_impact::GeneImpact::NonCodingTranscriptVariant => {
                    Ok(GeneImpact::NonCodingTranscriptVariant)
                }
                pbs::clinvar_data::gene_impact::GeneImpact::StopGained => {
                    Ok(GeneImpact::StopGained)
                }
                pbs::clinvar_data::gene_impact::GeneImpact::NoSequenceAlteration => {
                    Ok(GeneImpact::NoSequenceAlteration)
                }
                pbs::clinvar_data::gene_impact::GeneImpact::SpliceAcceptorVariant => {
                    Ok(GeneImpact::SpliceAcceptorVariant)
                }
                pbs::clinvar_data::gene_impact::GeneImpact::SpliceDonorVariant => {
                    Ok(GeneImpact::SpliceDonorVariant)
                }
                pbs::clinvar_data::gene_impact::GeneImpact::StopLost => Ok(GeneImpact::StopLost),
                pbs::clinvar_data::gene_impact::GeneImpact::SynonymousVariant => {
                    Ok(GeneImpact::SynonymousVariant)
                }
                pbs::clinvar_data::gene_impact::GeneImpact::UpstreamTranscriptVariant => {
                    Ok(GeneImpact::UpstreamTranscriptVariant)
                }
            }
        }
    }

    /// Stores the counts for a gene impact.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct ImpactCounts {
        /// The gene impact.
        pub gene_impact: GeneImpact,
        /// The counts for the benign impact.
        pub count_benign: u32,
        /// The counts for the likely benign impact.
        pub count_likely_benign: u32,
        /// The counts for the uncertain significance impact.
        pub count_uncertain_significance: u32,
        /// The counts for the likely pathogenic impact.
        pub count_likely_pathogenic: u32,
        /// The counts for the pathogenic impact.
        pub count_pathogenic: u32,
    }

    impl TryFrom<pbs::clinvar_data::gene_impact::gene_impact_counts::ImpactCounts> for ImpactCounts {
        type Error = anyhow::Error;

        fn try_from(
            value: pbs::clinvar_data::gene_impact::gene_impact_counts::ImpactCounts,
        ) -> Result<Self, Self::Error> {
            Ok(ImpactCounts {
                gene_impact: GeneImpact::try_from(
                    pbs::clinvar_data::gene_impact::GeneImpact::try_from(value.gene_impact)?,
                )?,
                count_benign: value.count_benign,
                count_likely_benign: value.count_likely_benign,
                count_uncertain_significance: value.count_uncertain_significance,
                count_likely_pathogenic: value.count_likely_pathogenic,
                count_pathogenic: value.count_pathogenic,
            })
        }
    }

    /// Entry for storing counts of `GeneImpact` and `ClinicalSignificance`.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GeneImpactCounts {
        /// The gene HGNC ID.
        pub hgnc_id: String,
        /// The impact counts.
        pub impact_counts: Vec<ImpactCounts>,
    }

    impl TryFrom<pbs::clinvar_data::gene_impact::GeneImpactCounts> for GeneImpactCounts {
        type Error = anyhow::Error;

        fn try_from(
            value: pbs::clinvar_data::gene_impact::GeneImpactCounts,
        ) -> Result<Self, Self::Error> {
            let impact_counts = value
                .impact_counts
                .into_iter()
                .map(ImpactCounts::try_from)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(GeneImpactCounts {
                hgnc_id: value.hgnc_id,
                impact_counts,
            })
        }
    }

    /// ClinVar detailed information per gene.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct ClinvarPerGeneRecord {
        /// Counts of variants per impact
        pub per_impact_counts: Option<GeneImpactCounts>,
        /// Counts of variants per impact / frequency
        pub per_freq_counts: Option<GeneCoarseClinsigFrequencyCounts>,
        /// Variants for the given gene.
        pub per_release_vars: Vec<ExtractedVariantsPerRelease>,
    }

    impl TryFrom<pbs::clinvar::per_gene::ClinvarPerGeneRecord> for ClinvarPerGeneRecord {
        type Error = anyhow::Error;

        fn try_from(
            value: pbs::clinvar::per_gene::ClinvarPerGeneRecord,
        ) -> Result<Self, Self::Error> {
            let per_impact_counts = value
                .per_impact_counts
                .map(GeneImpactCounts::try_from)
                .transpose()?;
            let per_freq_counts = value
                .per_freq_counts
                .map(GeneCoarseClinsigFrequencyCounts::try_from)
                .transpose()?;
            let per_release_vars = value
                .per_release_vars
                .into_iter()
                .map(ExtractedVariantsPerRelease::try_from)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ClinvarPerGeneRecord {
                per_impact_counts,
                per_freq_counts,
                per_release_vars,
            })
        }
    }

    /// One entry in the result.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesClinvarResponseEntry {
        /// HGNC ID of the gene.
        pub hgnc_id: String,
        /// The resulting per-gene record.
        pub record: ClinvarPerGeneRecord,
    }

    impl TryFrom<(String, pbs::clinvar::per_gene::ClinvarPerGeneRecord)> for GenesClinvarResponseEntry {
        type Error = anyhow::Error;

        fn try_from(
            (hgnc_id, record): (String, pbs::clinvar::per_gene::ClinvarPerGeneRecord),
        ) -> Result<Self, Self::Error> {
            let record = ClinvarPerGeneRecord::try_from(record)?;
            Ok(GenesClinvarResponseEntry { hgnc_id, record })
        }
    }

    /// Result for `handle_with_openapi`.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    #[serde_with::skip_serializing_none]
    pub struct GenesClinvarResponse {
        /// The resulting per-gene ClinVar information.
        pub genes: Vec<GenesClinvarResponseEntry>,
    }

    impl TryFrom<super::Container> for GenesClinvarResponse {
        type Error = anyhow::Error;

        fn try_from(container: super::Container) -> Result<Self, Self::Error> {
            let genes = container
                .genes
                .into_iter()
                .map(|(hgnc_id, record)| GenesClinvarResponseEntry { hgnc_id, record })
                .collect();
            Ok(GenesClinvarResponse { genes })
        }
    }
}

use response::*;

/// Query for ClinVar information for one or more genes.
#[utoipa::path(
    get,
    operation_id = "genesInfo",
    params(GenesClinvarQuery),
    responses(
        (status = 200, description = "Per-gene ClinVar information.", body = GenesClinvarResponse),
        (status = 500, description = "Internal server error.", body = CustomError)
    )
)]
#[get("/api/v1/genes/info")]
async fn handle_with_openapi(
    data: Data<crate::server::run::WebServerData>,
    _path: Path<()>,
    query: web::Query<GenesClinvarQuery>,
) -> actix_web::Result<Json<GenesClinvarResponse>, CustomError> {
    let container = handle_impl(data, _path, query).await?;
    let response = container
        .try_into()
        .map_err(|e| CustomError::new(anyhow::anyhow!("Failed to convert response: {}", e)))?;
    Ok(Json(response))
}
