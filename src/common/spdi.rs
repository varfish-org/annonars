//! Variants and coordinates in SPDI format.
//!
//! Also see:
//!
//! - Holmes JB, Moyer E, Phan L, Maglott D, Kattman B. [SPDI: data model for variants
//!   and applications at NCBI](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC7523648/).
//!   Bioinformatics. 2020 Mar 1;36(6):1902-1907.

use std::{fmt::Display, str::FromStr};

/// A variant in in SPDI format.
///
/// The SPDI format is described in [Holmes et al.
/// 2020](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC7523648/).
///
/// Note that the format uses 1-based positions and VCF-style allele strings.
///
/// # Example
///
/// ```
/// use std::str::FromStr;
/// use annonars::common::spdi::Var;
///
/// let var = Var::from_str("NC_000001.11:1000:G:A").unwrap();
/// assert_eq!(format!("{}", &var), "NC_000001.11:1000:G:A");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Var {
    /// Sequence identifier.
    pub sequence: String,
    /// Position information.
    pub position: i32,
    /// Deletion base string.
    pub deletion: String,
    /// Insertion base string.
    pub insertion: String,
}

impl Var {
    /// Create a new variant.
    pub fn new(sequence: String, position: i32, deletion: String, insertion: String) -> Self {
        Self {
            sequence,
            position,
            deletion,
            insertion,
        }
    }
}

impl FromStr for Var {
    type Err = anyhow::Error;

    fn from_str(spdi: &str) -> Result<Self, Self::Err> {
        let mut parts = spdi.rsplitn(4, ':');
        let insertion = parts.next().unwrap().to_string();
        let deletion = parts.next().unwrap().to_string();
        let position = parts
            .next()
            .unwrap()
            .parse::<i32>()
            .map_err(|e| anyhow::anyhow!("Could not parse position: {}", e))?;
        let sequence = parts.next().unwrap().to_string();
        Ok(Self {
            sequence,
            position,
            deletion,
            insertion,
        })
    }
}

impl Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}:{}",
            self.sequence, self.position, self.deletion, self.insertion
        )
    }
}

/// A SPDI-style position.
///
/// # Example
///
/// ```
/// use std::str::FromStr;
/// use annonars::common::spdi::Pos;
///
/// let pos = Pos::from_str("NC_000001.11:1000").unwrap();
/// assert_eq!(format!("{}", &pos), "NC_000001.11:1000");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pos {
    /// Sequence identifier.
    pub sequence: String,
    /// Position information.
    pub position: i32,
}

impl Pos {
    /// Create a new position.
    pub fn new(sequence: String, position: i32) -> Self {
        Self { sequence, position }
    }
}

impl FromStr for Pos {
    type Err = anyhow::Error;

    fn from_str(spdi: &str) -> Result<Self, Self::Err> {
        let mut parts = spdi.rsplitn(2, ':');
        let position = parts
            .next()
            .unwrap()
            .parse::<i32>()
            .map_err(|e| anyhow::anyhow!("Could not parse position: {}", e))?;
        let sequence = parts.next().unwrap().to_string();
        Ok(Self { sequence, position })
    }
}

impl Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.sequence, self.position)
    }
}

/// A SPDI-style range.
///
/// The range is inclusive of the 1-based start and end positions.
///
/// # Example
///
/// ```
/// use std::str::FromStr;
/// use annonars::common::spdi::Range;
///
/// let range = Range::from_str("NC_000001.11:1000:2000").unwrap();
/// assert_eq!(format!("{}", &range), "NC_000001.11:1000:2000");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Range {
    /// Sequence identifier.
    pub sequence: String,
    /// Start position.
    pub start: i32,
    /// End position.
    pub end: i32,
}

impl Range {
    /// Create a new range.
    pub fn new(sequence: String, start: i32, end: i32) -> Self {
        Self {
            sequence,
            start,
            end,
        }
    }
}

impl FromStr for Range {
    type Err = anyhow::Error;

    fn from_str(spdi: &str) -> Result<Self, Self::Err> {
        let mut parts = spdi.rsplitn(3, ':');
        let end = parts
            .next()
            .unwrap()
            .parse::<i32>()
            .map_err(|e| anyhow::anyhow!("Could not parse end position: {}", e))?;
        let start = parts
            .next()
            .unwrap()
            .parse::<i32>()
            .map_err(|e| anyhow::anyhow!("Could not parse start position: {}", e))?;
        let sequence = parts.next().unwrap().to_string();
        Ok(Self {
            sequence,
            start,
            end,
        })
    }
}

impl Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.sequence, self.start, self.end)
    }
}

impl From<Range> for (Pos, Pos) {
    fn from(val: Range) -> Self {
        (
            Pos::new(val.sequence.clone(), val.start),
            Pos::new(val.sequence, val.end),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn var_new() {
        let var = Var::new(
            String::from("NC_000001.11"),
            123,
            String::from("A"),
            String::from("T"),
        );
        assert_eq!(var.sequence, "NC_000001.11");
        assert_eq!(var.position, 123);
        assert_eq!(var.deletion, "A");
        assert_eq!(var.insertion, "T");
    }

    #[test]
    fn var_from_str() {
        let var = Var::from_str("NC_000001.11:123:A:T").unwrap();
        assert_eq!(var.sequence, "NC_000001.11");
        assert_eq!(var.position, 123);
        assert_eq!(var.deletion, "A");
        assert_eq!(var.insertion, "T");
    }

    #[test]
    fn var_display() {
        let var = Var::new(
            String::from("NC_000001.11"),
            123,
            String::from("A"),
            String::from("T"),
        );
        assert_eq!(var.to_string(), "NC_000001.11:123:A:T");
    }

    #[test]
    fn pos_new() {
        let pos = Pos::new(String::from("NC_000001.11"), 123);
        assert_eq!(pos.sequence, "NC_000001.11");
        assert_eq!(pos.position, 123);
    }

    #[test]
    fn pos_from_str() {
        let pos = Pos::from_str("NC_000001.11:123").unwrap();
        assert_eq!(pos.sequence, "NC_000001.11");
        assert_eq!(pos.position, 123);
    }

    #[test]
    fn pos_display() {
        let pos = Pos::new(String::from("NC_000001.11"), 123);
        assert_eq!(pos.to_string(), "NC_000001.11:123");
    }

    #[test]
    fn range_new() {
        let range = Range::new(String::from("NC_000001.11"), 123, 456);
        assert_eq!(range.sequence, "NC_000001.11");
        assert_eq!(range.start, 123);
        assert_eq!(range.end, 456);
    }

    #[test]
    fn range_from_str() {
        let range = Range::from_str("NC_000001.11:123:456").unwrap();
        assert_eq!(range.sequence, "NC_000001.11");
        assert_eq!(range.start, 123);
        assert_eq!(range.end, 456);
    }

    #[test]
    fn range_display() {
        let range = Range::new(String::from("NC_000001.11"), 123, 456);
        assert_eq!(range.to_string(), "NC_000001.11:123:456");
    }
}
