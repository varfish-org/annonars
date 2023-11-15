//! Structural variants in ClinVar.
//!
//! The implementation is as follows.
//!
//! - The structural variants are stored by their VCV identifier.
//! - On startup, all variants are loaded and interval trees are built mapping
//!   region to VCV identifier.

pub mod cli;
