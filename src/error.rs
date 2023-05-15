//! Error type definition.

use std::path::PathBuf;

use thiserror::Error;

/// Error type for `annonars`
#[derive(Error, Debug)]
pub enum Error {
    /// Invalid type.
    #[error("invalid type for {0}: {1}")]
    InvalidType(String, String),
    /// Invalid integer value.
    #[error("invalid integer value: {0}")]
    InvalidInt(#[from] std::num::ParseIntError),
    /// Invalid float value.
    #[error("invalid float value: {0}")]
    InvalidFloat(#[from] std::num::ParseFloatError),
    /// No null value defined.
    #[error("no null value defined")]
    NoNullValue,
    /// Invalid UTF-8 in string.
    #[error("invalid UTF-8 in string: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
    /// Cannot deserialize "Unknown" type.
    #[error("cannot deserialize \"Unknown\" type")]
    UnknownType,
    /// Unsupported value from `serde_json::Value`.
    #[error("unsupported value: {0}")]
    UnsupportedValue(serde_json::Value),
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
