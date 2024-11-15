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
pub struct GenesClinvarQuery {
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
    use crate::server::run::clinvar_data::ClinvarExtractedVcvRecord;

    /// Extracted variants per release.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesExtractedVariantsPerRelease {
        /// Release version.
        pub release: Option<String>,
        /// Variants per gene.
        pub variants: Vec<ClinvarExtractedVcvRecord>,
    }

    impl TryFrom<pbs::clinvar::per_gene::ExtractedVariantsPerRelease>
        for GenesExtractedVariantsPerRelease
    {
        type Error = anyhow::Error;

        fn try_from(
            value: pbs::clinvar::per_gene::ExtractedVariantsPerRelease,
        ) -> Result<Self, Self::Error> {
            let variants = value
                .variants
                .into_iter()
                .map(ClinvarExtractedVcvRecord::try_from)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(GenesExtractedVariantsPerRelease {
                release: value.release,
                variants,
            })
        }
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesCoarseClinsigFrequencyCounts {
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
        for GenesCoarseClinsigFrequencyCounts
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
    pub enum GenesGeneImpact {
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

    impl TryFrom<pbs::clinvar_data::gene_impact::GeneImpact> for GenesGeneImpact {
        type Error = anyhow::Error;

        fn try_from(
            value: pbs::clinvar_data::gene_impact::GeneImpact,
        ) -> Result<Self, Self::Error> {
            match value {
                pbs::clinvar_data::gene_impact::GeneImpact::Unspecified => {
                    Ok(GenesGeneImpact::Unspecified)
                }
                pbs::clinvar_data::gene_impact::GeneImpact::ThreePrimeUtrVariant => {
                    Ok(GenesGeneImpact::ThreePrimeUtrVariant)
                }
                pbs::clinvar_data::gene_impact::GeneImpact::FivePrimeUtrVariant => {
                    Ok(GenesGeneImpact::FivePrimeUtrVariant)
                }
                pbs::clinvar_data::gene_impact::GeneImpact::DownstreamTranscriptVariant => {
                    Ok(GenesGeneImpact::DownstreamTranscriptVariant)
                }
                pbs::clinvar_data::gene_impact::GeneImpact::FrameshiftVariant => {
                    Ok(GenesGeneImpact::FrameshiftVariant)
                }
                pbs::clinvar_data::gene_impact::GeneImpact::InframeIndel => {
                    Ok(GenesGeneImpact::InframeIndel)
                }
                pbs::clinvar_data::gene_impact::GeneImpact::StartLost => {
                    Ok(GenesGeneImpact::StartLost)
                }
                pbs::clinvar_data::gene_impact::GeneImpact::IntronVariant => {
                    Ok(GenesGeneImpact::IntronVariant)
                }
                pbs::clinvar_data::gene_impact::GeneImpact::MissenseVariant => {
                    Ok(GenesGeneImpact::MissenseVariant)
                }
                pbs::clinvar_data::gene_impact::GeneImpact::NonCodingTranscriptVariant => {
                    Ok(GenesGeneImpact::NonCodingTranscriptVariant)
                }
                pbs::clinvar_data::gene_impact::GeneImpact::StopGained => {
                    Ok(GenesGeneImpact::StopGained)
                }
                pbs::clinvar_data::gene_impact::GeneImpact::NoSequenceAlteration => {
                    Ok(GenesGeneImpact::NoSequenceAlteration)
                }
                pbs::clinvar_data::gene_impact::GeneImpact::SpliceAcceptorVariant => {
                    Ok(GenesGeneImpact::SpliceAcceptorVariant)
                }
                pbs::clinvar_data::gene_impact::GeneImpact::SpliceDonorVariant => {
                    Ok(GenesGeneImpact::SpliceDonorVariant)
                }
                pbs::clinvar_data::gene_impact::GeneImpact::StopLost => {
                    Ok(GenesGeneImpact::StopLost)
                }
                pbs::clinvar_data::gene_impact::GeneImpact::SynonymousVariant => {
                    Ok(GenesGeneImpact::SynonymousVariant)
                }
                pbs::clinvar_data::gene_impact::GeneImpact::UpstreamTranscriptVariant => {
                    Ok(GenesGeneImpact::UpstreamTranscriptVariant)
                }
            }
        }
    }

    /// Stores the counts for a gene impact.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesImpactCounts {
        /// The gene impact.
        pub gene_impact: GenesGeneImpact,
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

    impl TryFrom<pbs::clinvar_data::gene_impact::gene_impact_counts::ImpactCounts>
        for GenesImpactCounts
    {
        type Error = anyhow::Error;

        fn try_from(
            value: pbs::clinvar_data::gene_impact::gene_impact_counts::ImpactCounts,
        ) -> Result<Self, Self::Error> {
            Ok(GenesImpactCounts {
                gene_impact: GenesGeneImpact::try_from(
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
    pub struct GenesGeneImpactCounts {
        /// The gene HGNC ID.
        pub hgnc_id: String,
        /// The impact counts.
        pub impact_counts: Vec<GenesImpactCounts>,
    }

    impl TryFrom<pbs::clinvar_data::gene_impact::GeneImpactCounts> for GenesGeneImpactCounts {
        type Error = anyhow::Error;

        fn try_from(
            value: pbs::clinvar_data::gene_impact::GeneImpactCounts,
        ) -> Result<Self, Self::Error> {
            let impact_counts = value
                .impact_counts
                .into_iter()
                .map(GenesImpactCounts::try_from)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(GenesGeneImpactCounts {
                hgnc_id: value.hgnc_id,
                impact_counts,
            })
        }
    }

    /// ClinVar detailed information per gene.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesClinvarPerGeneRecord {
        /// Counts of variants per impact
        pub per_impact_counts: Option<GenesGeneImpactCounts>,
        /// Counts of variants per impact / frequency
        pub per_freq_counts: Option<GenesCoarseClinsigFrequencyCounts>,
        /// Variants for the given gene.
        pub per_release_vars: Vec<GenesExtractedVariantsPerRelease>,
    }

    impl TryFrom<pbs::clinvar::per_gene::ClinvarPerGeneRecord> for GenesClinvarPerGeneRecord {
        type Error = anyhow::Error;

        fn try_from(
            value: pbs::clinvar::per_gene::ClinvarPerGeneRecord,
        ) -> Result<Self, Self::Error> {
            let per_impact_counts = value
                .per_impact_counts
                .map(GenesGeneImpactCounts::try_from)
                .transpose()?;
            let per_freq_counts = value
                .per_freq_counts
                .map(GenesCoarseClinsigFrequencyCounts::try_from)
                .transpose()?;
            let per_release_vars = value
                .per_release_vars
                .into_iter()
                .map(GenesExtractedVariantsPerRelease::try_from)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(GenesClinvarPerGeneRecord {
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
        pub record: GenesClinvarPerGeneRecord,
    }

    impl TryFrom<(String, pbs::clinvar::per_gene::ClinvarPerGeneRecord)> for GenesClinvarResponseEntry {
        type Error = anyhow::Error;

        fn try_from(
            (hgnc_id, record): (String, pbs::clinvar::per_gene::ClinvarPerGeneRecord),
        ) -> Result<Self, Self::Error> {
            let record = GenesClinvarPerGeneRecord::try_from(record)?;
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
                .map(|(hgnc_id, record)| -> Result<_, anyhow::Error> {
                    Ok(GenesClinvarResponseEntry {
                        hgnc_id,
                        record: record.try_into()?,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
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
