//! Code for `/clinvar-sv/query` endpoint.

use actix_web::{
    get,
    web::{self, Data, Json, Path},
    Responder,
};

use crate::common::{cli::GenomeRelease, spdi};

use super::error::CustomError;
use serde_with::{formats::CommaSeparator, StringWithSeparator};

use crate::pbs::clinvar::sv::{PageInfo, ResponsePage, ResponseRecord};

/// The default page size to use.
const DEFAULT_PAGE_SIZE: u32 = 100;
/// The default minimal overlap.
const DEFAULT_MIN_OVERLAP: f64 = 0.5;

/// Enumeration for `Request::variant_type`.
///
/// We use the prefixed `SCREAMING_SNAKE_CASE` for consistency with the JSON
/// serialized protobufs.
#[serde_with::serde_as]
#[derive(
    serde_with::SerializeDisplay,
    serde_with::DeserializeFromStr,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
enum VariantType {
    Unknown,
    Deletion,
    Duplication,
    Indel,
    Insertion,
    Inversion,
    Snv,
}

impl std::fmt::Display for VariantType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariantType::Unknown => write!(f, "VARIANT_TYPE_UNKNOWN"),
            VariantType::Deletion => write!(f, "VARIANT_TYPE_DELETION"),
            VariantType::Duplication => write!(f, "VARIANT_TYPE_DUPLICATION"),
            VariantType::Indel => write!(f, "VARIANT_TYPE_INDEL"),
            VariantType::Insertion => write!(f, "VARIANT_TYPE_INSERTION"),
            VariantType::Inversion => write!(f, "VARIANT_TYPE_INVERSION"),
            VariantType::Snv => write!(f, "VARIANT_TYPE_SNV"),
        }
    }
}

impl std::str::FromStr for VariantType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "VARIANT_TYPE_UNKNOWN" => Ok(VariantType::Unknown),
            "VARIANT_TYPE_DELETION" => Ok(VariantType::Deletion),
            "VARIANT_TYPE_DUPLICATION" => Ok(VariantType::Duplication),
            "VARIANT_TYPE_INDEL" => Ok(VariantType::Indel),
            "VARIANT_TYPE_INSERTION" => Ok(VariantType::Insertion),
            "VARIANT_TYPE_INVERSION" => Ok(VariantType::Inversion),
            "VARIANT_TYPE_SNV" => Ok(VariantType::Snv),
            _ => Err(anyhow::anyhow!("unknown variant type: {}", s)),
        }
    }
}

// impl From<VariantType> for PbVariantType {
//     fn from(val: VariantType) -> Self {
//         match val {
//             VariantType::Unknown => PbVariantType::Unknown,
//             VariantType::Deletion => PbVariantType::Deletion,
//             VariantType::Duplication => PbVariantType::Duplication,
//             VariantType::Indel => PbVariantType::Indel,
//             VariantType::Insertion => PbVariantType::Insertion,
//             VariantType::Inversion => PbVariantType::Inversion,
//             VariantType::Snv => PbVariantType::Snv,
//         }
//     }
// }

/// Parameters for `handle()`.
///
/// We use `camelCase` for exposition on the REST API and consistency with
/// JSON serialized protobufs.
#[serde_with::skip_serializing_none]
#[serde_with::serde_as]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Request {
    /// Genome release specification.
    #[allow(dead_code)]
    pub genome_release: String,
    /// Chromosome name.
    pub chromosome: String,
    /// 1-based start position.
    pub start: u32,
    /// 1-based stop postion.
    pub stop: u32,
    /// Optionally, the variant types.
    #[serde_as(as = "Option<StringWithSeparator::<CommaSeparator, VariantType>>")]
    pub variant_type: Option<Vec<VariantType>>,
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
    (Into::<f64>::into(len_ovl)) / Into::<f64>::into(len_lhs + len_rhs)
}

/// Query for annotations for one variant.
#[allow(clippy::option_map_unit_fn)]
#[get("/clinvar-sv/query")]
async fn handle(
    data: Data<crate::server::WebServerData>,
    _path: Path<()>,
    query: web::Query<Request>,
) -> actix_web::Result<impl Responder, CustomError> {
    // Parse out genome release.
    let genome_release: GenomeRelease =
        query
            .clone()
            .into_inner()
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
    // let variant_types = query
    //     .variant_type
    //     .as_ref()
    //     .map(|vs| {
    //         vs.iter()
    //             .map(|v| PbVariantType::from(*v) as i32)
    //             .collect::<Vec<_>>()
    //     })
    //     .unwrap_or_default();
    // let records = {
    //     let mut records = records
    //         .into_iter()
    //         .map(|record| {
    //             let overlap = reciprocal_overlap(
    //                 &((query.start - 1)..query.stop),
    //                 &((record.start - 1)..record.stop),
    //             );
    //             ResponseRecord {
    //                 record: Some(record),
    //                 overlap,
    //             }
    //         })
    //         .filter(|record| {
    //             // filter by variant type if specified
    //             if !variant_types.is_empty() {
    //                 return variant_types
    //                     .contains(&record.record.as_ref().expect("no record").variant_type);
    //             }
    //             // filter by overlap if specified
    //             let min_overlap = query.min_overlap.unwrap_or(DEFAULT_MIN_OVERLAP);
    //             if record.overlap < min_overlap {
    //                 return false;
    //             }

    //             true
    //         })
    //         .collect::<Vec<_>>();
    //     records.sort_by(|a, b| b.overlap.partial_cmp(&a.overlap).unwrap());
    //     records
    // };
    // Compute pagination information.
    let per_page = query.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
    let total_pages = (records.len() as u32 + 1) / per_page;
    let current_page = std::cmp::max(query.page_no.unwrap_or(1), 1);
    let begin = ((current_page - 1) * per_page) as usize;
    let end = std::cmp::min(begin as u32 + per_page, records.len() as u32) as usize;
    let records = records[begin..end].to_vec();

    let page_info = PageInfo {
        total: records.len() as u32,
        per_page,
        current_page,
        total_pages,
    };

    Ok(Json(ResponsePage {
        // records,
        records: Default::default(),
        page_info: Some(page_info),
    }))
}
