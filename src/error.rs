//! Error type definition.

use std::path::PathBuf;

use thiserror::Error;

/// Error type for `annonars`
#[derive(Error, Debug)]
pub enum Error {
    /// Problem opening RocksDB.
    #[error("problem opening RocksDB at {0}: {1}")]
    RocksDBOpen(PathBuf, #[source] rocksdb::Error),
    /// Problem with RocksDB property query.
    #[error("problem accessing RocksDB property: {0}")]
    RocksDBProperty(#[source] rocksdb::Error),
    /// Other error.
    #[error("other error")]
    OtherError,
}
