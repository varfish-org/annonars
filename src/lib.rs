#![allow(clippy::must_use_candidate)]
#![allow(clippy::module_name_repetitions)]
#![warn(missing_docs)]
//! Genome annotation stored in `RocksDB`.

pub mod common;
mod error;
pub mod tsv;

pub use crate::error::*;

/// The version of the `annona-rs` package.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
