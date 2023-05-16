//! Data structures for UCSC 100 vertebrate conservation data.

/// A record in the UCSC conservation table.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Record {
    /// Chromosome name.
    #[serde(rename = "chromosome")]
    pub chrom: String,
    /// Start position (1-based).
    pub start: i32,
    /// End position (1-based, exclusive).
    #[serde(rename = "end")]
    pub stop: i32,
    /// HGNC identifier.
    pub hgnc_id: String,
    /// ENST identifier.
    pub enst_id: String,
    /// Alignment.
    pub alignment: String,
}

impl From<super::pbs::Record> for Record {
    fn from(pbs: super::pbs::Record) -> Self {
        Self {
            chrom: pbs.chrom,
            start: pbs.start,
            stop: pbs.stop,
            hgnc_id: pbs.hgnc_id,
            enst_id: pbs.enst_id,
            alignment: pbs.alignment,
        }
    }
}

impl From<Record> for super::pbs::Record {
    fn from(val: Record) -> Self {
        super::pbs::Record {
            chrom: val.chrom,
            start: val.start,
            stop: val.stop,
            hgnc_id: val.hgnc_id,
            enst_id: val.enst_id,
            alignment: val.alignment,
        }
    }
}
