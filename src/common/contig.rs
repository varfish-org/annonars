//! Assembly-agnostic contig dictionary.
//!
//! Maps contig names to compact `u32` IDs so that variant keys can be encoded
//! independently of any particular reference assembly (see
//! [`crate::common::keys::Var::encode_with_id`]). The dictionary is derived
//! from a FASTA index (`.fai`), so it works for arbitrary assemblies, and is
//! persisted alongside the data (e.g. in a RocksDB `meta` column family) so
//! that keys can be decoded again.

use std::io::BufRead;
use std::path::Path;

use indexmap::IndexMap;

use crate::common::cli::canonicalize;

/// An ordered contig dictionary mapping contig names to compact `u32` IDs.
///
/// IDs are assigned by position (ID = index), which for a `.fai` means the
/// order contigs appear in the reference. Both the raw name and its
/// canonicalized form (see [`canonicalize`]) resolve to the same ID, so inputs
/// using `chr1`/`1`/`M`/`MT` variants all match.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContigDict {
    /// Contig names in ID order (ID = index).
    names: Vec<String>,
    /// Contig lengths in ID order.
    lengths: Vec<u64>,
    /// Lookup from any accepted alias to its ID.
    alias_to_id: IndexMap<String, u32>,
}

impl ContigDict {
    /// The maximum number of contigs supported by the compact 24-bit key format
    /// (see [`crate::common::keys::Var::encode_with_id`]).
    pub const MAX_CONTIGS: usize = 1 << 24;

    /// Build from an ordered list of `(name, length)` entries, assigning IDs in
    /// iteration order.
    ///
    /// Returns an error if two entries collide on a name/alias or if the number
    /// of contigs exceeds [`Self::MAX_CONTIGS`].
    pub fn from_entries(
        entries: impl IntoIterator<Item = (String, u64)>,
    ) -> Result<Self, anyhow::Error> {
        let mut dict = Self::default();
        for (name, length) in entries {
            // Fail fast rather than panicking deep inside `encode_with_id`.
            if dict.names.len() >= Self::MAX_CONTIGS {
                anyhow::bail!(
                    "too many contigs: the compact key format supports at most {}",
                    Self::MAX_CONTIGS
                );
            }
            let id = dict.names.len() as u32;
            // A raw name and its canonicalized form may coincide; both must map
            // to *this* contig. A collision with a *different* contig's id would
            // silently misroute keys, so reject it.
            for alias in [name.clone(), canonicalize(&name)] {
                match dict.alias_to_id.get(&alias) {
                    Some(&existing) if existing != id => anyhow::bail!(
                        "contig alias {:?} for {:?} collides with existing contig id {}",
                        alias,
                        name,
                        existing
                    ),
                    Some(_) => {}
                    None => {
                        dict.alias_to_id.insert(alias, id);
                    }
                }
            }
            dict.names.push(name);
            dict.lengths.push(length);
        }
        Ok(dict)
    }

    /// Build from a FASTA index file (`.fai`), assigning IDs in file order.
    ///
    /// Each `.fai` line is `name\tlength\t...`; only the first two columns are used.
    pub fn from_fai(path: impl AsRef<Path>) -> Result<Self, anyhow::Error> {
        let path = path.as_ref();
        let file = std::fs::File::open(path)
            .map_err(|e| anyhow::anyhow!("failed to open .fai {}: {}", path.display(), e))?;
        let mut entries = Vec::new();
        for line in std::io::BufReader::new(file).lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let mut cols = line.split('\t');
            let name = cols
                .next()
                .ok_or_else(|| anyhow::anyhow!("malformed .fai line: {:?}", line))?
                .to_string();
            let length = cols
                .next()
                .ok_or_else(|| anyhow::anyhow!("missing length in .fai line: {:?}", line))?
                .parse::<u64>()
                .map_err(|e| anyhow::anyhow!("bad length in .fai line {:?}: {}", line, e))?;
            entries.push((name, length));
        }
        if entries.is_empty() {
            anyhow::bail!("no contigs found in .fai {}", path.display());
        }
        Self::from_entries(entries)
    }

    /// Number of contigs.
    pub fn len(&self) -> usize {
        self.names.len()
    }

    /// Whether the dictionary is empty.
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    /// Resolve a contig name/alias to its ID, falling back to the canonicalized name.
    pub fn id_of(&self, name: &str) -> Option<u32> {
        self.alias_to_id
            .get(name)
            .copied()
            .or_else(|| self.alias_to_id.get(&canonicalize(name)).copied())
    }

    /// Name for a given ID.
    pub fn name_of(&self, id: u32) -> Option<&str> {
        self.names.get(id as usize).map(String::as_str)
    }

    /// Length for a given ID.
    pub fn length_of(&self, id: u32) -> Option<u64> {
        self.lengths.get(id as usize).copied()
    }

    /// The id→name table for [`crate::common::keys::Var::decode_with_ctx`].
    pub fn id_to_name(&self) -> &[String] {
        &self.names
    }

    /// Serialize for storage in a DB `meta` value: one `id\tname\tlength` line per contig.
    pub fn to_meta_string(&self) -> String {
        let mut out = String::new();
        for (id, (name, length)) in self.names.iter().zip(self.lengths.iter()).enumerate() {
            out.push_str(&format!("{}\t{}\t{}\n", id, name, length));
        }
        out
    }

    /// Parse a dictionary written by [`Self::to_meta_string`].
    pub fn from_meta_string(s: &str) -> Result<Self, anyhow::Error> {
        let mut entries: Vec<(u32, String, u64)> = Vec::new();
        for line in s.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let mut cols = line.split('\t');
            let id = cols
                .next()
                .ok_or_else(|| anyhow::anyhow!("malformed contig meta line: {:?}", line))?
                .parse::<u32>()?;
            let name = cols
                .next()
                .ok_or_else(|| anyhow::anyhow!("missing name in contig meta line: {:?}", line))?
                .to_string();
            let length = cols
                .next()
                .ok_or_else(|| anyhow::anyhow!("missing length in contig meta line: {:?}", line))?
                .parse::<u64>()?;
            entries.push((id, name, length));
        }
        // Restore ID order explicitly rather than trusting file order.
        entries.sort_by_key(|(id, _, _)| *id);
        for (expected, (id, _, _)) in entries.iter().enumerate() {
            if *id as usize != expected {
                anyhow::bail!("contig meta IDs are not contiguous from 0 (got {})", id);
            }
        }
        Self::from_entries(entries.into_iter().map(|(_, name, length)| (name, length)))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn dict() -> ContigDict {
        ContigDict::from_entries([
            ("1".to_string(), 249_250_621),
            ("2".to_string(), 243_199_373),
            ("X".to_string(), 155_270_560),
            ("MT".to_string(), 16_569),
        ])
        .unwrap()
    }

    #[test]
    fn colliding_alias_is_rejected() {
        // "M" canonicalizes to "MT", so listing both as distinct contigs collides.
        let err = ContigDict::from_entries([("MT".to_string(), 16_569), ("M".to_string(), 16_569)]);
        assert!(err.is_err());
    }

    #[test]
    fn ids_assigned_in_order() {
        let d = dict();
        assert_eq!(d.len(), 4);
        assert_eq!(d.id_of("1"), Some(0));
        assert_eq!(d.id_of("X"), Some(2));
        assert_eq!(d.name_of(3), Some("MT"));
        assert_eq!(d.length_of(0), Some(249_250_621));
    }

    #[test]
    fn alias_resolution_via_canonicalize() {
        let d = dict();
        // "chr1" and "1" resolve to the same ID even though the dict stored "1".
        assert_eq!(d.id_of("chr1"), Some(0));
        // "chrM"/"M" canonicalize to "MT".
        assert_eq!(d.id_of("chrM"), Some(3));
        assert_eq!(d.id_of("M"), Some(3));
        assert_eq!(d.id_of("nonexistent"), None);
    }

    #[test]
    fn meta_string_roundtrip() {
        let d = dict();
        let parsed = ContigDict::from_meta_string(&d.to_meta_string()).unwrap();
        assert_eq!(parsed, d);
    }
}
