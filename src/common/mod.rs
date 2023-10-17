//! Common and utility code.

use std::path::{Path, PathBuf};

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

fn _readlink_f(path: &Path, level: u32) -> Result<PathBuf, anyhow::Error> {
    if level == 0 {
        anyhow::bail!(
            "maximal recursion depth for {:?}",
            path.as_os_str().to_string_lossy()
        )
    } else if path.is_symlink() {
        _readlink_f(&std::fs::read_link(path)?, level.saturating_sub(1))
    } else {
        Ok(path.to_path_buf())
    }
}

/// Recursively resolve the `path`.
pub fn readlink_f<P>(path: P) -> Result<PathBuf, anyhow::Error>
where
    P: AsRef<Path>,
{
    _readlink_f(path.as_ref(), 20)
}
