//! Utility code for rocksdb access.

use std::{path::Path, time::Instant};

use crate::error;

/// Tune `RocksDB` options for bulk insertion.
///
/// Example:
///
/// ```
/// use annonars::common::rocks_utils::tune_options;
///
/// let options = tune_options(rocksdb::Options::default(), None);
/// ```
///
/// # Arguments
///
/// * `options` - `RocksDB` options to tune.
/// * `wal_dir` - Optional directory for write-ahead log files.
///
/// # Returns
///
/// Tuned `RocksDB` options.
pub fn tune_options(options: rocksdb::Options, wal_dir: Option<&str>) -> rocksdb::Options {
    let mut options = options;

    options.create_if_missing(true);
    options.create_missing_column_families(true);

    options.prepare_for_bulk_load();

    options.set_max_background_jobs(16);
    options.set_max_subcompactions(8);
    options.increase_parallelism(8);
    options.optimize_level_style_compaction(1 << 30);
    options.set_min_write_buffer_number(1);
    options.set_min_write_buffer_number_to_merge(1);
    options.set_write_buffer_size(1 << 30);
    options.set_target_file_size_base(1 << 30);
    options.set_compaction_style(rocksdb::DBCompactionStyle::Universal);

    if let Some(wal_dir) = wal_dir {
        options.set_wal_dir(wal_dir);
    }

    // Compress everything with zstd.
    options.set_compression_per_level(&[]);
    options.set_bottommost_compression_options(-14, 10, 0, 1 << 14, true);
    options.set_bottommost_compression_type(rocksdb::DBCompressionType::Zstd);
    options.set_bottommost_zstd_max_train_bytes(1 << 22, true);
    options.optimize_for_point_lookup(1 << 26);

    options
}

/// Force manual compaction of all column families at the given path.
///
/// This function will enumerate all column families and start a compaction of all of them.
/// It will then wait for the completion of all such compactions.
///
/// # Arguments
///
/// * `path` - Path to the `RocksDB` database.
/// * `options` - `RocksDB` options to use for opening database and column families.
/// * `wait_msg_prefix` - Optional prefix for the wait message.
pub fn force_compaction<P>(
    path: P,
    options: &rocksdb::Options,
    wait_msg_prefix: Option<&str>,
) -> Result<(), error::Error>
where
    P: AsRef<Path>,
{
    let cf_names = rocksdb::DB::list_cf(options, path.as_ref())
        .map_err(|e| error::Error::RocksDBOpen(path.as_ref().to_owned(), e))?;
    let cfs = cf_names
        .iter()
        .map(|s| (s, options.clone()))
        .collect::<Vec<_>>();
    let db = rocksdb::DB::open_cf_with_opts(options, path.as_ref(), cfs)
        .map_err(|e| error::Error::RocksDBOpen(path.as_ref().to_owned(), e))?;

    let cf_names_str = cf_names
        .iter()
        .map(std::string::String::as_str)
        .collect::<Vec<_>>();
    force_compaction_cf(&db, cf_names_str, wait_msg_prefix)
}

/// Force manual compaction of the given column families in the given database.
///
/// The function will enforce compaction of the bottommost level of all column families.
/// The compression will depend on the options that the database was opened with.  Using the
/// `tune_options` function is recommended to optimize the resulting database.
///
/// # Arguments
///
/// * `db` - `RocksDB` database to compact.
/// * `cf_names` - Names of the column families to compact.
/// * `wait_msg_prefix` - Optional prefix for the wait message.
pub fn force_compaction_cf<I, N>(
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    cf_names: I,
    wait_msg_prefix: Option<&str>,
) -> Result<(), error::Error>
where
    I: IntoIterator<Item = N>,
    N: AsRef<str>,
{
    // Collect columns families to run compaction for.
    let cfs = cf_names
        .into_iter()
        .map(|cf| db.cf_handle(cf.as_ref()).unwrap())
        .collect::<Vec<_>>();

    // Create compaction options and enforce bottommost level compaction.
    let mut compact_opt = rocksdb::CompactOptions::default();
    compact_opt.set_exclusive_manual_compaction(true);
    compact_opt.set_bottommost_level_compaction(rocksdb::BottommostLevelCompaction::Force);

    // Start the compaction for each column family.
    cfs.iter()
        .for_each(|cf| db.compact_range_cf_opt(cf, None::<&[u8]>, None::<&[u8]>, &compact_opt));
    let compaction_start = Instant::now();
    let mut last_logged = compaction_start;

    // Wait until all compactions are done.
    while db
        .property_int_value(rocksdb::properties::COMPACTION_PENDING)
        .map_err(error::Error::RocksDBProperty)?
        .unwrap()
        > 0
        || db
            .property_int_value(rocksdb::properties::NUM_RUNNING_COMPACTIONS)
            .map_err(error::Error::RocksDBProperty)?
            .unwrap()
            > 0
    {
        std::thread::sleep(std::time::Duration::from_millis(100));
        // Log to info every second that compaction is still running.
        if let Some(wait_msg_prefix) = wait_msg_prefix {
            if last_logged.elapsed() > std::time::Duration::from_millis(1000) {
                tracing::info!(
                    "{}still waiting for RocksDB compaction (since {:?})",
                    wait_msg_prefix,
                    compaction_start.elapsed()
                );
                last_logged = Instant::now();
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use temp_testdir::TempDir;

    use super::*;

    /// Smoke test for the `tune_options` function.
    #[test]
    fn smoke_test_tune_options() -> Result<(), anyhow::Error> {
        let options = rocksdb::Options::default();
        let _tuned = tune_options(options, None);

        Ok(())
    }

    /// Smoke test for the `force_compaction` function.
    #[test]
    fn smoke_test_force_compaction() -> Result<(), anyhow::Error> {
        let temp = TempDir::default();
        let path_db = temp.join("rocksdb");

        let mut options = rocksdb::Options::default();
        options.create_if_missing(true);
        options.create_missing_column_families(true);
        {
            let cf_names = &["foo", "bar"];
            let _db = rocksdb::DB::open_cf(&options, &path_db, cf_names)?;
        }

        force_compaction(&path_db, &options, Some("msg"))?;

        Ok(())
    }

    /// Smoke test for the `force_compaction` function.
    #[test]
    fn smoke_test_force_compaction_cf() -> Result<(), anyhow::Error> {
        let temp = TempDir::default();
        let path_db = temp.join("rocksdb");

        let mut options = rocksdb::Options::default();
        options.create_if_missing(true);
        options.create_missing_column_families(true);
        let cf_names = &["foo", "bar"];
        let db = rocksdb::DB::open_cf(&options, path_db, cf_names)?;

        force_compaction_cf(&db, cf_names, Some("msg"))?;

        Ok(())
    }
}
