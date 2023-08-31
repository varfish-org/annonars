//! Implementation of `/genes/lookup` that allows to lookup genes by symbol or identifier.
//!
//! In contrast to gene search, more than one query may be given but this must match exactly
//! the symbol or HGNC/NCBI/ENSEMBL identifier.
use actix_web::{
    get,
    web::{self, Data, Json, Path},
    Responder,
};

use crate::server::GeneNames;

use super::error::CustomError;
use serde_with::{formats::CommaSeparator, StringWithSeparator};

/// Parameters for `handle`.
#[serde_with::skip_serializing_none]
#[serde_with::serde_as]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
struct Request {
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

/// Query for annotations for one variant.
#[allow(clippy::option_map_unit_fn)]
#[get("/genes/lookup")]
async fn handle(
    data: Data<crate::server::WebServerData>,
    _path: Path<()>,
    query: web::Query<Request>,
) -> actix_web::Result<impl Responder, CustomError> {
    if query.q.len() < 2 {
        return Ok(Json(Container {
            // server_version: VERSION.to_string(),
            // builder_version,
            genes: Default::default(),
        }));
    }

    let genes_db = data.genes.as_ref().ok_or(CustomError::new(anyhow::anyhow!(
        "genes database not available"
    )))?;

    let genes = indexmap::IndexMap::from_iter(query.q.iter().map(|q| {
        let v = genes_db
            .name_to_hgnc_idx
            .get(q)
            .map(|idx| genes_db.gene_names[*idx].clone());
        (q.clone(), v)
    }));

    Ok(Json(Container {
        // server_version: VERSION.to_string(),
        // builder_version,
        genes,
    }))
}
