//! Code for storing sequence variant frequency-only information.
//!
//! Rather than storing the full information from gnomAD etc, we store only the integer allele
//! counts that allow for annotating allele frequency.  This reduces the database size.  However,
//! it limits us to hard-coded database support which is acceptable:
//!
//! - for the following, we provide information on autosomal and gonosomal counts
//!   - gnomAD-exomes
//!   - gnomAD-genomes
//! - for the following, we provide information for mitochondrial DNA only
//! - gnomAD-mtDNA
//! - HelixMtDb

pub mod cli;
pub mod serialized;
