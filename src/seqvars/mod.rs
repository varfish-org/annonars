//! The unified sequence-variant database plus the helpers shared by its inputs.
//!
//! The native tracks ([`crate::cadd`], [`crate::spliceai`]) use
//! compact contig-ID keys (see [`crate::common::keys::Var::encode_with_id`])
//! based on a contig dictionary derived from a reference FASTA index (see
//! [`crate::common::contig::ContigDict`]), so they work for arbitrary
//! assemblies. The dictionary is persisted in the `meta` column family so keys
//! can be decoded again.

pub mod cli;

use std::sync::Arc;

use crate::common::contig::ContigDict;

/// Number of records buffered in a `rocksdb::WriteBatch` before flushing during
/// import; per-record `put_cf` calls are too slow for genome-wide inputs.
pub const WRITE_BATCH_SIZE: u64 = 50_000;

/// Open a track RocksDB for writing with a `meta` and a single data column family.
pub fn open_track_db_for_write(
    path: impl AsRef<std::path::Path>,
    data_cf: &str,
    wal_dir: Option<&str>,
) -> Result<(Arc<rocksdb::DB>, Vec<String>), anyhow::Error> {
    if data_cf == "meta" {
        anyhow::bail!("data column family name must not be \"meta\" (reserved)");
    }
    let options = rocksdb_utils_lookup::tune_options(rocksdb::Options::default(), wal_dir);
    let cf_names = vec!["meta".to_string(), data_cf.to_string()];
    let db = Arc::new(rocksdb::DB::open_cf_with_opts(
        &options,
        crate::common::readlink_f(path.as_ref())?,
        cf_names
            .iter()
            .map(|name| (name.clone(), options.clone()))
            .collect::<Vec<_>>(),
    )?);
    Ok((db, cf_names))
}

/// Write standard meta information plus the contig dictionary into the `meta` CF.
pub fn write_track_meta(
    db: &rocksdb::DB,
    db_name: &str,
    db_version: &str,
    assembly: &str,
    dict: &ContigDict,
) -> Result<(), anyhow::Error> {
    let cf_meta = db
        .cf_handle("meta")
        .ok_or_else(|| anyhow::anyhow!("meta column family missing"))?;
    db.put_cf(&cf_meta, "annonars-version", crate::VERSION)?;
    db.put_cf(&cf_meta, "genome-release", assembly)?;
    db.put_cf(&cf_meta, "db-name", db_name)?;
    db.put_cf(&cf_meta, "db-version", db_version)?;
    db.put_cf(&cf_meta, "contig-dict", dict.to_meta_string())?;
    Ok(())
}

/// Read back the contig dictionary written by [`write_track_meta`].
pub fn read_contig_dict(db: &rocksdb::DB) -> Result<ContigDict, anyhow::Error> {
    let cf_meta = db
        .cf_handle("meta")
        .ok_or_else(|| anyhow::anyhow!("meta column family missing"))?;
    let raw = db
        .get_cf(&cf_meta, "contig-dict")?
        .ok_or_else(|| anyhow::anyhow!("missing contig-dict in meta CF"))?;
    ContigDict::from_meta_string(&String::from_utf8(raw)?)
}
