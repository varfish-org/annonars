#![deny(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::module_name_repetitions)]
#![warn(missing_docs)]
//! Genome annotation stored in `RocksDB`.

mod error;

pub use crate::error::*;
