//! Implementation of endpoint `/api/v1/seqvars/annos`.
//!
//! Also includes the implementation of the `/annos/variant` endpoint (deprecated).

use actix_web::{
    get,
    web::{self, Data, Json, Path},
};
use strum::IntoEnumIterator;

use crate::{
    common::{keys, version},
    server::run::{fetch::fetch_pos_protobuf, AnnoDb},
};

use super::error::CustomError;
use super::fetch::{
    fetch_pos_protobuf_json, fetch_var_protobuf, fetch_var_protobuf_json, fetch_var_tsv_json,
};

/// Parameters for `variant_annos::handle`.
///
/// Defines a variant in VCF-style format with a genome release specification.
#[serde_with::skip_serializing_none]
#[serde_with::serde_as]
#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema, utoipa::IntoParams,
)]
pub struct SeqvarsAnnosQuery {
    /// Genome release specification.
    pub genome_release: String,
    /// Chromosome name.
    pub chromosome: String,
    /// 1-based position for VCF-style variant.
    pub pos: u32,
    /// Reference allele bases.
    pub reference: String,
    /// Alterantive allele bases.
    pub alternative: String,
}

impl From<SeqvarsAnnosQuery> for keys::Var {
    fn from(value: SeqvarsAnnosQuery) -> Self {
        keys::Var {
            chrom: value.chromosome,
            pos: value.pos as i32,
            reference: value.reference,
            alternative: value.alternative,
        }
    }
}

impl From<SeqvarsAnnosQuery> for keys::Pos {
    fn from(value: SeqvarsAnnosQuery) -> Self {
        keys::Pos {
            chrom: value.chromosome,
            pos: value.pos as i32,
        }
    }
}

/// Result for `handle`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde_with::skip_serializing_none]
struct Container {
    /// Version of the server code.
    pub server_version: String,
    /// The query parameters.
    pub query: SeqvarsAnnosQuery,
    /// Annotations for the variant from each database.
    pub result: std::collections::BTreeMap<AnnoDb, Option<serde_json::Value>>,
}

/// Query for annotations for one variant.
#[get("/annos/variant")]
async fn handle(
    data: Data<crate::server::run::WebServerData>,
    _path: Path<()>,
    query: web::Query<SeqvarsAnnosQuery>,
) -> actix_web::Result<Json<Container>, CustomError> {
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
                        fetch_var_protobuf_json::<
                            crate::pbs::clinvar::minimal::ExtractedVcvRecordList,
                        >(
                            &db.data,
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
                        fetch_var_tsv_json(
                            &db.data,
                            anno_db.cf_name(),
                            query.clone().into_inner().into(),
                        )
                    })
                    .transpose()?
                    .map(|v| annotations.insert(anno_db, v));
            }
            AnnoDb::Dbsnp => {
                data.annos[genome_release][anno_db]
                    .as_ref()
                    .map(|db| {
                        fetch_var_protobuf_json::<crate::dbsnp::pbs::Record>(
                            &db.data,
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
                        fetch_var_protobuf_json::<crate::helixmtdb::pbs::Record>(
                            &db.data,
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
                        fetch_var_protobuf_json::<crate::pbs::gnomad::mtdna::Record>(
                            &db.data,
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
                        let db_version = data.db_infos[genome_release][anno_db]
                            .as_ref()
                            .expect("must have db info here")
                            .db_version
                            .as_ref()
                            .expect("gnomAD must have db version");

                        if db_version.starts_with("2.") {
                            fetch_var_protobuf_json::<crate::pbs::gnomad::gnomad2::Record>(
                                &db.data,
                                anno_db.cf_name(),
                                query.clone().into_inner().into(),
                            )
                        } else if db_version.starts_with("4.") {
                            fetch_var_protobuf_json::<crate::pbs::gnomad::gnomad4::Record>(
                                &db.data,
                                anno_db.cf_name(),
                                query.clone().into_inner().into(),
                            )
                        } else {
                            Err(CustomError::new(anyhow::anyhow!(
                                "don't know how to handle gnomAD version {}",
                                db_version
                            )))
                        }
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
                            fetch_var_protobuf_json::<crate::pbs::gnomad::gnomad2::Record>(
                                &db.data,
                                anno_db.cf_name(),
                                query.clone().into_inner().into(),
                            )
                        } else if db_version.starts_with("3.") {
                            fetch_var_protobuf_json::<crate::pbs::gnomad::gnomad3::Record>(
                                &db.data,
                                anno_db.cf_name(),
                                query.clone().into_inner().into(),
                            )
                        } else if db_version.starts_with("4.") {
                            fetch_var_protobuf_json::<crate::pbs::gnomad::gnomad4::Record>(
                                &db.data,
                                anno_db.cf_name(),
                                query.clone().into_inner().into(),
                            )
                        } else {
                            Err(CustomError::new(anyhow::anyhow!(
                                "don't know how to handle gnomAD version {}",
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
                        fetch_pos_protobuf_json::<crate::pbs::cons::RecordList>(
                            &db.data,
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

/// `SeqvarsAnnosResponse` and related types.
pub mod response {
    use crate::server::run::clinvar_data::ClinvarExtractedVcvRecord;

    /// A HelixMtDb record.
    #[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct HelixMtDbRecord {
        /// Chromosome name.
        pub chrom: String,
        /// 1-based start position.
        pub pos: i32,
        /// Reference allele.
        pub ref_allele: String,
        /// / Alternate allele.
        pub alt_allele: String,
        /// Total number of individuals.
        pub num_total: i32,
        /// Number of homoplasmic carriers.
        pub num_het: i32,
        /// Number of heteroplasmic carriers.
        pub num_hom: i32,
        /// Feature type.
        pub feature_type: String,
        /// Gene name.
        pub gene_name: String,
    }

    impl Into<HelixMtDbRecord> for crate::pbs::helixmtdb::Record {
        fn into(self) -> HelixMtDbRecord {
            HelixMtDbRecord {
                chrom: self.chrom,
                pos: self.pos,
                ref_allele: self.ref_allele,
                alt_allele: self.alt_allele,
                num_total: self.num_total,
                num_het: self.num_het,
                num_hom: self.num_hom,
                feature_type: self.feature_type,
                gene_name: self.gene_name,
            }
        }
    }

    /// A UCSC conservation record.
    #[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct UcscConservationRecord {
        /// Chromosome name.
        pub chrom: String,
        /// 1-based, inclusive start position.
        pub start: i32,
        /// 1-based, inclusive stop position.
        pub stop: i32,
        /// HGNC identifier.
        pub hgnc_id: String,
        /// ENST identifier.
        pub enst_id: String,
        /// Exon number (1-based).
        pub exon_num: i32,
        /// Exon count.
        pub exon_count: i32,
        /// Alignment.
        pub alignment: String,
    }

    impl From<crate::pbs::cons::Record> for UcscConservationRecord {
        fn from(value: crate::pbs::cons::Record) -> Self {
            UcscConservationRecord {
                chrom: value.chrom,
                start: value.start,
                stop: value.stop,
                hgnc_id: value.hgnc_id,
                enst_id: value.enst_id,
                exon_num: value.exon_num,
                exon_count: value.exon_count,
                alignment: value.alignment,
            }
        }
    }

    /// List of `Record`s.
    #[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct UcscConservationRecordList {
        /// The records in the list.
        pub records: Vec<UcscConservationRecord>,
    }

    impl From<crate::pbs::cons::RecordList> for UcscConservationRecordList {
        fn from(value: crate::pbs::cons::RecordList) -> Self {
            UcscConservationRecordList {
                records: value.records.into_iter().map(Into::into).collect(),
            }
        }
    }
    /// List of `ClinvarExtractedVcvRecord`s.
    #[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct ExtractedVcvRecordList {
        /// The list of VCV records that may share a global variant.
        pub records: Vec<ClinvarExtractedVcvRecord>,
    }

    impl TryFrom<crate::pbs::clinvar::minimal::ExtractedVcvRecordList> for ExtractedVcvRecordList {
        type Error = anyhow::Error;

        fn try_from(
            value: crate::pbs::clinvar::minimal::ExtractedVcvRecordList,
        ) -> Result<Self, Self::Error> {
            Ok(ExtractedVcvRecordList {
                records: value
                    .records
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?,
            })
        }
    }

    /// Annotation for a sinngle variant.
    #[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct SeqvarsAnnoResponseRecord {
        /// Annotations from CADD (TSV annotation file).
        pub cadd: Option<bool>,
        /// Annotations from dbSNP.
        pub dbsnp: Option<bool>,
        /// Annotations from dbNSFP (TSV annotation file).
        pub dbnsfp: Option<bool>,
        /// Annotations from dbscSNV.
        pub dbscsnv: Option<bool>,
        /// Annotations from gnomAD-mtDNA.
        pub gnomad_mtdna: Option<bool>,
        /// Annotations from gnomAD-exomes.
        pub gnomad_exomes: Option<bool>,
        /// Annotations from gnomAD-genomes.
        pub gnomad_genomes: Option<bool>,
        /// Annotations from HelixMTdb.
        pub helixmtdb: Option<HelixMtDbRecord>,
        /// Annotations from UCSC conservation.
        pub ucsc_conservation: Option<UcscConservationRecordList>,
        /// Minimal extracted data from ClinVar.
        pub clinvar: Option<ExtractedVcvRecordList>,
    }

    /// Query response for `handle_with_openapi()`.
    #[derive(
        Debug,
        Default,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        utoipa::ToSchema,
        utoipa::ToResponse,
    )]
    pub struct SeqvarsAnnosResponse {
        /// The result records.
        pub result: SeqvarsAnnoResponseRecord,
    }
}

use response::*;

/// Query for annotations for a single variant.
#[utoipa::path(
    get,
    operation_id = "seqvarsAnosQuery",
    params(SeqvarsAnnosQuery),
    responses(
        (status = 200, description = "Annotation for a single variant.", body = SeqvarsAnnosResponse),
        (status = 500, description = "Internal server error.", body = CustomError)
    )
)]
#[get("/api/v1/genes/info")]
pub async fn handle_with_openapi(
    data: Data<crate::server::run::WebServerData>,
    _path: Path<()>,
    query: web::Query<SeqvarsAnnosQuery>,
) -> actix_web::Result<Json<SeqvarsAnnosResponse>, CustomError> {
    let genome_release = query
        .genome_release
        .parse()
        .map_err(|e: strum::ParseError| {
            CustomError::new(anyhow::anyhow!("problem getting genome release: {}", e))
        })?;

    let result = SeqvarsAnnoResponseRecord {
        // cadd: Option<bool>,
        // dbsnp: Option<bool>,
        // dbnsfp: Option<bool>,
        // dbscsnv: Option<bool>,
        // gnomad_mtdna: Option<bool>,
        // gnomad_exomes: Option<bool>,
        // gnomad_genomes: Option<bool>,
        helixmtdb: data.annos[genome_release][AnnoDb::Helixmtdb]
            .as_ref()
            .map(|db| {
                Ok(fetch_var_protobuf::<crate::pbs::helixmtdb::Record>(
                    &db.data,
                    AnnoDb::Helixmtdb.cf_name(),
                    query.clone().into_inner().into(),
                )?
                .map(Into::into))
            })
            .transpose()?
            .flatten(),
        ucsc_conservation: data.annos[genome_release][AnnoDb::UcscConservation]
            .as_ref()
            .map(|db| {
                let start: keys::Pos = query.clone().into_inner().into();
                let start = keys::Pos {
                    chrom: start.chrom,
                    pos: start.pos - 2,
                };
                let stop = query.clone().into_inner().into();
                Ok(fetch_pos_protobuf::<crate::pbs::cons::RecordList>(
                    &db.data,
                    AnnoDb::Clinvar.cf_name(),
                    start,
                    stop,
                )?
                .into_iter()
                .next()
                .map(Into::into))
            })
            .transpose()?
            .flatten(),
        clinvar: data.annos[genome_release][AnnoDb::Clinvar]
            .as_ref()
            .map(|db| {
                fetch_var_protobuf::<crate::pbs::clinvar::minimal::ExtractedVcvRecordList>(
                    &db.data,
                    AnnoDb::Clinvar.cf_name(),
                    query.clone().into_inner().into(),
                )?
                .map(TryInto::<ExtractedVcvRecordList>::try_into)
                .transpose()
                .map_err(|e| CustomError::new(e))
            })
            .transpose()?
            .flatten(),
        ..Default::default()
    };

    Ok(Json(SeqvarsAnnosResponse { result }))
}
