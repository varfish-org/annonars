//! Code for `/annos/db-info`.

use actix_web::{
    get,
    web::{self, Data, Json, Path},
    Responder,
};
use serde::{Deserialize, Serialize};

use crate::server::DbInfo;

use super::{error::CustomError, WebServerData};

/// Parameters for `variant_annos::handle`.
#[serde_with::skip_serializing_none]
#[serde_with::serde_as]
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
struct Request {
    pub genome_release: String,
}

/// Result for `handle`.
#[derive(Serialize, Debug, Clone)]
struct ResultEntry {
    /// Information for each database.
    pub db_info: indexmap::IndexMap<String, DbInfo>,
}

/// Query for annotations for one variant.
#[get("/annos/db-info")]
async fn handle(
    data: Data<WebServerData>,
    _path: Path<()>,
    query: web::Query<Request>,
) -> actix_web::Result<impl Responder, CustomError> {
    let genome_release =
        query
            .into_inner()
            .genome_release
            .parse()
            .map_err(|e: strum::ParseError| {
                CustomError::new(anyhow::anyhow!("problem getting genome release: {}", e))
            })?;
    Ok(Json(data.db_infos[genome_release].clone()))
}
