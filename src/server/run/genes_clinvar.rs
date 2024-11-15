//! Code for `/genes/clinvar`.

use actix_web::{
    get,
    web::{self, Data, Json, Path},
    Responder,
};
use prost::Message;

use crate::pbs::clinvar::per_gene::ClinvarPerGeneRecord;

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
    /// The resulting per-gene ClinVar information.
    pub genes: indexmap::IndexMap<String, ClinvarPerGeneRecord>,
}

/// Query for annotations for one variant.
#[get("/genes/clinvar")]
async fn handle(
    data: Data<crate::server::run::WebServerData>,
    _path: Path<()>,
    query: web::Query<Request>,
) -> actix_web::Result<impl Responder, CustomError> {
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

    Ok(Json(Container {
        // server_version: VERSION.to_string(),
        // builder_version,
        genes,
    }))
}
