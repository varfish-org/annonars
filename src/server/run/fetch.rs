//! Fetching of data for the Actix server.

use crate::common::keys;

use super::error::CustomError;

/// Function to fetch prost Message from a variant database.
pub fn fetch_var_protobuf<T>(
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    cf_name: &str,
    key: keys::Var,
) -> Result<Option<serde_json::Value>, CustomError>
where
    T: prost::Message + serde::Serialize + Default,
{
    let cf_data = db
        .cf_handle(cf_name)
        .unwrap_or_else(|| panic!("unknown column family: {}", cf_name));
    let key: Vec<u8> = key.into();

    let raw_data = db
        .get_cf(&cf_data, key)
        .map_err(|e| CustomError::new(anyhow::anyhow!("problem querying database: {}", e)))?;
    raw_data
        .map(|raw_data| {
            let msg: T = prost::Message::decode(&raw_data[..]).map_err(|e| {
                CustomError::new(anyhow::anyhow!(
                    "problem decoding protobuf from database (cf_name={}): {}",
                    cf_name,
                    e
                ))
            })?;
            serde_json::to_value(msg).map_err(|e| {
                CustomError::new(anyhow::anyhow!("problem decoding JSON from database: {e}",))
            })
        })
        .transpose()
}

/// Function to fetch prost Message from a position database.
pub fn fetch_pos_protobuf<T>(
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    cf_name: &str,
    start: keys::Pos,
    stop: keys::Pos,
) -> Result<Option<serde_json::Value>, CustomError>
where
    T: prost::Message + serde::Serialize + Default,
{
    let stop = crate::common::keys::Pos {
        chrom: stop.chrom.clone(),
        pos: stop.pos,
    };

    let cf_data = db.cf_handle(cf_name).unwrap();
    let mut iter = db.raw_iterator_cf(&cf_data);
    let start: Vec<u8> = start.into();
    iter.seek(&start);

    let mut result = Vec::new();
    while iter.valid() {
        if let Some(raw_value) = iter.value() {
            let iter_key = iter.key().unwrap();
            let iter_pos: crate::common::keys::Pos = iter_key.into();

            if iter_pos.chrom != stop.chrom || iter_pos.pos > stop.pos {
                break;
            }

            let msg: T = prost::Message::decode(raw_value).map_err(|e| {
                CustomError::new(anyhow::anyhow!(
                    "problem decoding protobuf from database (cf_name={}): {}",
                    cf_name,
                    e
                ))
            })?;
            result.push(serde_json::to_value(msg).map_err(|e| {
                CustomError::new(anyhow::anyhow!("problem decoding JSON from database: {e}",))
            })?);

            iter.next();
        }
    }

    Ok(Some(serde_json::Value::Array(result)))
}

/// Function to fetch a crate::tsv record from a database by variant.
pub fn fetch_var_tsv_json(
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    cf_name: &str,
    key: keys::Var,
) -> Result<Option<serde_json::Value>, CustomError> {
    let (db_schema, ctx) = fetch_tsv_json_prepare_db(db, cf_name)?;
    let cf_data = db
        .cf_handle(cf_name)
        .ok_or(CustomError::new(anyhow::anyhow!(
            "TSV data does not have a column family named {}",
            cf_name
        )))?;

    let key: Vec<u8> = key.into();
    let raw_value = db.get_cf(&cf_data, key).map_err(|e| {
        CustomError::new(anyhow::anyhow!(
            "problem querying database (cf_name={}): {}",
            cf_name,
            e
        ))
    })?;
    let values = if let Some(raw_value) = raw_value {
        let line = std::str::from_utf8(raw_value.as_slice()).map_err(|e| {
            CustomError::new(anyhow::anyhow!(
                "problem decoding value from database: {}",
                e
            ))
        })?;
        Some(ctx.line_to_values(line).map_err(|e| {
            CustomError::new(anyhow::anyhow!(
                "problem decoding value from database: {}",
                e
            ))
        })?)
    } else {
        None
    };

    fetch_tsv_json_prepare_result(values, db_schema)
}

/// Function to fetch a crate::tsv record from a database by position.
pub fn fetch_pos_tsv_json(
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    cf_name: &str,
    start: keys::Pos,
    stop: keys::Pos,
) -> Result<Option<serde_json::Value>, CustomError> {
    let (db_schema, ctx) = fetch_tsv_json_prepare_db(db, cf_name)?;
    let cf_data = db
        .cf_handle(cf_name)
        .ok_or(CustomError::new(anyhow::anyhow!(
            "TSV data does not have a column family named {}",
            cf_name
        )))?;

    // Obtain iterator and seek to start.
    let mut iter = db.raw_iterator_cf(&cf_data);
    let pos: keys::Pos = start;
    let key: Vec<u8> = pos.into();
    tracing::debug!("seeking to key {:?}", &key);
    iter.seek(&key);

    // Cast stop to `keys::Pos`.
    let stop: keys::Pos = stop;
    tracing::debug!("stop = {:?}", &stop);

    // Iterate over all variants until we are behind stop.
    let mut values = Vec::new();
    while iter.valid() {
        if let Some(raw_value) = iter.value() {
            tracing::trace!("iterator at {:?} => {:?}", &iter.key(), &raw_value);
            let iter_key = iter.key().unwrap();
            let iter_pos: keys::Pos = iter_key.into();

            if iter_pos.chrom != stop.chrom || iter_pos.pos > stop.pos {
                break;
            }

            let line = std::str::from_utf8(raw_value).map_err(|e| {
                CustomError::new(anyhow::anyhow!(
                    "problem decoding value from database: {}",
                    e
                ))
            })?;
            let mut tmp = ctx.line_to_values(line).map_err(|e| {
                CustomError::new(anyhow::anyhow!(
                    "problem decoding value from database: {}",
                    e
                ))
            })?;
            values.append(&mut tmp);

            iter.next();
        } else {
            break;
        }
    }

    fetch_tsv_json_prepare_result(Some(values), db_schema)
}
/// Helper function for `fetch_*_tsv_json`.
pub fn fetch_tsv_json_prepare_result(
    values: Option<Vec<serde_json::Value>>,
    db_schema: crate::tsv::schema::FileSchema,
) -> Result<Option<serde_json::Value>, CustomError> {
    Ok(values.as_ref().map(|values| {
        let mut m = serde_json::Map::new();
        for (col, value) in db_schema.columns.iter().zip(values.iter()) {
            m.insert(col.name.clone(), value.clone());
        }
        serde_json::Value::Object(m)
    }))
}

/// Helper function that opens the atabase for `fetch_*_tsv_json`.
pub fn fetch_tsv_json_prepare_db(
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    cf_name: &str,
) -> Result<(crate::tsv::schema::FileSchema, crate::tsv::coding::Context), CustomError> {
    let db_schema: crate::tsv::schema::FileSchema =
        rocksdb_utils_lookup::fetch_meta(db, "db-schema")
            .map_err(|e| CustomError::new(anyhow::anyhow!("problem loading metadata: {}", e)))?
            .map(|s| {
                serde_json::from_str(&s).map_err(|e| {
                    CustomError::new(anyhow::anyhow!(
                        "problem loading schema from JSON (cf_name={}): {}",
                        cf_name,
                        e
                    ))
                })
            })
            .transpose()?
            .ok_or(CustomError::new(anyhow::anyhow!(
                "db-schema not found in TSV data"
            )))?;
    let infer_config: crate::tsv::schema::infer::Config =
        rocksdb_utils_lookup::fetch_meta(db, "db-infer-config")
            .map_err(|e| CustomError::new(anyhow::anyhow!("problem loading metadata: {}", e)))?
            .map(|s| {
                serde_json::from_str(&s).map_err(|e| {
                    CustomError::new(anyhow::anyhow!(
                        "problem loading inference from JSON: {}",
                        e
                    ))
                })
            })
            .transpose()?
            .ok_or(CustomError::new(anyhow::anyhow!(
                "db-schema not found in TSV data"
            )))?;
    let ctx = crate::tsv::coding::Context::new(infer_config, db_schema.clone());

    Ok((db_schema, ctx))
}
