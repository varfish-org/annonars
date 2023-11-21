use annonars::{
    clinvar_genes, clinvar_minimal, clinvar_sv, common, cons, db_utils, dbsnp, freqs, functional,
    genes, gnomad_mtdna, gnomad_nuclear, gnomad_sv, helixmtdb, server, tsv,
};
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
    /// "genes" sub commands
    Gene(Gene),
    /// "tsv" sub commands
    Tsv(Tsv),
    /// "cons" sub commands
    Cons(Cons),
    /// "clinvar-genes" sub commands
    ClinvarGenes(ClinvarGenes),
    /// "clinvar-minimal" sub commands
    ClinvarMinimal(ClinvarMinimal),
    /// "clinvar-sv" sub commands
    ClinvarSv(ClinvarSv),
    /// "freqs" sub commands
    Freqs(Freqs),
    /// "functional" sub commands
    Functional(Functional),
    /// "dbsnp" sub commands
    Dbsnp(Dbsnp),
    /// "helixmtdb" sub commands
    Helixmtdb(Helixmtdb),
    /// "gnomad-mtdna" sub commands
    GnomadMtdna(GnomadMtdna),
    /// "gnomad-nuclear" sub commands
    GnomadNuclear(GnomadNuclear),
    /// "gnomad-sv" sub commands
    GnomadSv(GnomadSv),
    /// "db-utils" sub commands
    DbUtils(DbUtils),
    /// "run-server" command.
    RunServer(server::Args),
}

/// Parsing of "gene" subcommand
#[derive(Debug, Args, Clone)]
struct Gene {
    /// The sub command to run
    #[command(subcommand)]
    command: GeneCommands,
}

/// Enum supporting the parsing of "gene *" subcommands.
#[derive(Debug, Subcommand, Clone)]
enum GeneCommands {
    /// "import" sub command
    Import(Box<genes::cli::import::Args>),
    /// "query" sub command
    Query(Box<genes::cli::query::Args>),
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

/// Parsing of "clinvar-minimal" subcommand.
#[derive(Debug, Args, Clone)]
struct ClinvarGenes {
    /// The sub command to run
    #[command(subcommand)]
    command: ClinvarGeneCommands,
}

/// Enum supporting the parsing of "clinvar-gene *" subcommands.
#[derive(Debug, Subcommand, Clone)]
enum ClinvarGeneCommands {
    /// "import" sub command
    Import(clinvar_genes::cli::import::Args),
    /// "query" sub command
    Query(clinvar_genes::cli::query::Args),
}

/// Parsing of "clinvar-minimal" subcommand.
#[derive(Debug, Args, Clone)]
struct ClinvarMinimal {
    /// The sub command to run
    #[command(subcommand)]
    command: ClinvarMinimalCommands,
}

/// Enum supporting the parsing of "clinvar-minimal *" subcommands.
#[derive(Debug, Subcommand, Clone)]
enum ClinvarMinimalCommands {
    /// "import" sub command
    Import(clinvar_minimal::cli::import::Args),
    /// "query" sub command
    Query(clinvar_minimal::cli::query::Args),
}

/// Parsing of "clinvar-sv" subcommand.
#[derive(Debug, Args, Clone)]
struct ClinvarSv {
    /// The sub command to run
    #[command(subcommand)]
    command: ClinvarSvCommands,
}

/// Enum supporting the parsing of "clinvar-sv *" subcommands.
#[derive(Debug, Subcommand, Clone)]
enum ClinvarSvCommands {
    /// "import" sub command
    Import(clinvar_sv::cli::import::Args),
    /// "query" sub command
    Query(clinvar_sv::cli::query::Args),
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

/// Parsing of "freqs" subcommands.
#[derive(Debug, Args, Clone)]
struct Freqs {
    /// The sub command to run
    #[command(subcommand)]
    command: FreqsCommands,
}

/// Enum supporting the parsing of "freqs *" subcommands.
#[derive(Debug, Subcommand, Clone)]
enum FreqsCommands {
    /// "import" sub command
    Import(freqs::cli::import::Args),
    /// "query" sub command
    Query(freqs::cli::query::Args),
}

/// Parsing of "functional" subcommands.
#[derive(Debug, Args, Clone)]
struct Functional {
    /// The sub command to run
    #[command(subcommand)]
    command: FunctionalCommands,
}

/// Enum supporting the parsing of "functional *" subcommands.
#[derive(Debug, Subcommand, Clone)]
enum FunctionalCommands {
    /// "import" sub command
    Import(functional::cli::import::Args),
    /// "query" sub command
    Query(functional::cli::query::Args),
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

/// Enum supporting the parsing of "gnomad-nuclear *" subcommands.
#[derive(Debug, Subcommand, Clone)]
enum GnomadNuclearCommands {
    /// "import" sub command
    Import(gnomad_nuclear::cli::import::Args),
    /// "query" sub command
    Query(gnomad_nuclear::cli::query::Args),
}

/// Parsing of "gnomad-sv" subcommands.
#[derive(Debug, Args, Clone)]
struct GnomadSv {
    /// The sub command to run
    #[command(subcommand)]
    command: GnomadSvCommands,
}

/// Enum supporting the parsing of "gnomad-sv *" subcommands.
#[derive(Debug, Subcommand, Clone)]
enum GnomadSvCommands {
    /// "import" sub command
    Import(gnomad_sv::cli::import::Args),
    /// "query" sub command
    Query(gnomad_sv::cli::query::Args),
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
            Commands::Gene(args) => match &args.command {
                GeneCommands::Import(args) => genes::cli::import::run(&cli.common, args)?,
                GeneCommands::Query(args) => genes::cli::query::run(&cli.common, args)?,
            },
            Commands::Tsv(args) => match &args.command {
                TsvCommands::Import(args) => tsv::cli::import::run(&cli.common, args)?,
                TsvCommands::Query(args) => tsv::cli::query::run(&cli.common, args)?,
            },
            Commands::ClinvarGenes(args) => match &args.command {
                ClinvarGeneCommands::Import(args) => {
                    clinvar_genes::cli::import::run(&cli.common, args)?
                }
                ClinvarGeneCommands::Query(args) => {
                    clinvar_genes::cli::query::run(&cli.common, args)?
                }
            },
            Commands::ClinvarMinimal(args) => match &args.command {
                ClinvarMinimalCommands::Import(args) => {
                    clinvar_minimal::cli::import::run(&cli.common, args)?
                }
                ClinvarMinimalCommands::Query(args) => {
                    clinvar_minimal::cli::query::run(&cli.common, args)?
                }
            },
            Commands::ClinvarSv(args) => match &args.command {
                ClinvarSvCommands::Import(args) => clinvar_sv::cli::import::run(&cli.common, args)?,
                ClinvarSvCommands::Query(args) => clinvar_sv::cli::query::run(&cli.common, args)?,
            },
            Commands::Cons(args) => match &args.command {
                ConsCommands::Import(args) => cons::cli::import::run(&cli.common, args)?,
                ConsCommands::Query(args) => cons::cli::query::run(&cli.common, args)?,
            },
            Commands::Dbsnp(args) => match &args.command {
                DbsnpCommands::Import(args) => dbsnp::cli::import::run(&cli.common, args)?,
                DbsnpCommands::Query(args) => dbsnp::cli::query::run(&cli.common, args)?,
            },
            Commands::Freqs(args) => match &args.command {
                FreqsCommands::Import(args) => freqs::cli::import::run(&cli.common, args)?,
                FreqsCommands::Query(args) => freqs::cli::query::run(&cli.common, args)?,
            },
            Commands::Functional(args) => match &args.command {
                FunctionalCommands::Import(args) => {
                    functional::cli::import::run(&cli.common, args)?
                }
                FunctionalCommands::Query(args) => functional::cli::query::run(&cli.common, args)?,
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
            Commands::GnomadSv(args) => match &args.command {
                GnomadSvCommands::Import(args) => gnomad_sv::cli::import::run(&cli.common, args)?,
                GnomadSvCommands::Query(args) => gnomad_sv::cli::query::run(&cli.common, args)?,
            },
            Commands::DbUtils(args) => match &args.command {
                DbUtilsCommands::Copy(args) => db_utils::cli::copy::run(&cli.common, args)?,
                DbUtilsCommands::DumpMeta(args) => {
                    db_utils::cli::dump_meta::run(&cli.common, args)?
                }
            },
            Commands::RunServer(args) => server::run(&cli.common, args)?,
        }

        Ok::<(), Error>(())
    })?;

    tracing::info!("All done! Have a nice day.");

    Ok(())
}
