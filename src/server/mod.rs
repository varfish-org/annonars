//! Run REST API for serving entries from the annotations database

pub mod actix_server;

use std::time::Instant;

use clap::Parser;
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;

use crate::common::{self, cli::GenomeRelease};

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
)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
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
            AnnoDb::Other => panic!("cannot get meta version name name for 'Other'"),
        }
    }
}

/// Gene information database.
#[derive(Debug)]
pub struct GeneInfoDb {
    /// The database.
    pub db: rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
}

/// Genome-release specific annotation for each database.
pub type ReleaseAnnos =
    enum_map::EnumMap<AnnoDb, Option<rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>>>;

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

/// Data for the web server.
#[derive(Debug, Default)]
pub struct WebServerData {
    /// Gene information database.
    pub genes: Option<GeneInfoDb>,
    /// Release-specific annotations for each `GenomeRelease`.
    pub annos: enum_map::EnumMap<GenomeRelease, ReleaseAnnos>,
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
        path,
        ["meta", cf_name],
        true,
    )
    .map_err(|e| anyhow::anyhow!("problem opening database: {}", e));
    tracing::info!("...done opening database in {:?}", before_open.elapsed());
    res
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
    let before_opening = Instant::now();
    if let Some(path_genes) = args.path_genes.as_ref() {
        tracing::info!("Opening genes database {}...", path_genes);
        let before_open = Instant::now();
        let genes = open_db(path_genes, "genes")?;
        tracing::info!(
            "...done opening genes database in {:?}",
            before_open.elapsed()
        );
        let genes = GeneInfoDb { db: genes };
        tracing::info!(
            "...done opening genes database in {:?}",
            before_opening.elapsed()
        );
    }
    // Argument lists from the command line with the corresponding database enum value.
    let paths_db_pairs = vec![
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
    let mut data = WebServerData::default();
    path_db_pairs
        .par_iter()
        .progress_with(crate::common::cli::progress_bar(path_db_pairs.len()))
        .map(|(path, anno_db)| -> Result<_, anyhow::Error> {
            let db = open_db(path, anno_db.cf_name())?;
            let (genome_release, db_info) = fetch_db_info(&db, *anno_db)?;

            Ok((db_info, genome_release, db))
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .for_each(|(db_info, genome_release, db)| {
            let name = db_info.name;
            data.db_infos[genome_release][name] = Some(db_info);
            data.annos[genome_release][name] = Some(db);
        });
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
        "  try: http://{}:{}/genes/info?hgnc-id=HGNC:12403",
        args.listen_host.as_str(),
        args.listen_port
    );
    tracing::info!(
        "  try: http://{}:{}/annos/db-info?genome-release=grch37",
        args.listen_host.as_str(),
        args.listen_port
    );
    tracing::info!(
        "  try: http://{}:{}/annos/variant?genome-release=grch37&chromosome=1&pos=55505599&reference=C&alternative=G",
        args.listen_host.as_str(),
        args.listen_port
    );
    tracing::info!(
        "  try: http://{}:{}/annos/variant?genome-release=grch37&chromosome=1&pos=10001&reference=T&alternative=A",
        args.listen_host.as_str(),
        args.listen_port
    );
    tracing::info!(
        "  try: http://{}:{}/annos/range?genome-release=grch37&chromosome=1&start=1&stop=55516888",
        args.listen_host.as_str(),
        args.listen_port
    );
    actix_server::main(args, actix_web::web::Data::new(data))?;

    tracing::info!("All done. Have a nice day!");
    Ok(())
}
