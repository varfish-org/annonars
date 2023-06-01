use annonars::{common, cons, db_utils, dbsnp, gnomad_mtdna, gnomad_nuclear, helixmtdb, tsv};
use anyhow::Error;
use clap::{command, Args, Parser, Subcommand};

/// CLI parser based on clap.
#[derive(Debug, Clone, Parser)]
#[command(
    author,
    version,
    about = "RocksDB-based genome annotations",
    long_about = "Genome annotation stored in RocksDB."
)]
struct Cli {
    /// Commonly used arguments
    #[command(flatten)]
    common: common::cli::Args,

    /// The sub command to run
    #[command(subcommand)]
    command: Commands,
}

/// Enum supporting the parsing of top-level commands.
#[derive(Debug, Subcommand, Clone)]
enum Commands {
    /// "tsv" sub commands
    Tsv(Tsv),
    /// "cons" sub commands
    Cons(Cons),
    /// "dbsnp" sub commands
    Dbsnp(Dbsnp),
    /// "helixmtdb" sub commands
    Helixmtdb(Helixmtdb),
    /// "gnomad-mtdna" sub commands
    GnomadMtdna(GnomadMtdna),
    /// "gnomad-nuclear" sub commands
    GnomadNuclear(GnomadNuclear),
    /// "db-utils" sub commands
    DbUtils(DbUtils),
}

/// Parsing of "tsv" subcommand
#[derive(Debug, Args, Clone)]
struct Tsv {
    /// The sub command to run
    #[command(subcommand)]
    command: TsvCommands,
}

/// Enum supporting the parsing of "tsv *" subcommands.
#[derive(Debug, Subcommand, Clone)]
enum TsvCommands {
    /// "import" sub command
    Import(tsv::cli::import::Args),
    /// "query" sub command
    Query(tsv::cli::query::Args),
}

/// Parsing of "cons" subcommand.
#[derive(Debug, Args, Clone)]
struct Cons {
    /// The sub command to run
    #[command(subcommand)]
    command: ConsCommands,
}

/// Enum supporting the parsing of "dbsnp *" subcommands.
#[derive(Debug, Subcommand, Clone)]
enum ConsCommands {
    /// "import" sub command
    Import(cons::cli::import::Args),
    /// "query" sub command
    Query(cons::cli::query::Args),
}

/// Parsing of "dbsnp" subcommands.
#[derive(Debug, Args, Clone)]
struct Dbsnp {
    /// The sub command to run
    #[command(subcommand)]
    command: DbsnpCommands,
}

/// Enum supporting the parsing of "dbsnp *" subcommands.
#[derive(Debug, Subcommand, Clone)]
enum DbsnpCommands {
    /// "import" sub command
    Import(dbsnp::cli::import::Args),
    /// "query" sub command
    Query(dbsnp::cli::query::Args),
}

/// Parsing of "helixmtdb" subcommands.
#[derive(Debug, Args, Clone)]
struct Helixmtdb {
    /// The sub command to run
    #[command(subcommand)]
    command: HelixmtdbCommands,
}

/// Enum supporting the parsing of "helixmtdb *" subcommands.
#[derive(Debug, Subcommand, Clone)]
enum HelixmtdbCommands {
    /// "import" sub command
    Import(helixmtdb::cli::import::Args),
    /// "query" sub command
    Query(helixmtdb::cli::query::Args),
}

/// Parsing of "gnomad-mtdna" subcommands.
#[derive(Debug, Args, Clone)]
struct GnomadMtdna {
    /// The sub command to run
    #[command(subcommand)]
    command: GnomadMtdnaCommands,
}

/// Enum supporting the parsing of "gnomad-mtdna *" subcommands.
#[derive(Debug, Subcommand, Clone)]
enum GnomadMtdnaCommands {
    /// "import" sub command
    Import(gnomad_mtdna::cli::import::Args),
    /// "query" sub command
    Query(gnomad_mtdna::cli::query::Args),
}

/// Parsing of "gnomad-nuclear" subcommands.
#[derive(Debug, Args, Clone)]
struct GnomadNuclear {
    /// The sub command to run
    #[command(subcommand)]
    command: GnomadNuclearCommands,
}

/// Enum supporting the parsing of "helixmtdb *" subcommands.
#[derive(Debug, Subcommand, Clone)]
enum GnomadNuclearCommands {
    /// "import" sub command
    Import(gnomad_nuclear::cli::import::Args),
    /// "query" sub command
    Query(gnomad_nuclear::cli::query::Args),
}

/// Parsing of "db-utils" subcommands.
#[derive(Debug, Args, Clone)]
struct DbUtils {
    /// The sub command to run
    #[command(subcommand)]
    command: DbUtilsCommands,
}

/// Enum supporting the parsing of "db-utils *" subcommands.
#[derive(Debug, Subcommand, Clone)]
enum DbUtilsCommands {
    /// "copy" sub command
    Copy(db_utils::cli::copy::Args),
    /// "dump-meta" sub command
    DumpMeta(db_utils::cli::dump_meta::Args),
}

pub fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    // Build a tracing subscriber according to the configuration in `cli.common`.
    let collector = tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(match cli.common.verbose.log_level() {
            Some(level) => match level {
                log::Level::Error => tracing::Level::ERROR,
                log::Level::Warn => tracing::Level::WARN,
                log::Level::Info => tracing::Level::INFO,
                log::Level::Debug => tracing::Level::DEBUG,
                log::Level::Trace => tracing::Level::TRACE,
            },
            None => tracing::Level::INFO,
        })
        .compact()
        .finish();

    tracing::subscriber::with_default(collector, || {
        match &cli.command {
            Commands::Tsv(args) => match &args.command {
                TsvCommands::Import(args) => tsv::cli::import::run(&cli.common, args)?,
                TsvCommands::Query(args) => tsv::cli::query::run(&cli.common, args)?,
            },
            Commands::Cons(args) => match &args.command {
                ConsCommands::Import(args) => cons::cli::import::run(&cli.common, args)?,
                ConsCommands::Query(args) => cons::cli::query::run(&cli.common, args)?,
            },
            Commands::Dbsnp(args) => match &args.command {
                DbsnpCommands::Import(args) => dbsnp::cli::import::run(&cli.common, args)?,
                DbsnpCommands::Query(args) => dbsnp::cli::query::run(&cli.common, args)?,
            },
            Commands::Helixmtdb(args) => match &args.command {
                HelixmtdbCommands::Import(args) => helixmtdb::cli::import::run(&cli.common, args)?,
                HelixmtdbCommands::Query(args) => helixmtdb::cli::query::run(&cli.common, args)?,
            },
            Commands::GnomadMtdna(args) => match &args.command {
                GnomadMtdnaCommands::Import(args) => {
                    gnomad_mtdna::cli::import::run(&cli.common, args)?
                }
                GnomadMtdnaCommands::Query(args) => {
                    gnomad_mtdna::cli::query::run(&cli.common, args)?
                }
            },
            Commands::GnomadNuclear(args) => match &args.command {
                GnomadNuclearCommands::Import(args) => {
                    gnomad_nuclear::cli::import::run(&cli.common, args)?
                }
                GnomadNuclearCommands::Query(args) => {
                    gnomad_nuclear::cli::query::run(&cli.common, args)?
                }
            },
            Commands::DbUtils(args) => match &args.command {
                DbUtilsCommands::Copy(args) => db_utils::cli::copy::run(&cli.common, args)?,
                DbUtilsCommands::DumpMeta(args) => {
                    db_utils::cli::dump_meta::run(&cli.common, args)?
                }
            },
        }

        Ok::<(), Error>(())
    })?;

    tracing::info!("All done! Have a nice day.");

    Ok(())
}
