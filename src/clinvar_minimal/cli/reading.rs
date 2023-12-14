//! Reading `clinvar-data-jsonl` variant files.
//!
//! This code is shared for all reading of ClinVar JSONL data.

use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

/// Enumeration for ClinVar variant types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VariantType {
    /// Deletion
    Deletion,
    /// Duplication
    Duplication,
    /// Indel
    Indel,
    /// Insertion
    Insertion,
    /// Inversion
    Inversion,
    /// Snv
    Snv,
}

impl Display for VariantType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariantType::Deletion => write!(f, "deletion"),
            VariantType::Duplication => write!(f, "duplication"),
            VariantType::Indel => write!(f, "indel"),
            VariantType::Insertion => write!(f, "insertion"),
            VariantType::Inversion => write!(f, "inversion"),
            VariantType::Snv => write!(f, "single nucleotide variant"),
        }
    }
}

impl FromStr for VariantType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "copy number loss" | "deletion" => VariantType::Deletion,
            "copy number gain" | "duplication" | "tandem duplication" => VariantType::Duplication,
            "indel" => VariantType::Indel,
            "insertion" => VariantType::Insertion,
            "inversion" => VariantType::Inversion,
            "single nucleotide variant" => VariantType::Snv,
            _ => anyhow::bail!("Unknown variant type: {}", s),
        })
    }
}

impl Serialize for VariantType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for VariantType {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(d)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl From<VariantType> for crate::pbs::clinvar::minimal::VariantType {
    fn from(value: VariantType) -> Self {
        match value {
            VariantType::Deletion => crate::pbs::clinvar::minimal::VariantType::Deletion,
            VariantType::Duplication => crate::pbs::clinvar::minimal::VariantType::Duplication,
            VariantType::Indel => crate::pbs::clinvar::minimal::VariantType::Indel,
            VariantType::Insertion => crate::pbs::clinvar::minimal::VariantType::Insertion,
            VariantType::Inversion => crate::pbs::clinvar::minimal::VariantType::Inversion,
            VariantType::Snv => crate::pbs::clinvar::minimal::VariantType::Snv,
        }
    }
}

impl From<i32> for VariantType {
    fn from(value: i32) -> Self {
        match value {
            1 => VariantType::Deletion,
            2 => VariantType::Duplication,
            3 => VariantType::Indel,
            4 => VariantType::Insertion,
            5 => VariantType::Inversion,
            6 => VariantType::Snv,
            _ => unreachable!(),
        }
    }
}

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
        Ok(match s {
            "pathogenic" => ClinicalSignificance::Pathogenic,
            "likely pathogenic" => ClinicalSignificance::LikelyPathogenic,
            "uncertain significance" => ClinicalSignificance::UncertainSignificance,
            "likely benign" => ClinicalSignificance::LikelyBenign,
            "benign" => ClinicalSignificance::Benign,
            _ => anyhow::bail!("Unknown pathogenicity: {}", s),
        })
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

impl From<ClinicalSignificance> for crate::pbs::clinvar::minimal::ClinicalSignificance {
    fn from(value: ClinicalSignificance) -> Self {
        match value {
            ClinicalSignificance::Pathogenic => {
                crate::pbs::clinvar::minimal::ClinicalSignificance::Pathogenic
            }
            ClinicalSignificance::LikelyPathogenic => {
                crate::pbs::clinvar::minimal::ClinicalSignificance::LikelyPathogenic
            }
            ClinicalSignificance::UncertainSignificance => {
                crate::pbs::clinvar::minimal::ClinicalSignificance::UncertainSignificance
            }
            ClinicalSignificance::LikelyBenign => {
                crate::pbs::clinvar::minimal::ClinicalSignificance::LikelyBenign
            }
            ClinicalSignificance::Benign => {
                crate::pbs::clinvar::minimal::ClinicalSignificance::Benign
            }
        }
    }
}

impl From<i32> for ClinicalSignificance {
    fn from(value: i32) -> Self {
        match value {
            1 => ClinicalSignificance::Pathogenic,
            2 => ClinicalSignificance::LikelyPathogenic,
            3 => ClinicalSignificance::UncertainSignificance,
            4 => ClinicalSignificance::LikelyBenign,
            5 => ClinicalSignificance::Benign,
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
    /// "flagged submission",
    FlaggedSubmission,
    /// "no classifications from unflagged records",
    NoClassificationsFromUnflaggedRecords,
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
            ReviewStatus::FlaggedSubmission => write!(f, "flagged submission"),
            ReviewStatus::NoClassificationsFromUnflaggedRecords => {
                write!(f, "no classifications from unflagged records")
            }
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
            "flagged submission" => Ok(ReviewStatus::FlaggedSubmission),
            "no classifications from unflagged records" => {
                Ok(ReviewStatus::NoClassificationsFromUnflaggedRecords)
            }
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

impl From<ReviewStatus> for crate::pbs::clinvar::minimal::ReviewStatus {
    fn from(value: ReviewStatus) -> Self {
        match value {
            ReviewStatus::NoAssertionProvided => crate::pbs::clinvar::minimal::ReviewStatus::NoAssertionProvided,
            ReviewStatus::NoAssertionCriteriaProvided => {
                crate::pbs::clinvar::minimal::ReviewStatus::NoAssertionCriteriaProvided
            }
            ReviewStatus::CriteriaProvidedConflictingInterpretations => {
                crate::pbs::clinvar::minimal::ReviewStatus::CriteriaProvidedConflictingInterpretations
            }
            ReviewStatus::CriteriaProvidedSingleSubmitter => {
                crate::pbs::clinvar::minimal::ReviewStatus::CriteriaProvidedSingleSubmitter
            }
            ReviewStatus::CriteriaProvidedMultipleSubmittersNoConflicts => {
                crate::pbs::clinvar::minimal::ReviewStatus::CriteriaProvidedMultipleSubmittersNoConflicts
            }
            ReviewStatus::ReviewedByExpertPanel => crate::pbs::clinvar::minimal::ReviewStatus::ReviewedByExpertPanel,
            ReviewStatus::PracticeGuideline => crate::pbs::clinvar::minimal::ReviewStatus::PracticeGuideline,
            ReviewStatus::FlaggedSubmission => crate::pbs::clinvar::minimal::ReviewStatus::FlaggedSubmission,
            ReviewStatus::NoClassificationsFromUnflaggedRecords => crate::pbs::clinvar::minimal::ReviewStatus::NoClassificationsFromUnflaggedRecords,
        }
    }
}

impl From<i32> for ReviewStatus {
    fn from(value: i32) -> Self {
        match value {
            1 => ReviewStatus::PracticeGuideline,
            2 => ReviewStatus::ReviewedByExpertPanel,
            3 => ReviewStatus::CriteriaProvidedMultipleSubmittersNoConflicts,
            4 => ReviewStatus::CriteriaProvidedSingleSubmitter,
            5 => ReviewStatus::CriteriaProvidedConflictingInterpretations,
            6 => ReviewStatus::NoAssertionCriteriaProvided,
            7 => ReviewStatus::NoAssertionProvided,
            8 => ReviewStatus::FlaggedSubmission,
            9 => ReviewStatus::NoClassificationsFromUnflaggedRecords,
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
    /// The variant type.
    pub variant_type: VariantType,
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
    pub start: Option<u32>,
    /// 1-based stop position.
    pub stop: Option<u32>,
    /// Reference allele bases in VCF notation.
    pub reference_allele_vcf: Option<String>,
    /// Alternative allele bases in VCF notation.
    pub alternate_allele_vcf: Option<String>,

    /// 1-based inner start position.
    pub inner_start: Option<u32>,
    /// 1-based inner stop position.
    pub inner_stop: Option<u32>,
    /// 1-based outer start position.
    pub outer_start: Option<u32>,
    /// 1-based outer stop position.
    pub outer_stop: Option<u32>,
}
