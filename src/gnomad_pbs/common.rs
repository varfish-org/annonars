//! Common code for gnomAD mtDNA and nuclear.

/// The cohorts that are available in the gnomAD-exomes/genomes VCFs.
pub static COHORTS: &[&str] = &["controls", "non_cancer", "non_neuro", "non_topmed"];

/// The populations that are available in the gnomAD-exomes/genomes VCFs.
///
/// This includes the "global" population represented by an empty string.
pub static POPS: &[&str] = &["afr", "amr", "asj", "eas", "fin", "nfe", "oth", "sas"];

/// Define the token used for encoding the XX/XY/male/female karyotypes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SexCoding {
    /// As "female" or "male".
    FemaleMale,
    /// As "XX" or "XY".
    XxXy,
}

impl SexCoding {
    /// Return `INFO` field suffixes for the given `SexCoding`.
    pub fn to_suffixes(self) -> (&'static str, &'static str) {
        match self {
            SexCoding::FemaleMale => ("_female", "_male"),
            SexCoding::XxXy => ("_XX", "_XY"),
        }
    }
}
