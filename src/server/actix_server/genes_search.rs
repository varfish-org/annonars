//! Implementation of `/genes/search` that allows to search for genes by symbol etc.
use actix_web::{
    get,
    web::{self, Data, Json, Path},
    Responder,
};

use crate::server::GeneNames;

use super::error::CustomError;
use serde_with::{formats::CommaSeparator, StringWithSeparator};

/// The allowed fields to search in.
#[derive(
    serde::Serialize,
    serde::Deserialize,
    strum::Display,
    strum::EnumString,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
enum Fields {
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
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
struct Request {
    /// The string to search for.
    pub q: String,
    /// The fields to search in.
    #[serde_as(as = "Option<StringWithSeparator::<CommaSeparator, Fields>>")]
    pub fields: Option<Vec<Fields>>,
}

/// A scored result.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct Scored<T> {
    /// The score.
    pub score: f32,
    /// The result.
    pub data: T,
}

/// Result for `handle`.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde_with::skip_serializing_none]
struct Container {
    // TODO: add data version
    /// The resulting gene information.
    pub genes: Vec<Scored<GeneNames>>,
}

/// Query for annotations for one variant.
#[allow(clippy::option_map_unit_fn)]
#[get("/genes/search")]
async fn handle(
    data: Data<crate::server::WebServerData>,
    _path: Path<()>,
    query: web::Query<Request>,
) -> actix_web::Result<impl Responder, CustomError> {
    if query.q.len() < 2 {
        return Ok(Json(Container {
            // server_version: VERSION.to_string(),
            // builder_version,
            genes: Vec::new(),
        }));
    }

    let genes_db = data.genes.as_ref().ok_or(CustomError::new(anyhow::anyhow!(
        "genes database not available"
    )))?;

    let max_items = 100;

    let q = &query.q;
    let fields: Vec<Fields> = if let Some(fields) = query.fields.as_ref() {
        fields.clone()
    } else {
        Vec::new()
    };

    // The fields contain the given field or are empty.
    let fields_contains = |field: &Fields| -> bool { fields.is_empty() || fields.contains(field) };

    let mut genes = genes_db
        .gene_strings
        .iter()
        .map(|gn| -> Scored<GeneNames> {
            let score = if (fields_contains(&Fields::HgncId) && &gn.hgnc_id == q)
                || (fields_contains(&Fields::Symbol) && &gn.symbol == q)
                || (fields_contains(&Fields::Symbol) && &gn.symbol == q)
                || (fields_contains(&Fields::Name) && &gn.name == q)
                || (fields_contains(&Fields::EnsemblGeneId)
                    && gn.ensembl_gene_id.iter().any(|s| s == q))
                || (fields_contains(&Fields::NcbiGeneId)
                    && gn.ensembl_gene_id.iter().any(|s| s == q))
            {
                1f32
            } else if fields_contains(&Fields::Symbol) && gn.symbol.contains(q) {
                q.len() as f32 / gn.symbol.len() as f32
            } else if fields_contains(&Fields::Name) && gn.name.contains(q) {
                q.len() as f32 / gn.name.len() as f32
            } else if fields_contains(&Fields::AliasSymbol)
                && gn.alias_symbol.iter().any(|s| s.contains(q))
            {
                gn.alias_symbol
                    .iter()
                    .map(|s| {
                        if s.contains(q) {
                            q.len() as f32 / s.len() as f32
                        } else {
                            0f32
                        }
                    })
                    .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                    .unwrap_or(0f32)
            } else if fields_contains(&Fields::AliasName)
                && gn.alias_name.iter().any(|s| s.contains(q))
            {
                gn.alias_name
                    .iter()
                    .map(|s| {
                        if s.contains(q) {
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

    Ok(Json(Container {
        // server_version: VERSION.to_string(),
        // builder_version,
        genes,
    }))
}
