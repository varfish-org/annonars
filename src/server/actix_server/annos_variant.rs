//! Code for `/annos/variant`.
use actix_web::{
    get,
    web::{self, Data, Json, Path},
    Responder,
};
use strum::IntoEnumIterator;

use crate::{
    common::{keys, version},
    server::AnnoDb,
};

use super::error::CustomError;
use super::fetch::{fetch_pos_protobuf, fetch_var_protobuf, fetch_var_tsv_json};

/// Parameters for `variant_annos::handle`.
///
/// Defines a variant in SPDI format with a genome release specification.
#[serde_with::skip_serializing_none]
#[serde_with::serde_as]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
struct Request {
    /// Genome release specification.
    #[allow(dead_code)]
    pub genome_release: String,
    /// Chromosome name.
    pub chromosome: String,
    /// 1-based position for SPDI.
    pub pos: u32,
    /// Reference allele bases.
    pub reference: String,
    /// Alterantive allele bases.
    pub alternative: String,
}

impl From<Request> for keys::Var {
    fn from(value: Request) -> Self {
        keys::Var {
            chrom: value.chromosome,
            pos: value.pos as i32,
            reference: value.reference,
            alternative: value.alternative,
        }
    }
}

impl From<Request> for keys::Pos {
    fn from(value: Request) -> Self {
        keys::Pos {
            chrom: value.chromosome,
            pos: value.pos as i32,
        }
    }
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
#[allow(clippy::option_map_unit_fn)]
#[get("/annos/variant")]
async fn handle(
    data: Data<crate::server::WebServerData>,
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
                        fetch_var_protobuf::<crate::pbs::clinvar::minimal::Record>(
                            db,
                            anno_db.cf_name(),
                            query.clone().into_inner().into(),
                        )
                    })
                    .transpose()?
                    .map(|v| annotations.insert(anno_db, v));
            }
            AnnoDb::Cadd | AnnoDb::Dbnsfp | AnnoDb::Dbscsnv => {
                data.annos[genome_release][anno_db]
                    .as_ref()
                    .map(|db| {
                        fetch_var_tsv_json(db, anno_db.cf_name(), query.clone().into_inner().into())
                    })
                    .transpose()?
                    .map(|v| annotations.insert(anno_db, v));
            }
            AnnoDb::Dbsnp => {
                data.annos[genome_release][anno_db]
                    .as_ref()
                    .map(|db| {
                        fetch_var_protobuf::<crate::dbsnp::pbs::Record>(
                            db,
                            anno_db.cf_name(),
                            query.clone().into_inner().into(),
                        )
                    })
                    .transpose()?
                    .map(|v| annotations.insert(anno_db, v));
            }
            AnnoDb::Helixmtdb => {
                data.annos[genome_release][anno_db]
                    .as_ref()
                    .map(|db| {
                        fetch_var_protobuf::<crate::helixmtdb::pbs::Record>(
                            db,
                            anno_db.cf_name(),
                            query.clone().into_inner().into(),
                        )
                    })
                    .transpose()?
                    .map(|v| annotations.insert(anno_db, v));
            }
            AnnoDb::GnomadMtdna => {
                data.annos[genome_release][anno_db]
                    .as_ref()
                    .map(|db| {
                        fetch_var_protobuf::<crate::pbs::gnomad::mtdna::Record>(
                            db,
                            anno_db.cf_name(),
                            query.clone().into_inner().into(),
                        )
                    })
                    .transpose()?
                    .map(|v| annotations.insert(anno_db, v));
            }
            AnnoDb::GnomadExomes => {
                data.annos[genome_release][anno_db]
                    .as_ref()
                    .map(|db| {
                        fetch_var_protobuf::<crate::pbs::gnomad::gnomad2::Record>(
                            db,
                            anno_db.cf_name(),
                            query.clone().into_inner().into(),
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
                            fetch_var_protobuf::<crate::pbs::gnomad::gnomad2::Record>(
                                db,
                                anno_db.cf_name(),
                                query.clone().into_inner().into(),
                            )
                        } else if db_version.starts_with("3.") {
                            fetch_var_protobuf::<crate::pbs::gnomad::gnomad3::Record>(
                                db,
                                anno_db.cf_name(),
                                query.clone().into_inner().into(),
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
                        let start: keys::Pos = query.clone().into_inner().into();
                        let start = keys::Pos {
                            chrom: start.chrom,
                            pos: start.pos - 2,
                        };
                        let stop = query.clone().into_inner().into();
                        fetch_pos_protobuf::<crate::pbs::cons::RecordList>(
                            db,
                            anno_db.cf_name(),
                            start,
                            stop,
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
