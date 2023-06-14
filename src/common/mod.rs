//! Common and utility code.

pub mod cli;
pub mod keys;
pub mod noodles;
pub mod spdi;

/// The version of `annonars` package.
#[cfg(not(test))]
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// This allows us to override the version to `0.0.0` in tests.
pub fn version() -> &'static str {
    #[cfg(test)]
    return "0.0.0";
    #[cfg(not(test))]
    return VERSION;
}
