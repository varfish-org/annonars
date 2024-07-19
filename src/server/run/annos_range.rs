//! Code for `/annos/range`.

use actix_web::{
    get,
    web::{self, Data, Json, Path},
    Responder,
};
use strum::IntoEnumIterator;

use crate::{
    common::{cli::GenomeRelease, keys, version},
    server::run::{
        fetch::{fetch_pos_protobuf, fetch_pos_tsv_json},
        AnnoDb,
    },
};

use super::{error::CustomError, fetch::FetchConfig, WebServerData};

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
struct Response {
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

/// Helper that performs the fetching of annotations with a given configuration.
fn fetch_annotations(
    data: Data<WebServerData>,
    genome_release: GenomeRelease,
    start_pos: keys::Pos,
    stop_pos: keys::Pos,
    fetch_config: &FetchConfig,
) -> Result<std::collections::BTreeMap<AnnoDb, Option<serde_json::Value>>, anyhow::Error> {
    let mut annotations = std::collections::BTreeMap::default();

    for anno_db in AnnoDb::iter() {
        match anno_db {
            AnnoDb::Other => (),
            AnnoDb::Clinvar => {
                data.annos[genome_release][anno_db]
                    .as_ref()
                    .map(|db| {
                        fetch_pos_protobuf::<crate::pbs::clinvar::minimal::ExtractedVcvRecordList>(
                            &db.data,
                            anno_db.cf_name(),
                            start_pos.clone(),
                            stop_pos.clone(),
                            fetch_config,
                        )
                    })
                    .transpose()
                    .map(|v| annotations.insert(anno_db, v.flatten()));
            }
            AnnoDb::Cadd | AnnoDb::Dbnsfp | AnnoDb::Dbscsnv => {
                data.annos[genome_release][anno_db]
                    .as_ref()
                    .map(|db| {
                        fetch_pos_tsv_json(
                            &db.data,
                            anno_db.cf_name(),
                            start_pos.clone(),
                            stop_pos.clone(),
                            fetch_config,
                        )
                    })
                    .transpose()
                    .map(|v| annotations.insert(anno_db, v.flatten()));
            }
            AnnoDb::Dbsnp => {
                data.annos[genome_release][anno_db]
                    .as_ref()
                    .map(|db| {
                        fetch_pos_protobuf::<crate::dbsnp::pbs::Record>(
                            &db.data,
                            anno_db.cf_name(),
                            start_pos.clone(),
                            stop_pos.clone(),
                            fetch_config,
                        )
                    })
                    .transpose()
                    .map(|v| annotations.insert(anno_db, v.flatten()));
            }
            AnnoDb::Helixmtdb => {
                data.annos[genome_release][anno_db]
                    .as_ref()
                    .map(|db| {
                        fetch_pos_protobuf::<crate::helixmtdb::pbs::Record>(
                            &db.data,
                            anno_db.cf_name(),
                            start_pos.clone(),
                            stop_pos.clone(),
                            fetch_config,
                        )
                    })
                    .transpose()
                    .map(|v| annotations.insert(anno_db, v.flatten()));
            }
            AnnoDb::GnomadMtdna => {
                data.annos[genome_release][anno_db]
                    .as_ref()
                    .map(|db| {
                        fetch_pos_protobuf::<crate::pbs::gnomad::mtdna::Record>(
                            &db.data,
                            anno_db.cf_name(),
                            start_pos.clone(),
                            stop_pos.clone(),
                            fetch_config,
                        )
                    })
                    .transpose()
                    .map(|v| annotations.insert(anno_db, v.flatten()));
            }
            AnnoDb::GnomadExomes => {
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
                                &db.data,
                                anno_db.cf_name(),
                                start_pos.clone(),
                                stop_pos.clone(),
                                fetch_config,
                            )
                        } else if db_version.starts_with("4.") {
                            fetch_pos_protobuf::<crate::pbs::gnomad::gnomad4::Record>(
                                &db.data,
                                anno_db.cf_name(),
                                start_pos.clone(),
                                stop_pos.clone(),
                                fetch_config,
                            )
                        } else {
                            Err(CustomError::new(anyhow::anyhow!(
                                "don't know how to handle gnomAD version {}",
                                db_version
                            )))
                        }
                    })
                    .transpose()
                    .map(|v| annotations.insert(anno_db, v.flatten()));
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
                                &db.data,
                                anno_db.cf_name(),
                                start_pos.clone(),
                                stop_pos.clone(),
                                fetch_config,
                            )
                        } else if db_version.starts_with("3.") {
                            fetch_pos_protobuf::<crate::pbs::gnomad::gnomad3::Record>(
                                &db.data,
                                anno_db.cf_name(),
                                start_pos.clone(),
                                stop_pos.clone(),
                                fetch_config,
                            )
                        } else if db_version.starts_with("4.") {
                            fetch_pos_protobuf::<crate::pbs::gnomad::gnomad4::Record>(
                                &db.data,
                                anno_db.cf_name(),
                                start_pos.clone(),
                                stop_pos.clone(),
                                fetch_config,
                            )
                        } else {
                            Err(CustomError::new(anyhow::anyhow!(
                                "don't know how to handle gnomAD version {}",
                                db_version
                            )))
                        }
                    })
                    .transpose()
                    .map(|v| annotations.insert(anno_db, v.flatten()));
            }
            AnnoDb::UcscConservation => {
                data.annos[genome_release][anno_db]
                    .as_ref()
                    .map(|db| {
                        fetch_pos_protobuf::<crate::pbs::cons::RecordList>(
                            &db.data,
                            anno_db.cf_name(),
                            start_pos.clone(),
                            stop_pos.clone(),
                            fetch_config,
                        )
                    })
                    .transpose()
                    .map(|v| annotations.insert(anno_db, v.flatten()));
            }
        }
    }

    Ok(annotations)
}

/// Paginated retrieval of annotations.
pub mod paginated {
    use super::*;
    use crate::{
        pbs::server::interface::{AnnosRangeQuery, AnnosRangeResponse, GenomeRelease},
        server::run::fetch::FetchConfig,
    };

    /// Maximal number of records to fetch per page.
    static PAGE_SIZE: i32 = 100;

    impl AnnosRangeQuery {
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

    #[get("/v1/annos/range")]
    pub async fn handle(
        data: Data<WebServerData>,
        _path: Path<()>,
        query: web::Query<AnnosRangeQuery>,
    ) -> actix_web::Result<impl Responder, CustomError> {
        let query = query.into_inner();
        let genome_release = crate::common::cli::GenomeRelease::try_from(
            GenomeRelease::try_from(query.genome_release).map_err(|e| {
                CustomError::new(anyhow::anyhow!("problem getting genome release: {}", e))
            })?,
        )
        .map_err(|e| {
            CustomError::new(anyhow::anyhow!("could not convert genome release: {}", e))
        })?;
        let page_size = std::cmp::min(query.page_size.unwrap_or(PAGE_SIZE), PAGE_SIZE);
        let mut results = Vec::new();
        let mut next_page_token = None;

        let fetch_config = FetchConfig {
            limit: Some(page_size as usize),
        };

        // First, collect up to `page_size` annotations...
        let tmp = fetch_annotations(
            data,
            genome_release,
            query.start_pos(),
            query.stop_pos(),
            &fetch_config,
        ).map_err(|e| {
            CustomError::new(anyhow::anyhow!("problem querying variants: {}", e))
        })?;

        // Now obtain sorted list of keys.
        // And build result of the first page.

        Ok(Json(AnnosRangeResponse {
            query: Some(query),
            next_page_token,
            results,
        }))
    }
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

    let result = fetch_annotations(
        data,
        genome_release,
        query.start_pos(),
        query.stop_pos(),
        &Default::default(),
    ).map_err(|e| {
        CustomError::new(anyhow::anyhow!("problem querying variants: {}", e))
    })?;

    let result = Container {
        server_version: version().to_string(),
        query: query.into_inner(),
        result,
    };

    Ok(Json(result))
}
