//! Reading `clinvar-data-jsonl` sequence variant files.

use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

/// Enumeration for ClinVar clinical significance for (de)serialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ClinicalSignificance {
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

impl Display for ClinicalSignificance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClinicalSignificance::Pathogenic => write!(f, "pathogenic"),
            ClinicalSignificance::LikelyPathogenic => write!(f, "likely pathogenic"),
            ClinicalSignificance::UncertainSignificance => write!(f, "uncertain significance"),
            ClinicalSignificance::LikelyBenign => write!(f, "likely benign"),
            ClinicalSignificance::Benign => write!(f, "benign"),
        }
    }
}

impl FromStr for ClinicalSignificance {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pathogenic" => Ok(ClinicalSignificance::Pathogenic),
            "likely pathogenic" => Ok(ClinicalSignificance::LikelyPathogenic),
            "uncertain significance" => Ok(ClinicalSignificance::UncertainSignificance),
            "likely benign" => Ok(ClinicalSignificance::LikelyBenign),
            "benign" => Ok(ClinicalSignificance::Benign),
            _ => anyhow::bail!("Unknown pathogenicity: {}", s),
        }
    }
}

impl Serialize for ClinicalSignificance {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ClinicalSignificance {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(d)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl From<ClinicalSignificance> for crate::clinvar_minimal::pbs::ClinicalSignificance {
    fn from(value: ClinicalSignificance) -> Self {
        match value {
            ClinicalSignificance::Pathogenic => {
                crate::clinvar_minimal::pbs::ClinicalSignificance::Pathogenic
            }
            ClinicalSignificance::LikelyPathogenic => {
                crate::clinvar_minimal::pbs::ClinicalSignificance::LikelyPathogenic
            }
            ClinicalSignificance::UncertainSignificance => {
                crate::clinvar_minimal::pbs::ClinicalSignificance::UncertainSignificance
            }
            ClinicalSignificance::LikelyBenign => {
                crate::clinvar_minimal::pbs::ClinicalSignificance::LikelyBenign
            }
            ClinicalSignificance::Benign => {
                crate::clinvar_minimal::pbs::ClinicalSignificance::Benign
            }
        }
    }
}

impl From<ClinicalSignificance> for crate::clinvar_genes::pbs::ClinicalSignificance {
    fn from(value: ClinicalSignificance) -> Self {
        match value {
            ClinicalSignificance::Pathogenic => {
                crate::clinvar_genes::pbs::ClinicalSignificance::Pathogenic
            }
            ClinicalSignificance::LikelyPathogenic => {
                crate::clinvar_genes::pbs::ClinicalSignificance::LikelyPathogenic
            }
            ClinicalSignificance::UncertainSignificance => {
                crate::clinvar_genes::pbs::ClinicalSignificance::UncertainSignificance
            }
            ClinicalSignificance::LikelyBenign => {
                crate::clinvar_genes::pbs::ClinicalSignificance::LikelyBenign
            }
            ClinicalSignificance::Benign => crate::clinvar_genes::pbs::ClinicalSignificance::Benign,
        }
    }
}

impl From<i32> for ClinicalSignificance {
    fn from(value: i32) -> Self {
        match value {
            0 => ClinicalSignificance::Pathogenic,
            1 => ClinicalSignificance::LikelyPathogenic,
            2 => ClinicalSignificance::UncertainSignificance,
            3 => ClinicalSignificance::LikelyBenign,
            4 => ClinicalSignificance::Benign,
            _ => unreachable!(),
        }
    }
}

/// Enumeration for ClinVar review status for (de)serialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReviewStatus {
    /// "practice guideline"
    PracticeGuideline,
    /// "reviewed by expert panel"
    ReviewedByExpertPanel,
    /// "criteria provided, multiple submitters, no conflicts"
    CriteriaProvidedMultipleSubmittersNoConflicts,
    /// "criteria provided, single submitter"
    CriteriaProvidedSingleSubmitter,
    /// "criteria provided, conflicting interpretations"
    CriteriaProvidedConflictingInterpretations,
    /// "no assertion criteria provided"
    NoAssertionCriteriaProvided,
    /// "no assertion provided"
    NoAssertionProvided,
}

impl Display for ReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReviewStatus::CriteriaProvidedConflictingInterpretations => {
                write!(f, "criteria provided, conflicting interpretations")
            }
            ReviewStatus::CriteriaProvidedMultipleSubmittersNoConflicts => {
                write!(f, "criteria provided, multiple submitters, no conflicts")
            }
            ReviewStatus::CriteriaProvidedSingleSubmitter => {
                write!(f, "criteria provided, single submitter")
            }
            ReviewStatus::NoAssertionCriteriaProvided => {
                write!(f, "no assertion criteria provided")
            }
            ReviewStatus::NoAssertionProvided => write!(f, "no assertion provided"),
            ReviewStatus::PracticeGuideline => write!(f, "practice guideline"),
            ReviewStatus::ReviewedByExpertPanel => write!(f, "reviewed by expert panel"),
        }
    }
}

impl FromStr for ReviewStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "criteria provided, conflicting interpretations" => {
                Ok(ReviewStatus::CriteriaProvidedConflictingInterpretations)
            }
            "criteria provided, multiple submitters, no conflicts" => {
                Ok(ReviewStatus::CriteriaProvidedMultipleSubmittersNoConflicts)
            }
            "criteria provided, single submitter" => {
                Ok(ReviewStatus::CriteriaProvidedSingleSubmitter)
            }
            "no assertion criteria provided" => Ok(ReviewStatus::NoAssertionCriteriaProvided),
            "no assertion provided" => Ok(ReviewStatus::NoAssertionProvided),
            "practice guideline" => Ok(ReviewStatus::PracticeGuideline),
            "reviewed by expert panel" => Ok(ReviewStatus::ReviewedByExpertPanel),
            _ => anyhow::bail!("Unknown review status: {}", s),
        }
    }
}

impl Serialize for ReviewStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ReviewStatus {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(d)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl From<ReviewStatus> for crate::clinvar_minimal::pbs::ReviewStatus {
    fn from(value: ReviewStatus) -> Self {
        match value {
            ReviewStatus::NoAssertionProvided => crate::clinvar_minimal::pbs::ReviewStatus::NoAssertionProvided,
            ReviewStatus::NoAssertionCriteriaProvided => {
                crate::clinvar_minimal::pbs::ReviewStatus::NoAssertionCriteriaProvided
            }
            ReviewStatus::CriteriaProvidedConflictingInterpretations => {
                crate::clinvar_minimal::pbs::ReviewStatus::CriteriaProvidedConflictingInterpretations
            }
            ReviewStatus::CriteriaProvidedSingleSubmitter => {
                crate::clinvar_minimal::pbs::ReviewStatus::CriteriaProvidedSingleSubmitter
            }
            ReviewStatus::CriteriaProvidedMultipleSubmittersNoConflicts => {
                crate::clinvar_minimal::pbs::ReviewStatus::CriteriaProvidedMultipleSubmittersNoConflicts
            }
            ReviewStatus::ReviewedByExpertPanel => crate::clinvar_minimal::pbs::ReviewStatus::ReviewedByExpertPanel,
            ReviewStatus::PracticeGuideline => crate::clinvar_minimal::pbs::ReviewStatus::PracticeGuideline,
        }
    }
}

impl From<ReviewStatus> for crate::clinvar_genes::pbs::ReviewStatus {
    fn from(value: ReviewStatus) -> Self {
        match value {
            ReviewStatus::NoAssertionProvided => crate::clinvar_genes::pbs::ReviewStatus::NoAssertionProvided,
            ReviewStatus::NoAssertionCriteriaProvided => {
                crate::clinvar_genes::pbs::ReviewStatus::NoAssertionCriteriaProvided
            }
            ReviewStatus::CriteriaProvidedConflictingInterpretations => {
                crate::clinvar_genes::pbs::ReviewStatus::CriteriaProvidedConflictingInterpretations
            }
            ReviewStatus::CriteriaProvidedSingleSubmitter => {
                crate::clinvar_genes::pbs::ReviewStatus::CriteriaProvidedSingleSubmitter
            }
            ReviewStatus::CriteriaProvidedMultipleSubmittersNoConflicts => {
                crate::clinvar_genes::pbs::ReviewStatus::CriteriaProvidedMultipleSubmittersNoConflicts
            }
            ReviewStatus::ReviewedByExpertPanel => crate::clinvar_genes::pbs::ReviewStatus::ReviewedByExpertPanel,
            ReviewStatus::PracticeGuideline => crate::clinvar_genes::pbs::ReviewStatus::PracticeGuideline,
        }
    }
}
impl From<i32> for ReviewStatus {
    fn from(value: i32) -> Self {
        match value {
            0 => ReviewStatus::PracticeGuideline,
            1 => ReviewStatus::ReviewedByExpertPanel,
            2 => ReviewStatus::CriteriaProvidedMultipleSubmittersNoConflicts,
            3 => ReviewStatus::CriteriaProvidedSingleSubmitter,
            4 => ReviewStatus::CriteriaProvidedConflictingInterpretations,
            5 => ReviewStatus::NoAssertionCriteriaProvided,
            6 => ReviewStatus::NoAssertionProvided,
            _ => unreachable!(),
        }
    }
}

/// Representation of a record from the `clinvar-data-jsonl` output.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Record {
    /// RCV accession identifier.
    pub rcv: String,
    /// VCV accession identifier.
    pub vcv: String,
    /// RCV title.
    pub title: String,
    /// HGNC ids
    pub hgnc_ids: Vec<String>,
    /// ClinVar clinical significance
    pub clinical_significance: ClinicalSignificance,
    /// ClinVar review status
    pub review_status: ReviewStatus,
    /// Sequence location
    pub sequence_location: SequenceLocation,
}

/// Representation of a sequence location record.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SequenceLocation {
    /// Genome release.
    pub assembly: String,
    /// Chromosome name.
    pub chr: String,
    /// 1-based start position.
    pub start: u32,
    /// 1-based stop position.
    pub stop: u32,
    /// Reference allele bases in VCF notation.
    pub reference_allele_vcf: Option<String>,
    /// Alternative allele bases in VCF notation.
    pub alternate_allele_vcf: Option<String>,
}
