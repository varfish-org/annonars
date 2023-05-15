//! Example for running code from the `annonars` crate.

use anyhow::Error;
use clap::{arg, command, Args, Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};

/// Commonly used command line arguments.
#[derive(Parser, Debug)]
pub struct CommonArgs {
    /// Verbosity of the program
    #[clap(flatten)]
    pub verbose: Verbosity<InfoLevel>,
}

/// CLI parser based on clap.
#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "RocksDB-based genome annotations",
    long_about = "Genome annotation stored in RocksDB."
)]
struct Cli {
    /// Commonly used arguments
    #[command(flatten)]
    common: CommonArgs,

    /// The sub command to run
    #[command(subcommand)]
    command: Commands,
}

/// Enum supporting the parsing of top-level commands.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Subcommand)]
enum Commands {
    /// "foo" sub command
    Foo(FooArgs),
}

/// Parsing of "foo" subcommand
#[derive(Debug, Args)]
struct FooArgs {
    /// The arguments.
    #[arg()]
    pub args: Vec<String>,
}

/// Implementation of "export" command.
fn main_foo(common_args: &CommonArgs, args: &FooArgs) -> Result<(), Error> {
    tracing::debug!("common_args = {:?}", &common_args);
    tracing::debug!("args = {:?}", &args);

    tracing::info!("args = {:#?}", &args);

    Ok(())
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
            Commands::Foo(args) => {
                main_foo(&cli.common, args)?;
            }
        }

        Ok::<(), Error>(())
    })?;

    tracing::info!("All done! Have a nice day.");

    Ok(())
}

#[cfg(test)]
mod test {
    use clap_verbosity_flag::Verbosity;

    use super::main_foo;
    use crate::{CommonArgs, FooArgs};

    #[test]
    fn run_cmd() -> Result<(), Error> {
        main_foo(
            &CommonArgs {
                verbose: Verbosity::new(0, 0),
            },
            &FooArgs {
                args: vec![String::from("foo")],
            },
        )
    }
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
