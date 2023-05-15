//! Error type definition.

use thiserror::Error;

/// Error type for `annonars`
#[derive(Error, Debug)]
pub enum Error {
    /// Other error.
    #[error("other error")]
    OtherError,
}
