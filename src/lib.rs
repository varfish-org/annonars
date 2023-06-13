//! Genome annotation stored in `RocksDB`.
#![allow(clippy::must_use_candidate)]
#![allow(clippy::module_name_repetitions)]
#![warn(missing_docs)]

pub mod clinvar_minimal;
pub mod common;
pub mod cons;
pub mod db_utils;
pub mod dbsnp;
mod error;
pub mod freqs;
pub mod gnomad_mtdna;
pub mod gnomad_nuclear;
pub mod gnomad_pbs;
pub mod helixmtdb;
pub mod server;
pub mod tsv;

pub use crate::error::*;

/// The version of the `annona-rs` package.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
