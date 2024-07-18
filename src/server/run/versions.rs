//! Provide information about the software and database versions.

use actix_web::{
    get,
    web::{self, Data, Json, Path},
    Responder,
};

use super::{error::CustomError, WebServerData};

/// Code for deserializing the version `spec.yaml` files.
pub mod schema {
    use std::path::Path;

    use crate::pbs;

    /// Information about input data.
    #[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
    pub struct CreatedFrom {
        /// Data source name.
        pub name: String,
        /// Version of the data source.
        pub version: String,
    }

    impl Into<pbs::common::versions::CreatedFrom> for CreatedFrom {
        fn into(self) -> pbs::common::versions::CreatedFrom {
            let Self { name, version } = self;
            pbs::common::versions::CreatedFrom { name, version }
        }
    }

    /// Version specification.
    #[serde_with::skip_serializing_none]
    #[serde_with::serde_as]
    #[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
    pub struct VersionSpec {
        /// Identifier of the data.
        #[serde(rename = "dc.identifier")]
        pub identifier: String,
        /// Title of the data.
        #[serde(rename = "dc.title")]
        pub title: String,
        /// Creator of the data.
        #[serde(rename = "dc.creator")]
        pub creator: String,
        /// Contributors of the data.
        #[serde(rename = "dc.contributor")]
        pub contributor: Option<Vec<String>>,
        /// Format of the data.
        #[serde(rename = "dc.format")]
        pub format: String,
        /// Date of the data.
        #[serde(rename = "dc.date")]
        pub date: String,
        /// Version of the data.
        #[serde(rename = "x-version")]
        pub version: String,
        /// Optional genome release.
        #[serde(rename = "x-genome-release")]
        pub genome_release: Option<String>,
        /// Data description.
        #[serde(rename = "dc.description")]
        pub description: String,
        /// Data source.
        #[serde(rename = "dc.source")]
        pub source: Vec<String>,
        /// Created from information.
        #[serde(rename = "x-created-from")]
        pub created_from: Vec<CreatedFrom>,
    }

    impl VersionSpec {
        /// Read a `VersionSpec` from a YAML file.
        pub fn from_path<P>(p: P) -> Result<Self, anyhow::Error>
        where
            P: AsRef<Path>,
        {
            let full_path = p.as_ref().to_str().ok_or_else(|| {
                anyhow::anyhow!("problem converting path to string: {:?}", p.as_ref())
            })?;
            let yaml_str = std::fs::read_to_string(&full_path)
                .map_err(|e| anyhow::anyhow!("problem reading file {}: {}", &full_path, e))?;
            serde_yaml::from_str(&yaml_str)
                .map_err(|e| anyhow::anyhow!("problem deserializing {}: {}", full_path, e))
        }
    }

    impl Into<pbs::common::versions::VersionSpec> for VersionSpec {
        fn into(self) -> pbs::common::versions::VersionSpec {
            let Self {
                identifier,
                title,
                creator,
                contributor,
                format,
                date,
                version,
                genome_release,
                description,
                source,
                created_from,
            } = self;
            pbs::common::versions::VersionSpec {
                identifier,
                title,
                creator,
                contributor: contributor.unwrap_or_default(),
                format,
                date,
                version,
                genome_release,
                description,
                source,
                created_from: created_from.into_iter().map(Into::into).collect(),
            }
        }
    }
}

/// Query parameters for `handle()`.
#[derive(Debug, Clone, serde::Deserialize)]
struct Request {
    pub genome_release: String,
}

/// Query for annotations for one variant.
#[get("/v1/versions")]
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

#[cfg(test)]
pub mod test {
    #[rstest::rstest]
    #[case("alphamissense-grch37-1+0.33.0")]
    #[case("alphamissense-grch38-1+0.33.0")]
    #[case("functional-grch37-105.20201022+0.33.0")]
    #[case("functional-grch38-110+0.33.0")]
    #[case("genes-3.1+4.0+4.5+20230606+10.1+20240105+0.33.0")]
    #[case("gnomad-mtdna-grch37-3.1+0.33.0")]
    #[case("gnomad-mtdna-grch38-3.1+0.33.0")]
    #[case("helixmtdb-grch37-20200327+0.33.0")]
    #[case("helixmtdb-grch38-20200327+0.33.0")]
    fn test_deserialize_spec_yaml(#[case] name: &str) -> Result<(), anyhow::Error> {
        crate::common::set_snapshot_suffix!("{}", &name);

        let full_path = format!("tests/server/annonars/{}/spec.yaml", &name);
        let spec = super::schema::VersionSpec::from_path(&full_path)?;
        insta::assert_yaml_snapshot!(&spec);

        let proto_spec: crate::pbs::common::versions::VersionSpec = spec.into();
        insta::assert_yaml_snapshot!(&proto_spec);

        Ok(())
    }
}
