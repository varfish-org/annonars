//! Genome annotation stored in `RocksDB`.
#![allow(clippy::must_use_candidate)]
#![allow(clippy::module_name_repetitions)]
#![warn(missing_docs)]

pub mod clinvar_genes;
pub mod clinvar_minimal;
pub mod clinvar_sv;
pub mod common;
pub mod cons;
pub mod db_utils;
pub mod dbsnp;
mod error;
pub mod freqs;
pub mod functional;
pub mod genes;
pub mod gnomad_mtdna;
pub mod gnomad_nuclear;
pub mod gnomad_pbs;
pub mod gnomad_sv;
pub mod helixmtdb;
pub mod pbs;
pub mod server;
pub mod tsv;

pub use crate::error::*;

/// The version of the `annonars` package.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
