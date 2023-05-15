use annonars::{common, tsv};
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
            },
        }

        Ok::<(), Error>(())
    })?;

    tracing::info!("All done! Have a nice day.");

    Ok(())
}

// <LICENSE>
// Copyright 2023 annonars Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
// </LICENSE>
