//! Implementation of the actix server.

pub mod annos_db_info;
pub mod annos_range;
pub mod annos_variant;
pub mod clinvar_sv;
pub mod error;
pub mod fetch;
pub mod genes_clinvar;
pub mod genes_info;
pub mod genes_lookup;
pub mod genes_search;
pub mod versions;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    str::FromStr as _,
    time::Instant,
};

use clap::Parser;
use indicatif::ParallelProgressIterator;
use prost::Message;
use rayon::prelude::*;
use utoipa::OpenApi as _;

use crate::{
    clinvar_sv::cli::query::{self as clinvarsv_query, IntervalTrees as ClinvarsvIntervalTrees},
    common::{self, cli::GenomeRelease},
    pbs::{self, genes},
};

use actix_web::{middleware::Logger, web::Data, App, HttpServer};

/// Module with OpenAPI documentation.
pub mod openapi {
    use super::*;
    use crate::{
        common::cli::GenomeRelease,
        pbs::common::versions::{CreatedFrom, VersionSpec},
        pbs::server::interface::{AnnosRangeQuery, VcfVariant},
        server::run::{
            versions::{AnnoVersionInfo, ReleaseVersionInfos},
            AnnoDb,
        },
    };

    use super::versions::{self, VersionInfoResponse};

    /// Utoipa-based `OpenAPI` generation helper.
    #[derive(utoipa::OpenApi)]
    #[openapi(
        paths(versions::handle),
        components(schemas(
            AnnoVersionInfo,
            ReleaseVersionInfos,
            VersionInfoResponse,
            CreatedFrom,
            VersionSpec,
            GenomeRelease,
            AnnoDb,
            AnnosRangeQuery,
            VcfVariant,
        ))
    )]
    pub struct ApiDoc;
}

/// Main entry point for the actix server.
///
/// # Errors
///
/// If the server cannot be started.
#[actix_web::main]
pub async fn main(args: &Args, dbs: Data<WebServerData>) -> std::io::Result<()> {
    let openapi = openapi::ApiDoc::openapi();

    HttpServer::new(move || {
        let app = App::new()
            .app_data(dbs.clone())
            .service(annos_variant::handle)
            .service(annos_range::handle)
            .service(annos_range::paginated::handle)
            .service(annos_db_info::handle)
            .service(clinvar_sv::handle)
            .service(genes_clinvar::handle)
            .service(genes_info::handle)
            .service(genes_search::handle)
            .service(genes_lookup::handle)
            .service(versions::handle)
            .service(
                utoipa_swagger_ui::SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", openapi.clone()),
            );
        app.wrap(Logger::default())
    })
    .bind((args.listen_host.as_str(), args.listen_port))?
    .run()
    .await
}

/// Encode annotation database.
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    strum::Display,
    strum::EnumString,
    strum::EnumIter,
    serde::Serialize,
    serde::Deserialize,
    enum_map::Enum,
    utoipa::ToSchema,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AnnoDb {
    /// Other database.
    #[default]
    Other,
    /// CADD annotations.
    Cadd,
    /// dbSNP annotations.
    Dbsnp,
    /// dbNSFP annotations.
    Dbnsfp,
    /// dbscSNV annotations.
    Dbscsnv,
    /// gnomAD mtDNA annotations.
    GnomadMtdna,
    /// gnomAD exomes annotations.
    GnomadExomes,
    /// gnomAD genomes annotations.
    GnomadGenomes,
    /// HelixMtDb annotations.
    Helixmtdb,
    /// UCSC conservation annotations.
    UcscConservation,
    /// ClinVar with minimal data extracted.
    Clinvar,
}

impl AnnoDb {
    /// Return the expected column family name of the database.
    pub fn cf_name(self) -> &'static str {
        match self {
            AnnoDb::Cadd => "tsv_data",
            AnnoDb::Dbsnp => "dbsnp_data",
            AnnoDb::Dbnsfp => "tsv_data",
            AnnoDb::Dbscsnv => "tsv_data",
            AnnoDb::GnomadMtdna => "gnomad_mtdna_data",
            AnnoDb::GnomadExomes => "gnomad_nuclear_data",
            AnnoDb::GnomadGenomes => "gnomad_nuclear_data",
            AnnoDb::Helixmtdb => "helixmtdb_data",
            AnnoDb::UcscConservation => "ucsc_conservation",
            AnnoDb::Clinvar => "clinvar",
            AnnoDb::Other => panic!("cannot get CF name for 'Other'"),
        }
    }

    /// Return the key for the database version.
    fn db_version_meta(&self) -> Option<&'static str> {
        match self {
            AnnoDb::Cadd => Some("db-version"),
            AnnoDb::Dbsnp => Some("db-version"),
            AnnoDb::Dbnsfp => Some("db-version"),
            AnnoDb::Dbscsnv => Some("db-version"),
            AnnoDb::GnomadMtdna => Some("gnomad-version"),
            AnnoDb::GnomadExomes => Some("gnomad-version"),
            AnnoDb::GnomadGenomes => Some("gnomad-version"),
            AnnoDb::Helixmtdb => None,
            AnnoDb::UcscConservation => None,
            AnnoDb::Clinvar => None,
            AnnoDb::Other => panic!("cannot get meta version name name for 'Other'"),
        }
    }
}

/// Identifier / name information for one gene.
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct GeneNames {
    /// HGNC gene ID.
    pub hgnc_id: String,
    /// HGNC gene symbol.
    pub symbol: String,
    /// Gene name from HGNC.
    pub name: String,
    /// HGNC alias symbols.
    pub alias_symbol: Vec<String>,
    /// HGNC alias names.
    pub alias_name: Vec<String>,
    /// ENSEMBL gene ID.
    pub ensembl_gene_id: Option<String>,
    /// NCBI gene ID.
    pub ncbi_gene_id: Option<String>,
}

/// Gene information database.
#[derive(Debug)]
pub struct GeneInfoDb {
    /// The database with overall genes information.
    pub db: rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    /// ClinVar gene information.
    pub db_clinvar: Option<rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>>,
    /// Gene information to keep in memory (for `/genes/search`).
    pub gene_names: Vec<GeneNames>,
    /// Mapping from allowed gene name string to index in `gene_names`.
    pub name_to_hgnc_idx: HashMap<String, usize>,
}

/// Genome-release specific annotation for each database.
pub type ReleaseAnnos = enum_map::EnumMap<
    AnnoDb,
    Option<WithVersionSpec<rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>>>,
>;

/// Database information
#[derive(serde::Serialize, Debug, Clone, Default)]
pub struct DbInfo {
    /// Identifier of the database.
    pub name: AnnoDb,
    /// Version of the database.
    pub db_version: Option<String>,
    /// Version of the builder code.
    pub builder_version: String,
}

/// Fetch database information from the given RocksDB.
fn fetch_db_info(
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    name: AnnoDb,
) -> Result<(GenomeRelease, DbInfo), anyhow::Error> {
    let genome_release: GenomeRelease = rocksdb_utils_lookup::fetch_meta(db, "genome-release")?
        .ok_or(anyhow::anyhow!("meta:genome-release not found in data"))?
        .as_str()
        .parse()?;
    let db_version = name
        .db_version_meta()
        .map(|db_version_meta| {
            rocksdb_utils_lookup::fetch_meta(db, db_version_meta)?.ok_or(anyhow::anyhow!(
                "meta:{} not found in database",
                db_version_meta
            ))
        })
        .transpose()?;
    let builder_version =
        rocksdb_utils_lookup::fetch_meta(db, "annonars-version")?.ok_or(anyhow::anyhow!(
            "meta:annonars-version not found in database {}",
            db.path().display()
        ))?;
    let db_info = DbInfo {
        name,
        db_version,
        builder_version,
    };
    Ok((genome_release, db_info))
}

/// Generic type to store a database together with version specification.
#[derive(Debug)]
pub struct WithVersionSpec<T: std::fmt::Debug> {
    /// The actual data.
    pub data: T,
    /// Version specification.
    pub version_spec: pbs::common::versions::VersionSpec,
}

impl<T> WithVersionSpec<T>
where
    T: std::fmt::Debug,
{
    /// Construct with the given data and path to specification YAML file.
    pub fn from_data_and_path<P>(data: T, path: P) -> Result<Self, anyhow::Error>
    where
        P: AsRef<Path>,
    {
        let version_spec: pbs::common::versions::VersionSpec =
            versions::schema::VersionSpec::from_path(path)?.into();
        Ok(Self { data, version_spec })
    }
}

/// Data for the web server.
#[derive(Debug, Default)]
pub struct WebServerData {
    /// Gene information database.
    pub genes: Option<WithVersionSpec<GeneInfoDb>>,
    /// Release-specific annotations for each `GenomeRelease`.
    pub annos: enum_map::EnumMap<GenomeRelease, ReleaseAnnos>,
    /// Release-specific ClinVar SV interval tree indexed databased.
    pub clinvar_svs: enum_map::EnumMap<GenomeRelease, Option<ClinvarsvIntervalTrees>>,
    /// Version information for each database.
    pub db_infos: enum_map::EnumMap<GenomeRelease, enum_map::EnumMap<AnnoDb, Option<DbInfo>>>,
}

/// Command line arguments for `server rest` sub command.
///
/// Each path can be given more than one time to support multiple releases.  When the server
/// is started, it needs to be given a file for each database with each release.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "Run annonars REST API", long_about = None)]
pub struct Args {
    /// Path to genes database.
    #[arg(long)]
    pub path_genes: Option<String>,
    /// ClinVar per-gene database(s), one for each release.
    #[arg(long)]
    pub path_clinvar_genes: Option<String>,
    /// ClinVar database(s), one for each release.
    #[arg(long)]
    pub path_clinvar: Vec<String>,
    /// ClinVar SV database(s), one for each release.
    #[arg(long)]
    pub path_clinvar_sv: Vec<String>,
    /// CADD database(s), one for each release.
    #[arg(long)]
    pub path_cadd: Vec<String>,
    /// dbSNP database(s), one for each release.
    #[arg(long)]
    pub path_dbsnp: Vec<String>,
    /// dbNSFP database(s), one for each release.
    #[arg(long)]
    pub path_dbnsfp: Vec<String>,
    /// PdbscSNV database(s), one for each release.
    #[arg(long)]
    pub path_dbscsnv: Vec<String>,
    /// gnomAD mtDNA database(s), one for each release.
    #[arg(long)]
    pub path_gnomad_mtdna: Vec<String>,
    /// gnomAD-exomes database(s), one for each release.
    #[arg(long)]
    pub path_gnomad_exomes: Vec<String>,
    /// gnomAD-genomes database(s), one for each release.
    #[arg(long)]
    pub path_gnomad_genomes: Vec<String>,
    /// HelixMtDB database(s), one for each release.
    #[arg(long)]
    pub path_helixmtdb: Vec<String>,
    /// UCSC conservation database(s), one for each release.
    #[arg(long)]
    pub path_ucsc_conservation: Vec<String>,

    /// IP to listen on.
    #[arg(long, default_value = "127.0.0.1")]
    pub listen_host: String,
    /// Port to listen on.
    #[arg(long, default_value_t = 8081)]
    pub listen_port: u16,
}

/// Open a RocksDB database.
///
/// # Arguments
///
/// * `path` - Path to the database.
/// * `cf_name` - Name of the column family to open (besides the mandatory `meta` column family).
fn open_db(
    path: &str,
    cf_name: &str,
) -> Result<rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>, anyhow::Error> {
    tracing::info!("Opening database {}...", path);
    let before_open = Instant::now();
    let res = rocksdb::DB::open_cf_for_read_only(
        &rocksdb::Options::default(),
        common::readlink_f(path)?,
        ["meta", cf_name],
        true,
    )
    .map_err(|e| anyhow::anyhow!("problem opening database: {}", e));
    tracing::info!("...done opening database in {:?}", before_open.elapsed());
    res
}

/// Obtain gene names from the genes RocksDB.
fn extract_gene_names(
    genes_db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
) -> Result<Vec<GeneNames>, anyhow::Error> {
    let mut result = Vec::new();

    let cf_read = genes_db.cf_handle("genes").unwrap();
    let mut iter = genes_db.raw_iterator_cf(&cf_read);
    iter.seek(b"");
    while iter.valid() {
        if let Some(iter_value) = iter.value() {
            let record = genes::base::Record::decode(std::io::Cursor::new(iter_value))?;
            let genes::base::Record { hgnc, .. } = record;
            if let Some(hgnc) = hgnc {
                let genes::base::HgncRecord {
                    hgnc_id,
                    symbol,
                    name,
                    alias_symbol,
                    alias_name,
                    ensembl_gene_id,
                    entrez_id,
                    ..
                } = hgnc;
                result.push(GeneNames {
                    hgnc_id,
                    symbol,
                    name,
                    alias_symbol,
                    alias_name,
                    ensembl_gene_id,
                    ncbi_gene_id: entrez_id,
                })
            }
        }
        iter.next();
    }

    Ok(result)
}

/// Main entry point for `server rest` sub command.
pub fn run(args_common: &common::cli::Args, args: &Args) -> Result<(), anyhow::Error> {
    tracing::info!("args_common = {:?}", &args_common);
    tracing::info!("args = {:?}", &args);

    if let Some(level) = args_common.verbose.log_level() {
        match level {
            log::Level::Trace | log::Level::Debug => {
                std::env::set_var("RUST_LOG", "debug");
                env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
            }
            _ => (),
        }
    }

    tracing::info!("Opening databases...");
    let mut data = WebServerData::default();
    let before_opening = Instant::now();

    if let Some(path_genes) = args.path_genes.as_ref() {
        tracing::info!("Opening genes database {}...", path_genes);
        let before_open = Instant::now();
        let db = open_db(path_genes, "genes")?;
        tracing::info!(
            "...done opening genes database in {:?}",
            before_open.elapsed()
        );

        let db_clinvar = if let Some(path_clinvar_genes) = args.path_clinvar_genes.as_ref() {
            tracing::info!("Opening ClinVar genes database {}...", path_clinvar_genes);
            let before_open = Instant::now();
            let clinvar_db = open_db(path_clinvar_genes, "clinvar-genes")?;
            tracing::info!(
                "...done opening ClinVar genes database in {:?}",
                before_open.elapsed()
            );
            Some(clinvar_db)
        } else {
            None
        };

        tracing::info!("Building gene names...");
        let before_open = Instant::now();
        let gene_names = extract_gene_names(&db)?;
        let name_to_hgnc_idx = {
            let mut result = HashMap::new();
            for (idx, gene_name) in gene_names.iter().enumerate() {
                result.insert(gene_name.hgnc_id.clone(), idx);
                if let Some(ensembl_gene_id) = gene_name.ensembl_gene_id.as_ref() {
                    result.insert(ensembl_gene_id.clone(), idx);
                }
                if let Some(ncbi_gene_id) = gene_name.ncbi_gene_id.as_ref() {
                    result.insert(ncbi_gene_id.clone(), idx);
                }
                result.insert(gene_name.symbol.clone(), idx);
            }
            result
        };
        tracing::info!("...done building genes names {:?}", before_open.elapsed());
        let gene_info_db = GeneInfoDb {
            db,
            db_clinvar,
            gene_names,
            name_to_hgnc_idx,
        };
        let path_buf = PathBuf::from_str(path_genes)?
            .parent()
            .ok_or_else(|| anyhow::anyhow!("cannot get parent directory of path {}", path_genes))?
            .join("spec.yaml");
        data.genes = Some(
            WithVersionSpec::from_data_and_path(gene_info_db, &path_buf).map_err(|e| {
                anyhow::anyhow!(
                    "problem loading gene info spec from {}: {}",
                    path_buf.display(),
                    e
                )
            })?,
        );
    }

    tracing::info!("Opening ClinVar SV databases...");
    let before_clinvar_sv = Instant::now();
    for path_clinvar_sv in &args.path_clinvar_sv {
        tracing::info!("  - {}", path_clinvar_sv);
        let (clinvar_sv_db, clinvar_sv_meta) = clinvarsv_query::open_rocksdb(
            path_clinvar_sv,
            "clinvar_sv",
            "meta",
            "clinvar_sv_by_rcv",
        )
        .map_err(|e| anyhow::anyhow!("problem opening RocksDB database: {}", e))?;
        let genome_release: GenomeRelease = clinvar_sv_meta.genome_release.parse()?;
        tracing::info!("    => {}", genome_release);
        let clinvar_sv_interval_trees =
            ClinvarsvIntervalTrees::with_db(clinvar_sv_db, "clinvar_sv", clinvar_sv_meta)
                .map_err(|e| anyhow::anyhow!("problem building interval trees: {}", e))?;
        data.clinvar_svs[genome_release] = Some(clinvar_sv_interval_trees);
    }
    tracing::info!(
        "...done opening ClinVar SV databases in {:?}",
        before_clinvar_sv.elapsed()
    );

    // Argument lists from the command line with the corresponding database enum value.
    let paths_db_pairs = [
        (&args.path_clinvar, AnnoDb::Clinvar),
        (&args.path_cadd, AnnoDb::Cadd),
        (&args.path_dbnsfp, AnnoDb::Dbnsfp),
        (&args.path_dbsnp, AnnoDb::Dbsnp),
        (&args.path_dbscsnv, AnnoDb::Dbscsnv),
        (&args.path_gnomad_mtdna, AnnoDb::GnomadMtdna),
        (&args.path_gnomad_exomes, AnnoDb::GnomadExomes),
        (&args.path_gnomad_genomes, AnnoDb::GnomadGenomes),
        (&args.path_helixmtdb, AnnoDb::Helixmtdb),
        (&args.path_ucsc_conservation, AnnoDb::UcscConservation),
    ];
    // "Unpack" the list of paths to single paths.
    let path_db_pairs = paths_db_pairs
        .iter()
        .map(|(paths, anno_db)| {
            paths
                .iter()
                .map(|path| (path.clone(), *anno_db))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    // Open the corresponding databases in parallel and extract database infos.  Store the
    // resulting database infos in `data`.
    path_db_pairs
        .par_iter()
        .progress_with(crate::common::cli::progress_bar(path_db_pairs.len()))
        .map(|(path, anno_db)| -> Result<_, anyhow::Error> {
            let db = open_db(path, anno_db.cf_name())?;
            let (genome_release, db_info) = fetch_db_info(&db, *anno_db)?;

            Ok((path, db_info, genome_release, db))
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .try_for_each(
            |(path_rocksdb, db_info, genome_release, db)| -> Result<(), anyhow::Error> {
                let spec_path = PathBuf::from_str(path_rocksdb)?
                    .parent()
                    .ok_or_else(|| {
                        anyhow::anyhow!("cannot get parent directory of path {}", path_rocksdb)
                    })?
                    .join("spec.yaml");
                let name = db_info.name;
                data.db_infos[genome_release][name] = Some(db_info);
                data.annos[genome_release][name] = Some(
                    WithVersionSpec::from_data_and_path(db, &spec_path).map_err(|e| {
                        anyhow::anyhow!(
                            "problem loading gene info spec from {}: {}",
                            spec_path.display(),
                            e
                        )
                    })?,
                );

                Ok(())
            },
        )?;
    tracing::info!(
        "...done opening databases in {:?}",
        before_opening.elapsed()
    );

    tracing::info!(
        "Launching server main on http://{}:{} ...",
        args.listen_host.as_str(),
        args.listen_port
    );
    tracing::info!(
        "  try: http://{}:{}/genes/search?q=BRCA",
        args.listen_host.as_str(),
        args.listen_port
    );
    tracing::info!(
        "  try: http://{}:{}/genes/search?q=BRCA&fields=hgnc_id,ensembl_gene_id,ncbi_gene_id,symbol",
        args.listen_host.as_str(),
        args.listen_port
    );
    tracing::info!(
        "  try: http://{}:{}/genes/lookup?q=BRCA,BRCA1,HGNC:1100",
        args.listen_host.as_str(),
        args.listen_port
    );
    tracing::info!(
        "  try: http://{}:{}/genes/info?hgnc_id=HGNC:12403",
        args.listen_host.as_str(),
        args.listen_port
    );
    tracing::info!(
        "  try: http://{}:{}/genes/clinvar?hgnc_id=HGNC:12403",
        args.listen_host.as_str(),
        args.listen_port
    );
    tracing::info!(
        "  try: http://{}:{}/annos/db-info?genome_release=grch37",
        args.listen_host.as_str(),
        args.listen_port
    );
    tracing::info!(
        "  try: http://{}:{}/annos/variant?genome_release=grch37&chromosome=1&pos=55505599&reference=C&alternative=G",
        args.listen_host.as_str(),
        args.listen_port
    );
    tracing::info!(
        "  try: http://{}:{}/annos/variant?genome_release=grch37&chromosome=1&pos=10001&reference=T&alternative=A",
        args.listen_host.as_str(),
        args.listen_port
    );
    tracing::info!(
        "  try: http://{}:{}/annos/range?genome_release=grch37&chromosome=1&start=1&stop=55516888",
        args.listen_host.as_str(),
        args.listen_port
    );
    main(args, actix_web::web::Data::new(data))?;

    tracing::info!("All done. Have a nice day!");
    Ok(())
}
