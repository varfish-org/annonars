//! Error type definition.

use std::path::PathBuf;

use thiserror::Error;

/// Error type for `annonars`
#[derive(Error, Debug)]
pub enum Error {
    /// Header missing in TSV file.
    #[error("header missing in TSV file")]
    HeaderMissing,
    /// Error in underlying I/O.
    #[error("I/O error: {0}")]
    Io(#[source] std::io::Error),
    /// Mismatching column counts.
    #[error("mismatching number of columns: {0} != {1}")]
    ColumnCount(usize, usize),
    /// Mismatching column names..
    #[error("mismatching number of column names: {0} != {1}")]
    ColumnName(String, String),
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
