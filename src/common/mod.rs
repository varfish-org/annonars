//! Common and utility code.

use std::path::{Path, PathBuf};

pub mod cli;
pub mod contig;
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

/// Allows to set the rstest snapshot suffix.
#[macro_export]
macro_rules! set_snapshot_suffix {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_suffix(format!($($expr,)*));
        let _guard = settings.bind_to_scope();
    }
}

pub use set_snapshot_suffix;

/// Assembly to be passed on the command line.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    clap::ValueEnum,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Assembly {
    /// GRCh37
    Grch37,
    /// GRCh38
    Grch38,
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    /// Databases written by RocksDB >= 11 use table format version 7 by default (e.g., the
    /// bioconda build of annonars) and must be readable.
    #[test]
    fn test_read_rocksdb_table_format_version_7() -> Result<(), anyhow::Error> {
        let tmp = temp_testdir::TempDir::default();

        let mut block_opts = rocksdb::BlockBasedOptions::default();
        block_opts.set_format_version(7);
        let mut options = rocksdb::Options::default();
        options.create_if_missing(true);
        options.set_block_based_table_factory(&block_opts);
        {
            let db = rocksdb::DB::open(&options, tmp.as_ref())?;
            db.put(b"key", b"value")?;
            db.flush()?;
        }

        let db =
            rocksdb::DB::open_for_read_only(&rocksdb::Options::default(), tmp.as_ref(), false)?;
        assert_eq!(db.get(b"key")?, Some(b"value".to_vec()));

        Ok(())
    }
}
