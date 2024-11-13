//! Implementation of endpoint `/api/v1/genes/search`.
//!
//! Also includes the implementation of the `/genes/search` endpoint (deprecated).
//!
//! Gene identifiers (HGNC, NCBI, ENSEMBL) must match.  As for symbols and names, the
//! search string may also be a substring.
use actix_web::{
    get,
    web::{self, Data, Json, Path},
};

use crate::server::run::GeneNames;

use super::error::CustomError;
use serde_with::{formats::CommaSeparator, StringWithSeparator};

/// The allowed fields to search in.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    strum::Display,
    strum::EnumString,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub(crate) enum GenesFields {
    /// HGNC ID field
    HgncId,
    /// Symbol field
    Symbol,
    /// Name field
    Name,
    /// Alias symbols field
    AliasSymbol,
    /// Alias names field
    AliasName,
    /// ENSEMBL gene ID
    EnsemblGeneId,
    /// NCBI gene ID
    NcbiGeneId,
}

/// Parameters for `handle`.
#[serde_with::skip_serializing_none]
#[serde_with::serde_as]
#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema, utoipa::IntoParams,
)]
#[serde(rename_all = "snake_case")]
pub(crate) struct GenesSearchQuery {
    /// The string to search for.
    pub q: String,
    /// The fields to search in.
    #[serde_as(as = "Option<StringWithSeparator::<CommaSeparator, GenesFields>>")]
    pub fields: Option<Vec<GenesFields>>,
    /// Enable case sensitive search.
    pub case_sensitive: Option<bool>,
}

/// A scored result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub(crate) struct Scored<T> {
    /// The score.
    pub score: f32,
    /// The result.
    pub data: T,
}

/// Alias for scored genes names.
pub(crate) type GenesScoredGeneNames = Scored<GeneNames>;

/// Result for `handle`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde_with::skip_serializing_none]
pub(crate) struct GenesSearchResponse {
    /// The resulting gene information.
    pub genes: Vec<GenesScoredGeneNames>,
}

/// Implementation of both endpoints.
async fn handle_impl(
    data: Data<crate::server::run::WebServerData>,
    _path: Path<()>,
    query: web::Query<GenesSearchQuery>,
) -> actix_web::Result<Json<GenesSearchResponse>, CustomError> {
    if query.q.len() < 2 {
        return Ok(Json(GenesSearchResponse {
            // server_version: VERSION.to_string(),
            // builder_version,
            genes: Vec::new(),
        }));
    }

    let genes_db = data.genes.as_ref().ok_or(CustomError::new(anyhow::anyhow!(
        "genes database not available"
    )))?;

    let max_items = 100;

    let case_sensitive: bool = query.case_sensitive.unwrap_or(false);

    let q = if case_sensitive {
        query.q.clone()
    } else {
        query.q.to_lowercase()
    };
    let equals_q = |val: &str| {
        if case_sensitive {
            val == q
        } else {
            val.to_lowercase() == q
        }
    };
    let contains_q = |val: &str| {
        if case_sensitive {
            val.contains(&q)
        } else {
            val.to_lowercase().contains(&q)
        }
    };
    let fields: Vec<GenesFields> = if let Some(fields) = query.fields.as_ref() {
        fields.clone()
    } else {
        Vec::new()
    };

    // The fields contain the given field or are empty.
    let fields_contains =
        |field: &GenesFields| -> bool { fields.is_empty() || fields.contains(field) };

    let mut genes = genes_db
        .data
        .gene_names
        .iter()
        .map(|gn| -> Scored<GeneNames> {
            let score = if (fields_contains(&GenesFields::HgncId) && equals_q(&gn.hgnc_id))
                || (fields_contains(&GenesFields::Symbol) && equals_q(&gn.symbol))
                || (fields_contains(&GenesFields::Symbol) && equals_q(&gn.symbol))
                || (fields_contains(&GenesFields::Name) && equals_q(&gn.name))
                || (fields_contains(&GenesFields::EnsemblGeneId)
                    && gn.ensembl_gene_id.iter().any(|s| equals_q(s)))
                || (fields_contains(&GenesFields::NcbiGeneId)
                    && gn.ncbi_gene_id.iter().any(|s| equals_q(s)))
            {
                1f32
            } else if fields_contains(&GenesFields::Symbol) && contains_q(&gn.symbol) {
                q.len() as f32 / gn.symbol.len() as f32
            } else if fields_contains(&GenesFields::Name) && contains_q(&gn.name) {
                q.len() as f32 / gn.name.len() as f32
            } else if fields_contains(&GenesFields::AliasSymbol)
                && gn.alias_symbol.iter().any(|s| contains_q(s))
            {
                gn.alias_symbol
                    .iter()
                    .map(|s| {
                        if contains_q(s) {
                            q.len() as f32 / s.len() as f32
                        } else {
                            0f32
                        }
                    })
                    .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                    .unwrap_or(0f32)
            } else if fields_contains(&GenesFields::AliasName)
                && gn.alias_name.iter().any(|s| contains_q(s))
            {
                gn.alias_name
                    .iter()
                    .map(|s| {
                        if contains_q(s) {
                            q.len() as f32 / s.len() as f32
                        } else {
                            0f32
                        }
                    })
                    .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                    .unwrap_or(0f32)
            } else {
                0f32
            };
            Scored {
                score,
                data: gn.clone(),
            }
        })
        .filter(|s| s.score > 0.0)
        .take(max_items)
        .collect::<Vec<_>>();

    genes.sort_by(|a, b| {
        (b.score, &b.data.symbol)
            .partial_cmp(&(a.score, &b.data.symbol))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(Json(GenesSearchResponse {
        // server_version: VERSION.to_string(),
        // builder_version,
        genes,
    }))
}

/// Search for genes.
#[get("/genes/search")]
async fn handle(
    data: Data<crate::server::run::WebServerData>,
    path: Path<()>,
    query: web::Query<GenesSearchQuery>,
) -> actix_web::Result<Json<GenesSearchResponse>, CustomError> {
    handle_impl(data, path, query).await
}

/// Search for genes.
#[utoipa::path(
    get,
    operation_id = "genesSearch",
    params(GenesSearchQuery),
    responses(
        (status = 200, description = "Genes search results.", body = GenesSearchResponse),
        (status = 500, description = "Internal server error.", body = CustomError)
    )
)]
#[get("/api/v1/genes/search")]
async fn handle_with_openapi(
    data: Data<crate::server::run::WebServerData>,
    path: Path<()>,
    query: web::Query<GenesSearchQuery>,
) -> actix_web::Result<Json<GenesSearchResponse>, CustomError> {
    handle_impl(data, path, query).await
}
