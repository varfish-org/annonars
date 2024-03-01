//! Code for `/genes/info`.

use actix_web::{
    get,
    web::{self, Data, Json, Path},
    Responder,
};
use prost::Message;

use crate::pbs::genes;

use super::error::CustomError;
use serde_with::{formats::CommaSeparator, StringWithSeparator};

/// Parameters for `handle`.
#[serde_with::skip_serializing_none]
#[serde_with::serde_as]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
struct Request {
    /// The HGNC IDs to search for.
    #[serde_as(as = "Option<StringWithSeparator::<CommaSeparator, String>>")]
    pub hgnc_id: Option<Vec<String>>,
}

/// Result for `handle`.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde_with::skip_serializing_none]
struct Container {
    // TODO: add data version
    /// The resulting gene information.
    pub genes: indexmap::IndexMap<String, genes::base::Record>,
}

/// Query for annotations for one variant.
#[allow(clippy::option_map_unit_fn)]
#[get("/genes/info")]
async fn handle(
    data: Data<crate::server::WebServerData>,
    _path: Path<()>,
    query: web::Query<Request>,
) -> actix_web::Result<impl Responder, CustomError> {
    let genes_db = data.genes.as_ref().ok_or(CustomError::new(anyhow::anyhow!(
        "genes database not available"
    )))?;
    let cf_genes = genes_db
        .db
        .cf_handle("genes")
        .expect("no 'genes' column family");
    let mut genes = indexmap::IndexMap::new();
    if let Some(hgnc_id) = query.hgnc_id.as_ref() {
        for hgnc_id in hgnc_id {
            if let Some(raw_buf) = genes_db.db.get_cf(&cf_genes, hgnc_id).map_err(|e| {
                CustomError::new(anyhow::anyhow!("problem querying database: {}", e))
            })? {
                let record =
                    genes::base::Record::decode(std::io::Cursor::new(raw_buf)).map_err(|e| {
                        CustomError::new(anyhow::anyhow!("problem decoding value: {}", e))
                    })?;
                genes.insert(hgnc_id.to_string(), record);
            } else {
                tracing::debug!("no such gene: {}", hgnc_id);
            }
        }
    }

    let cf_meta = genes_db
        .db
        .cf_handle("meta")
        .expect("no 'meta' column family");
    let raw_builder_version = &genes_db
        .db
        .get_cf(&cf_meta, "builder-version")
        .map_err(|e| CustomError::new(anyhow::anyhow!("problem querying database: {}", e)))?
        .expect("database missing 'builder-version' key?");
    let _builder_version = std::str::from_utf8(raw_builder_version)
        .map_err(|e| CustomError::new(anyhow::anyhow!("problem decoding value: {}", e)))?
        .to_string();

    Ok(Json(Container {
        // server_version: VERSION.to_string(),
        // builder_version,
        genes,
    }))
}
