//! Reading `clinvar-tsv` sequence variant files.

use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::clinvar_minimal::pbs;

/// Enumeration for ClinVar pathogenicity for (de)serialization.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Pathogenicity {
    /// Pathogenic.
    Pathogenic,
    /// Likely pathogenic.
    LikelyPathogenic,
    /// Uncertain significance.
    UncertainSignificance,
    /// Likely benign.
    LikelyBenign,
    /// Benign.
    Benign,
}

impl Display for Pathogenicity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pathogenicity::Pathogenic => write!(f, "pathogenic"),
            Pathogenicity::LikelyPathogenic => write!(f, "likely pathogenic"),
            Pathogenicity::UncertainSignificance => write!(f, "uncertain significance"),
            Pathogenicity::LikelyBenign => write!(f, "likely benign"),
            Pathogenicity::Benign => write!(f, "benign"),
        }
    }
}

impl FromStr for Pathogenicity {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pathogenic" => Ok(Pathogenicity::Pathogenic),
            "likely pathogenic" => Ok(Pathogenicity::LikelyPathogenic),
            "uncertain significance" => Ok(Pathogenicity::UncertainSignificance),
            "likely benign" => Ok(Pathogenicity::LikelyBenign),
            "benign" => Ok(Pathogenicity::Benign),
            _ => anyhow::bail!("Unknown pathogenicity: {}", s),
        }
    }
}

impl Serialize for Pathogenicity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Pathogenicity {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(d)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl From<Pathogenicity> for pbs::Pathogenicity {
    fn from(value: Pathogenicity) -> Self {
        match value {
            Pathogenicity::Pathogenic => pbs::Pathogenicity::Pathogenic,
            Pathogenicity::LikelyPathogenic => pbs::Pathogenicity::LikelyPathogenic,
            Pathogenicity::UncertainSignificance => pbs::Pathogenicity::UncertainSignificance,
            Pathogenicity::LikelyBenign => pbs::Pathogenicity::LikelyBenign,
            Pathogenicity::Benign => pbs::Pathogenicity::Benign,
        }
    }
}

/// Representation of a record from the `clinvar-tsv` output.
///
/// Note that the pathogenicity and review status are available in two fashions.  The first is
/// "ClinVar style" and attempts to follow the ClinVar approach.  Here, variant assessments
/// with a higher star rating override those a lowe rone.  This is what most users want.
/// The assessment "paranoid" uses all assessments, including those without a star rating,
/// on the same level.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Record {
    /// Genome release.
    pub release: String,
    /// Chromosome name.
    pub chromosome: String,
    /// 1-based start position.
    pub start: u32,
    /// 1-based end position.
    pub end: u32,
    /// Reference allele bases in VCF notation.
    pub reference: String,
    /// Alternative allele bases in VCF notation.
    pub alternative: String,
    /// VCV accession identifier.
    pub vcv: String,
    /// Pathogenicity summary for the variant (ClinVar style).
    #[serde(deserialize_with = "deserialize_pathogenicity")]
    pub summary_clinvar_pathogenicity: Vec<Pathogenicity>,
    /// Pathogenicity gold stars (ClinVar style).
    pub summary_clinvar_gold_stars: u32,
    /// Pathogenicity summary for the variant ("paranoid" style).
    #[serde(deserialize_with = "deserialize_pathogenicity")]
    pub summary_paranoid_pathogenicity: Vec<Pathogenicity>,
    /// Pathogenicity gold stars ("paranoid" style).
    pub summary_paranoid_gold_stars: u32,
}

fn deserialize_pathogenicity<'de, D>(deserializer: D) -> Result<Vec<Pathogenicity>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    let s = s.replace('{', "[").replace('}', "]");
    serde_json::from_str(&s).map_err(serde::de::Error::custom)
}
