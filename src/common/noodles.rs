//! Noodles utility code.

use std::str::FromStr;

use noodles_vcf::record::info::field;

/// Extract a `String` field from a record.
pub fn get_string(record: &noodles_vcf::Record, name: &str) -> Result<String, anyhow::Error> {
    if let Some(Some(field::Value::String(v))) = record.info().get(&field::Key::from_str(name)?) {
        Ok(v.to_string())
    } else if let Some(Some(field::Value::Array(field::value::Array::String(vs)))) =
        record.info().get(&field::Key::from_str(name)?)
    {
        Ok(vs.get(0).unwrap().as_ref().unwrap().to_string())
    } else {
        anyhow::bail!("missing INFO/{} in gnomAD record", name)
    }
}

/// Extract a flag field from a record.
pub fn get_flag(record: &noodles_vcf::Record, name: &str) -> Result<bool, anyhow::Error> {
    Ok(matches!(
        record.info().get(&field::Key::from_str(name)?),
        Some(Some(field::Value::Flag))
    ))
}

/// Extract an `i32` field from a record.
pub fn get_i32(record: &noodles_vcf::Record, name: &str) -> Result<i32, anyhow::Error> {
    if let Some(Some(field::Value::Integer(v))) = record.info().get(&field::Key::from_str(name)?) {
        Ok(*v)
    } else if let Some(Some(field::Value::Array(field::value::Array::Integer(vs)))) =
        record.info().get(&field::Key::from_str(name)?)
    {
        Ok(vs.get(0).unwrap().unwrap())
    } else {
        anyhow::bail!("missing INFO/{} in gnomAD record", name)
    }
}

/// Extract an `f32` field from a record.
pub fn get_f32(record: &noodles_vcf::Record, name: &str) -> Result<f32, anyhow::Error> {
    if let Some(Some(field::Value::Float(v))) = record.info().get(&field::Key::from_str(name)?) {
        Ok(*v)
    } else if let Some(Some(field::Value::Array(field::value::Array::Float(vs)))) =
        record.info().get(&field::Key::from_str(name)?)
    {
        Ok(vs.get(0).unwrap().unwrap())
    } else {
        anyhow::bail!("missing INFO/{} in gnomAD record", name)
    }
}

/// Extract an `Vec<FromStr>` field from a record encoded as a pipe symbol separated string.
pub fn get_vec<T>(record: &noodles_vcf::Record, name: &str) -> Result<Vec<T>, anyhow::Error>
where
    T: FromStr,
{
    if let Some(Some(field::Value::String(v))) = record.info().get(&field::Key::from_str(name)?) {
        v.split('|')
            .map(|s| s.parse())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| anyhow::anyhow!("failed to parse INFO/{} as Vec<_>", name))
    } else {
        anyhow::bail!("missing INFO/{} in gnomAD record", name)
    }
}

/// Extract an `Vec<Vec<FromStr>>` field from a record encoded as a list of pipe symbol
/// separated string.
pub fn get_vec_vec<T>(record: &noodles_vcf::Record, name: &str) -> Result<Vec<T>, anyhow::Error>
where
    T: FromStr,
{
    if let Some(Some(field::Value::Array(field::value::Array::String(value)))) =
        record.info().get(&field::Key::from_str(name)?)
    {
        Ok(value
            .iter()
            .map(|s| {
                s.as_ref()
                    .ok_or(anyhow::anyhow!("missing value in INFO/hap_hl_hist"))
                    .map(|s| {
                        s.split('|')
                            .map(|s| s.parse())
                            .collect::<Result<Vec<T>, _>>()
                    })
            })
            .map(|r| match r {
                Ok(Ok(v)) => Ok(v),
                _ => anyhow::bail!("missing value in INFO/hap_hl_hist"),
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| anyhow::anyhow!("failed to parse INFO/{} as Vec<Vec<_>>: {}", name, e))?
            .into_iter()
            .flatten()
            .collect())
    } else {
        anyhow::bail!("missing INFO/{} in gnomAD record", name)
    }
}
