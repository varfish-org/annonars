//! Implementation of endpoint `/api/v1/genes/lookup`.
//!
//! Also includes the implementation of the `/genes/lookup` endpoint (deprecated).
//!
//! In contrast to gene search, more than one query may be given but this must match exactly
//! the symbol or HGNC/NCBI/ENSEMBL identifier.
use actix_web::{
    get,
    web::{self, Data, Json, Path},
};

use crate::server::run::GeneNames;

use super::error::CustomError;
use serde_with::{formats::CommaSeparator, StringWithSeparator};

/// Parameters for `handle`.
#[serde_with::skip_serializing_none]
#[serde_with::serde_as]
#[derive(
    serde::Serialize, serde::Deserialize, Debug, Clone, utoipa::ToSchema, utoipa::IntoParams,
)]
#[serde(rename_all = "snake_case")]
pub(crate) struct GenesLookupQuery {
    /// The strings to search for.
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, String>")]
    pub q: Vec<String>,
}

/// Result for `handle`.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde_with::skip_serializing_none]
struct Container {
    // TODO: add data version
    /// The resulting gene information.
    pub genes: indexmap::IndexMap<String, Option<GeneNames>>,
}

/// Implementation of both endpoints.
async fn handle_impl(
    data: Data<crate::server::run::WebServerData>,
    _path: Path<()>,
    query: web::Query<GenesLookupQuery>,
) -> actix_web::Result<Container, CustomError> {
    let genes_db = data.genes.as_ref().ok_or(CustomError::new(anyhow::anyhow!(
        "genes database not available"
    )))?;

    let genes = indexmap::IndexMap::from_iter(query.q.iter().map(|q| {
        let v = genes_db
            .data
            .name_to_hgnc_idx
            .get(q)
            .map(|idx| genes_db.data.gene_names[*idx].clone());
        (q.clone(), v)
    }));

    Ok(Container {
        // server_version: VERSION.to_string(),
        // builder_version,
        genes,
    })
}

/// Query for annotations for one variant.
#[get("/genes/lookup")]
async fn handle(
    data: Data<crate::server::run::WebServerData>,
    path: Path<()>,
    query: web::Query<GenesLookupQuery>,
) -> actix_web::Result<Json<Container>, CustomError> {
    Ok(Json(handle_impl(data, path, query).await?))
}

/// One result entry in the response.
#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema, utoipa::ToResponse,
)]
pub(crate) struct GenesLookupResultEntry {
    /// The query string,
    pub query: String,
    /// The gene names information.
    pub gene_names: Option<GeneNames>,
}

/// Result for `async fn handle_with_openapi(
#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema, utoipa::ToResponse,
)]
pub(crate) struct GenesLookupResponse {
    /// The resulting gene information.
    pub genes: Vec<GenesLookupResultEntry>,
}

impl From<Container> for GenesLookupResponse {
    fn from(container: Container) -> Self {
        Self {
            genes: container
                .genes
                .into_iter()
                .map(|(query, gene_names)| GenesLookupResultEntry { query, gene_names })
                .collect(),
        }
    }
}

/// Search for genes.
#[utoipa::path(
    get,
    operation_id = "genesLookup",
    params(GenesLookupQuery),
    responses(
        (status = 200, description = "Genes search results.", body = GenesLookupResponse),
        (status = 500, description = "Internal server error.", body = CustomError)
    )
)]
#[get("/api/v1/genes/lookup")]
async fn handle_with_openapi(
    data: Data<crate::server::run::WebServerData>,
    path: Path<()>,
    query: web::Query<GenesLookupQuery>,
) -> actix_web::Result<Json<GenesLookupResponse>, CustomError> {
    Ok(Json(handle_impl(data, path, query).await?.into()))
}
