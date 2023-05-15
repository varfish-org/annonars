//! Schema inference and representation for TSV files.

use crate::error;

/// Possible column types.
///
/// We allow for a subset allowed in JSON.
///
/// The enum type is sorted in the sense that smaller value are more generic/less specific
/// than the larger ones.
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum ColumnType {
    /// String type.
    String,
    /// Float type; will be store as `f32`.
    Float,
    /// Integer type; will be stored as `i32`.
    Integer,
    /// Unknown, seen only null values.
    #[default]
    Unknown,
}

impl ColumnType {
    /// Make the column type as general as necessary to hold the given value.
    ///
    /// # Arguments
    ///
    /// * `val` - Value to extend the column type with.
    /// * `null_values` - List of null values.
    pub(crate) fn extend(&self, val: &str, null_values: &[&str]) -> Self {
        if null_values.contains(&val) {
            *self
        } else {
            let compat = if val.parse::<i64>().is_ok() {
                ColumnType::Integer
            } else if val.parse::<f64>().is_ok() {
                ColumnType::Float
            } else {
                ColumnType::String
            };

            *self.min(&compat)
        }
    }

    /// Merge the types of two columns and return the most general one.
    ///
    /// # Arguments
    ///
    /// * `other` - Other column type to merge with.
    pub(crate) fn merge(&self, other: &ColumnType) -> Self {
        *self.min(other)
    }
}

/// Schema description for one column.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ColumnSchema {
    /// Column name.
    pub name: String,
    /// Column type.
    pub typ: ColumnType,
}

impl FileSchema {
    /// Ensure that all column names are the same and return an error if not.
    /// Otherwise, perform a column-wise extension of the column type.
    pub fn merge(&self, other: &FileSchema) -> Result<FileSchema, error::Error> {
        // Check that the column names are the same and in the same order.
        if self.columns.len() != other.columns.len() {
            return Err(error::Error::ColumnCount(
                self.columns.len(),
                other.columns.len(),
            ));
        }
        for (col1, col2) in self.columns.iter().zip(other.columns.iter()) {
            if col1.name != col2.name {
                return Err(error::Error::ColumnName(
                    col1.name.clone(),
                    col2.name.clone(),
                ));
            }
        }
        // Now merge the column types.
        let columns = self
            .columns
            .iter()
            .zip(other.columns.iter())
            .map(|(col1, col2)| ColumnSchema {
                name: col1.name.clone(),
                typ: col1.typ.merge(&col2.typ),
            })
            .collect();

        Ok(FileSchema { columns })
    }
}

/// Schema description for a table.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FileSchema {
    /// The columns.
    pub columns: Vec<ColumnSchema>,
}

/// Schema inference.
pub mod infer {
    use std::io::BufRead;

    use crate::error;

    use super::{ColumnSchema, ColumnType, FileSchema};

    /// Configuration for schema inference.
    ///
    /// The `Default` trait provides appropriate defaults that could be used using
    /// VCF-style headers.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Config {
        /// Field delimiter to use.
        pub field_delimiter: char,
        /// Allow different number of columns in different rows.
        pub flexible: bool,
        /// Values to use for null.
        pub null_values: Vec<String>,
        /// Header prefix to strip (OK if missing).
        pub header_prefix: String,
        /// Number of rows to use for inferring schema.
        pub num_rows: usize,
        /// Number of rows to skip.
        pub skip_rows: usize,

        /// Column name for chromosome.
        pub col_chromosome: String,
        /// Column name for (start) position.
        pub col_start: String,
        /// Column name for reference allele.
        pub col_reference: String,
        /// Column name for alternative allele.
        pub col_alternative: String,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                field_delimiter: '\t',
                flexible: false,
                null_values: vec![String::from(""), String::from("."), String::from("NA")],
                header_prefix: String::from("#"),
                num_rows: 10_000,
                skip_rows: 0,
                col_chromosome: String::from("CHROM"),
                col_start: String::from("POS"),
                col_reference: String::from("REF"),
                col_alternative: String::from("ALT"),
            }
        }
    }

    /// Context for running the inference algorithm.
    #[derive(Debug, Clone, Default)]
    pub struct Context {
        /// Configuration for schema inference.
        config: Config,
    }

    impl Context {
        /// Create a new context with given configuration.
        ///
        /// # Arguments
        ///
        /// * `config` - Configuration for schema inference.
        pub fn with_config(config: Config) -> Self {
            Self { config }
        }

        /// Get default configuration for the given row.
        fn default_column_config(&self, name: &str) -> ColumnType {
            if name == self.config.col_chromosome
                || name == self.config.col_reference
                || name == self.config.col_alternative
            {
                ColumnType::String
            } else if name == self.config.col_start {
                ColumnType::Integer
            } else {
                ColumnType::Unknown
            }
        }

        /// Run the schema inference algorithm.
        pub fn infer_schema<R: BufRead>(&self, reader: R) -> Result<FileSchema, error::Error> {
            // Skip the first few rows, as configured.
            let mut reader = reader;
            for _i in 0..self.config.skip_rows {
                let mut buf = String::new();
                reader.read_line(&mut buf).map_err(error::Error::Io)?;
            }

            // Get the null values as `&str` into a shortcut variable.
            let null_values = self
                .config
                .null_values
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>();
            let mut columns: Option<Vec<ColumnSchema>> = None;
            let mut seen_rows = 0;

            // Read the first few rows as configured and infer the schema.
            for result in reader.lines() {
                let line = result.map_err(error::Error::Io)?;
                let record = line.split(self.config.field_delimiter).collect::<Vec<_>>();
                // Track number of seen rows.
                seen_rows += 1;

                if let Some(columns) = columns.as_mut() {
                    // We have seen the header and can extend the schema.
                    record
                        .into_iter()
                        .zip(columns.iter_mut())
                        .for_each(|(record, column)| {
                            column.typ = column.typ.extend(record, &null_values);
                        })
                } else {
                    // Assign header for first row, optionally strip header prefix.
                    columns = Some(
                        record
                            .into_iter()
                            .enumerate()
                            .map(|(i, val)| {
                                let val = if i == 0 {
                                    val.strip_prefix(&self.config.header_prefix).unwrap_or(val)
                                } else {
                                    val
                                };

                                ColumnSchema {
                                    name: val.to_string(),
                                    typ: self.default_column_config(&val),
                                }
                            })
                            .collect::<Vec<_>>(),
                    );
                }

                if seen_rows > self.config.num_rows {
                    break;
                }
            }

            if let Some(columns) = columns {
                if seen_rows == 1 {
                    tracing::warn!("only seen header row, assuming strings");
                    Ok(FileSchema {
                        columns: columns
                            .iter()
                            .map(|c| ColumnSchema {
                                name: c.name.clone(),
                                typ: self.default_column_config(&c.name),
                            })
                            .collect(),
                    })
                } else {
                    Ok(FileSchema { columns })
                }
            } else {
                Err(error::Error::HeaderMissing)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::BufReader;

    use super::*;

    #[test]
    fn column_type_extend() {
        assert_eq!(ColumnType::Integer.extend("x", &["."]), ColumnType::String,);
        assert_eq!(ColumnType::Integer.extend("1.0", &["."]), ColumnType::Float,);
        assert_eq!(ColumnType::Integer.extend("1", &["."]), ColumnType::Integer,);
        assert_eq!(ColumnType::Integer.extend(".", &["."]), ColumnType::Integer,);

        assert_eq!(ColumnType::Float.extend("x", &["."]), ColumnType::String,);
        assert_eq!(ColumnType::Float.extend("1.0", &["."]), ColumnType::Float,);
        assert_eq!(ColumnType::Float.extend("1", &["."]), ColumnType::Float,);
        assert_eq!(ColumnType::Float.extend(".", &["."]), ColumnType::Float,);

        assert_eq!(ColumnType::String.extend("x", &["."]), ColumnType::String,);
        assert_eq!(ColumnType::String.extend("1.0", &["."]), ColumnType::String,);
        assert_eq!(ColumnType::String.extend("1", &["."]), ColumnType::String,);
        assert_eq!(ColumnType::String.extend(".", &["."]), ColumnType::String,);
    }

    #[test]
    fn infer_schema_empty() -> Result<(), anyhow::Error> {
        let config = infer::Config {
            field_delimiter: '\t',
            flexible: true,
            null_values: vec![String::from(".")],
            header_prefix: String::from("#"),
            num_rows: 100,
            skip_rows: 0,
            ..Default::default()
        };
        let mut reader = BufReader::new(std::fs::File::open("tests/tsv/schema/empty.tsv")?);
        let res = infer::Context::with_config(config).infer_schema(&mut reader);

        assert!(res.is_err());

        Ok(())
    }

    #[test]
    fn infer_schema_header() -> Result<(), anyhow::Error> {
        let config = infer::Config {
            field_delimiter: '\t',
            flexible: true,
            null_values: vec![String::from(".")],
            header_prefix: String::from("#"),
            num_rows: 100,
            skip_rows: 0,
            ..Default::default()
        };
        let mut reader = BufReader::new(std::fs::File::open("tests/tsv/schema/header.tsv")?);
        let record = infer::Context::with_config(config).infer_schema(&mut reader)?;

        insta::assert_debug_snapshot!(record);

        Ok(())
    }

    #[test]
    fn infer_schema_values() -> Result<(), anyhow::Error> {
        let config = infer::Config {
            field_delimiter: '\t',
            flexible: true,
            null_values: vec![String::from(".")],
            header_prefix: String::from("#"),
            num_rows: 100,
            skip_rows: 0,
            ..Default::default()
        };
        let mut reader = BufReader::new(std::fs::File::open("tests/tsv/schema/values.tsv")?);
        let record = infer::Context::with_config(config).infer_schema(&mut reader)?;

        insta::assert_debug_snapshot!(record);

        Ok(())
    }
}
