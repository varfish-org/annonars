//! Code for `/annos/range`.

use actix_web::{
    get,
    web::{self, Data, Json, Path},
    Responder,
};
use strum::IntoEnumIterator;

use crate::{
    common::{keys, version},
    server::{
        actix_server::fetch::{fetch_pos_protobuf, fetch_pos_tsv_json},
        AnnoDb,
    },
};

use super::{error::CustomError, WebServerData};

/// Parameters for `variant_annos::handle`.
#[serde_with::skip_serializing_none]
#[serde_with::serde_as]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
struct Request {
    /// Genome release version.
    pub genome_release: String,
    /// Chromosome name.
    pub chromosome: String,
    /// 1-based start position.
    pub start: u32,
    /// 1-based stop position.
    pub stop: u32,
}

impl Request {
    /// Conver to start `keys::Pos`.
    pub fn start_pos(&self) -> keys::Pos {
        keys::Pos {
            chrom: self.chromosome.clone(),
            pos: self.start as i32,
        }
    }

    /// Conver to stop `keys::Pos`.
    pub fn stop_pos(&self) -> keys::Pos {
        keys::Pos {
            chrom: self.chromosome.clone(),
            pos: self.stop as i32,
        }
    }
}

/// Result for `handle`.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct Result {
    /// Version of the server code.
    pub server_version: String,
    /// Version of the builder code.
    pub builder_version: String,
}

/// Result for `handle`.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde_with::skip_serializing_none]
struct Container {
    /// Version of the server code.
    pub server_version: String,
    /// The query parameters.
    pub query: Request,
    /// Annotations for the variant from each database.
    pub result: std::collections::BTreeMap<AnnoDb, Option<serde_json::Value>>,
}

/// Query for annotations for one variant.
#[get("/annos/range")]
async fn handle(
    data: Data<WebServerData>,
    _path: Path<()>,
    query: web::Query<Request>,
) -> actix_web::Result<impl Responder, CustomError> {
    let genome_release =
        query
            .clone()
            .into_inner()
            .genome_release
            .parse()
            .map_err(|e: strum::ParseError| {
                CustomError::new(anyhow::anyhow!("problem getting genome release: {}", e))
            })?;

    let mut annotations = std::collections::BTreeMap::default();
    for anno_db in AnnoDb::iter() {
        match anno_db {
            AnnoDb::Other => (),
            AnnoDb::Clinvar => {
                data.annos[genome_release][anno_db]
                    .as_ref()
                    .map(|db| {
                        fetch_pos_protobuf::<crate::pbs::clinvar::minimal::Record>(
                            db,
                            anno_db.cf_name(),
                            query.start_pos(),
                            query.stop_pos(),
                        )
                    })
                    .transpose()?
                    .map(|v| annotations.insert(anno_db, v));
            }
            AnnoDb::Cadd | AnnoDb::Dbnsfp | AnnoDb::Dbscsnv => {
                data.annos[genome_release][anno_db]
                    .as_ref()
                    .map(|db| {
                        fetch_pos_tsv_json(
                            db,
                            anno_db.cf_name(),
                            query.start_pos(),
                            query.stop_pos(),
                        )
                    })
                    .transpose()?
                    .map(|v| annotations.insert(anno_db, v));
            }
            AnnoDb::Dbsnp => {
                data.annos[genome_release][anno_db]
                    .as_ref()
                    .map(|db| {
                        fetch_pos_protobuf::<crate::dbsnp::pbs::Record>(
                            db,
                            anno_db.cf_name(),
                            query.start_pos(),
                            query.stop_pos(),
                        )
                    })
                    .transpose()?
                    .map(|v| annotations.insert(anno_db, v));
            }
            AnnoDb::Helixmtdb => {
                data.annos[genome_release][anno_db]
                    .as_ref()
                    .map(|db| {
                        fetch_pos_protobuf::<crate::helixmtdb::pbs::Record>(
                            db,
                            anno_db.cf_name(),
                            query.start_pos(),
                            query.stop_pos(),
                        )
                    })
                    .transpose()?
                    .map(|v| annotations.insert(anno_db, v));
            }
            AnnoDb::GnomadMtdna => {
                data.annos[genome_release][anno_db]
                    .as_ref()
                    .map(|db| {
                        fetch_pos_protobuf::<crate::pbs::gnomad::mtdna::Record>(
                            db,
                            anno_db.cf_name(),
                            query.start_pos(),
                            query.stop_pos(),
                        )
                    })
                    .transpose()?
                    .map(|v| annotations.insert(anno_db, v));
            }
            AnnoDb::GnomadExomes => {
                data.annos[genome_release][anno_db]
                    .as_ref()
                    .map(|db| {
                        fetch_pos_protobuf::<crate::pbs::gnomad::gnomad2::Record>(
                            db,
                            anno_db.cf_name(),
                            query.start_pos(),
                            query.stop_pos(),
                        )
                    })
                    .transpose()?
                    .map(|v| annotations.insert(anno_db, v));
            }
            AnnoDb::GnomadGenomes => {
                data.annos[genome_release][anno_db]
                    .as_ref()
                    .map(|db| {
                        let db_version = data.db_infos[genome_release][anno_db]
                            .as_ref()
                            .expect("must have db info here")
                            .db_version
                            .as_ref()
                            .expect("gnomAD must have db version");
                        if db_version.starts_with("2.") {
                            fetch_pos_protobuf::<crate::pbs::gnomad::gnomad2::Record>(
                                db,
                                anno_db.cf_name(),
                                query.start_pos(),
                                query.stop_pos(),
                            )
                        } else if db_version.starts_with("3.") {
                            fetch_pos_protobuf::<crate::pbs::gnomad::gnomad3::Record>(
                                db,
                                anno_db.cf_name(),
                                query.start_pos(),
                                query.stop_pos(),
                            )
                        } else {
                            Err(CustomError::new(anyhow::anyhow!(
                                "don't know how to tread gnomAD version {}",
                                db_version
                            )))
                        }
                    })
                    .transpose()?
                    .map(|v| annotations.insert(anno_db, v));
            }
            AnnoDb::UcscConservation => {
                data.annos[genome_release][anno_db]
                    .as_ref()
                    .map(|db| {
                        fetch_pos_protobuf::<crate::pbs::cons::RecordList>(
                            db,
                            anno_db.cf_name(),
                            query.start_pos(),
                            query.stop_pos(),
                        )
                    })
                    .transpose()?
                    .map(|v| annotations.insert(anno_db, v));
            }
        }
    }

    let result = Container {
        server_version: version().to_string(),
        query: query.into_inner(),
        result: annotations,
    };

    Ok(Json(result))
}
