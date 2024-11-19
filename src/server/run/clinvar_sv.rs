//! Implementation of endpoint `/api/v1/strucvars/clinvar/query`.
//!
//! Also includes the implementation of the `/clinvar-sv/query` endpoint (deprecated).

use actix_web::{
    get,
    web::{self, Data, Json, Path},
};

use crate::common::{cli::GenomeRelease, spdi};

use super::error::CustomError;
use serde_with::{formats::CommaSeparator, StringWithSeparator};

use crate::pbs::clinvar_data::extracted_vars::VariationType as PbVariationType;
use crate::server::run::clinvar_data::ClinvarExtractedVariationType;

/// The default page size to use.
const DEFAULT_PAGE_SIZE: u32 = 100;
/// The default minimal overlap.
const DEFAULT_MIN_OVERLAP: f64 = 0.5;

/// Parameters for `handle()`.
///
/// We use `camelCase` for exposition on the REST API and consistency with
/// JSON serialized protobufs.
#[serde_with::skip_serializing_none]
#[serde_with::serde_as]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Request {
    /// Genome release specification.
    pub genome_release: String,
    /// Chromosome name.
    pub chromosome: String,
    /// 1-based start position.
    pub start: u32,
    /// 1-based stop postion.
    pub stop: u32,
    /// Optionally, the variant types.
    #[serde_as(as = "Option<StringWithSeparator::<CommaSeparator, PbVariationType>>")]
    pub variation_types: Option<Vec<PbVariationType>>,
    /// Optionally, minimal overlap.
    pub min_overlap: Option<f64>,
    /// Optional 1-based page number.
    pub page_no: Option<u32>,
    /// Optional page size.
    pub page_size: Option<u32>,
}

/// Compute reciprocal overlap between two ranges.
fn reciprocal_overlap<T>(lhs: &std::ops::Range<T>, rhs: &std::ops::Range<T>) -> f64
where
    T: std::cmp::Ord + std::ops::Sub<Output = T> + std::ops::Add<Output = T> + Copy + Into<f64>,
{
    // bail out if the intervals don't overlap
    if lhs.end <= rhs.start || rhs.end <= lhs.start {
        return 0.0;
    }
    // otherwise, compute and return reciprocal overlap
    let len_lhs = lhs.end - lhs.start;
    let len_rhs = rhs.end - rhs.start;
    let len_ovl = std::cmp::min(lhs.end, rhs.end) - std::cmp::max(lhs.start, rhs.start);
    let res_lhs = Into::<f64>::into(len_ovl) / Into::<f64>::into(len_lhs);
    let res_rhs = Into::<f64>::into(len_ovl) / Into::<f64>::into(len_rhs);
    if res_lhs < res_rhs {
        res_lhs
    } else {
        res_rhs
    }
}

/// Implementation of both endpoints.
async fn handle_impl(
    data: Data<crate::server::run::WebServerData>,
    _path: Path<()>,
    query: Request,
) -> actix_web::Result<crate::pbs::clinvar::sv::ResponsePage, CustomError> {
    // Parse out genome release.
    let genome_release: GenomeRelease =
        query
            .clone()
            .genome_release
            .parse()
            .map_err(|e: strum::ParseError| {
                CustomError::new(anyhow::anyhow!("problem getting genome release: {}", e))
            })?;
    // Obtain handle of interval trees datastructure for genome release.
    let trees = if let Some(trees) = data.clinvar_svs[genome_release].as_ref() {
        trees
    } else {
        Err(anyhow::anyhow!(
            "no clinvar-sv database for genome release {}",
            genome_release
        ))
        .map_err(CustomError::new)?
    };
    // Create SPDI range to query for and query the tree with this.
    let spdi_range = spdi::Range {
        sequence: query.chromosome.replace("chr", "").to_string(),
        start: query.start as i32,
        end: query.stop as i32,
    };
    let records = trees.query(&spdi_range).map_err(|e| {
        CustomError::new(anyhow::anyhow!(
            "problem querying clinvar-sv database: {}",
            e
        ))
    })?;
    // Filter the records.
    let variation_types = query
        .variation_types
        .as_ref()
        .map(|vs| vs.iter().map(|v| *v as i32).collect::<Vec<_>>())
        .unwrap_or_default();
    let records = {
        let mut records = records
            .into_iter()
            .filter_map(|record| {
                let crate::pbs::clinvar_data::clinvar_public::location::SequenceLocation {
                    start,
                    stop,
                    inner_start,
                    inner_stop,
                    outer_start,
                    outer_stop,
                    ..
                } = record
                    .sequence_location
                    .clone()
                    .expect("missing sequence_location");
                let (start, stop) = if let (Some(start), Some(stop)) = (start, stop) {
                    (start, stop)
                } else if let (Some(inner_start), Some(inner_stop)) = (inner_start, inner_stop) {
                    (inner_start, inner_stop)
                } else if let (Some(outer_start), Some(outer_stop)) = (outer_start, outer_stop) {
                    (outer_start, outer_stop)
                } else {
                    let accession = record.accession.clone().expect("missing accession");
                    let vcv = format!("{}.{}", &accession.accession, &accession.version);
                    tracing::warn!("skipping record because no start/stop: {}", &vcv);
                    return None;
                };

                let overlap =
                    reciprocal_overlap(&((query.start - 1)..query.stop), &((start - 1)..stop));
                Some(crate::pbs::clinvar::sv::ResponseRecord {
                    record: Some(record),
                    overlap,
                })
            })
            .filter(|record| {
                // filter by variant type if specified
                if !variation_types.is_empty() {
                    return variation_types
                        .contains(&record.record.as_ref().expect("no record").variation_type);
                }
                // filter by overlap if specified
                let min_overlap = query.min_overlap.unwrap_or(DEFAULT_MIN_OVERLAP);
                if record.overlap < min_overlap {
                    return false;
                }

                true
            })
            .collect::<Vec<_>>();
        records.sort_by(|a, b| b.overlap.partial_cmp(&a.overlap).unwrap());
        records
    };
    // Compute pagination information.
    let per_page = query.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
    let total_pages = (records.len() as u32 + 1) / per_page;
    let current_page = std::cmp::max(query.page_no.unwrap_or(1), 1);
    let begin = ((current_page - 1) * per_page) as usize;
    let end = std::cmp::min(begin as u32 + per_page, records.len() as u32) as usize;
    let records = records[begin..end].to_vec();

    let page_info = crate::pbs::clinvar::sv::PageInfo {
        total: records.len() as u32,
        per_page,
        current_page,
        total_pages,
    };

    Ok(crate::pbs::clinvar::sv::ResponsePage {
        records,
        page_info: Some(page_info),
    })
}

/// Query for annotations for one variant.
#[get("/clinvar-sv/query")]
async fn handle(
    data: Data<crate::server::run::WebServerData>,
    path: Path<()>,
    query: web::Query<Request>,
) -> actix_web::Result<Json<crate::pbs::clinvar::sv::ResponsePage>, CustomError> {
    Ok(Json(handle_impl(data, path, query.into_inner()).await?))
}

/// Query of the `/api/v1/strucvars/clinvar-annos/query` endpoint.
#[serde_with::skip_serializing_none]
#[serde_with::serde_as]
#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema, utoipa::IntoParams,
)]
pub(crate) struct StrucvarsClinvarQuery {
    /// Genome release specification.
    pub genome_release: GenomeRelease,
    /// Chromosome name.
    pub chromosome: String,
    /// 1-based start position.
    pub start: u32,
    /// 1-based stop postion.
    pub stop: u32,
    /// Optionally, the variant types.
    #[serde_as(
        as = "Option<StringWithSeparator::<CommaSeparator, ClinvarExtractedVariationType>>"
    )]
    pub variation_types: Option<Vec<ClinvarExtractedVariationType>>,
    /// Optionally, minimal overlap.
    pub min_overlap: Option<f64>,
    /// Optional 1-based page number.
    pub page_no: Option<u32>,
    /// Optional page size.
    pub page_size: Option<u32>,
}

impl From<StrucvarsClinvarQuery> for Request {
    fn from(val: StrucvarsClinvarQuery) -> Self {
        Request {
            genome_release: val.genome_release.to_string(),
            chromosome: val.chromosome,
            start: val.start,
            stop: val.stop,
            variation_types: val
                .variation_types
                .map(|v| v.into_iter().map(Into::into).collect()),
            min_overlap: val.min_overlap,
            page_no: val.page_no,
            page_size: val.page_size,
        }
    }
}

/// Module with response information.
pub mod response {
    use crate::server::run::clinvar_data::ClinvarExtractedVcvRecord;

    /// Information on one response record.
    #[derive(
        Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema, utoipa::ToResponse,
    )]
    pub struct StrucvarsClinvarResponseRecord {
        /// The record.
        pub record: Option<ClinvarExtractedVcvRecord>,
        /// The reciprocal overlap with the query.
        pub overlap: f64,
    }

    impl TryFrom<crate::pbs::clinvar::sv::ResponseRecord> for StrucvarsClinvarResponseRecord {
        type Error = anyhow::Error;

        fn try_from(value: crate::pbs::clinvar::sv::ResponseRecord) -> Result<Self, Self::Error> {
            Ok(Self {
                record: value.record.map(|record| record.try_into()).transpose()?,
                overlap: value.overlap,
            })
        }
    }

    /// Information regarding the pagination.
    #[derive(
        Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema, utoipa::ToResponse,
    )]
    pub struct StrucvarsClinvarPageInfo {
        /// The total number of records.
        pub total: u32,
        /// The number of records per page.
        pub per_page: u32,
        /// The current page number.
        pub current_page: u32,
        /// The total number of pages.
        pub total_pages: u32,
    }

    impl From<crate::pbs::clinvar::sv::PageInfo> for StrucvarsClinvarPageInfo {
        fn from(value: crate::pbs::clinvar::sv::PageInfo) -> Self {
            Self {
                total: value.total,
                per_page: value.per_page,
                current_page: value.current_page,
                total_pages: value.total_pages,
            }
        }
    }

    /// Response of the `/api/v1/strucvars/clinvar-annos/query` endpoint.
    #[derive(
        Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema, utoipa::ToResponse,
    )]
    pub struct StrucvarsClinvarResponse {
        /// The records in this page.
        pub records: Vec<StrucvarsClinvarResponseRecord>,
        /// Pagination information.
        pub page_info: StrucvarsClinvarPageInfo,
    }

    impl TryFrom<crate::pbs::clinvar::sv::ResponsePage> for StrucvarsClinvarResponse {
        type Error = anyhow::Error;

        fn try_from(value: crate::pbs::clinvar::sv::ResponsePage) -> Result<Self, Self::Error> {
            Ok(Self {
                records: value
                    .records
                    .into_iter()
                    .map(|record| record.try_into())
                    .collect::<Result<_, _>>()?,
                page_info: value
                    .page_info
                    .map(|page_info| page_info.into())
                    .ok_or_else(|| anyhow::anyhow!("missing page_info in response"))?,
            })
        }
    }
}

use response::*;

/// Endpoint for querying ClinVar SV annotations.
#[utoipa::path(
    get,
    operation_id = "strucvarsClinvarQuery",
    params(StrucvarsClinvarQuery),
    responses(
        (status = 200, description = "Clinvar strucvars information.", body = StrucvarsClinvarResponse),
        (status = 500, description = "Internal server error.", body = CustomError)
    )
)]
#[get("/api/v1/strucvars/clinvar/query")]
async fn handle_with_openapi(
    data: Data<crate::server::run::WebServerData>,
    path: Path<()>,
    query: web::Query<StrucvarsClinvarQuery>,
) -> actix_web::Result<Json<StrucvarsClinvarResponse>, CustomError> {
    Ok(Json(
        handle_impl(data, path, Into::<Request>::into(query.into_inner()))
            .await
            .map_err(|e| CustomError::new(anyhow::anyhow!("Implementaion error: {:?}", e)))?
            .try_into()
            .map_err(|e| CustomError::new(anyhow::anyhow!("Response conversion error: {:?}", e)))?,
    ))
}
