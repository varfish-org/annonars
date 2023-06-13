//! Errors for the Actix servers.

use actix_web::ResponseError;

/// Custom error type for the Actix server.
#[derive(Debug)]
pub struct CustomError {
    err: anyhow::Error,
}

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.err)
    }
}

impl CustomError {
    /// Create from `anyhow::Error`.
    pub fn new(err: anyhow::Error) -> Self {
        CustomError { err }
    }
}

impl ResponseError for CustomError {}
