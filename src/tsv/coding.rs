//! Coding and decoding of values for character separated text (CSV/TSV) files.

use byteorder::{BigEndian, ByteOrder};

use crate::{common, error};

use super::schema;

/// Encapsulate the coding and decoding of character-separated lines.
#[derive(Debug, Clone)]
pub struct Context {
    /// The configuration that was used for schema inference.
    pub config: schema::infer::Config,
    /// The schema to use for the coding and decoding.
    pub schema: schema::FileSchema,
}

impl Context {
    /// Return number of columns in the schema.
    pub fn num_columns(&self) -> usize {
        self.schema.columns.len()
    }

    /// Create a new context for coding and decoding.
    pub fn new(config: schema::infer::Config, schema: schema::FileSchema) -> Self {
        Self { config, schema }
    }

    /// Convert a line of text into a sequence of `serde_json::Value`s.
    ///
    /// Note that only the value types `Null`, `String`, `Number`, and `Float` are used.
    pub fn line_to_values(&self, line: &str) -> Result<Vec<serde_json::Value>, error::Error> {
        let entries = line.split(self.config.field_delimiter).collect::<Vec<_>>();
        if entries.len() != self.num_columns() {
            return Err(error::Error::ColumnCount(entries.len(), self.num_columns()));
        }

        let mut res = Vec::new();
        for (val, col_schema) in entries.into_iter().zip(self.schema.columns.iter()) {
            if self.config.null_values.iter().any(|nv| nv == val) {
                res.push(serde_json::Value::Null);
                continue;
            }
            match col_schema.typ {
                schema::ColumnType::String => {
                    res.push(val.into());
                }
                schema::ColumnType::Float => {
                    let val: f64 = val
                        .parse()
                        .map_err(|e| error::Error::InvalidFloat(val.to_string(), e))?;
                    res.push(val.into());
                }
                schema::ColumnType::Integer => {
                    let val: i32 = val
                        .parse()
                        .map_err(|e| error::Error::InvalidInt(val.to_string(), e))?;
                    res.push(val.into());
                }
                schema::ColumnType::Unknown => return Err(error::Error::UnknownType),
            }
        }

        Ok(res)
    }

    /// Convert a sequence of `serde_json::Value`s into a line of text.
    pub fn values_to_line(&self, values: &[serde_json::Value]) -> Result<String, error::Error> {
        let mut res = String::new();

        for (i, val) in values.iter().enumerate() {
            if i > 0 {
                res.push(self.config.field_delimiter);
            }
            match val {
                serde_json::Value::Null => {
                    if let Some(null_value) = self.config.null_values.first() {
                        res.push_str(null_value);
                    } else {
                        return Err(error::Error::NoNullValue);
                    }
                }
                serde_json::Value::Number(n) => {
                    res.push_str(&format!("{n}"));
                }
                serde_json::Value::String(s) => {
                    res.push_str(s);
                }
                serde_json::Value::Bool(_)
                | serde_json::Value::Array(_)
                | serde_json::Value::Object(_) => {
                    return Err(error::Error::UnsupportedValue((*val).clone()));
                }
            }
        }

        Ok(res)
    }

    /// Encode a vector of `serde_json::Value`s into a vector of bytes.
    pub fn encode_values(&self, values: &[&serde_json::Value]) -> Result<Vec<u8>, error::Error> {
        // Create bit mask buffer initialized with all zeros.
        let mut mask = boolvec::BoolVec::filled_with(self.num_columns(), false);
        // Pre-allocate space for bitmask in result.
        let mask_bytes = (self.num_columns() + 7) / 8;
        let mut res = vec![0; mask_bytes];

        for (i, val) in values.iter().enumerate() {
            match val {
                serde_json::Value::Null => {
                    // nothing to write out, all good
                }
                serde_json::Value::String(s) => {
                    // set bit in mask
                    mask.set(i, true);
                    // write out as null-terminated string
                    s.as_bytes().iter().for_each(|b| res.push(*b));
                    res.push(b'\0');
                }
                serde_json::Value::Number(n) => {
                    // set bit in mask
                    mask.set(i, true);
                    // write out as 32-bit integer or float
                    if n.is_u64() {
                        let val = n
                            .as_u64()
                            .ok_or_else(|| error::Error::UnsupportedValue((*val).clone()))?;
                        let val = val as i32;
                        let mut buf = [0; 4];
                        BigEndian::write_i32(&mut buf, val);
                        buf.into_iter().for_each(|b| res.push(b));
                    } else if n.is_i64() {
                        let val = n
                            .as_i64()
                            .ok_or_else(|| error::Error::UnsupportedValue((*val).clone()))?;
                        let val = val as i32;
                        let mut buf = [0; 4];
                        BigEndian::write_i32(&mut buf, val);
                        buf.into_iter().for_each(|b| res.push(b));
                    } else if n.is_f64() {
                        let val = n
                            .as_f64()
                            .ok_or_else(|| error::Error::UnsupportedValue((*val).clone()))?;
                        let mut buf = [0; 8];
                        BigEndian::write_f64(&mut buf, val);
                        buf.into_iter().for_each(|b| res.push(b));
                    } else {
                        return Err(error::Error::UnsupportedValue((*val).clone()));
                    }
                }
                serde_json::Value::Bool(_)
                | serde_json::Value::Array(_)
                | serde_json::Value::Object(_) => {
                    return Err(error::Error::UnsupportedValue((*val).clone()));
                }
            }
        }

        // Copy the bit mask bytes into the output.
        for (i, b) in mask.bytes().enumerate() {
            res[i] = *b;
        }

        Ok(res)
    }

    /// Decode vector of bytes to a vector of `serde_json::Value`s.
    pub fn decode_values(&self, bytes: &[u8]) -> Result<Vec<serde_json::Value>, error::Error> {
        // Get bitmask bytes from input and update input slice.
        let mask_bytes = (self.num_columns() + 7) / 8;
        let mask = boolvec::BoolVec::from_vec(bytes[..mask_bytes].to_vec());
        let bytes = &bytes[mask_bytes..];

        // Now decode the values with the help of the bitmask.
        let mut res = Vec::new();
        let mut offset = 0;
        for (mask_bit, col_schema) in mask.iter().zip(self.schema.columns.iter()) {
            if !mask_bit {
                res.push(serde_json::Value::Null);
                continue;
            }

            match col_schema.typ {
                schema::ColumnType::String => {
                    let mut val = Vec::new();
                    while offset < bytes.len() && bytes[offset] != b'\0' {
                        val.push(bytes[offset]);
                        offset += 1;
                    }
                    offset += 1;
                    let val = String::from_utf8(val).map_err(error::Error::InvalidUtf8)?;
                    res.push(val.into());
                }
                schema::ColumnType::Float => {
                    let val = BigEndian::read_f64(&bytes[offset..offset + 8]);
                    res.push(val.into());
                    offset += 8;
                }
                schema::ColumnType::Integer => {
                    let val = BigEndian::read_i32(&bytes[offset..offset + 4]);
                    res.push(val.into());
                    offset += 4;
                }
                schema::ColumnType::Unknown => return Err(error::Error::UnknownType),
            }
        }

        Ok(res)
    }

    /// Extract `common::keys::Var` from a vector of `serde_json::Value`s.
    pub fn values_to_var(
        &self,
        values: &[&serde_json::Value],
    ) -> Result<Option<common::keys::Var>, error::Error> {
        let mut res = common::keys::Var::default();

        for (val, col) in values.iter().zip(self.schema.columns.iter()) {
            if col.name == self.config.col_chrom {
                if val.is_null() {
                    // skip if not lifted to this genome build
                    return Ok(None);
                } else if let serde_json::Value::String(chrom) = val {
                    res.chrom.clone_from(chrom);
                } else {
                    return Err(error::Error::InvalidType(
                        self.config.col_chrom.clone(),
                        format!("{}", val),
                    ));
                }
            } else if col.name == self.config.col_start {
                if let serde_json::Value::Number(n) = val {
                    if n.is_i64() && n.as_i64().is_some() {
                        res.pos = n.as_i64().unwrap() as i32;
                    } else if n.is_u64() && n.as_u64().is_some() {
                        res.pos = n.as_u64().unwrap() as i32;
                    } else {
                        return Err(error::Error::InvalidType(
                            self.config.col_start.clone(),
                            format!("{}", val),
                        ));
                    }
                } else {
                    return Err(error::Error::InvalidType(
                        self.config.col_start.clone(),
                        format!("{}", val),
                    ));
                }
            } else if col.name == self.config.col_ref {
                if let serde_json::Value::String(reference) = val {
                    res.reference.clone_from(reference);
                } else {
                    return Err(error::Error::InvalidType(
                        self.config.col_ref.clone(),
                        format!("{}", val),
                    ));
                }
            } else if col.name == self.config.col_alt {
                if let serde_json::Value::String(alternative) = val {
                    res.alternative.clone_from(alternative);
                } else {
                    return Err(error::Error::InvalidType(
                        self.config.col_alt.clone(),
                        format!("{}", val),
                    ));
                }
            }
        }

        if !common::cli::is_canonical(&res.chrom) {
            tracing::trace!("skipping on non-canonical chrom: {}", &res.chrom);
            return Ok(None);
        }

        Ok(Some(res))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn simple_schema_context() -> Context {
        let config = schema::infer::Config {
            null_values: vec![String::from("NA")],
            ..schema::infer::Config::default()
        };
        let schema = schema::FileSchema::from(
            vec![
                schema::ColumnSchema::from("a", schema::ColumnType::String),
                schema::ColumnSchema::from("b", schema::ColumnType::Integer),
                schema::ColumnSchema::from("c", schema::ColumnType::Float),
                schema::ColumnSchema::from("d", schema::ColumnType::String),
            ],
            vec![String::from(".")],
        );
        Context::new(config, schema)
    }

    fn example_values() -> Vec<serde_json::Value> {
        vec![
            serde_json::Value::Null,
            1i32.into(),
            2.1f64.into(),
            "hello".into(),
        ]
    }

    fn example_line() -> String {
        String::from("NA\t1\t2.1\thello\0")
    }

    #[test]
    fn context_encode_values() -> Result<(), anyhow::Error> {
        let ctx = simple_schema_context();
        let values = example_values();

        let res = ctx.encode_values(&values.iter().collect::<Vec<_>>())?;
        insta::assert_yaml_snapshot!(res);

        Ok(())
    }

    #[test]
    fn context_decode_values() -> Result<(), anyhow::Error> {
        let ctx = simple_schema_context();
        let values = example_values();
        let tmp = ctx.encode_values(&values.iter().collect::<Vec<_>>())?;

        let res = ctx.decode_values(&tmp)?;
        insta::assert_yaml_snapshot!(res);

        Ok(())
    }

    #[test]
    fn context_values_to_line() -> Result<(), anyhow::Error> {
        let ctx = simple_schema_context();
        let values = example_values();

        let res = ctx.values_to_line(&values)?;
        insta::assert_yaml_snapshot!(res);

        Ok(())
    }

    #[test]
    fn context_line_to_values() -> Result<(), anyhow::Error> {
        let ctx = simple_schema_context();

        let res = ctx.line_to_values(&example_line())?;
        insta::assert_yaml_snapshot!(res);

        Ok(())
    }
}
