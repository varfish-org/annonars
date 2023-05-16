use annonars::{common, cons, tsv};
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
    /// "tsv" sub command
    Tsv(Tsv),
    /// "cons" sub command
    Cons(Cons),
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

/// Parsing of "cons" subcommand
#[derive(Debug, Args, Clone)]
struct Cons {
    /// The sub command to run
    #[command(subcommand)]
    command: ConsCommands,
}

/// Enum supporting the parsing of "cons *" subcommands.
#[derive(Debug, Subcommand, Clone)]
enum ConsCommands {
    /// "import" sub command
    Import(cons::cli::import::Args),
    /// "query" sub command
    Query(cons::cli::query::Args),
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
        }

        Ok::<(), Error>(())
    })?;

    tracing::info!("All done! Have a nice day.");

    Ok(())
}
