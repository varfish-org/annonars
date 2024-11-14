//! Data structures for serde/Utoipa corresponding to the ones from clinvar protobufs.

use crate::pbs;

/// A structure to support reporting unformatted content, with type and
/// source specified.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Comment {
    /// The comment's value.
    pub value: String,
    /// The optional comment data source.
    pub data_source: Option<String>,
    /// The comment's type.
    pub r#type: Option<CommentType>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Comment> for Comment {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Comment) -> Result<Self, Self::Error> {
        Ok(Self {
            value: value.value,
            data_source: value.data_source,
            r#type: value
                .r#type
                .map(|t| {
                    CommentType::try_from(pbs::clinvar_data::clinvar_public::CommentType::try_from(
                        t,
                    )?)
                })
                .transpose()?,
        })
    }
}

/// This structure is used to represent how an object described in the
/// submission relates to objects in other databases.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Xref {
    /// The name of the database. When there is an overlap with sequence
    /// databases, that name is used.
    pub db: String,
    /// The identifier used by the database. Being exported as a string
    /// even though internally the database has rules for defining which datases use
    /// integer identifers.
    pub id: String,
    /// Used to differentiate between different types of identifiers that
    /// a database may provide.
    pub r#type: Option<String>,
    /// Optional URL to the database entry.
    pub url: Option<String>,
    /// The status; defaults to "current".
    pub status: Option<Status>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Xref> for Xref {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Xref) -> Result<Self, Self::Error> {
        Ok(Xref {
            db: value.db,
            id: value.id,
            r#type: value.r#type,
            url: value.url,
            status: value.status.map(|x| x.try_into()).transpose()?,
        })
    }
}

/// Description of a citation.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Citation {
    /// Optional list of IDs.
    pub ids: Vec<IdType>,
    /// Optional URL.
    pub url: Option<String>,
    /// Optional citation text.
    pub citation_text: Option<String>,
    /// This maintained distinct from publication types in PubMed and
    /// established by GTR curators.  The default is 'general'.
    pub r#type: Option<String>,
    /// Corresponds to the abbreviation reported by GTR.
    pub abbrev: Option<String>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Citation> for Citation {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Citation) -> Result<Self, Self::Error> {
        Ok(Citation {
            ids: value.ids.into_iter().map(IdType::from).collect()?,
            url: value.url,
            citation_text: value.citation_text,
            r#type: value.r#type,
            abbrev: value.abbrev,
        })
    }
}

/// Local ID with source.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct IdType {
    /// The citation's value.
    pub value: String,
    /// If there is an identifier, what database provides it.
    pub source: String,
}

impl From<pbs::clinvar_data::clinvar_public::citation::IdType> for IdType {
    fn from(value: pbs::clinvar_data::clinvar_public::citation::IdType) -> Self {
        IdType {
            value: value.value,
            source: value.source,
        }
    }
}

/// The attribute is a general element to represent a defined set of data
/// qualified by an enumerated set of types. For each attribute element, the value will
/// be a character string and is optional. Source shall be used to store identifiers for
/// supplied data from source other than the submitter (e.g. SequenceOntology). The data
/// submitted where Type="variation" shall be validated against sequence_alternation in
/// Sequence Ontology <http://www.sequenceontology.org/.> This is to be a generic version
/// of AttributeType and should be used with extension when it is used to specify Type
/// and its enumerations.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct BaseAttribute {
    /// The attribute's value; can be empty.
    pub value: Option<String>,
    /// The optional integer value.
    pub integer_value: Option<i64>,
    /// The optional date value.
    pub date_value: Option<chrono::DateTime<chrono::Utc>>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::BaseAttribute> for BaseAttribute {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::BaseAttribute,
    ) -> Result<Self, Self::Error> {
        Ok(BaseAttribute {
            value: value.value,
            integer_value: value.integer_value,
            date_value: value.date_value.map(|x| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(x.seconds, x.nanos as u32)
                    .unwrap_or_default()
            }),
        })
    }
}

/// Description of a nucleotide sequence expression.
///
/// Corresponds to `typeNucleotideSequenceExpression`
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct HgvsNucleotideExpression {
    /// The expression values.
    pub expression: String,
    /// The type of the nucleotide sequence.
    pub sequence_type: Option<NucleotideSequence>,
    /// Optional sequence accession version.
    pub sequence_accession_version: Option<String>,
    /// Optional sequence accession.
    pub sequence_accession: Option<String>,
    /// Optional sequence version.
    pub sequence_version: Option<i32>,
    /// Optional description of the change.
    pub change: Option<String>,
    /// Optional assembly information.
    pub assembly: Option<String>,
    /// Optional submission information.
    pub submitted: Option<String>,
    /// Optional MANE Select flag.
    pub mane_select: Option<bool>,
    /// Optional MANE Plus Clinical flag.
    pub mane_plus_clinical: Option<bool>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::HgvsNucleotideExpression>
    for HgvsNucleotideExpression
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::HgvsNucleotideExpression,
    ) -> Result<Self, Self::Error> {
        Ok(HgvsNucleotideExpression {
            expression: value.expression,
            sequence_type: value.sequence_type.map(|x| x.try_into()).transpose()?,
            sequence_accession_version: value.sequence_accession_version,
            sequence_accession: value.sequence_accession,
            sequence_version: value.sequence_version,
            change: value.change,
            assembly: value.assembly,
            submitted: value.submitted,
            mane_select: value.mane_select,
            mane_plus_clinical: value.mane_plus_clinical,
        })
    }
}

/// Description of a protein sequence expression.
///
/// Corresponds to `typeProteinSequenceExpression` in XSD.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct HgvsProteinExpression {
    /// The expression values.
    pub expression: String,
    /// Optional sequence accession version.
    pub sequence_accession_version: Option<String>,
    /// Optional sequence accession.
    pub sequence_accession: Option<String>,
    /// Optional sequence version.
    pub sequence_version: Option<i32>,
    /// Optional description of the change.
    pub change: Option<String>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::HgvsProteinExpression> for HgvsProteinExpression {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::HgvsProteinExpression,
    ) -> Result<Self, Self::Error> {
        Ok(HgvsProteinExpression {
            expression: value.expression,
            sequence_accession_version: value.sequence_accession_version,
            sequence_accession: value.sequence_accession,
            sequence_version: value.sequence_version,
            change: value.change,
        })
    }
}

/// A structure to represent an HGVS expression for a nucleotide sequence
/// variant, along with the predicted protein change and the predicted molecular
/// consequence. Also used to represent only the protein change if that is all that has
/// been reported.
///
/// Corresponds to `typeHVSExpression` in XSD.
///
/// nested elements
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct HgvsExpression {
    /// Optional nucleotide sequence expression.
    pub nucleotide_expression: Option<HgvsNucleotideExpression>,
    /// Optional protein sequence expression.
    pub protein_expression: Option<HgvsProteinExpression>,
    /// List of molecular consequences.
    pub molecular_consequences: Vec<Xref>,
    /// Type of HGVS expression.
    pub r#type: HgvsType,
    /// Optional assembly.
    pub assembly: Option<String>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::HgvsExpression> for HgvsExpression {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::HgvsExpression,
    ) -> Result<Self, Self::Error> {
        Ok(HgvsExpression {
            nucleotide_expression: value
                .nucleotide_expression
                .map(|x| x.try_into())
                .transpose()?,
            protein_expression: value.protein_expression.map(|x| x.try_into()).transpose()?,
            molecular_consequences: value
                .molecular_consequences
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            r#type: value.r#type.map(|x| x.try_into()).transpose()?,
            assembly: value.assembly,
        })
    }
}

/// Description of a software.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Software {
    /// Name of the software.
    pub name: String,
    /// Version of the software; optional.
    pub version: Option<String>,
    /// Purpose of the software; optional.
    pub purpose: Option<String>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Software> for Software {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Software) -> Result<Self, Self::Error> {
        Ok(Software {
            name: value.name,
            version: value.version,
            purpose: value.purpose,
        })
    }
}

/// Description of the history of a record.
///
/// Called ``typeDescriptionHistory`` in XSD.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct DescriptionHistory {
    /// The pathogenicity description.
    pub description: String,
    /// The date of the description.
    pub dated: Option<chrono::DateTime<chrono::Utc>>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::DescriptionHistory> for DescriptionHistory {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::DescriptionHistory,
    ) -> Result<Self, Self::Error> {
        Ok(DescriptionHistory {
            description: value.description,
            dated: value.dated.map(|x| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(x.seconds, x.nanos as u32)
                    .unwrap_or_default()
            }),
        })
    }
}

/// Entry in an element set.
///
/// Called ``SetElementSetType`` in XSD.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct GenericSetElement {
    /// The element's value.
    pub value: String,
    /// The element's type.
    pub r#type: String,
    /// List of citations.
    pub citations: Vec<Citation>,
    /// List of xrefs.
    pub xrefs: Vec<Xref>,
    /// List of comments.
    pub comments: Vec<Comment>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::GenericSetElement> for GenericSetElement {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::GenericSetElement,
    ) -> Result<Self, Self::Error> {
        Ok(GenericSetElement {
            value: value.value,
            r#type: value.r#type,
            citations: value
                .citations
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            xrefs: value
                .xrefs
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            comments: value
                .comments
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
        })
    }
}

/// Common type for an entry in a set of attributes.
///
/// Called ``typeAttributeSet`` in XSD.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AttributeSetElement {
    /// The attribute value.
    pub attribute: Option<Attribute>,
    /// List of xrefs.
    pub xrefs: Vec<Xref>,
    /// List of citations.
    pub citations: Vec<Citation>,
    /// List of comments.
    pub comments: Vec<Comment>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::AttributeSetElement> for AttributeSetElement {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::AttributeSetElement,
    ) -> Result<Self, Self::Error> {
        Ok(AttributeSetElement {
            attribute: value.attribute.map(|x| x.try_into()).transpose()?,
            xrefs: value
                .xrefs
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            citations: value
                .citations
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            comments: value
                .comments
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
        })
    }
}

/// Extend the BaseAttribute with a `type` field.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Attribute {
    /// The base value.
    pub base: Option<BaseAttribute>,
    /// The type of the attribute.
    pub r#type: String,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::attribute_set_element::Attribute> for Attribute {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::attribute_set_element::Attribute,
    ) -> Result<Self, Self::Error> {
        Ok(Attribute {
            base: value.base.map(|x| x.try_into()).transpose()?,
            r#type: value.r#type,
        })
    }
}

/// Type to describe traits in various places.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Trait {
    /// names
    pub names: Vec<GenericSetElement>,
    /// symbols
    pub symbols: Vec<GenericSetElement>,
    /// attributes
    pub attributes: Vec<AttributeSetElement>,
    /// Trait relationships
    pub trait_relationships: Vec<TraitRelationship>,
    /// Citation list.
    pub citations: Vec<Citation>,
    /// Xref list.
    pub xrefs: Vec<Xref>,
    /// Comment list.
    pub comments: Vec<Comment>,
    /// Sources
    pub sources: Vec<String>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Trait> for Trait {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Trait) -> Result<Self, Self::Error> {
        Ok(Trait {
            names: value
                .names
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            symbols: value
                .symbols
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            attributes: value
                .attributes
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            trait_relationships: value
                .trait_relationships
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            citations: value
                .citations
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            xrefs: value
                .xrefs
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            comments: value
                .comments
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            sources: value.sources,
        })
    }
}

/// Local type for trait relationship.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct TraitRelationship {
    /// Name(s) of the trait.
    pub names: Vec<GenericSetElement>,
    /// Citation list.
    pub citations: Vec<Citation>,
    /// Xref list.
    pub xrefs: Vec<Xref>,
    /// Comment list.
    pub comments: Vec<Comment>,
    /// Sources
    pub sources: Vec<String>,
    /// Trait type.
    pub r#type: TraitRelationshipType,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::r#trait::TraitRelationship> for TraitRelationship {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::r#trait::TraitRelationship,
    ) -> Result<Self, Self::Error> {
        Ok(TraitRelationship {
            names: value
                .names
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            citations: value
                .citations
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            xrefs: value
                .xrefs
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            comments: value
                .comments
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            sources: value.sources,
            r#type: value.r#type.map(|x| x.try_into()).transpose()?,
        })
    }
}

/// Local enumeration for trait types.
///
/// NB: only DrugResponseAndDisease is used in the XML.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum TraitRelationshipType {
    /// corresponds to "phenotype"
    Phenotype,
    /// corresponds to "Subphenotype"
    Subphenotype,
    /// corresponds to "DrugResponseAndDisease"
    DrugResponseAndDisease,
    /// corresponds to "co-occuring condition"
    CoOccuringCondition,
    /// corresponds to "Finding member"
    FindingMember,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::r#trait::r#trait_relationship::Type>
    for TraitRelationshipType
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::r#trait::r#trait_relationship::Type,
    ) -> Result<Self, Self::Error> {
        match value {
            pbs::clinvar_data::clinvar_public::r#trait::r#trait_relationship::Type::Phenotype => {
                Ok(TraitRelationshipType::Phenotype)
            }
            pbs::clinvar_data::clinvar_public::r#trait::r#trait_relationship::Type::Subphenotype => {
                Ok(TraitRelationshipType::Subphenotype)
            }
            pbs::clinvar_data::clinvar_public::r#trait::r#trait_relationship::Type::DrugResponseAndDisease => {
                Ok(TraitRelationshipType::DrugResponseAndDisease)
            }
            pbs::clinvar_data::clinvar_public::r#trait::r#trait_relationship::Type::CoOccuringCondition => {
                Ok(TraitRelationshipType::CoOccuringCondition)
            }
            pbs::clinvar_data::clinvar_public::r#trait::r#trait_relationship::Type::FindingMember => {
                Ok(TraitRelationshipType::FindingMember)
            }
            _ => Err(anyhow::anyhow!(
                "Invalid value for TraitRelationshipType: {:?}",
                value
            )),
        }
    }
}

/// Describes an indication.
///
/// NB: Called "IndicationType" in the XSD.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Indication {
    /// Represents the value for the test indication as a name of a trait.
    pub traits: Vec<Trait>,
    /// List of names.
    pub names: Vec<GenericSetElement>,
    /// List of attributes.
    pub attributes: Vec<AttributeSetElement>,
    /// List of xrefs.
    pub xrefs: Vec<Xref>,
    /// List of citations.
    pub citations: Vec<Citation>,
    /// List of comments.
    pub comments: Vec<Comment>,
    /// The type of indication.
    pub r#type: IndicationType,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Indication> for Indication {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Indication) -> Result<Self, Self::Error> {
        Ok(Indication {
            traits: value
                .traits
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            names: value
                .names
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            attributes: value
                .attributes
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            xrefs: value
                .xrefs
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            citations: value
                .citations
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            comments: value
                .comments
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            r#type: value.r#type.map(|x| x.try_into()).transpose()?,
        })
    }
}

/// Enumeration for the indication type.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum IndicationType {
    /// corresponds to "Indication"
    Indication,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::indication::Type> for IndicationType {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::indication::Type,
    ) -> Result<Self, Self::Error> {
        match value {
            pbs::clinvar_data::clinvar_public::indication::Type::Indication => {
                Ok(IndicationType::Indication)
            }
            _ => Err(anyhow::anyhow!(
                "Invalid value for IndicationType: {:?}",
                value
            )),
        }
    }
}

/// A set of ``Trait`` objects.
///
/// NB: Called "ClinAsserTraitSetType" in the XSD.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct TraitSet {
    /// The traits.
    pub traits: Vec<Trait>,
    /// The names.
    pub names: Vec<GenericSetElement>,
    /// The symbols.
    pub symbols: Vec<GenericSetElement>,
    /// The attributes.
    pub attributes: Vec<AttributeSetElement>,
    /// List of xrefs.
    pub xrefs: Vec<Xref>,
    /// List of citations.
    pub citations: Vec<Citation>,
    /// List of comments.
    pub comments: Vec<Comment>,
    /// The type.
    pub r#type: TraitSetType,
    /// Date of last evaluation.
    pub date_last_evaluated: Option<chrono::DateTime<chrono::Utc>>,
    /// ID.
    pub id: Option<i64>,
    /// Whether contributes to aggregate classification.
    pub contributes_to_aggregate_classification: Option<bool>,
    /// Lower level of evidence.
    pub lower_level_of_evidence: Option<bool>,
    /// Explanation of or multiple conditions.
    pub multiple_condition_explanation: Option<String>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::TraitSet> for TraitSet {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::TraitSet) -> Result<Self, Self::Error> {
        Ok(TraitSet {
            traits: value
                .traits
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            names: value
                .names
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            symbols: value
                .symbols
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            attributes: value
                .attributes
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            xrefs: value
                .xrefs
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            citations: value
                .citations
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            comments: value
                .comments
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            r#type: value.r#type.map(|x| x.try_into()).transpose()?,
            date_last_evaluated: value.date_last_evaluated.map(|x| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(x.seconds, x.nanos as u32)
            }),
            id: value.id,
            contributes_to_aggregate_classification: value.contributes_to_aggregate_classification,
            lower_level_of_evidence: value.lower_level_of_evidence,
            multiple_condition_explanation: value.multiple_condition_explanation,
        })
    }
}

/// Local type.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum TraitSetType {
    /// corresponds to "Disease"
    Disease,
    /// corresponds to "DrugResponse"
    DrugResponse,
    /// corresponds to "Finding"
    Finding,
    /// corresponds to "PhenotypeInstruction"
    PhenotypeInstruction,
    /// corresponds to "TraitChoice"
    TraitChoice,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::trait_set::Type> for TraitSetType {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::trait_set::Type,
    ) -> Result<Self, Self::Error> {
        match value {
            pbs::clinvar_data::clinvar_public::trait_set::Type::Disease => {
                Ok(TraitSetType::Disease)
            }
            pbs::clinvar_data::clinvar_public::trait_set::Type::DrugResponse => {
                Ok(TraitSetType::DrugResponse)
            }
            pbs::clinvar_data::clinvar_public::trait_set::Type::Finding => {
                Ok(TraitSetType::Finding)
            }
            pbs::clinvar_data::clinvar_public::trait_set::Type::PhenotypeInstruction => {
                Ok(TraitSetType::PhenotypeInstruction)
            }
            pbs::clinvar_data::clinvar_public::trait_set::Type::TraitChoice => {
                Ok(TraitSetType::TraitChoice)
            }
            _ => Err(anyhow::anyhow!(
                "Invalid value for TraitSetType: {:?}",
                value
            )),
        }
    }
}

/// Aggregated germline classification info.
///
/// Corresponds to ``typeAggregatedGermlineClassification`` in XSD.
///
/// nested elements
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AggregatedGermlineClassification {
    /// The aggregate review status based on all germline submissions
    /// for this record.
    pub review_status: AggregateGermlineReviewStatus,
    /// We are not providing an enumeration for the values we report
    /// for germline classification within the xsd. Details are in
    /// <https://github.com/ncbi/clinvar/ClassificationOnClinVar.md>
    ///
    pub description: Option<String>,
    /// Explanation is used only when the description is 'conflicting
    /// data from submitters' The element summarizes the conflict.
    pub explanation: Option<Comment>,
    /// List of xrefs.
    pub xrefs: Vec<Xref>,
    /// List of citations.
    pub citations: Vec<Citation>,
    /// List of comments.
    pub comments: Vec<Comment>,
    /// History information.
    pub history_records: Vec<DescriptionHistory>,
    /// List of conditions.
    pub conditions: Vec<TraitSet>,
    /// Date of last evaluation.
    pub date_last_evaluated: Option<chrono::DateTime<chrono::Utc>>,
    /// Date of creation.
    pub date_created: Option<chrono::DateTime<chrono::Utc>>,
    /// Date of most recent submission.
    pub most_recent_submission: Option<chrono::DateTime<chrono::Utc>>,
    /// Number of submitters.
    pub number_of_submitters: Option<i32>,
    /// Number of submissions.
    pub number_of_submissions: Option<i32>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::AggregatedGermlineClassification>
    for AggregatedGermlineClassification
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::AggregatedGermlineClassification,
    ) -> Result<Self, Self::Error> {
        Ok(AggregatedGermlineClassification {
            review_status: value.review_status.map(|x| x.try_into()).transpose()?,
            description: value.description,
            explanation: value.explanation.map(|x| x.try_into()).transpose()?,
            xrefs: value
                .xrefs
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            citations: value
                .citations
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            comments: value
                .comments
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            history_records: value
                .history_records
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            conditions: value
                .conditions
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            date_last_evaluated: value.date_last_evaluated.map(|x| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(x.seconds, x.nanos as u32)
                    .unwrap_or_default()
            }),
            date_created: value.date_created.map(|x| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(x.seconds, x.nanos as u32)
                    .unwrap_or_default()
            }),
            most_recent_submission: value.most_recent_submission.map(|x| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(x.seconds, x.nanos as u32)
                    .unwrap_or_default()
            }),
            number_of_submitters: value.number_of_submitters,
            number_of_submissions: value.number_of_submissions,
        })
    }
}

/// Aggregated somatic clinical impact info.
///
/// nested elements
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AggregatedSomaticClinicalImpact {
    /// The aggregate review status based on all somatic clinical
    /// impact submissions for this record.
    pub review_status: AggregateSomaticClinicalImpactReviewStatus,
    /// We are not providing an enumeration for the values we report
    /// for somatic clinical impact classification within the xsd. Details are in
    /// <https://github.com/ncbi/clinvar/ClassificationOnClinVar.md>
    pub description: Option<String>,
    /// List of xrefs.
    pub xrefs: Vec<Xref>,
    /// List of citations.
    pub citations: Vec<Citation>,
    /// List of comments.
    pub comments: Vec<Comment>,
    /// History information.
    pub history_records: Vec<DescriptionHistory>,
    /// List of conditions.
    pub conditions: Vec<TraitSet>,
    /// Date of last evaluation.
    pub date_last_evaluated: Option<chrono::DateTime<chrono::Utc>>,
    /// Date of creation.
    pub date_created: Option<chrono::DateTime<chrono::Utc>>,
    /// Date of most recent submission.
    pub most_recent_submission: Option<chrono::DateTime<chrono::Utc>>,
    /// Number of submitters.
    pub number_of_submitters: Option<i32>,
    /// Number of submissions.
    pub number_of_submissions: Option<i32>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::AggregatedSomaticClinicalImpact>
    for AggregatedSomaticClinicalImpact
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::AggregatedSomaticClinicalImpact,
    ) -> Result<Self, Self::Error> {
        Ok(AggregatedSomaticClinicalImpact {
            review_status: value.review_status.map(|x| x.try_into()).transpose()?,
            description: value.description,
            xrefs: value
                .xrefs
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            citations: value
                .citations
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            comments: value
                .comments
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            history_records: value
                .history_records
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            conditions: value
                .conditions
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            date_last_evaluated: value.date_last_evaluated.map(|x| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(x.seconds, x.nanos as u32)
                    .unwrap_or_default()
            }),
            date_created: value.date_created.map(|x| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(x.seconds, x.nanos as u32)
                    .unwrap_or_default()
            }),
            most_recent_submission: value.most_recent_submission.map(|x| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(x.seconds, x.nanos as u32)
                    .unwrap_or_default()
            }),
            number_of_submitters: value.number_of_submitters,
            number_of_submissions: value.number_of_submissions,
        })
    }
}

/// Aggregated oncogenicity classification info.
///
/// nested elements
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AggregatedOncogenicityClassification {
    /// The aggregate review status based on all somatic clinical
    /// impact submissions for this record.
    pub review_status: AggregateOncogenicityReviewStatus,
    /// We are not providing an enumeration for the values we report
    /// for somatic clinical impact classification within the xsd. Details are in
    /// <https://github.com/ncbi/clinvar/ClassificationOnClinVar.md>
    pub description: Option<String>,
    /// List of xrefs.
    pub xrefs: Vec<Xref>,
    /// List of citations.
    pub citations: Vec<Citation>,
    /// List of comments.
    pub comments: Vec<Comment>,
    /// History information.
    pub history_records: Vec<DescriptionHistory>,
    /// List of conditions.
    pub conditions: Vec<TraitSet>,
    /// Date of last evaluation.
    pub date_last_evaluated: Option<chrono::DateTime<chrono::Utc>>,
    /// Date of creation.
    pub date_created: Option<chrono::DateTime<chrono::Utc>>,
    /// Date of most recent submission.
    pub most_recent_submission: Option<chrono::DateTime<chrono::Utc>>,
    /// Number of submitters.
    pub number_of_submitters: Option<i32>,
    /// Number of submissions.
    pub number_of_submissions: Option<i32>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::AggregatedOncogenicityClassification>
    for AggregatedOncogenicityClassification
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::AggregatedOncogenicityClassification,
    ) -> Result<Self, Self::Error> {
        Ok(AggregatedOncogenicityClassification {
            review_status: value.review_status.map(|x| x.try_into()).transpose()?,
            description: value.description,
            xrefs: value
                .xrefs
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            citations: value
                .citations
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            comments: value
                .comments
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            history_records: value
                .history_records
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            conditions: value
                .conditions
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            date_last_evaluated: value.date_last_evaluated.map(|x| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(x.seconds, x.nanos as u32)
                    .unwrap_or_default()
            }),
            date_created: value.date_created.map(|x| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(x.seconds, x.nanos as u32)
                    .unwrap_or_default()
            }),
            most_recent_submission: value.most_recent_submission.map(|x| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(x.seconds, x.nanos as u32)
                    .unwrap_or_default()
            }),
            number_of_submitters: value.number_of_submitters,
            number_of_submissions: value.number_of_submissions,
        })
    }
}

/// Used to bundle different types of Classifications (germline,
/// oncogenic, somatic clinical impact) ; Supports summary as
/// well as submission details.
///
/// NB: called "typeAggregateClassificationSet" in XSD.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AggregateClassificationSet {
    /// The aggregate germline classification.
    pub germline_classification: Option<AggregatedGermlineClassification>,
    /// The aggregate somatic clinical impact.
    pub somatic_clinical_impact: Option<AggregatedSomaticClinicalImpact>,
    /// The aggregate oncogenicity classification.
    pub oncogenicity_classification: Option<AggregatedOncogenicityClassification>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::AggregateClassificationSet>
    for AggregateClassificationSet
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::AggregateClassificationSet,
    ) -> Result<Self, Self::Error> {
        Ok(AggregateClassificationSet {
            germline_classification: value
                .germline_classification
                .map(|x| x.try_into())
                .transpose()?,
            somatic_clinical_impact: value
                .somatic_clinical_impact
                .map(|x| x.try_into())
                .transpose()?,
            oncogenicity_classification: value
                .oncogenicity_classification
                .map(|x| x.try_into())
                .transpose()?,
        })
    }
}

/// Describes the clinical significance of a variant.
///
/// Corresponds to `ClinicalSignificanceType` in XSD.
///
/// contained elements
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinicalSignificance {
    /// The optional review status.
    pub review_status: Option<SubmitterReviewStatus>,
    /// Structure used to support old data of AlleleDescriptionSet
    /// within Co-occurenceSet.
    ///
    /// NB: unused in XML
    pub description: Option<String>,
    /// Optional explanatory comment.
    ///
    /// Explanation is used only when the description is 'conflicting
    /// data from submitters' The element summarizes the conflict.
    ///
    /// NB: unused in XML
    pub explanation: Option<Comment>,
    /// Optional list of xrefs.
    pub xrefs: Vec<Xref>,
    /// Optional list of citations.
    pub citations: Vec<Citation>,
    /// Optional list of comments.
    pub comments: Vec<Comment>,
    /// Date of last evaluation.
    ///
    /// NB: unused in XML
    pub date_last_evaluated: Option<chrono::DateTime<chrono::Utc>>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::ClinicalSignificance> for ClinicalSignificance {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::ClinicalSignificance,
    ) -> Result<Self, Self::Error> {
        Ok(ClinicalSignificance {
            review_status: value.review_status.map(|x| x.try_into()).transpose()?,
            description: value.description,
            explanation: value.explanation.map(|x| x.try_into()).transpose()?,
            xrefs: value
                .xrefs
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            citations: value
                .citations
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            comments: value
                .comments
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            date_last_evaluated: value.date_last_evaluated.map(|x| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(x.seconds, x.nanos as u32)
                    .unwrap_or_default()
            }),
        })
    }
}

/// This is to be used within co-occurrence set.
///
/// Corresponds to `typeAlleleDescr` in XSD.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AlleleDescription {
    /// The name of the allele.
    pub name: String,
    /// Optional relative orientation.
    ///
    /// NB: Unused in XML
    pub relative_orientation: Option<RelativeOrientation>,
    /// Optional zygosity.
    pub zygosity: Option<Zygosity>,
    /// Optional clinical significance.
    ///
    /// Corresponds to `ClinicalSignificanceType` in XSD.
    pub clinical_significance: Option<ClinicalSignificance>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::AlleleDescription> for AlleleDescription {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::AlleleDescription,
    ) -> Result<Self, Self::Error> {
        Ok(AlleleDescription {
            name: value.name,
            relative_orientation: value
                .relative_orientation
                .map(|x| x.try_into())
                .transpose()?,
            zygosity: value.zygosity.map(|x| x.try_into()).transpose()?,
            clinical_significance: value
                .clinical_significance
                .map(|x| x.try_into())
                .transpose()?,
        })
    }
}

/// Enumeration for relative orientation.
///
/// NB: unused in XML
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum RelativeOrientation {
    /// corresponds to "cis"
    Cis,
    /// corresponds to "trans"
    Trans,
    /// corresponds to "unknown"
    Unknown,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::allele_description::RelativeOrientation>
    for RelativeOrientation
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::allele_description::RelativeOrientation,
    ) -> Result<Self, Self::Error> {
        match value {
            pbs::clinvar_data::clinvar_public::allele_description::RelativeOrientation::Cis => {
                Ok(RelativeOrientation::Cis)
            }
            pbs::clinvar_data::clinvar_public::allele_description::RelativeOrientation::Trans => {
                Ok(RelativeOrientation::Trans)
            }
            pbs::clinvar_data::clinvar_public::allele_description::RelativeOrientation::Unknown => {
                Ok(RelativeOrientation::Unknown)
            }
            _ => Err(anyhow::anyhow!(
                "Invalid value for RelativeOrientation: {:?}",
                value
            )),
        }
    }
}

/// A structure to support reporting of an accession, its version, the
/// date its status changed, and text describing that change.
///
/// Corresponds to `typeRecordHistory` in XSD.
///
/// nested elements
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct RecordHistory {
    /// Optional comment on the history record.
    pub comment: Option<Comment>,
    /// The accession.
    pub accession: String,
    /// The version.
    pub version: i32,
    /// The date the record.
    pub date_changed: Option<chrono::DateTime<chrono::Utc>>,
    /// Attribute @VaritionID is only populated for VCV, where @Accession
    /// is like VCV000000009
    pub variation_id: Option<i64>,
}

impl From<pbs::clinvar_data::clinvar_public::RecordHistory> for RecordHistory {
    fn from(record_history: pbs::clinvar_data::clinvar_public::RecordHistory) -> Self {
        Self {
            comment: record_history.comment.map(|x| x.into()),
            accession: record_history.accession,
            version: record_history.version,
            date_changed: record_history.date_changed.map(|x| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(x.seconds, x.nanos as u32)
                    .unwrap_or_default()
            }),
            variation_id: record_history.variation_id,
        }
    }
}

/// Report classification of a variant for a SCV.
///
/// Corresponds to `ClassificationTypeSCV` in XSD.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClassificationScv {
    /// The field's review status.
    pub review_status: SubmitterReviewStatus,
    /// The germline classification; mutually exlusive with `somatic_clinical_impact`
    /// and `oncogenicity_classification`.
    pub germline_classification: Option<String>,
    /// Information on the clinical impact; mutually exlusive with `germline_classification`
    /// and `oncogenicity_classification`.
    pub somatic_clinical_impact: Option<ClassificationScvSomaticClinicalImpact>,
    /// The oncogenicity classification; mutually exlusive with `germline_classification`
    /// and `oncogenicity_classification`.
    pub oncogenicity_classification: Option<String>,
    /// Optional explanation of classification.
    pub explanation_of_classification: Option<String>,
    /// List of classification scores.
    pub classification_scores: Vec<ClassificationScore>,
    /// List of xrefs.
    pub xrefs: Vec<Xref>,
    /// List of citations.
    pub citations: Vec<Citation>,
    /// List of comments.
    pub comments: Vec<Comment>,
    /// Date of last evaluation.
    pub date_last_evaluated: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<pbs::clinvar_data::clinvar_public::ClassificationScv> for ClassificationScv {
    fn from(classification_scv: pbs::clinvar_data::clinvar_public::ClassificationScv) -> Self {
        Self {
            review_status: classification_scv.review_status.into(),
            germline_classification: classification_scv.germline_classification,
            somatic_clinical_impact: classification_scv.somatic_clinical_impact.map(|x| x.into()),
            oncogenicity_classification: classification_scv.oncogenicity_classification,
            explanation_of_classification: classification_scv.explanation_of_classification,
            classification_scores: classification_scv
                .classification_scores
                .into_iter()
                .map(|x| x.into())
                .collect(),
            xrefs: classification_scv
                .xrefs
                .into_iter()
                .map(|x| x.into())
                .collect(),
            citations: classification_scv
                .citations
                .into_iter()
                .map(|x| x.into())
                .collect(),
            comments: classification_scv
                .comments
                .into_iter()
                .map(|x| x.into())
                .collect(),
            date_last_evaluated: classification_scv.date_last_evaluated.map(|x| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(x.seconds, x.nanos as u32)
                    .unwrap_or_default()
            }),
        }
    }
}

/// Clinical impact of a somatic variatn.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClassificationScvSomaticClinicalImpact {
    /// The somatic clinical impact value.
    pub value: String,
    /// Type of the clinical impact assertion.
    pub clinical_impact_assertion_type: Option<String>,
    /// Clinical impact significance.
    pub clinical_impact_clinical_significance: Option<String>,
    /// Name of the drug for the therapeutic assertion.
    pub drug_for_therapeutic_assertion: Option<String>,
}

impl From<pbs::clinvar_data::clinvar_public::classification_scv::SomaticClinicalImpact>
    for ClassificationScvSomaticClinicalImpact
{
    fn from(
        classification_scv_somatic_clinical_impact: pbs::clinvar_data::clinvar_public::classification_scv::SomaticClinicalImpact,
    ) -> Self {
        Self {
            value: classification_scv_somatic_clinical_impact.value,
            clinical_impact_assertion_type: classification_scv_somatic_clinical_impact
                .clinical_impact_assertion_type,
            clinical_impact_clinical_significance: classification_scv_somatic_clinical_impact
                .clinical_impact_clinical_significance,
            drug_for_therapeutic_assertion: classification_scv_somatic_clinical_impact
                .drug_for_therapeutic_assertion,
        }
    }
}

/// Classification score description.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClassificationScore {
    /// The score's value.
    pub value: f64,
    /// The score's type; optional.
    pub r#type: Option<String>,
}

impl From<pbs::clinvar_data::clinvar_public::classification_scv::ClassificationScore>
    for ClassificationScore
{
    fn from(
        classification_score: pbs::clinvar_data::clinvar_public::classification_scv::ClassificationScore,
    ) -> Self {
        Self {
            value: classification_score.value,
            r#type: classification_score.r#type,
        }
    }
}

/// Set of attributes for the primary submitter. Any addtional submitters
/// are captured in the AdditionalSubmitters element.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct SubmitterIdentifiers {
    /// Name of submitter.
    pub submitter_name: String,
    /// Organization ID.
    pub org_id: i64,
    /// Organization category.
    pub org_category: String,
    /// Organization abbreviation; optional.
    pub org_abbreviation: Option<String>,
}

impl From<pbs::clinvar_data::clinvar_public::SubmitterIdentifiers> for SubmitterIdentifiers {
    fn from(
        submitter_identifiers: pbs::clinvar_data::clinvar_public::SubmitterIdentifiers,
    ) -> Self {
        Self {
            submitter_name: submitter_identifiers.submitter_name,
            org_id: submitter_identifiers.org_id,
            org_category: submitter_identifiers.org_category,
            org_abbreviation: submitter_identifiers.org_abbreviation,
        }
    }
}

/// Definition of a species.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Species {
    /// Name of the species.
    pub name: String,
    /// Optional taxonomy ID.
    pub taxonomy_id: Option<i32>,
}

impl From<pbs::clinvar_data::clinvar_public::Species> for Species {
    fn from(species: pbs::clinvar_data::clinvar_public::Species) -> Self {
        Self {
            name: species.name,
            taxonomy_id: species.taxonomy_id,
        }
    }
}

/// Interpreted condition for an RCV record.
///
/// Corresponds to `typeRCVInterpretedCondition` in XSD.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClassifiedCondition {
    /// Condition value.
    pub value: String,
    /// Database name.
    pub db: Option<String>,
    /// Identifier in database.
    pub id: Option<String>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::ClassifiedCondition> for ClassifiedCondition {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::ClassifiedCondition,
    ) -> Result<Self, Self::Error> {
        Ok(ClassifiedCondition {
            value: value.value,
            db: value.db,
            id: value.id,
        })
    }
}
/// Inside ClinicalAssertion, a structure to support reporting of an
/// accession, its version, the date its status changed, and text describing that
/// change.
///
/// Corresponds to `typeClinicalAssertionRecordHistory` in XSD.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinicalAssertionRecordHistory {
    /// Optional comment.
    pub comment: Option<Comment>,
    /// Accession.
    pub accession: String,
    /// Optional version.
    pub version: Option<i32>,
    /// Date of the record.
    pub date_changed: Option<chrono::DateTime<chrono::Utc>>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::ClinicalAssertionRecordHistory>
    for ClinicalAssertionRecordHistory
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::ClinicalAssertionRecordHistory,
    ) -> Result<Self, Self::Error> {
        Ok(ClinicalAssertionRecordHistory {
            comment: value.comment.map(|x| x.try_into()).transpose()?,
            accession: value.accession,
            version: value.version,
            date_changed: value.date_changed.map(|x| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(x.seconds, x.nanos as u32)
                    .unwrap_or_default()
            }),
        })
    }
}

/// Description of a functional consequence.
///
/// Corresponds to `typeFunctionalConsequence` in XSD.
///
/// nested elements
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct FunctionalConsequence {
    /// List of xrefs.
    pub xrefs: Vec<Xref>,
    /// List of citations.
    pub citations: Vec<Citation>,
    /// List of comments.
    pub comments: Vec<Comment>,
    /// Value of functional consequence.
    pub value: String,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::FunctionalConsequence> for FunctionalConsequence {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::FunctionalConsequence,
    ) -> Result<Self, Self::Error> {
        Ok(FunctionalConsequence {
            xrefs: value
                .xrefs
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            citations: value
                .citations
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            comments: value
                .comments
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            value: value.value,
        })
    }
}

/// Type for the tag `GeneralCitations`.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct GeneralCitations {
    /// List of xrefs.
    pub xrefs: Vec<Xref>,
    /// List of citations.
    pub citations: Vec<Citation>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::GeneralCitations> for GeneralCitations {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::GeneralCitations,
    ) -> Result<Self, Self::Error> {
        Ok(GeneralCitations {
            xrefs: value
                .xrefs
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            citations: value
                .citations
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
        })
    }
}

/// This refers to the zygosity of the variant being asserted.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Cooccurrence {
    /// Optional zygosity.
    pub zygosity: Option<Zygosity>,
    /// The allele descriptions.
    pub allele_descriptions: Vec<AlleleDescription>,
    /// The optional count.
    pub count: Option<i32>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Cooccurrence> for Cooccurrence {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::Cooccurrence,
    ) -> Result<Self, Self::Error> {
        Ok(Cooccurrence {
            zygosity: value.zygosity.map(|x| x.try_into()).transpose()?,
            allele_descriptions: value
                .allele_descriptions
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            count: value.count,
        })
    }
}

/// A structure to support reporting the name of a submitter, its
/// organization id, and its abbreviation and type.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Submitter {
    /// The submitter's identifier.
    pub submitter_identifiers: Option<SubmitterIdentifiers>,
    /// The submitter type.
    pub r#type: SubmitterType,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Submitter> for Submitter {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Submitter) -> Result<Self, Self::Error> {
        Ok(Submitter {
            submitter_identifiers: value
                .submitter_identifiers
                .map(|x| x.try_into())
                .transpose()?,
            r#type: value.r#type.try_into()?,
        })
    }
}

/// Enumeration of submitter kind.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum SubmitterType {
    /// corresponds to "primary"
    Primary,
    /// corresponds to "secondary"
    Secondary,
    /// corresponds to "behalf"
    Behalf,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::submitter::Type> for SubmitterType {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::submitter::Type,
    ) -> Result<Self, Self::Error> {
        match value {
            pbs::clinvar_data::clinvar_public::submitter::Type::Primary => {
                Ok(SubmitterType::Primary)
            }
            pbs::clinvar_data::clinvar_public::submitter::Type::Secondary => {
                Ok(SubmitterType::Secondary)
            }
            pbs::clinvar_data::clinvar_public::submitter::Type::Behalf => Ok(SubmitterType::Behalf),
            _ => Err(anyhow::anyhow!(
                "Invalid value for SubmitterType: {:?}",
                value
            )),
        }
    }
}

/// Haploinsufficiency/Triplosensitivity of gene.
///
/// nested elements
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct DosageSensitivity {
    /// Value.
    pub value: String,
    /// Optional last evaluated date.
    pub last_evaluated: Option<chrono::DateTime<chrono::Utc>>,
    /// URL to ClinGen.
    pub clingen: Option<String>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::DosageSensitivity> for DosageSensitivity {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::DosageSensitivity,
    ) -> Result<Self, Self::Error> {
        Ok(DosageSensitivity {
            value: value.value,
            last_evaluated: value.last_evaluated.map(|x| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(x.seconds, x.nanos as u32)
                    .unwrap_or_default()
            }),
            clingen: value.clingen,
        })
    }
}

/// A name with an optional type.
///
/// Corresponds to `typeNames` in XSD.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct OtherName {
    /// The name's value.
    pub value: String,
    /// The name's type.
    pub r#type: Option<String>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::OtherName> for OtherName {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::OtherName) -> Result<Self, Self::Error> {
        Ok(OtherName {
            value: value.value,
            r#type: value.r#type,
        })
    }
}

/// A structure to support reporting of an accession, its version, the
/// date it was deleted and a free-text summary of why it was deleted.
///
/// Corresponds to `typeDeletedSCV`.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct DeletedScv {
    /// The accession.
    pub accession: String,
    /// The version.
    pub version: i32,
    /// The date of deletion.
    pub date_deleted: Option<chrono::DateTime<chrono::Utc>>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::DeletedScv> for DeletedScv {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::DeletedScv) -> Result<Self, Self::Error> {
        Ok(DeletedScv {
            accession: value.accession,
            version: value.version,
            date_deleted: value.date_deleted.map(|x| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(x.seconds, x.nanos as u32)
                    .unwrap_or_default()
            }),
        })
    }
}

/// There can be multiple types of location, and the locations may have
/// identifiers in other databases.
///
/// Corresponds to `typeLocation` in XSD.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Location {
    /// Cytogenetic location is maintained independent of sequence
    /// location, and can be submitted or computed from the sequence location.
    ///
    /// Between 0 and 4 entries.
    pub cytogenetic_locations: Vec<String>,
    /// Location on a defined sequence, with reference and alternate
    /// allele, and start /stop values depending on the specificity with which the
    /// variant location is known. The number system of offset 1, and
    /// right-justified to be consistent with HGVS location data.
    pub sequence_locations: Vec<SequenceLocation>,
    /// The location of the variant relative to features within the gene.
    pub gene_locations: Vec<String>,
    /// List of xrefs.
    pub xrefs: Vec<Xref>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Location> for Location {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Location) -> Result<Self, Self::Error> {
        Ok(Location {
            cytogenetic_locations: value.cytogenetic_locations,
            sequence_locations: value
                .sequence_locations
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
            gene_locations: value.gene_locations,
            xrefs: value
                .xrefs
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<_, _>>()?,
        })
    }
}

/// Local type for sequence location.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct SequenceLocation {
    /// forDisplay value.
    pub for_display: Option<bool>,
    /// Name of assembly.
    pub assembly: String,
    /// Chromosomeof variant.
    pub chr: Chromosome,
    /// Optional chromosome accession.
    pub accession: Option<String>,
    /// Outer start position.
    pub outer_start: Option<u32>,
    /// Inner start position.
    pub inner_start: Option<u32>,
    /// Start position.
    pub start: Option<u32>,
    /// Stop position.
    pub stop: Option<u32>,
    /// Inner stop position.
    pub inner_stop: Option<u32>,
    /// Outer stop position.
    pub outer_stop: Option<u32>,
    /// Display start position.
    pub display_start: Option<u32>,
    /// Display stop position.
    pub display_stop: Option<u32>,
    /// Strand.
    pub strand: Option<String>,
    /// Variant length.
    pub variant_length: Option<u32>,
    /// Reference allele.
    pub reference_allele: Option<String>,
    /// Alternate allele.
    pub alternate_allele: Option<String>,
    /// Assembly accession version.
    pub assembly_accession_version: Option<String>,
    /// Assembly status.
    pub assembly_status: Option<AssemblyStatus>,
    /// Position in VCF.
    pub position_vcf: Option<u32>,
    /// Reference allele in VCF.
    pub reference_allele_vcf: Option<String>,
    /// Alternate allele in VCF.
    pub alternate_allele_vcf: Option<String>,
    /// For display length.
    pub for_display_length: Option<u32>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::location::SequenceLocation> for SequenceLocation {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::location::SequenceLocation,
    ) -> Result<Self, Self::Error> {
        Ok(SequenceLocation {
            for_display: value.for_display,
            assembly: value.assembly,
            chr: value.chr.try_into()?,
            accession: value.accession,
            outer_start: value.outer_start,
            inner_start: value.inner_start,
            start: value.start,
            stop: value.stop,
            inner_stop: value.inner_stop,
            outer_stop: value.outer_stop,
            display_start: value.display_start,
            display_stop: value.display_stop,
            strand: value.strand,
            variant_length: value.variant_length,
            reference_allele: value.reference_allele,
            alternate_allele: value.alternate_allele,
            assembly_accession_version: value.assembly_accession_version,
            assembly_status: value.assembly_status.map(|x| x.try_into()).transpose()?,
            position_vcf: value.position_vcf,
            reference_allele_vcf: value.reference_allele_vcf,
            alternate_allele_vcf: value.alternate_allele_vcf,
            for_display_length: value.for_display_length,
        })
    }
}

/// Local enum for the assembly status.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum AssemblyStatus {
    /// corresponds to "current"
    Current,
    /// corresponds to "previous"
    Previous,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::location::sequence_location::AssemblyStatus>
    for AssemblyStatus
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::location::sequence_location::AssemblyStatus,
    ) -> Result<Self, Self::Error> {
        match value {
            pbs::clinvar_data::clinvar_public::location::sequence_location::AssemblyStatus::Current => {
                Ok(AssemblyStatus::Current)
            }
            pbs::clinvar_data::clinvar_public::location::sequence_location::AssemblyStatus::Previous => {
                Ok(AssemblyStatus::Previous)
            }
            _ => Err(anyhow::anyhow!("Unknown AssemblyStatus value: {:?}", value)),
        }
    }
}

/// Description of a SCV.
///
/// Corresponds to "typeSCV" in XSD.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Scv {
    /// Optional title.
    pub title: Option<String>,
    /// Accession.
    pub accession: String,
    /// Version.
    pub version: i32,
}

impl From<pbs::clinvar_data::clinvar_public::Scv> for Scv {
    fn from(value: pbs::clinvar_data::clinvar_public::Scv) -> Self {
        Scv {
            title: value.title,
            accession: value.accession,
            version: value.version,
        }
    }
}

/// Structure to describe attributes of any family data in an observation.
/// If the details of the number of families and the de-identified pedigree id are not
/// available, use FamilyHistory to describe what type of family data is available. Can
/// also be used to report 'Yes' or 'No' if there are no more details.
///
/// Corresponds to "FamilyInfo" in XSD.
///
/// nested elements
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct FamilyData {
    /// Optional family history.
    pub family_history: Option<String>,
    /// Number of families.
    pub num_families: Option<i32>,
    /// Number of families with variant.
    pub num_families_with_variant: Option<i32>,
    /// Number of families with segregation observed.
    pub num_families_with_segregation_observed: Option<i32>,
    /// Pedigree ID.
    pub pedigree_id: Option<String>,
    /// Segregation oberved (yes, no, number)
    pub segregation_observed: Option<String>,
}

impl From<pbs::clinvar_data::clinvar_public::FamilyData> for FamilyData {
    fn from(value: pbs::clinvar_data::clinvar_public::FamilyData) -> Self {
        FamilyData {
            family_history: value.family_history,
            num_families: value.num_families,
            num_families_with_variant: value.num_families_with_variant,
            num_families_with_segregation_observed: value.num_families_with_segregation_observed,
            pedigree_id: value.pedigree_id,
            segregation_observed: value.segregation_observed,
        }
    }
}

/// Description of a sample.
///
/// Corresponds to `typeSample` in XSD.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Sample {
    /// The sample description.
    pub sample_description: Option<SampleDescription>,
    /// The sample origin.
    pub origin: Option<Origin>,
    /// Sample ethnicity.
    pub ethnicity: Option<String>,
    /// Sample geographic origin.
    pub geographic_origin: Option<String>,
    /// Sample tissue.
    pub tissue: Option<String>,
    /// Presence of variant in normal tissue.
    pub somatic_variant_in_normal_tissue: Option<SomaticVariantInNormalTissue>,
    /// Somatic variant allele fraction.
    pub somatic_variant_allele_fraction: Option<String>,
    /// Cell line name.
    pub cell_line: Option<String>,
    /// Species.
    pub species: Option<Species>,
    /// Age (range), max. size of 2.
    pub ages: Vec<Age>,
    /// Strain.
    pub strain: Option<String>,
    /// Affected status.
    pub affected_status: Option<AffectedStatus>,
    /// Denominator, total individuals included in this observation set.
    pub numer_tested: Option<i32>,
    /// Denominator, total males included in this observation set.
    pub number_males: Option<i32>,
    /// Denominator, total females included in this observation set.
    pub number_females: Option<i32>,
    /// Denominator, total number chromosomes tested. Number affected
    /// and unaffected are captured in the element NumberObserved.
    pub number_chr_tested: Option<i32>,
    /// Gender should be used ONLY if explicit values are not
    /// available for number of males or females, and there is a need to indicate
    /// that the genders in the sample are known.
    pub gender: Option<Gender>,
    /// Family information.
    pub family_data: Option<FamilyData>,
    /// Optional proband ID.
    pub proband: Option<String>,
    /// Optional indication.
    pub indication: Option<Indication>,
    /// List of citations.
    pub citations: Vec<Citation>,
    /// List of xrefs.
    pub xrefs: Vec<Xref>,
    /// List of comments.
    pub comments: Vec<Comment>,
    /// Source type.
    pub source_type: Option<SampleSourceType>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Sample> for Sample {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Sample) -> Result<Self, Self::Error> {
        Ok(Sample {
            sample_description: value
                .sample_description
                .map(SampleDescription::try_from)
                .transpose()?,
            origin: value.origin.map(Origin::try_from).transpose()?,
            ethnicity: value.ethnicity,
            geographic_origin: value.geographic_origin,
            tissue: value.tissue,
            somatic_variant_in_normal_tissue: value
                .somatic_variant_in_normal_tissue
                .map(SomaticVariantInNormalTissue::try_from)
                .transpose()?,
            somatic_variant_allele_fraction: value.somatic_variant_allele_fraction,
            cell_line: value.cell_line,
            species: value.species.map(Species::try_from).transpose()?,
            ages: value
                .ages
                .into_iter()
                .map(Age::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            strain: value.strain,
            affected_status: value
                .affected_status
                .map(AffectedStatus::try_from)
                .transpose()?,
            numer_tested: value.numer_tested,
            number_males: value.number_males,
            number_females: value.number_females,
            number_chr_tested: value.number_chr_tested,
            gender: value
                .gender
                .map(Gender::try_from(
                    pbs::clinvar_data::clinvar_public::sample::Gender::try_from(value.gender)?,
                ))
                .transpose()?,
            family_data: value.family_data.map(FamilyData::try_from).transpose()?,
            proband: value.proband,
            indication: value.indication.map(Indication::try_from).transpose()?,
            citations: value
                .citations
                .into_iter()
                .map(Citation::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            xrefs: value
                .xrefs
                .into_iter()
                .map(Xref::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            comments: value
                .comments
                .into_iter()
                .map(Comment::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            source_type: value
                .source_type
                .map(SampleSourceType::try_from)
                .transpose()?,
        })
    }
}

/// Local type for sample description.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct SampleDescription {
    /// Description of sample.
    pub description: Option<Comment>,
    /// Citation.
    pub citation: Option<Citation>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::sample::SampleDescription> for SampleDescription {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::sample::SampleDescription,
    ) -> Result<Self, Self::Error> {
        Ok(SampleDescription {
            description: value.description.map(Comment::try_from).transpose()?,
            citation: value.citation.map(Citation::try_from).transpose()?,
        })
    }
}

/// Local type for an age.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Age {
    /// The age value.
    pub value: i32,
    /// The age unit.
    pub unit: AgeUnit,
    /// The age type.
    pub r#type: AgeType,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::sample::Age> for Age {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::sample::Age,
    ) -> Result<Self, Self::Error> {
        Ok(Age {
            value: value.value,
            unit: AgeUnit::try_from(value.unit)?,
            r#type: AgeType::try_from(value.r#type)?,
        })
    }
}

/// Local enumeration for presence in normal tissue.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum SomaticVariantInNormalTissue {
    /// corresponds to "present"
    Present,
    /// corresponds to "absent"
    Absent,
    /// corresponds to "not tested"
    NotTested,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::sample::SomaticVariantInNormalTissue>
    for SomaticVariantInNormalTissue
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::sample::SomaticVariantInNormalTissue,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::sample::SomaticVariantInNormalTissue::Present => {
                SomaticVariantInNormalTissue::Present
            }
            pbs::clinvar_data::clinvar_public::sample::SomaticVariantInNormalTissue::Absent => {
                SomaticVariantInNormalTissue::Absent
            }
            pbs::clinvar_data::clinvar_public::sample::SomaticVariantInNormalTissue::NotTested => {
                SomaticVariantInNormalTissue::NotTested
            }
            _ => anyhow::bail!("Invalid sample::SomaticVariantInNormalTissue {:?}", value),
        })
    }
}

/// Local enumeration for an age unit.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum AgeUnit {
    /// corresponds to "days"
    Days,
    /// corresponds to "weeks"
    Weeks,
    /// corresponds to "months"
    Months,
    /// corresponds to "years"
    Years,
    /// corresponds to "weeks gestation"
    WeeksGestation,
    /// corresponds to "months gestation"
    MonthsGestation,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::sample::AgeUnit> for AgeUnit {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::sample::AgeUnit,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::sample::AgeUnit::Days => AgeUnit::Days,
            pbs::clinvar_data::clinvar_public::sample::AgeUnit::Weeks => AgeUnit::Weeks,
            pbs::clinvar_data::clinvar_public::sample::AgeUnit::Months => AgeUnit::Months,
            pbs::clinvar_data::clinvar_public::sample::AgeUnit::Years => AgeUnit::Years,
            pbs::clinvar_data::clinvar_public::sample::AgeUnit::WeeksGestation => {
                AgeUnit::WeeksGestation
            }
            pbs::clinvar_data::clinvar_public::sample::AgeUnit::MonthsGestation => {
                AgeUnit::MonthsGestation
            }
            _ => anyhow::bail!("Invalid sample::AgeUnit {:?}", value),
        })
    }
}

/// Local enumeration for an age type.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum AgeType {
    /// corresponds to "minimum"
    Minimum,
    /// corresponds to "maximum"
    Maximum,
    /// corresponds to "single"
    Single,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::sample::AgeType> for AgeType {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::sample::AgeType,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::sample::AgeType::Minimum => AgeType::Minimum,
            pbs::clinvar_data::clinvar_public::sample::AgeType::Maximum => AgeType::Maximum,
            pbs::clinvar_data::clinvar_public::sample::AgeType::Single => AgeType::Single,
            _ => anyhow::bail!("Invalid sample::AgeType {:?}", value),
        })
    }
}

/// Local enumeration for the affected status.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum AffectedStatus {
    /// corresponds to "yes"
    Yes,
    /// corresponds to "no"
    No,
    /// corresponds to "not provided"
    NotProvided,
    /// corresponds to "unknown"
    Unknown,
    /// corresponds to "not applicable"
    NotApplicable,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::sample::AffectedStatus> for AffectedStatus {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::sample::AffectedStatus,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::sample::AffectedStatus::Yes => AffectedStatus::Yes,
            pbs::clinvar_data::clinvar_public::sample::AffectedStatus::No => AffectedStatus::No,
            pbs::clinvar_data::clinvar_public::sample::AffectedStatus::NotProvided => {
                AffectedStatus::NotProvided
            }
            pbs::clinvar_data::clinvar_public::sample::AffectedStatus::Unknown => {
                AffectedStatus::Unknown
            }
            pbs::clinvar_data::clinvar_public::sample::AffectedStatus::NotApplicable => {
                AffectedStatus::NotApplicable
            }
            _ => anyhow::bail!("Invalid sample::AffectedStatus {:?}", value),
        })
    }
}

/// Local enumeration for gender.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum Gender {
    /// corresponds to "male"
    Male,
    /// corresponds to "female"
    Female,
    /// corresponds to "mixed"
    Mixed,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::sample::Gender> for Gender {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::sample::Gender,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::sample::Gender::Male => Gender::Male,
            pbs::clinvar_data::clinvar_public::sample::Gender::Female => Gender::Female,
            pbs::clinvar_data::clinvar_public::sample::Gender::Mixed => Gender::Mixed,
            _ => anyhow::bail!("Invalid sample::Gender {:?}", value),
        })
    }
}

/// Local enumeration for SourceType.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum SampleSourceType {
    /// corresponds to "submitter-generated"
    SubmitterGenerated,
    /// corresponds to "data mining"
    DataMining,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::sample::SourceType> for SampleSourceType {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::sample::SourceType,
    ) -> Result<Self, Self::Error> {
        match value {
            pbs::clinvar_data::clinvar_public::sample::SourceType::SubmitterGenerated => {
                Ok(SampleSourceType::SubmitterGenerated)
            }
            pbs::clinvar_data::clinvar_public::sample::SourceType::DataMining => {
                Ok(SampleSourceType::DataMining)
            }
            _ => Err(anyhow::anyhow!(
                "Invalid value for sample::SourceType: {:?}",
                value
            )),
        }
    }
}
/// Details of a method used to generate variant calls or predict/report
/// functional consequence. The name of the platform should represent a sequencer or an
/// array, e.g. sequencing or array , e.g. capillary, 454, Helicos, Solexa, SOLiD. This
/// structure should also be used if the method is 'Curation'.
///
/// Corresponds to `MethodType` in XSD.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Method {
    /// Platform name.
    pub name_platform: Option<String>,
    /// Platform type.
    pub type_platform: Option<String>,
    /// Method purpose.
    pub purpose: Option<String>,
    /// Method result type.
    pub result_type: Option<ResultType>,
    /// Smallest reported.
    pub min_reported: Option<String>,
    /// Largest reported.
    pub max_reported: Option<String>,
    /// Reference standard.
    pub reference_standard: Option<String>,
    /// List of citations.
    pub citations: Vec<Citation>,
    /// List of xrefs.
    pub xrefs: Vec<Xref>,
    /// Free text to enrich the description of the method and to
    /// provide information not captured in specific fields.
    pub description: Option<String>,
    /// List of softwares used.
    pub software: Vec<Software>,
    /// Source type.
    pub source_type: Option<MethodSourceType>,
    /// Method type.
    pub method_type: MethodListType,
    /// Method attribute.
    pub method_attributes: Vec<MethodAttribute>,
    /// ObsMethodAttribute is used to indicate an attribute specific
    /// to a particular method in conjunction with a particular observation .
    pub obs_method_attributes: Vec<ObsMethodAttribute>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Method> for Method {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Method) -> Result<Self, Self::Error> {
        Ok(Self {
            name_platform: value.name_platform,
            type_platform: value.type_platform,
            purpose: value.purpose,
            result_type: value
                .result_type
                .map(|result_type| {
                    ResultType::try_from(
                        pbs::clinvar_data::clinvar_public::method::ResultType::try_from(
                            result_type,
                        )?,
                    )
                })
                .transpose()?,
            min_reported: value.min_reported,
            max_reported: value.max_reported,
            reference_standard: value.reference_standard,
            citations: value
                .citations
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            xrefs: value
                .xrefs
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            description: value.description,
            software: value
                .software
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            source_type: value.source_type.map(TryInto::try_into).transpose()?,
            method_type: value.method_type.try_into()?,
            method_attributes: value
                .method_attributes
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            obs_method_attributes: value
                .obs_method_attributes
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

/// Local type for method attribute.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct MethodAttribute {
    /// The base value.
    pub base: Option<BaseAttribute>,
    /// The attribute type.
    pub r#type: MethodAttributeType,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::method::MethodAttribute> for MethodAttribute {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::method::MethodAttribute,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            base: value.base.map(TryInto::try_into).transpose()?,
            r#type: MethodAttributeType::try_from(
                pbs::clinvar_data::clinvar_public::method::method_attribute::AttributeType::try_from(
                    value.r#type
                )?
            )?
        })
    }
}

/// Local enumeration of attribute type.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum MethodAttributeType {
    /// corresponds to "Location"
    Location,
    /// corresponds to "ControlsAppropriate"
    ControlsAppropriate,
    /// corresponds to "MethodAppropriate"
    MethodAppropriate,
    /// corresponds to "TestName"
    TestName,
    /// corresponds to "StructVarMethod"
    StructVarMethodType,
    /// corresponds to "ProbeAccession"
    ProbeAccession,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::method::method_attribute::AttributeType>
    for MethodAttributeType
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::method::method_attribute::AttributeType,
    ) -> Result<Self, Self::Error> {
        match value {
            pbs::clinvar_data::clinvar_public::method::method_attribute::AttributeType::Location => Ok(MethodAttributeType::Location),
            pbs::clinvar_data::clinvar_public::method::method_attribute::AttributeType::ControlsAppropriate => {
                Ok(MethodAttributeType::ControlsAppropriate)
            }
            pbs::clinvar_data::clinvar_public::method::method_attribute::AttributeType::MethodAppropriate => {
                Ok(MethodAttributeType::MethodAppropriate)
            }
            pbs::clinvar_data::clinvar_public::method::method_attribute::AttributeType::TestName => Ok(MethodAttributeType::TestName),
            pbs::clinvar_data::clinvar_public::method::method_attribute::AttributeType::StructVarMethodType => {
                Ok(MethodAttributeType::StructVarMethodType)
            }
            pbs::clinvar_data::clinvar_public::method::method_attribute::AttributeType::ProbeAccession => {
                Ok(MethodAttributeType::ProbeAccession)
            }
            _ => anyhow::bail!("Invalid AttributeType {:?}", value)
        }
    }
}

/// Local type for observation method attribute.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ObsMethodAttribute {
    /// The base value.
    pub base: Option<BaseAttribute>,
    /// The attribute type.
    pub r#type: ObsMethodAttributeType,
    /// Optional comments.
    pub comments: Vec<Comment>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::method::ObsMethodAttribute> for ObsMethodAttribute {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::method::ObsMethodAttribute,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            base: value.base.map(TryInto::try_into).transpose()?,
            r#type: value.r#type.try_into()?,
            comments: value
                .comments
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

/// Local enumeration for attribute type.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum ObsMethodAttributeType {
    /// corresponds to "MethodResult"
    MethodResult,
    /// corresponds to "TestingLaboratory"
    TestingLaboratory,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::method::obs_method_attribute::AttributeType>
    for ObsMethodAttributeType
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::method::obs_method_attribute::AttributeType,
    ) -> Result<Self, Self::Error> {
        match value {
            pbs::clinvar_data::clinvar_public::method::obs_method_attribute::AttributeType::MethodResult => Ok(ObsMethodAttributeType::MethodResult),
            pbs::clinvar_data::clinvar_public::method::obs_method_attribute::AttributeType::TestingLaboratory => Ok(ObsMethodAttributeType::TestingLaboratory),
            _ => anyhow::bail!("Invalid obs_method_attribute::AttributeType: {:?}", value),
        }
    }
}

/// Local enumeration for result types.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum ResultType {
    /// corresponds to "number of occurrences"
    NumberOfOccurrences,
    /// corresponds to "p value"
    PValue,
    /// corresponds to "odds ratio"
    OddsRatio,
    /// corresponds to "variant call"
    VariantCall,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::method::ResultType> for ResultType {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::method::ResultType,
    ) -> Result<Self, Self::Error> {
        match value {
            pbs::clinvar_data::clinvar_public::method::ResultType::NumberOfOccurrences => {
                Ok(ResultType::NumberOfOccurrences)
            }
            pbs::clinvar_data::clinvar_public::method::ResultType::PValue => Ok(ResultType::PValue),
            pbs::clinvar_data::clinvar_public::method::ResultType::OddsRatio => {
                Ok(ResultType::OddsRatio)
            }
            pbs::clinvar_data::clinvar_public::method::ResultType::VariantCall => {
                Ok(ResultType::VariantCall)
            }
            _ => anyhow::bail!("Invalid method::ResultType {:?}", value),
        }
    }
}

/// Local enumeration for SourceType.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum MethodSourceType {
    /// corresponds to "submitter-generated"
    SubmitterGenerated,
    /// corresponds to "data mining"
    DataMining,
    /// corresponds to "data review"
    DataReview,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::method::SourceType> for MethodSourceType {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::method::SourceType,
    ) -> Result<Self, Self::Error> {
        match value {
            pbs::clinvar_data::clinvar_public::method::SourceType::SubmitterGenerated => {
                Ok(MethodSourceType::SubmitterGenerated)
            }
            pbs::clinvar_data::clinvar_public::method::SourceType::DataMining => {
                Ok(MethodSourceType::DataMining)
            }
            pbs::clinvar_data::clinvar_public::method::SourceType::DataReview => {
                Ok(MethodSourceType::DataReview)
            }
            _ => anyhow::bail!("Invalid method::SourceType {:?}", value),
        }
    }
}

/// This is a record per variant (Measure/@ID,AlleleID) as submitted for
/// accessioning in an SCV.
///
/// Corresponds to "typeAlleleSCV" in XSD.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AlleleScv {
    /// 0 to many genes (and related data ) related to the allele
    /// being reported.
    pub genes: Vec<AlleleScvGene>,
    /// Name provided by the submitter.
    pub name: Option<OtherName>,
    /// Variant type.
    pub variant_type: Option<String>,
    /// Location.
    pub location: Option<Location>,
    /// List of other names.
    pub other_names: Vec<OtherName>,
    /// Single letter representation of the amino acid change and its
    /// location.
    pub protein_changes: Vec<String>,
    /// List of xrefs.
    pub xrefs: Vec<Xref>,
    /// List of citations.
    pub citations: Vec<Citation>,
    /// List of comments.
    pub comments: Vec<Comment>,
    /// Currently redundant with the MolecularConsequence element of
    /// the HGVS element?
    pub molecular_consequences: Vec<MolecularConsequence>,
    /// Functional consequences.
    pub functional_consequences: Vec<FunctionalConsequence>,
    /// Attributes.
    pub attributes: Vec<AttributeSetElement>,
    /// Allele ID.
    pub allele_id: Option<i64>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::AlleleScv> for AlleleScv {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::AlleleScv) -> Result<Self, Self::Error> {
        Ok(Self {
            genes: value
                .genes
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            name: value.name.map(TryInto::try_into).transpose()?,
            variant_type: value.variant_type,
            location: value.location.map(TryInto::try_into).transpose()?,
            other_names: value
                .other_names
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            protein_changes: value.protein_changes,
            xrefs: value
                .xrefs
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            citations: value
                .citations
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            comments: value
                .comments
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            molecular_consequences: value
                .molecular_consequences
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            functional_consequences: value
                .functional_consequences
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            attributes: value
                .attributes
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            allele_id: value.allele_id,
        })
    }
}
/// Local type for Gene.
///
/// nested elements
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AlleleScvGene {
    /// Gene name.
    pub name: Option<String>,
    /// Used to set key words for retrieval or
    /// display about a gene, such as genes listed by the
    /// ACMG guidelines.
    pub properties: Vec<String>,
    /// Used for gene specific identifiers
    /// such as MIM number, Gene ID, HGNC ID, etc.
    pub xrefs: Vec<Xref>,
    /// Optional gene symbol.
    pub symbol: Option<String>,
    /// Relationship between gene and variant.
    pub relationship_type: Option<GeneVariantRelationship>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::allele_scv::Gene> for AlleleScvGene {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::allele_scv::Gene,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            properties: value.properties,
            xrefs: value
                .xrefs
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            symbol: value.symbol,
            relationship_type: value.relationship_type.map(TryInto::try_into).transpose()?,
        })
    }
}

/// Local type for MolecularConsequence.
///
/// nested elements
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct MolecularConsequence {
    /// Xref list.
    pub xrefs: Vec<Xref>,
    /// Citation list.
    pub citations: Vec<Citation>,
    /// Comment list.
    pub comments: Vec<Comment>,
    /// RS id.
    pub rs: Option<i64>,
    /// Optional HGVS expression.
    pub hgvs: Option<String>,
    /// Optional SO id.
    pub so_id: Option<String>,
    /// Function.
    pub function: String,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::allele_scv::MolecularConsequence>
    for MolecularConsequence
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::allele_scv::MolecularConsequence,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            xrefs: value
                .xrefs
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            citations: value
                .citations
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            comments: value
                .comments
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            rs: value.rs,
            hgvs: value.hgvs,
            so_id: value.so_id,
            function: value.function,
        })
    }
}

/// This is a record of a haplotype in SCV.
///
/// Corresponds to `typeHaplotypeSCV` in XSD.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct HaplotypeScv {
    /// The list of alleles in the haplotype.
    pub simple_alleles: Vec<AlleleScv>,
    /// The preferred representation of the haplotype.
    pub name: Option<String>,
    /// Names other than 'preferred' used for the haplotype.
    pub other_names: Vec<OtherName>,
    /// Classification of the variant.
    pub classifications: Option<AggregateClassificationSet>,
    /// Functional consequences of the variant.
    pub functional_consequences: Vec<FunctionalConsequence>,
    /// List of attributes.
    pub attributes: Vec<AttributeSetElement>,
    /// List of citations.
    pub citations: Vec<Citation>,
    /// List of cross-references.
    pub xrefs: Vec<Xref>,
    /// List of comments.
    pub comments: Vec<Comment>,
    /// Variation ID.
    pub variation_id: Option<i64>,
    /// Number of copies.
    pub number_of_copies: Option<i32>,
    /// Number of chromosomes.
    pub number_of_chromosomes: Option<i32>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::HaplotypeScv> for HaplotypeScv {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::HaplotypeScv,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            simple_alleles: value
                .simple_alleles
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            name: value.name,
            other_names: value
                .other_names
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            classifications: value.classifications.map(TryInto::try_into).transpose()?,
            functional_consequences: value
                .functional_consequences
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            attributes: value
                .attributes
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            citations: value
                .citations
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            xrefs: value
                .xrefs
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            comments: value
                .comments
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            variation_id: value.variation_id,
            number_of_copies: value.number_of_copies,
            number_of_chromosomes: value.number_of_chromosomes,
        })
    }
}

/// Used to report genotypes, be they simple or complex diplotypes.
///
/// Corresponds to "typeGenotypeSCV" in XSD.
///
/// nested elements
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct GenotypeScv {
    /// Simple alleles; mutually exclusive with `haplotypes`.
    pub simple_alleles: Vec<AlleleScv>,
    /// Haplotype; mutually exclusive with `simple_alleles`.
    ///
    /// Allows more than 2 haplotypes per genotype to support
    /// representation of ploidy.
    pub haplotypes: Vec<HaplotypeScv>,
    /// Optional name.
    pub name: Option<String>,
    /// Other names used for the genotype.
    pub other_names: Vec<OtherName>,
    /// The variation type.
    pub variation_type: VariationType,
    /// Functional consequences.
    pub functional_consequences: Vec<FunctionalConsequence>,
    /// Attributes.
    pub attributes: Vec<AttributeSetElement>,
    /// List of citations.
    pub citations: Vec<Citation>,
    /// List of xrefs.
    pub xrefs: Vec<Xref>,
    /// List of comments.
    pub comments: Vec<Comment>,
    /// Variation ID.
    pub variation_id: Option<i64>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::GenotypeScv> for GenotypeScv {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::GenotypeScv,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            simple_alleles: value
                .simple_alleles
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            haplotypes: value
                .haplotypes
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            name: value.name,
            other_names: value
                .other_names
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            variation_type: value.variation_type.try_into()?,
            functional_consequences: value
                .functional_consequences
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            attributes: value
                .attributes
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            citations: value
                .citations
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            xrefs: value
                .xrefs
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            comments: value
                .comments
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            variation_id: value.variation_id,
        })
    }
}
/// Documents in what populations or samples an allele or genotype has
/// been observed relative to the described trait. Summary observations can be
/// registered per submitted assertion, grouped by common citation, study type, origin,
/// ethnicity, tissue, cell line, and species data. Not all options are valid per study
/// type, but these will not be validated in the xsd.
///
/// Corresponds to `ObservationSet` in XSD.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ObservedIn {
    /// Sample.
    pub sample: Option<Sample>,
    /// Observed data.
    pub observed_data: Vec<ObservedData>,
    /// Co-occurence set.
    pub cooccurrence_sets: Vec<Cooccurrence>,
    /// TraitSet.
    pub trait_set: Option<TraitSet>,
    /// Citation list.
    pub citations: Vec<Citation>,
    /// Xref list.
    pub xrefs: Vec<Xref>,
    /// Comment list.
    pub comments: Vec<Comment>,
}

/// Local struct for attributes based on `BaseAttribute`.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ObservedDataAttribute {
    /// base
    pub base: Option<BaseAttribute>,
    /// type
    pub r#type: ObservedDataAttributeType,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::observed_in::ObservedDataAttribute>
    for ObservedDataAttribute
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::observed_in::ObservedDataAttribute,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            base: value.base.map(TryInto::try_into).transpose()?,
            r#type: value.r#type.try_into()?,
        })
    }
}

/// Local enum for the observed data type.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum ObservedDataAttributeType {
    /// corresponds to "Description"
    Description,
    /// corresponds to "VariantAlleles"
    VariantAlleles,
    /// corresponds to "SubjectsWithVariant"
    SubjectsWithVariant,
    /// corresponds to "SubjectsWithDifferentCausativeVariant"
    SubjectsWithDifferentCausativeVariant,
    /// corresponds to "VariantChromosomes"
    VariantChromosomes,
    /// corresponds to "IndependentObservations"
    IndependentObservations,
    /// corresponds to "SingleHeterozygote"
    SingleHeterozygous,
    /// corresponds to "CompoundHeterozygote"
    CompoundHeterozygous,
    /// corresponds to "Homozygote"
    Homozygous,
    /// corresponds to "Hemizygote"
    Hemizygous,
    /// corresponds to "NumberMosaic"
    NumberMosaic,
    /// corresponds to "ObservedUnspecified"
    ObservedUnspecified,
    /// corresponds to "AlleleFrequency"
    AlleleFrequency,
    /// corresponds to "SecondaryFinding"
    SecondaryFinding,
    /// corresponds to "GenotypeAndMOIConsistent"
    GenotypeAndMoiConsistent,
    /// corresponds to "UnaffectedFamilyMemberWithCausativeVariant"
    UnaffectedFamilyMemberWithCausativeVariant,
    /// corresponds to "HetParentTransmitNormalAllele"
    HetParentTransmitNormalAllele,
    /// corresponds to "CosegregatingFamilies"
    CosegregatingFamilies,
    /// corresponds to "InformativeMeioses"
    InformativeMeioses,
    /// corresponds to "SampleLocalID"
    SampleLocalId,
    /// corresponds to "SampleVariantID"
    SampleVariantId,
    /// corresponds to "FamilyHistory"
    FamilyHistory,
    /// corresponds to "NumFamiliesWithVariant"
    NumFamiliesWithVariant,
    /// corresponds to "NumFamiliesWithSegregationObserved"
    NumFamiliesWithSegregationObserved,
    /// corresponds to "SegregationObserved"
    SegregationObserved,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type>
    for ObservedDataAttributeType
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::Description => {
                ObservedDataAttributeType::Description
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::VariantAlleles => {
                ObservedDataAttributeType::VariantAlleles
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::SubjectsWithVariant => {
                ObservedDataAttributeType::SubjectsWithVariant
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::SubjectsWithDifferentCausativeVariant => {
                ObservedDataAttributeType::SubjectsWithDifferentCausativeVariant
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::VariantChromosomes => {
                ObservedDataAttributeType::VariantChromosomes
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::IndependentObservations => {
                ObservedDataAttributeType::IndependentObservations
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::SingleHeterozygous => {
                ObservedDataAttributeType::SingleHeterozygous
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::CompoundHeterozygous => {
                ObservedDataAttributeType::CompoundHeterozygous
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::Homozygous => {
                ObservedDataAttributeType::Homozygous
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::Hemizygous => {
                ObservedDataAttributeType::Hemizygous
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::NumberMosaic => {
                ObservedDataAttributeType::NumberMosaic
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::ObservedUnspecified => {
                ObservedDataAttributeType::ObservedUnspecified
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::AlleleFrequency => {
                ObservedDataAttributeType::AlleleFrequency
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::SecondaryFinding => {
                ObservedDataAttributeType::SecondaryFinding
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::GenotypeAndMoiConsistent => {
                ObservedDataAttributeType::GenotypeAndMoiConsistent
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::UnaffectedFamilyMemberWithCausativeVariant => {
                ObservedDataAttributeType::UnaffectedFamilyMemberWithCausativeVariant
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::HetParentTransmitNormalAllele => {
                ObservedDataAttributeType::HetParentTransmitNormalAllele
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::CosegregatingFamilies => {
                ObservedDataAttributeType::CosegregatingFamilies
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::InformativeMeioses => {
                ObservedDataAttributeType::InformativeMeioses
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::SampleLocalId => {
                ObservedDataAttributeType::SampleLocalId
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::SampleVariantId => {
                ObservedDataAttributeType::SampleVariantId
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::FamilyHistory => {
                ObservedDataAttributeType::FamilyHistory
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::NumFamiliesWithVariant => {
                ObservedDataAttributeType::NumFamiliesWithVariant
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::NumFamiliesWithSegregationObserved => {
                ObservedDataAttributeType::NumFamiliesWithSegregationObserved
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::SegregationObserved => {
                ObservedDataAttributeType::SegregationObserved
            }
            _ =>  anyhow::bail!("Invalid observed_data_attribute::Type: {:?}", value)
        })
    }
}

/// This is an AttributeSet, there will be 1 attribute supported
/// by optional citations, xrefs and comment. There must be at least one
/// ObservedData Set, but can be any number. For each ObservedData set the
/// Attribute will be either decimal or string depending on type. The value will
/// be stored here, but decimals will be entered to the database as a string.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ObservedData {
    /// Attributes.
    pub attributes: Vec<ObservedDataAttribute>,
    /// Severity.
    pub severity: Option<Severity>,
    /// Citation list.
    pub citations: Vec<Citation>,
    /// Xref list.
    pub xrefs: Vec<Xref>,
    /// Comment list.
    pub comments: Vec<Comment>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::observed_in::ObservedData> for ObservedData {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::observed_in::ObservedData,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            attributes: value
                .attributes
                .into_iter()
                .map(ObservedDataAttribute::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            severity: value.severity.map(Severity::try_from).transpose()?,
            citations: value
                .citations
                .into_iter()
                .map(Citation::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            xrefs: value
                .xrefs
                .into_iter()
                .map(Xref::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            comments: value
                .comments
                .into_iter()
                .map(Comment::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

/// Local enum for the method type.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum MethodType {
    /// corresponds to "literature only"
    LiteratureOnly,
    /// corresponds to "reference population"
    ReferencePopulation,
    /// corresponds to "case-control"
    CaseControl,
    /// corresponds to "clinical testing"
    ClinicalTesting,
    /// corresponds to "in vitro"
    InVitro,
    /// corresponds to "in vivo"
    InVivo,
    /// corresponds to "inferred from source"
    InferredFromSource,
    /// corresponds to "research"
    Research,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::observed_in::MethodType> for MethodType {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::observed_in::MethodType,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::observed_in::MethodType::LiteratureOnly => {
                MethodType::LiteratureOnly
            }
            pbs::clinvar_data::clinvar_public::observed_in::MethodType::ReferencePopulation => {
                MethodType::ReferencePopulation
            }
            pbs::clinvar_data::clinvar_public::observed_in::MethodType::CaseControl => {
                MethodType::CaseControl
            }
            pbs::clinvar_data::clinvar_public::observed_in::MethodType::ClinicalTesting => {
                MethodType::ClinicalTesting
            }
            pbs::clinvar_data::clinvar_public::observed_in::MethodType::InVitro => {
                MethodType::InVitro
            }
            pbs::clinvar_data::clinvar_public::observed_in::MethodType::InVivo => {
                MethodType::InVivo
            }
            pbs::clinvar_data::clinvar_public::observed_in::MethodType::InferredFromSource => {
                MethodType::InferredFromSource
            }
            pbs::clinvar_data::clinvar_public::observed_in::MethodType::Research => {
                MethodType::Research
            }
            _ => anyhow::bail!("Invalid observed_in::MethodType: {:?}", value),
        })
    }
}

/// A clinical assertion as submitted (SCV record).
///
/// Corresponds to `MeasureTraitType` in XSD and `<ClinicalAssertion>` in XML
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinicalAssertion {
    /// The ClinVar submission ID.
    pub clinvar_submission_id: Option<ClinvarSubmissionId>,
    /// The ClinVar SCV accessions.
    pub clinvar_accession: Option<ClinvarAccession>,
    /// Optional list of additional submitters.
    pub additional_submitters: Vec<Submitter>,
    /// Record status.
    pub record_status: ClinicalAssertionRecordStatus,
    /// Replaces; mutually exclusive with replaceds
    pub replaces: Vec<String>,
    /// Replaced list; mutually exclusive with replaces
    pub replaceds: Vec<ClinicalAssertionRecordHistory>,
    /// SCV classification.
    pub classifications: Option<ClassificationScv>,
    /// The assertion.
    pub assertion: Assertion,
    /// Attributes.
    pub attributes: Vec<ClinicalAssertionAttributeSetElement>,
    /// Observed in.
    pub observed_ins: Vec<ObservedIn>,
    /// Allele in SCV; mutually exclusive with haplotype/genotype.
    pub simple_allele: Option<AlleleScv>,
    /// Haplotype in SCV; mutually exclusive with allele/genotype.
    pub haplotype: Option<HaplotypeScv>,
    /// Genotype in SCV; mutually exclusive with allele/haplotype.
    pub genotype: Option<GenotypeScv>,
    /// The trait set.
    pub trait_set: Option<TraitSet>,
    /// List of citations.
    pub citations: Vec<Citation>,
    /// Optional study name.
    pub study_name: Option<String>,
    /// Optional study description.
    pub study_description: Option<String>,
    /// List of comments.
    pub comments: Vec<Comment>,
    /// List of submissions.
    pub submission_names: Vec<String>,
    /// Date of creation.
    pub date_created: Option<chrono::DateTime<chrono::Utc>>,
    /// Date of creation.
    pub date_last_updated: Option<chrono::DateTime<chrono::Utc>>,
    /// Date of creation.
    pub submission_date: Option<chrono::DateTime<chrono::Utc>>,
    /// ID.
    pub id: Option<u64>,
    /// Whether it is an FDA recognized database.
    pub fda_recognized_database: Option<bool>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::ClinicalAssertion> for ClinicalAssertion {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::ClinicalAssertion,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            clinvar_submission_id: value.clinvar_submission_id.map(ClinvarSubmissionId::from),
            clinvar_accession: value
                .clinvar_accession
                .map(ClinvarAccession::try_from)
                .transpose()?,
            additional_submitters: value
                .additional_submitters
                .into_iter()
                .map(Submitter::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            record_status: value.record_status.try_into()?,
            replaces: value.replaces,
            replaceds: value
                .replaceds
                .into_iter()
                .map(ClinicalAssertionRecordHistory::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            classifications: value
                .classifications
                .map(ClassificationScv::try_from)
                .transpose()?,
            assertion: value.assertion.try_into()?,
            attributes: value
                .attributes
                .into_iter()
                .map(ClinicalAssertionAttributeSetElement::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            observed_ins: value
                .observed_ins
                .into_iter()
                .map(ObservedIn::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            simple_allele: value.simple_allele.map(AlleleScv::try_from).transpose()?,
            haplotype: value.haplotype.map(HaplotypeScv::try_from).transpose()?,
            genotype: value.genotype.map(GenotypeScv::try_from).transpose()?,
            trait_set: value.trait_set.map(TraitSet::try_from).transpose()?,
            citations: value
                .citations
                .into_iter()
                .map(Citation::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            study_name: value.study_name,
            study_description: value.study_description,
            comments: value
                .comments
                .into_iter()
                .map(Comment::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            submission_names: value.submission_names,
            date_created: value.date_created.map(|x| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(x.seconds, x.nanos as u32)
                    .unwrap_or_default()
            }),
            date_last_updated: value.date_last_updated.map(|x| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(x.seconds, x.nanos as u32)
                    .unwrap_or_default()
            }),
            submission_date: value.submission_date.map(|x| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(x.seconds, x.nanos as u32)
                    .unwrap_or_default()
            }),
            id: value.id,
            fda_recognized_database: value.fda_recognized_database,
        })
    }
}

/// Local type for ClinVarSubmissionID.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarSubmissionId {
    /// The identifier provided by the submitter to facilitate
    /// identification of records corresponding to their submissions. If not
    /// provided by a submitter, NCBI generates one. If provided by
    /// submitter, that is represented in localKeyIsSubmitted.
    pub local_key: String,
    /// Optional title.
    pub title: Option<String>,
    /// Optional indication whether local key has been submitted.
    pub local_key_is_submitted: Option<bool>,
    /// Optional assembly of submission.
    pub submitted_assembly: Option<String>,
}

impl From<pbs::clinvar_data::clinvar_public::clinical_assertion::ClinvarSubmissionId>
    for ClinvarSubmissionId
{
    fn from(
        value: pbs::clinvar_data::clinvar_public::clinical_assertion::ClinvarSubmissionId,
    ) -> Self {
        Self {
            local_key: value.local_key,
            title: value.title,
            local_key_is_submitted: value.local_key_is_submitted,
            submitted_assembly: value.submitted_assembly,
        }
    }
}

/// Local type for attribute set.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinicalAssertionAttributeSetElement {
    /// The base value.
    pub attribute: Option<BaseAttribute>,
    /// The type of the attribute.
    pub r#type: AttributeSetElementType,
    /// List of xrefs.
    pub xrefs: Vec<Xref>,
    /// List of citations.
    pub citations: Vec<Citation>,
    /// List of comments.
    pub comments: Vec<Comment>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::clinical_assertion::AttributeSetElement>
    for ClinicalAssertionAttributeSetElement
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::clinical_assertion::AttributeSetElement,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            attribute: value.attribute.map(BaseAttribute::try_from).transpose()?,
            r#type: AttributeSetElementType::try_from(
                pbs::clinvar_data::clinvar_public::clinical_assertion::attribute_set_element::Type::try_from(
                    value.r#type
                )?
            )?,
            xrefs: value
                .xrefs
                .into_iter()
                .map(Xref::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            citations: value
                .citations
                .into_iter()
                .map(Citation::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            comments: value
                .comments
                .into_iter()
                .map(Comment::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

/// Local enum for types.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum AttributeSetElementType {
    /// Corresponds to "ModeOfInheritance"
    ModeOfInheritance,
    /// Corresponds to "Penetrance"
    Penetrance,
    /// Corresponds to "AgeOfOnset"
    AgeOfOnset,
    /// Corresponds to "Severity"
    Severity,
    /// Corresponds to "ClassificationHistory"
    ClassificationHistory,
    /// Corresponds to "SeverityDescription"
    SeverityDescription,
    /// Corresponds to "AssertionMethod"
    AssertionMethod,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::clinical_assertion::attribute_set_element::Type>
    for AttributeSetElementType
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::clinical_assertion::attribute_set_element::Type,
    ) -> Result<Self, Self::Error> {
        match value {
            pbs::clinvar_data::clinvar_public::clinical_assertion::attribute_set_element::Type::ModeOfInheritance => Ok(Self::ModeOfInheritance),
            pbs::clinvar_data::clinvar_public::clinical_assertion::attribute_set_element::Type::Penetrance => Ok(Self::Penetrance),
            pbs::clinvar_data::clinvar_public::clinical_assertion::attribute_set_element::Type::AgeOfOnset => Ok(Self::AgeOfOnset),
            pbs::clinvar_data::clinvar_public::clinical_assertion::attribute_set_element::Type::Severity => Ok(Self::Severity),
            pbs::clinvar_data::clinvar_public::clinical_assertion::attribute_set_element::Type::ClassificationHistory => Ok(Self::ClassificationHistory),
            pbs::clinvar_data::clinvar_public::clinical_assertion::attribute_set_element::Type::SeverityDescription => Ok(Self::SeverityDescription),
            pbs::clinvar_data::clinvar_public::clinical_assertion::attribute_set_element::Type::AssertionMethod => Ok(Self::AssertionMethod),
            _ => anyhow::bail!("Invalid attribute_set_element::Type {:?}", value)
        }
    }
}

/// Local type for `ClinVarAccession`
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarAccession {
    /// Accession.
    pub accession: String,
    /// Version.
    pub version: i32,
    /// The submitter's identifier.
    pub submitter_identifiers: Option<SubmitterIdentifiers>,
    /// The date that the latest update to the submitted
    /// record (SCV) became public in ClinVar.
    pub date_updated: Option<chrono::DateTime<chrono::Utc>>,
    /// DateCreated is the date when the record first became
    /// public in ClinVar.
    pub date_created: Option<chrono::DateTime<chrono::Utc>>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::clinical_assertion::ClinvarAccession>
    for ClinvarAccession
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::clinical_assertion::ClinvarAccession,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            accession: value.accession,
            version: value.version,
            submitter_identifiers: value
                .submitter_identifiers
                .map(SubmitterIdentifiers::try_from)
                .transpose()?,
            date_updated: value
                .date_updated
                .map(chrono::DateTime::<chrono::Utc>::from),
            date_created: value
                .date_created
                .map(chrono::DateTime::<chrono::Utc>::from),
        })
    }
}

/// Local enum for record status.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum ClinicalAssertionRecordStatus {
    /// corresponds to "current"
    Current,
    /// corresponds to "replaced"
    Replaced,
    /// corresponds to "removed"
    Removed,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::clinical_assertion::RecordStatus>
    for ClinicalAssertionRecordStatus
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::clinical_assertion::RecordStatus,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::clinical_assertion::RecordStatus::Current => {
                Self::Current
            }
            pbs::clinvar_data::clinvar_public::clinical_assertion::RecordStatus::Replaced => {
                Self::Replaced
            }
            pbs::clinvar_data::clinvar_public::clinical_assertion::RecordStatus::Removed => {
                Self::Removed
            }
            _ => anyhow::bail!("Invalid clinical_assertion::RecordStatus: {:?}", value),
        })
    }
}

/// This is a record per variant (Measure/@ID,AlleleID).
///
/// Corresponds to "typeAllele" in XSD.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Allele {
    /// Gene list.
    pub genes: Vec<AlleleGene>,
    /// Name.
    pub name: String,
    /// Canonical SPDI.
    pub canonical_spdi: Option<String>,
    /// Variant type(s).
    pub variant_types: Vec<String>,
    /// Location.
    pub locations: Vec<Location>,
    /// List of other names.
    pub other_names: Vec<OtherName>,
    /// These are the single-letter representations of the protein change.
    pub protein_changes: Vec<String>,
    /// List of HGVS expressions.
    pub hgvs_expressions: Vec<HgvsExpression>,
    /// Aggregated classifications.
    pub classifications: Option<AggregateClassificationSet>,
    /// List of xrefs.
    pub xrefs: Vec<Xref>,
    /// List of comments.
    pub comments: Vec<Comment>,
    /// List of functional consequences.
    pub functional_consequences: Vec<FunctionalConsequence>,
    /// Allele frequencies.
    pub allele_frequencies: Vec<AlleleFrequency>,
    /// Global minor allele frequencies.
    pub global_minor_allele_frequency: Option<GlobalMinorAlleleFrequency>,
    /// Allele ID.
    pub allele_id: i64,
    /// Variation ID.
    pub variation_id: i64,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Allele> for Allele {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Allele) -> Result<Self, Self::Error> {
        Ok(Self {
            genes: value
                .genes
                .into_iter()
                .map(AlleleGene::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            name: value.name,
            canonical_spdi: value.canonical_spdi,
            variant_types: value.variant_types,
            locations: value
                .locations
                .into_iter()
                .map(Location::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            other_names: value
                .other_names
                .into_iter()
                .map(OtherName::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            protein_changes: value.protein_changes,
            hgvs_expressions: value
                .hgvs_expressions
                .into_iter()
                .map(HgvsExpression::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            classifications: value
                .classifications
                .map(AggregateClassificationSet::try_from)
                .transpose()?,
            xrefs: value
                .xrefs
                .into_iter()
                .map(Xref::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            comments: value
                .comments
                .into_iter()
                .map(Comment::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            functional_consequences: value
                .functional_consequences
                .into_iter()
                .map(FunctionalConsequence::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            allele_frequencies: value
                .allele_frequencies
                .into_iter()
                .map(AlleleFrequency::from)
                .collect(),
            global_minor_allele_frequency: value
                .global_minor_allele_frequency
                .map(GlobalMinorAlleleFrequency::from),
            allele_id: value.allele_id,
            variation_id: value.variation_id,
        })
    }
}

/// Local type for Gene.
///
/// nested elements
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AlleleGene {
    /// Gene's locations.
    pub locations: Vec<Location>,
    /// OMIM ID.
    pub omims: Vec<u64>,
    /// Haploinsuffiency.
    pub haploinsufficiency: Option<DosageSensitivity>,
    /// Triplosensitivity.
    pub triplosensitivity: Option<DosageSensitivity>,
    /// Used to set key words for retrieval or
    /// display about a gene, such as genes listed by the
    /// ACMG guidelines.
    pub properties: Vec<String>,
    /// Optional gene symbol.
    pub symbol: Option<String>,
    /// Full gene name.
    pub full_name: String,
    /// Gene ID.
    pub gene_id: i64,
    /// Optional HGNC ID.
    pub hgnc_id: Option<String>,
    /// Source of gene (calculated or submitted).
    pub source: String,
    /// Relationship between gene and variant.
    pub relationship_type: Option<GeneVariantRelationship>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::allele::Gene> for AlleleGene {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::allele::Gene,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            locations: value
                .locations
                .into_iter()
                .map(Location::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            omims: value.omims,
            haploinsufficiency: value
                .haploinsufficiency
                .map(DosageSensitivity::try_from)
                .transpose()?,
            triplosensitivity: value
                .triplosensitivity
                .map(DosageSensitivity::try_from)
                .transpose()?,
            properties: value.properties,
            symbol: value.symbol,
            full_name: value.full_name,
            gene_id: value.gene_id,
            hgnc_id: value.hgnc_id,
            source: value.source,
            relationship_type: value
                .relationship_type
                .map(|x| {
                    crate::server::run::clinvar_data::GeneVariantRelationship::try_from(
                        pbs::clinvar_data::clinvar_public::GeneVariantRelationship::try_from(x)?,
                    )
                })
                .transpose()?,
        })
    }
}
/// Local type for allele frequency.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AlleleFrequency {
    /// Value.
    pub value: f64,
    /// Source.
    pub source: String,
    /// URL.
    pub url: Option<String>,
}

impl From<pbs::clinvar_data::clinvar_public::allele::AlleleFrequency> for AlleleFrequency {
    fn from(value: pbs::clinvar_data::clinvar_public::allele::AlleleFrequency) -> Self {
        Self {
            value: value.value,
            source: value.source,
            url: value.url,
        }
    }
}
/// Local type for GlobalMinorAlleleFrequency.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct GlobalMinorAlleleFrequency {
    /// Value.
    pub value: f64,
    /// Source.
    pub source: String,
    /// Minor allele.
    pub minor_allele: Option<String>,
    /// URL.
    pub url: Option<String>,
}

impl From<pbs::clinvar_data::clinvar_public::allele::GlobalMinorAlleleFrequency>
    for GlobalMinorAlleleFrequency
{
    fn from(value: pbs::clinvar_data::clinvar_public::allele::GlobalMinorAlleleFrequency) -> Self {
        Self {
            value: value.value,
            source: value.source,
            minor_allele: value.minor_allele,
            url: value.url,
        }
    }
}

/// Local type for allele name.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AlleleName {
    /// The name's value.
    pub value: String,
    /// The name's type.
    pub r#type: Option<String>,
}

impl From<pbs::clinvar_data::clinvar_public::allele::Name> for AlleleName {
    fn from(value: pbs::clinvar_data::clinvar_public::allele::Name) -> Self {
        Self {
            value: value.value,
            r#type: value.r#type,
        }
    }
}
/// This is a record of one or more simple alleles on the same chromosome
/// molecule.
///
/// Corresponds to `typeHaplotype` in XSD
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Haplotype {
    /// The list of alleles in the haplotype.
    pub simple_alleles: Vec<Allele>,
    /// The preferred representation of the haplotype.
    pub name: String,
    /// The type of the haplotype.
    pub variation_type: VariationType,
    /// Names other than 'preferred' used for the haplotype.
    pub other_names: Vec<OtherName>,
    /// List of all the HGVS expressions valid for, or used to submit,
    /// a variant.
    pub hgvs_expressions: Vec<HgvsExpression>,
    /// Classifications of the variant.
    pub classifications: Option<AggregateClassificationSet>,
    /// Functional consequences of the variant.
    pub functional_consequences: Vec<FunctionalConsequence>,
    /// List of cross-references.
    pub xrefs: Vec<Xref>,
    /// List of comments.
    pub comments: Vec<Comment>,
    /// Variation ID.
    pub variation_id: i64,
    /// Number of copies.
    pub number_of_copies: Option<i32>,
    /// Number of chromosomes.
    pub number_of_chromosomes: Option<i32>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Haplotype> for Haplotype {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Haplotype) -> Result<Self, Self::Error> {
        Ok(Self {
            simple_alleles: value
                .simple_alleles
                .into_iter()
                .map(Allele::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            name: value.name,
            variation_type: VariationType::try_from(
                pbs::clinvar_data::clinvar_public::VariationType::try_from(value.variation_type)?,
            )?,
            other_names: value
                .other_names
                .into_iter()
                .map(OtherName::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            hgvs_expressions: value
                .hgvs_expressions
                .into_iter()
                .map(HgvsExpression::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            classifications: value
                .classifications
                .map(AggregateClassificationSet::try_from)
                .transpose()?,
            functional_consequences: value
                .functional_consequences
                .into_iter()
                .map(FunctionalConsequence::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            xrefs: value
                .xrefs
                .into_iter()
                .map(Xref::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            comments: value
                .comments
                .into_iter()
                .map(Comment::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            variation_id: value.variation_id,
            number_of_copies: value.number_of_copies,
            number_of_chromosomes: value.number_of_chromosomes,
        })
    }
}
/// This element is used for alleles that were not directly part of a
/// submission but were part of a complex submission. They have no direct submitted
/// classification, but are being reported for a complete representation of all alleles
/// in ClinVar. Compare to ClassifiedRecord.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct IncludedRecord {
    /// Simple allele; mutually exclusive with haplotype.
    pub simple_allele: Option<Allele>,
    /// Haplotype; mutually exclusive with simple_allele.
    pub haplotype: Option<Haplotype>,
    /// Aggregate classification sets.
    pub classifications: Option<AggregateClassificationSet>,
    /// List of submitted records.
    pub submitted_classifications: Vec<Scv>,
    /// Maintains the list of classified variants represented in
    /// this submission, although not submitted with an Classification
    /// independently.
    pub classified_variations: Vec<ClassifiedVariation>,
    /// List of general citations.
    pub general_citations: Vec<GeneralCitations>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::IncludedRecord> for IncludedRecord {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::IncludedRecord,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            simple_allele: value.simple_allele.map(Allele::try_from).transpose()?,
            haplotype: value.haplotype.map(Haplotype::try_from).transpose()?,
            classifications: value
                .classifications
                .map(AggregateClassificationSet::try_from)
                .transpose()?,
            submitted_classifications: value
                .submitted_classifications
                .into_iter()
                .map(Scv::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            classified_variations: value
                .classified_variations
                .into_iter()
                .map(ClassifiedVariation::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            general_citations: value
                .general_citations
                .into_iter()
                .map(GeneralCitations::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

/// Local type for tag `ClassifiedVariation`.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClassifiedVariation {
    /// Variation ID.
    pub variation_id: i64,
    /// Optional accession.
    pub accession: Option<String>,
    /// Version.
    pub version: i32,
}

impl From<pbs::clinvar_data::clinvar_public::included_record::ClassifiedVariation>
    for ClassifiedVariation
{
    fn from(
        value: pbs::clinvar_data::clinvar_public::included_record::ClassifiedVariation,
    ) -> Self {
        Self {
            variation_id: value.variation_id,
            accession: value.accession,
            version: value.version,
        }
    }
}

/// Used to report genotypes, be they simple or complex diplotypes.
///
/// Corresponds to "typeGenotype" in XSD.
///
/// nested elements
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Genotype {
    /// Simple allele; mutually exclusive with `haplotype`.
    pub simple_alleles: Vec<Allele>,
    /// Haplotype; mutually exclusive with `simple_allele`.
    ///
    /// Allows more than 2 haplotypes per genotype to support
    /// representation of ploidy.
    pub haplotypes: Vec<Haplotype>,
    /// Optional name.
    pub name: String,
    /// The variation type.
    pub variation_type: VariationType,
    /// Names other than 'preferred' used for the Genotype.
    pub other_names: Vec<OtherName>,
    /// HGVS descriptions.
    pub hgvs_expressions: Vec<HgvsExpression>,
    /// Functional consequences.
    pub functional_consequences: Vec<FunctionalConsequence>,
    /// Aggregated classifications.
    pub classifications: Option<AggregateClassificationSet>,
    /// List of xrefs.
    pub xrefs: Vec<Xref>,
    /// List of citations.
    pub citations: Vec<Citation>,
    /// List of comments.
    pub comments: Vec<Comment>,
    /// Attributes.
    pub attributes: Vec<AttributeSetElement>,
    /// Variation ID.
    pub variation_id: Option<i64>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Genotype> for Genotype {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Genotype) -> Result<Self, Self::Error> {
        Ok(Self {
            simple_alleles: value
                .simple_alleles
                .into_iter()
                .map(Allele::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            haplotypes: value
                .haplotypes
                .into_iter()
                .map(Haplotype::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            name: value.name,
            variation_type: VariationType::try_from(
                pbs::clinvar_data::clinvar_public::VariationType::try_from(value.variation_type)?,
            )?,
            other_names: value
                .other_names
                .into_iter()
                .map(OtherName::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            hgvs_expressions: value
                .hgvs_expressions
                .into_iter()
                .map(HgvsExpression::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            functional_consequences: value
                .functional_consequences
                .into_iter()
                .map(FunctionalConsequence::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            classifications: value
                .classifications
                .map(AggregateClassificationSet::try_from)
                .transpose()?,
            xrefs: value
                .xrefs
                .into_iter()
                .map(Xref::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            citations: value
                .citations
                .into_iter()
                .map(Citation::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            comments: value
                .comments
                .into_iter()
                .map(Comment::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            attributes: value
                .attributes
                .into_iter()
                .map(AttributeSetElement::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            variation_id: value.variation_id,
        })
    }
}
/// Corresponds to "typeRCV" in XSD.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct RcvAccession {
    /// The list of classified conditions.
    pub classified_condition_list: Option<RcvClassifiedConditionList>,
    /// The list of RCV classifications.
    pub rcv_classifications: Option<RcvClassifications>,
    /// The list of RCV accessions this record has replaced.
    pub replaceds: Vec<RecordHistory>,
    /// Optional title.
    pub title: Option<String>,
    /// Accession.
    pub accession: String,
    /// Version.
    pub version: i32,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::RcvAccession> for RcvAccession {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::RcvAccession,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            classified_condition_list: value
                .classified_condition_list
                .map(RcvClassifiedConditionList::try_from)
                .transpose()?,
            rcv_classifications: value
                .rcv_classifications
                .map(RcvClassifications::try_from)
                .transpose()?,
            replaceds: value
                .replaceds
                .into_iter()
                .map(RecordHistory::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            title: value.title,
            accession: value.accession,
            version: value.version,
        })
    }
}

/// Local type for ClassifiedConditionList.
///
/// nested elements
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct RcvClassifiedConditionList {
    /// List of interpreted conditions.
    pub classified_conditions: Vec<ClassifiedCondition>,
    /// Trait set ID.
    pub trait_set_id: Option<i64>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::rcv_accession::ClassifiedConditionList>
    for RcvClassifiedConditionList
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::rcv_accession::ClassifiedConditionList,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            classified_conditions: value
                .classified_conditions
                .into_iter()
                .map(ClassifiedCondition::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            trait_set_id: value.trait_set_id,
        })
    }
}

/// Local type for GermlineClassification.
///
/// The aggregate review status based on
/// all germline submissions for this record.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct RcvGermlineClassification {
    /// The aggregate review status based on
    /// all somatic clinical impact submissions for this
    /// record.
    pub review_status: AggregateGermlineReviewStatus,
    /// The oncogenicity description.
    pub description: Option<RcvGermlineClassificationDescription>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::rcv_accession::GermlineClassification>
    for RcvGermlineClassification
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::rcv_accession::GermlineClassification,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            review_status: AggregateGermlineReviewStatus::try_from(
                pbs::clinvar_data::clinvar_public::AggregateGermlineReviewStatus::try_from(
                    value.review_status,
                )?,
            )?,
            description: value
                .description
                .map(RcvGermlineClassificationDescription::try_from)
                .transpose()?,
        })
    }
}

/// Local type for Description.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct RcvGermlineClassificationDescription {
    /// The description.
    pub value: String,
    /// The date of the description.
    pub date_last_evaluated: Option<chrono::DateTime<chrono::Utc>>,
    /// The number of submissions.
    pub submission_count: Option<u32>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::rcv_accession::germline_classification::Description>
    for RcvGermlineClassificationDescription
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::rcv_accession::germline_classification::Description,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            value: value.value,
            date_last_evaluated: value
                .date_last_evaluated
                .map(chrono::DateTime::<chrono::Utc>::from),
            submission_count: value.submission_count,
        })
    }
}

/// Local type for SomaticClinicalImpact.
///
/// The aggregate review status based on
/// all somatic clinical impact submissions for this
/// record.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct RcvAccessionSomaticClinicalImpact {
    /// The aggregate review status based on
    /// all somatic clinical impact submissions for this
    /// record.
    pub review_status: AggregateSomaticClinicalImpactReviewStatus,
    /// The oncogenicity description.
    pub descriptions: Vec<RcvSomaticClinicalImpactDescription>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::rcv_accession::SomaticClinicalImpact>
    for RcvAccessionSomaticClinicalImpact
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::rcv_accession::SomaticClinicalImpact,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            review_status: AggregateSomaticClinicalImpactReviewStatus::try_from(
                pbs::clinvar_data::clinvar_public::AggregateSomaticClinicalImpactReviewStatus::try_from(
                    value.review_status
                )?
            )?,
            descriptions: value.descriptions.into_iter().map(RcvSomaticClinicalImpactDescription::try_from).collect::<Result<Vec<_>, _>>()?,
        })
    }
}

/// Local type for Description.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct RcvSomaticClinicalImpactDescription {
    /// The description.
    pub value: String,
    /// Clinical impact assertion type.
    pub clinical_impact_assertion_type: Option<String>,
    /// Clinical impact significance
    pub clinical_impact_clinical_significance: Option<String>,
    /// The date of the description.
    pub date_last_evaluated: Option<chrono::DateTime<chrono::Utc>>,
    /// The number of submissions.
    pub submission_count: Option<u32>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::rcv_accession::somatic_clinical_impact::Description>
    for RcvSomaticClinicalImpactDescription
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::rcv_accession::somatic_clinical_impact::Description,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            value: value.value,
            clinical_impact_assertion_type: value.clinical_impact_assertion_type,
            clinical_impact_clinical_significance: value.clinical_impact_clinical_significance,
            date_last_evaluated: value
                .date_last_evaluated
                .map(chrono::DateTime::<chrono::Utc>::from),
            submission_count: value.submission_count,
        })
    }
}

/// Local type for OncogenicityClassification.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct RcvOncogenicityClassification {
    /// The aggregate review status based on
    /// all oncogenic submissions for this record.
    pub review_status: AggregateGermlineReviewStatus,
    /// The oncogenicity description.
    pub description: Option<RcvOncogenicityDescription>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::rcv_accession::OncogenicityClassification>
    for RcvOncogenicityClassification
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::rcv_accession::OncogenicityClassification,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            review_status: AggregateGermlineReviewStatus::try_from(
                pbs::clinvar_data::clinvar_public::AggregateGermlineReviewStatus::try_from(
                    value.review_status,
                )?,
            )?,
            description: value
                .description
                .map(RcvOncogenicityDescription::try_from)
                .transpose()?,
        })
    }
}
/// Local type for Description.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct RcvOncogenicityDescription {
    /// The description.
    pub value: String,
    /// The date of the description.
    pub date_last_evaluated: Option<chrono::DateTime<chrono::Utc>>,
    /// The number of submissions.
    pub submission_count: Option<u32>,
}

impl
    TryFrom<
        pbs::clinvar_data::clinvar_public::rcv_accession::oncogenicity_classification::Description,
    > for RcvOncogenicityDescription
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::rcv_accession::oncogenicity_classification::Description,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            value: value.value,
            date_last_evaluated: value
                .date_last_evaluated
                .map(chrono::DateTime::<chrono::Utc>::from),
            submission_count: value.submission_count,
        })
    }
}

/// Local type for RCV classifications.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct RcvClassifications {
    /// Germline classification.
    pub germline_classification: Option<RcvGermlineClassification>,
    /// Somatic clinical impact.
    pub somatic_clinical_impact: Option<RcvAccessionSomaticClinicalImpact>,
    /// Oncogenicity classification.
    pub oncogenicity_classification: Option<RcvOncogenicityClassification>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::rcv_accession::RcvClassifications>
    for RcvClassifications
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::rcv_accession::RcvClassifications,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            germline_classification: value
                .germline_classification
                .map(RcvGermlineClassification::try_from)
                .transpose()?,
            somatic_clinical_impact: value
                .somatic_clinical_impact
                .map(RcvAccessionSomaticClinicalImpact::try_from)
                .transpose()?,
            oncogenicity_classification: value
                .oncogenicity_classification
                .map(RcvOncogenicityClassification::try_from)
                .transpose()?,
        })
    }
}

/// This element is restricted to variation records for which an explicit
/// classification was submitted.  Compare to IncludedRecord, which provides aggregate
/// information about variants that are part of another submission, but for which
/// ClinVar has *not* received a submission specific to that variant independently.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClassifiedRecord {
    /// Describes a single sequence change relative to a
    /// contiguous region of a chromosome or the mitochondrion.
    ///
    /// Mutually exclusive with `haplotype` and `genotype`.
    pub simple_allele: Option<Allele>,
    /// Describes multiple sequence changes on one of the
    /// chromosomes of a homologous pair or on the mitochondrion.
    ///
    /// Mutually exclusive with `simple_allele` and `genotype`.
    pub haplotype: Option<Haplotype>,
    /// Describes the combination of sequence changes on each
    /// chromosome of a homologous pair.
    ///
    /// Mutually exclusive with `simple_allele` and `haplotype`.
    pub genotype: Option<Genotype>,
    /// List of RCV records.
    pub rcv_list: Option<RcvList>,
    /// List of classifications.
    pub classifications: Option<AggregateClassificationSet>,
    /// List of clinical assertions.
    pub clinical_assertions: Vec<ClinicalAssertion>,
    /// This element is used to report how each user-submitted
    /// trait name was mapped to a MedGen CUI identifier and a preferred name.
    /// The structure may be used in the future to report, when a trait is
    /// identified by a source's identifier (e.g. MIM number), the preferred
    /// name used by that source at the time of submission. For MappingType
    /// XRef, MappingRef is the database name and MappingValue is the database's
    /// identifier. For MappingType Name, MappingRef is Alternate or Preferred,
    /// and MappingValue is the submitted name of the trait. ClinicalAssertionID
    /// is an integer identifier that corresponds 1:1 to the SCV assigned to the
    /// submission.
    pub trait_mappings: Vec<RcvTraitMapping>,
    /// List of deleted SCVs.
    pub deleted_scvs: Vec<DeletedScv>,
    /// List of general citations.
    pub general_citations: Vec<GeneralCitations>,
}

// XXX
/// Local type for tag `RCVList`.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct RcvList {
    /// The RCV record.
    pub rcv_accessions: Vec<RcvAccession>,
    /// The number of submissions (SCV accessions) referencing the VariationID.
    pub submission_count: Option<i32>,
    /// The number of idependent observations.
    pub independent_observations: Option<i32>,
}

impl From<pbs::clinvar_data::clinvar_public::classified_record::RcvList> for RcvList {
    fn from(value: pbs::clinvar_data::clinvar_public::classified_record::RcvList) -> Self {
        Self {
            rcv_accessions: value
                .rcv_accessions
                .into_iter()
                .map(RcvAccession::from)
                .collect(),
            submission_count: value.submission_count,
            independent_observations: value.independent_observations,
        }
    }
}

/// Local type for the tag `TraitMapping`.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct RcvTraitMapping {
    /// nested elements
    pub medgens: Vec<RcvTraitMappingMedgen>,
    /// ID of clinical assertion.
    pub clinical_assertion_id: i64,
    /// The trait type.
    pub trait_type: String,
    /// The mapping type.
    pub mapping_type: RcvTraitMappingType,
    /// The mapping value.
    pub mapping_value: String,
    /// The mapping reference.
    pub mapping_ref: String,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::classified_record::TraitMapping>
    for RcvTraitMapping
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::classified_record::TraitMapping,
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            medgens: value
                .medgens
                .into_iter()
                .map(RcvTraitMappingMedgen::from)
                .collect(),
            clinical_assertion_id: value.clinical_assertion_id,
            trait_type: value.trait_type,
            mapping_type: RcvTraitMappingType::try_from(
                pbs::clinvar_data::clinvar_public::classified_record::MappingType::try_from(
                    value.mapping_type,
                )?,
            )?,
            mapping_value: value.mapping_value,
            mapping_ref: value.mapping_ref,
        })
    }
}

/// Local type for the tag "MedGen"
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct RcvTraitMappingMedgen {
    /// Name.
    pub name: String,
    /// CUI.
    pub cui: String,
}

impl From<pbs::clinvar_data::clinvar_public::classified_record::trait_mapping::Medgen>
    for RcvTraitMappingMedgen
{
    fn from(
        value: pbs::clinvar_data::clinvar_public::classified_record::trait_mapping::Medgen,
    ) -> Self {
        Self {
            name: value.name,
            cui: value.cui,
        }
    }
}

/// Local type for the attribute `@MappingType`.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum RcvTraitMappingType {
    /// corresponds to "Name"
    Name,
    /// corresponds to "Xref"
    Xref,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::classified_record::MappingType>
    for RcvTraitMappingType
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::classified_record::MappingType,
    ) -> Result<Self, Self::Error> {
        match value {
            pbs::clinvar_data::clinvar_public::classified_record::MappingType::Name => {
                Ok(Self::Name)
            }
            pbs::clinvar_data::clinvar_public::classified_record::MappingType::Xref => {
                Ok(Self::Xref)
            }
            _ => Err(anyhow::anyhow!("Unknown value: {:?}", value)),
        }
    }
}
/// This element groups the set of data specific to a VariationArchive
/// record, namely the summary data of what has been submitted about a
/// VariationID AND for Classified records only, the content each
/// submission (SCV) provided.
///
/// Type for the `<VariationArchive>` type.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct VariationArchive {
    /// Numeric variation ID.
    pub variation_id: i64,
    /// This is ClinVar's name for the variant.  ClinVar uses this term in
    /// its web displays
    pub variation_name: String,
    /// Type of the variant.
    pub variation_type: String,
    /// DateCreated is the date when the record first became public in
    /// ClinVar.
    pub date_created: Option<chrono::DateTime<chrono::Utc>>,
    /// The date the record was last updated in the public database. The
    /// update may be a change to one of the submitted records (SCVs) or
    /// annotation added to the aggregate record by NCBI staff. This date
    /// is independent of a version change; annotated added by NCBI may
    /// change without representing a change in the version.
    pub date_last_updated: Option<chrono::DateTime<chrono::Utc>>,
    /// This date is of the most recent submitted record (SCV) for the
    /// VCV; it may reflect a new submitted record or an update to a submitted record.
    pub most_recent_submission: Option<chrono::DateTime<chrono::Utc>>,
    /// Accession assigned to the variant, or set of variants, that was
    /// Classified
    pub accession: String,
    /// Version of record and suffix for accession.
    pub version: i32,
    /// Number of submitters in record.
    pub number_of_submitters: i32,
    /// Number of submissions in record.
    pub number_of_submissions: i32,
    /// Record type.
    pub record_type: VariationArchiveRecordType,
    /// The record's status.
    pub record_status: VariationArchiveRecordStatus,
    /// Pointer to the replacing record; optional.
    pub replaced_by: Option<RecordHistory>,
    /// The list of VCV accessions this record has replaced.
    pub replaceds: Vec<RecordHistory>,
    /// Comment on the record; optional.
    pub comment: Option<Comment>,
    /// Specification of the species.
    pub species: Option<Species>,
    /// This element describes the classification of a single
    /// allele, haplotype, or genotype based on all submissions to ClinVar. This
    /// differs from the element IncludedRecord, which describes simple alleles
    /// or haplotypes, referenced in ClassifiedRecord, but for which no explicit
    /// classification was submitted. Once that variation is described, details
    /// are added about the phenotypes being classified, the classification, the
    /// submitters providing the classifications, and all supported evidence.
    ///
    /// NB: mutually exclusive with `included_record`.
    pub classified_record: Option<ClassifiedRecord>,
    /// This element describes a single allele or haplotype
    /// included in submissions to ClinVar, but for which no explicit
    /// classification was submitted. It also references the submissions and the
    /// Classified records that include them.
    ///
    /// NB: mutually exclusive with `classified_record`.
    pub included_record: Option<IncludedRecord>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::VariationArchive> for VariationArchive {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::VariationArchive,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            variation_id: value.variation_id,
            variation_name: value.variation_name,
            variation_type: value.variation_type,
            date_created: value
                .date_created
                .map(chrono::DateTime::<chrono::Utc>::from),
            date_last_updated: value
                .date_last_updated
                .map(chrono::DateTime::<chrono::Utc>::from),
            most_recent_submission: value
                .most_recent_submission
                .map(chrono::DateTime::<chrono::Utc>::from),
            accession: value.accession,
            version: value.version,
            number_of_submitters: value.number_of_submitters,
            number_of_submissions: value.number_of_submissions,
            record_type: value.record_type.try_into()?,
            record_status: value.record_status.try_into()?,
            replaced_by: value.replaced_by.map(TryInto::try_into).transpose()?,
            replaceds: value
                .replaceds
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            comment: value.comment.map(TryInto::try_into).transpose()?,
            species: value.species.map(TryInto::try_into).transpose()?,
            classified_record: value.classified_record.map(TryInto::try_into).transpose()?,
            included_record: value.included_record.map(TryInto::try_into).transpose()?,
        })
    }
}

/// Enumeration for `@RecordType`.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum VariationArchiveRecordType {
    /// corresponds to "included"
    Included,
    /// corresponds to "classified"
    Classified,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::variation_archive::RecordType>
    for VariationArchiveRecordType
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::variation_archive::RecordType,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::variation_archive::RecordType::Included => {
                VariationArchiveRecordType::Included
            }
            pbs::clinvar_data::clinvar_public::variation_archive::RecordType::Classified => {
                VariationArchiveRecordType::Classified
            }
            _ => anyhow::bail!("Unknown variation_archive::RecordType: {:?}", value),
        })
    }
}

/// Enumeration for `@RecordStatus`.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum VariationArchiveRecordStatus {
    /// corresponds to "current"
    Current,
    /// corresponds to "previous"
    Previous,
    /// corresponds to "replaced"
    Replaced,
    /// correspodns to "deleted"
    Deleted,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::variation_archive::RecordStatus>
    for VariationArchiveRecordStatus
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::variation_archive::RecordStatus,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::variation_archive::RecordStatus::Current => {
                VariationArchiveRecordStatus::Current
            }
            pbs::clinvar_data::clinvar_public::variation_archive::RecordStatus::Previous => {
                VariationArchiveRecordStatus::Previous
            }
            pbs::clinvar_data::clinvar_public::variation_archive::RecordStatus::Replaced => {
                VariationArchiveRecordStatus::Replaced
            }
            pbs::clinvar_data::clinvar_public::variation_archive::RecordStatus::Deleted => {
                VariationArchiveRecordStatus::Deleted
            }
            _ => anyhow::bail!("Unknown variation_archive::RecordStatus: {:?}", value),
        })
    }
}

/// The element to group each VariationArchive element in the release
///
/// Type for the `<ClinVarVariationRelease>` tag.
///
/// attributes
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarVariationRelease {
    /// The current release.
    pub release_date: Option<chrono::DateTime<chrono::Utc>>,
    /// List of `<VariationArchive>` tags.
    pub variation_archives: Vec<VariationArchive>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::ClinvarVariationRelease>
    for ClinvarVariationRelease
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::ClinvarVariationRelease,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            release_date: value
                .release_date
                .map(chrono::DateTime::<chrono::Utc>::from),
            variation_archives: value
                .variation_archives
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

/// Enumeration describing connection between genes and variants.
///
/// Corresponds to XSD type "GeneVariantRelationship".
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum GeneVariantRelationship {
    /// corresponds to "variant within gene"
    VariantWithinGene,
    /// corresponds to "gene overlapped by variant" and
    /// (legacy:) "genes overlapped by variant"
    GeneOverlappedByVariant,
    /// corresponds to "variant near gene, upstream" and
    /// (legacy:) "near gene, upstream"
    NearGeneUpstream,
    /// corresponds to "variant near gene, downstream" and
    /// (legacy:) "near gene, downstream"
    NearGeneDownstream,
    /// corresponds to "asserted, but not computed"
    AssertedButNotComputed,
    /// corresponds to "within multiple genes by overlap"
    WithinMultipleGenesByOverlap,
    /// corresponds to "within single gene"
    WithinSingleGene,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::GeneVariantRelationship>
    for GeneVariantRelationship
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::GeneVariantRelationship,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::GeneVariantRelationship::VariantWithinGene => {
                GeneVariantRelationship::VariantWithinGene
            }
            pbs::clinvar_data::clinvar_public::GeneVariantRelationship::GeneOverlappedByVariant => {
                GeneVariantRelationship::GeneOverlappedByVariant
            }
            pbs::clinvar_data::clinvar_public::GeneVariantRelationship::NearGeneUpstream => {
                GeneVariantRelationship::NearGeneUpstream
            }
            pbs::clinvar_data::clinvar_public::GeneVariantRelationship::NearGeneDownstream => {
                GeneVariantRelationship::NearGeneDownstream
            }
            pbs::clinvar_data::clinvar_public::GeneVariantRelationship::AssertedButNotComputed => {
                GeneVariantRelationship::AssertedButNotComputed
            }
            pbs::clinvar_data::clinvar_public::GeneVariantRelationship::WithinMultipleGenesByOverlap => {
                GeneVariantRelationship::WithinMultipleGenesByOverlap
            }
            pbs::clinvar_data::clinvar_public::GeneVariantRelationship::WithinSingleGene => {
                GeneVariantRelationship::WithinSingleGene
            }
            _ => anyhow::bail!("Unknown GeneVariantRelationship: {:?}", value),
        })
    }
}

/// Enumeration describing severity.
///
/// Corresponds to XSD type "typeSeverity"
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum Severity {
    /// corresponds to "mild"
    Mild,
    /// corresponds to "moderate"
    Moderate,
    /// corresponds to "sever"
    Severe,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Severity> for Severity {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Severity) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::Severity::Mild => Severity::Mild,
            pbs::clinvar_data::clinvar_public::Severity::Moderate => Severity::Moderate,
            pbs::clinvar_data::clinvar_public::Severity::Severe => Severity::Severe,
            _ => anyhow::bail!("Unknown Severity: {:?}", value),
        })
    }
}

/// Enumeration describing status.
///
/// Corresponds to `typeStatus` in XSD.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum Status {
    /// corresponds to "current"
    Current,
    /// corresponds to "completed and retired"
    CompletedAndRetired,
    /// corresponds to "delete"
    Delete,
    /// corresponds to "in development"
    InDevelopment,
    /// corresponds to "reclassified"
    Reclassified,
    /// corresponds to "reject"
    Reject,
    /// corresponds to "secondary"
    Secondary,
    /// corresponds to "suppressed"
    Suppressed,
    /// corresponds to "under review"
    UnderReview,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Status> for Status {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Status) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::Status::Current => Status::Current,
            pbs::clinvar_data::clinvar_public::Status::CompletedAndRetired => {
                Status::CompletedAndRetired
            }
            pbs::clinvar_data::clinvar_public::Status::Delete => Status::Delete,
            pbs::clinvar_data::clinvar_public::Status::InDevelopment => Status::InDevelopment,
            pbs::clinvar_data::clinvar_public::Status::Reclassified => Status::Reclassified,
            pbs::clinvar_data::clinvar_public::Status::Reject => Status::Reject,
            pbs::clinvar_data::clinvar_public::Status::Secondary => Status::Secondary,
            pbs::clinvar_data::clinvar_public::Status::Suppressed => Status::Suppressed,
            pbs::clinvar_data::clinvar_public::Status::UnderReview => Status::UnderReview,
            _ => anyhow::bail!("Unknown Status: {:?}", value),
        })
    }
}

/// Enumeration describing submitter review status.
///
/// Corresponds to `typeSubmitterReviewStatusValue` in XSD.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum SubmitterReviewStatus {
    /// corresponds to "no classification provided"
    NoClassificationProvided,
    /// corresponds to "no assertion criteria provided"
    NoAssertionCriteriaProvided,
    /// corresponds to "criteria provided, single submitter"
    CriteriaProvidedSingleSubmitter,
    /// corresponds to "reviewed by expert panel"
    ReviewedByExpertPanel,
    /// corresponds to "practice guideline"
    PracticeGuideline,
    /// corresponds to "flagged submission"
    FlaggedSubmission,
    /// corresponds to "criteria provided, multiple submitters, no conflicts"
    CriteriaProvidedMultipleSubmittersNoConflicts,
    /// corresponds to "criteria provided, conflicting classifications"
    CriteriaProvidedConflictingClassifications,
    /// corresponds to "classified by single submitter"
    ClassifiedBySingleSubmitter,
    /// corresponds to "reviewed by professional society"
    ReviewedByProfessionalSociety,
    /// corresponds to "not classified by submitter"
    NotClassifiedBySubmitter,
    /// corresponds to "classified by multiple submitters"
    ClassifiedByMultipleSubmitters,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::SubmitterReviewStatus> for SubmitterReviewStatus {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::SubmitterReviewStatus,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::SubmitterReviewStatus::NoClassificationProvided => SubmitterReviewStatus::NoClassificationProvided,
            pbs::clinvar_data::clinvar_public::SubmitterReviewStatus::NoAssertionCriteriaProvided => SubmitterReviewStatus::NoAssertionCriteriaProvided,
            pbs::clinvar_data::clinvar_public::SubmitterReviewStatus::CriteriaProvidedSingleSubmitter => SubmitterReviewStatus::CriteriaProvidedSingleSubmitter,
            pbs::clinvar_data::clinvar_public::SubmitterReviewStatus::ReviewedByExpertPanel => SubmitterReviewStatus::ReviewedByExpertPanel,
            pbs::clinvar_data::clinvar_public::SubmitterReviewStatus::PracticeGuideline => SubmitterReviewStatus::PracticeGuideline,
            pbs::clinvar_data::clinvar_public::SubmitterReviewStatus::FlaggedSubmission => SubmitterReviewStatus::FlaggedSubmission,
            pbs::clinvar_data::clinvar_public::SubmitterReviewStatus::CriteriaProvidedMultipleSubmittersNoConflicts => SubmitterReviewStatus::CriteriaProvidedMultipleSubmittersNoConflicts,
            pbs::clinvar_data::clinvar_public::SubmitterReviewStatus::CriteriaProvidedConflictingClassifications => SubmitterReviewStatus::CriteriaProvidedConflictingClassifications,
            pbs::clinvar_data::clinvar_public::SubmitterReviewStatus::ClassifiedBySingleSubmitter => SubmitterReviewStatus::ClassifiedBySingleSubmitter,
            pbs::clinvar_data::clinvar_public::SubmitterReviewStatus::ReviewedByProfessionalSociety => SubmitterReviewStatus::ReviewedByProfessionalSociety,
            pbs::clinvar_data::clinvar_public::SubmitterReviewStatus::NotClassifiedBySubmitter => SubmitterReviewStatus::NotClassifiedBySubmitter,
            pbs::clinvar_data::clinvar_public::SubmitterReviewStatus::ClassifiedByMultipleSubmitters => SubmitterReviewStatus::ClassifiedByMultipleSubmitters,
            _ => anyhow::bail!("Unknown SubmitterReviewStatus: {:?}", value),
        })
    }
}

/// Enumeration describing zygosity.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum Zygosity {
    /// corresponds to "Homozygote"
    Homozygote,
    /// corresponds to "SingleHeterozygote"
    SingleHeterozygote,
    /// corresponds to "CompoundHeterozygote"
    CompoundHeterozygote,
    /// corresponds to "Hemizygote"
    Hemizygote,
    /// corresponds to "not provided"
    NotProvided,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Zygosity> for Zygosity {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Zygosity) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::Zygosity::Homozygote => Zygosity::Homozygote,
            pbs::clinvar_data::clinvar_public::Zygosity::SingleHeterozygote => {
                Zygosity::SingleHeterozygote
            }
            pbs::clinvar_data::clinvar_public::Zygosity::CompoundHeterozygote => {
                Zygosity::CompoundHeterozygote
            }
            pbs::clinvar_data::clinvar_public::Zygosity::Hemizygote => Zygosity::Hemizygote,
            pbs::clinvar_data::clinvar_public::Zygosity::NotProvided => Zygosity::NotProvided,
            _ => anyhow::bail!("Unknown Zygosity: {:?}", value),
        })
    }
}

/// Enumeration describing assertion type attributes.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum Assertion {
    /// corresponds to "variation to disease"
    VariationToDisease,
    /// corresponds to "variation to included disease"
    VariationToIncludedDisease,
    /// corresponds to "variation in modifier gene to disease"
    VariationInModifierGeneToDisease,
    /// corresponds to "confers sensitivity"
    ConfersSensitivity,
    /// corresponds to "confers resistance"
    ConfersResistance,
    /// corresponds to "variant to named protein"
    VariantToNamedProtein,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Assertion> for Assertion {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Assertion) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::Assertion::VariationToDisease => {
                Assertion::VariationToDisease
            }
            pbs::clinvar_data::clinvar_public::Assertion::VariationToIncludedDisease => {
                Assertion::VariationToIncludedDisease
            }
            pbs::clinvar_data::clinvar_public::Assertion::VariationInModifierGeneToDisease => {
                Assertion::VariationInModifierGeneToDisease
            }
            pbs::clinvar_data::clinvar_public::Assertion::ConfersSensitivity => {
                Assertion::ConfersSensitivity
            }
            pbs::clinvar_data::clinvar_public::Assertion::ConfersResistance => {
                Assertion::ConfersResistance
            }
            pbs::clinvar_data::clinvar_public::Assertion::VariantToNamedProtein => {
                Assertion::VariantToNamedProtein
            }
            _ => anyhow::bail!("Unknown Assertion: {:?}", value),
        })
    }
}

/// Enumeration describing aggregate germline review status value.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum AggregateGermlineReviewStatus {
    /// corresponds to "no classification provided"
    NoClassificationProvided,
    /// corresponds to "no assertion criteria provided"
    NoAssertionCriteriaProvided,
    /// corresponds to "criteria provided, single submitter"
    CriteriaProvidedSingleSubmitter,
    /// corresponds to "criteria provided, multiple submitters, no conflicts"
    CriteriaProvidedMultipleSubmittersNoConflicts,
    /// corresponds to "criteria provided, conflicting classifications"
    CriteriaProvidedConflictingClassifications,
    /// corresponds to "reviewed by expert panel"
    ReviewedByExpertPanel,
    /// corresponds to "practice guideline"
    PracticeGuideline,
    /// corresponds to "no classifications from unflagged records"
    NoClassificationsFromUnflaggedRecords,
    /// corresponds to "no classification for the single variant"
    NoClassificationForTheSingleVariant,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::AggregateGermlineReviewStatus>
    for AggregateGermlineReviewStatus
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::AggregateGermlineReviewStatus,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::AggregateGermlineReviewStatus::NoClassificationProvided => {
                AggregateGermlineReviewStatus::NoClassificationProvided
            }
            pbs::clinvar_data::clinvar_public::AggregateGermlineReviewStatus::NoAssertionCriteriaProvided => {
                AggregateGermlineReviewStatus::NoAssertionCriteriaProvided
            }
            pbs::clinvar_data::clinvar_public::AggregateGermlineReviewStatus::CriteriaProvidedSingleSubmitter => {
                AggregateGermlineReviewStatus::CriteriaProvidedSingleSubmitter
            }
            pbs::clinvar_data::clinvar_public::AggregateGermlineReviewStatus::CriteriaProvidedMultipleSubmittersNoConflicts => {
                AggregateGermlineReviewStatus::CriteriaProvidedMultipleSubmittersNoConflicts
            }
            pbs::clinvar_data::clinvar_public::AggregateGermlineReviewStatus::CriteriaProvidedConflictingClassifications => {
                AggregateGermlineReviewStatus::CriteriaProvidedConflictingClassifications
            }
            pbs::clinvar_data::clinvar_public::AggregateGermlineReviewStatus::ReviewedByExpertPanel => {
                AggregateGermlineReviewStatus::ReviewedByExpertPanel
            }
            pbs::clinvar_data::clinvar_public::AggregateGermlineReviewStatus::PracticeGuideline => {
                AggregateGermlineReviewStatus::PracticeGuideline
            }
            pbs::clinvar_data::clinvar_public::AggregateGermlineReviewStatus::NoClassificationsFromUnflaggedRecords => {
                AggregateGermlineReviewStatus::NoClassificationsFromUnflaggedRecords
            }
            pbs::clinvar_data::clinvar_public::AggregateGermlineReviewStatus::NoClassificationForTheSingleVariant => {
                AggregateGermlineReviewStatus::NoClassificationForTheSingleVariant
            }
            _ => anyhow::bail!("Unknown AggregateGermlineReviewStatus: {:?}", value),
        })
    }
}

/// Enumeration describing aggregate somatic clinical impact review status value.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum AggregateSomaticClinicalImpactReviewStatus {
    /// corresponds to "no classification provided"
    NoClassificationProvided,
    /// corresponds to "no assertion criteria provided"
    NoAssertionCriteriaProvided,
    /// corresponds to "criteria provided, single submitter"
    CriteriaProvidedSingleSubmitter,
    /// corresponds to "criteria provided, multiple submitters"
    CriteriaProvidedMultipleSubmitters,
    /// corresponds to "reviewed by expert panel"
    ReviewedByExpertPanel,
    /// corresponds to "practice guideline"
    PracticeGuideline,
    /// corresponds to "no classifications from unflagged records"
    NoClassificationsFromUnflaggedRecords,
    /// corresponds to "no classification for the single variant"
    NoClassificationForTheSingleVariant,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::AggregateSomaticClinicalImpactReviewStatus>
    for AggregateSomaticClinicalImpactReviewStatus
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::AggregateSomaticClinicalImpactReviewStatus,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::AggregateSomaticClinicalImpactReviewStatus::NoClassificationProvided => {
                AggregateSomaticClinicalImpactReviewStatus::NoClassificationProvided
            }
            pbs::clinvar_data::clinvar_public::AggregateSomaticClinicalImpactReviewStatus::NoAssertionCriteriaProvided => {
                AggregateSomaticClinicalImpactReviewStatus::NoAssertionCriteriaProvided
            }
            pbs::clinvar_data::clinvar_public::AggregateSomaticClinicalImpactReviewStatus::CriteriaProvidedSingleSubmitter => {
                AggregateSomaticClinicalImpactReviewStatus::CriteriaProvidedSingleSubmitter
            }
            pbs::clinvar_data::clinvar_public::AggregateSomaticClinicalImpactReviewStatus::CriteriaProvidedMultipleSubmitters => {
                AggregateSomaticClinicalImpactReviewStatus::CriteriaProvidedMultipleSubmitters
            }
            pbs::clinvar_data::clinvar_public::AggregateSomaticClinicalImpactReviewStatus::ReviewedByExpertPanel => {
                AggregateSomaticClinicalImpactReviewStatus::ReviewedByExpertPanel
            }
            pbs::clinvar_data::clinvar_public::AggregateSomaticClinicalImpactReviewStatus::PracticeGuideline => {
                AggregateSomaticClinicalImpactReviewStatus::PracticeGuideline
            }
            pbs::clinvar_data::clinvar_public::AggregateSomaticClinicalImpactReviewStatus::NoClassificationsFromUnflaggedRecords => {
                AggregateSomaticClinicalImpactReviewStatus::NoClassificationsFromUnflaggedRecords
            }
            pbs::clinvar_data::clinvar_public::AggregateSomaticClinicalImpactReviewStatus::NoClassificationForTheSingleVariant => {
                AggregateSomaticClinicalImpactReviewStatus::NoClassificationForTheSingleVariant
            }
            _ => anyhow::bail!("Unknown AggregateSomaticClinicalImpactReviewStatus: {:?}", value),
        })
    }
}

/// Enumeration describing aggregate oncogenicity review status value.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum AggregateOncogenicityReviewStatus {
    /// corresponds to "no classification provided"
    NoClassificationProvided,
    /// corresponds to "no assertion criteria provided"
    NoAssertionCriteriaProvided,
    /// corresponds to "criteria provided, single submitter"
    CriteriaProvidedSingleSubmitter,
    /// corresponds to "criteria provided, multiple submitters, no conflicts"
    CriteriaProvidedMultipleSubmittersNoConflicts,
    /// corresponds to "criteria provided, conflicting classifications"
    CriteriaProvidedConflictingClassifications,
    /// corresponds to "reviewed by expert panel"
    ReviewedByExpertPanel,
    /// corresponds to "practice guideline"
    PracticeGuideline,
    /// corresponds to "no classifications from unflagged records"
    NoClassificationsFromUnflaggedRecords,
    /// corresponds to "no classification for the single variant"
    NoClassificationForTheSingleVariant,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::AggregateOncogenicityReviewStatus>
    for AggregateOncogenicityReviewStatus
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::AggregateOncogenicityReviewStatus,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::AggregateOncogenicityReviewStatus::NoClassificationProvided => {
                AggregateOncogenicityReviewStatus::NoClassificationProvided
            }
            pbs::clinvar_data::clinvar_public::AggregateOncogenicityReviewStatus::NoAssertionCriteriaProvided => {
                AggregateOncogenicityReviewStatus::NoAssertionCriteriaProvided
            }
            pbs::clinvar_data::clinvar_public::AggregateOncogenicityReviewStatus::CriteriaProvidedSingleSubmitter => {
                AggregateOncogenicityReviewStatus::CriteriaProvidedSingleSubmitter
            }
            pbs::clinvar_data::clinvar_public::AggregateOncogenicityReviewStatus::CriteriaProvidedMultipleSubmittersNoConflicts => {
                AggregateOncogenicityReviewStatus::CriteriaProvidedMultipleSubmittersNoConflicts
            }
            pbs::clinvar_data::clinvar_public::AggregateOncogenicityReviewStatus::CriteriaProvidedConflictingClassifications => {
                AggregateOncogenicityReviewStatus::CriteriaProvidedConflictingClassifications
            }
            pbs::clinvar_data::clinvar_public::AggregateOncogenicityReviewStatus::ReviewedByExpertPanel => {
                AggregateOncogenicityReviewStatus::ReviewedByExpertPanel
            }
            pbs::clinvar_data::clinvar_public::AggregateOncogenicityReviewStatus::PracticeGuideline => {
                AggregateOncogenicityReviewStatus::PracticeGuideline
            }
            pbs::clinvar_data::clinvar_public::AggregateOncogenicityReviewStatus::NoClassificationsFromUnflaggedRecords => {
                AggregateOncogenicityReviewStatus::NoClassificationsFromUnflaggedRecords
            }
            pbs::clinvar_data::clinvar_public::AggregateOncogenicityReviewStatus::NoClassificationForTheSingleVariant => {
                AggregateOncogenicityReviewStatus::NoClassificationForTheSingleVariant
            }
            _ => anyhow::bail!("Unknown AggregateGermlineReviewStatus: {:?}", value),
        })
    }
}

/// Enumeration describing origin.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum Origin {
    /// corresponds to "germline"
    Germline,
    /// corresponds to "somatic"
    Somatic,
    /// corresponds to "de novo"
    DeNovo,
    /// corresponds to "not provided"
    NotProvided,
    /// corresponds to "inherited"
    Inherited,
    /// corresponds to "maternal"
    Maternal,
    /// corresponds to "paternal"
    Paternal,
    /// corresponds to "uniparental"
    Uniparental,
    /// corresponds to "biparental"
    Biparental,
    /// corresponds to "not-reported"
    NotReported,
    /// corresponds to "tested-inconclusive"
    TestedInconclusive,
    /// corresponds to "unknown"
    Unknown,
    /// corresponds to "not applicable"
    NotApplicable,
    /// corresponds to "experimentally generated"
    ExperimentallyGenerated,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Origin> for Origin {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Origin) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::Origin::Germline => Origin::Germline,
            pbs::clinvar_data::clinvar_public::Origin::Somatic => Origin::Somatic,
            pbs::clinvar_data::clinvar_public::Origin::DeNovo => Origin::DeNovo,
            pbs::clinvar_data::clinvar_public::Origin::NotProvided => Origin::NotProvided,
            pbs::clinvar_data::clinvar_public::Origin::Inherited => Origin::Inherited,
            pbs::clinvar_data::clinvar_public::Origin::Maternal => Origin::Maternal,
            pbs::clinvar_data::clinvar_public::Origin::Paternal => Origin::Paternal,
            pbs::clinvar_data::clinvar_public::Origin::Uniparental => Origin::Uniparental,
            pbs::clinvar_data::clinvar_public::Origin::Biparental => Origin::Biparental,
            pbs::clinvar_data::clinvar_public::Origin::NotReported => Origin::NotReported,
            pbs::clinvar_data::clinvar_public::Origin::TestedInconclusive => {
                Origin::TestedInconclusive
            }
            pbs::clinvar_data::clinvar_public::Origin::Unknown => Origin::Unknown,
            pbs::clinvar_data::clinvar_public::Origin::NotApplicable => Origin::NotApplicable,
            pbs::clinvar_data::clinvar_public::Origin::ExperimentallyGenerated => {
                Origin::ExperimentallyGenerated
            }
            _ => anyhow::bail!("Unknown Origin: {:?}", value),
        })
    }
}

/// Enumeration describing chromosome.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum Chromosome {
    /// corresponds to "1"
    Chromosome1,
    /// corresponds to "2"
    Chromosome2,
    /// corresponds to "3"
    Chromosome3,
    /// corresponds to "4"
    Chromosome4,
    /// corresponds to "5"
    Chromosome5,
    /// corresponds to "6"
    Chromosome6,
    /// corresponds to "7"
    Chromosome7,
    /// corresponds to "8"
    Chromosome8,
    /// corresponds to "9"
    Chromosome9,
    /// corresponds to "10"
    Chromosome10,
    /// corresponds to "11"
    Chromosome11,
    /// corresponds to "12"
    Chromosome12,
    /// corresponds to "13"
    Chromosome13,
    /// corresponds to "14"
    Chromosome14,
    /// corresponds to "15"
    Chromosome15,
    /// corresponds to "16"
    Chromosome16,
    /// corresponds to "17"
    Chromosome17,
    /// corresponds to "18"
    Chromosome18,
    /// corresponds to "19"
    Chromosome19,
    /// corresponds to "20"
    Chromosome20,
    /// corresponds to "21"
    Chromosome21,
    /// corresponds to "22"
    Chromosome22,
    /// corresponds to "X"
    X,
    /// corresponds to "Y"
    Y,
    /// corresponds to "MT"
    Mt,
    /// corresponds to "PAR"
    Par,
    /// corresponds to "Un"
    Un,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Chromosome> for Chromosome {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Chromosome) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome1 => Chromosome::Chromosome1,
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome2 => Chromosome::Chromosome2,
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome3 => Chromosome::Chromosome3,
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome4 => Chromosome::Chromosome4,
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome5 => Chromosome::Chromosome5,
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome6 => Chromosome::Chromosome6,
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome7 => Chromosome::Chromosome7,
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome8 => Chromosome::Chromosome8,
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome9 => Chromosome::Chromosome9,
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome10 => Chromosome::Chromosome10,
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome11 => Chromosome::Chromosome11,
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome12 => Chromosome::Chromosome12,
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome13 => Chromosome::Chromosome13,
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome14 => Chromosome::Chromosome14,
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome15 => Chromosome::Chromosome15,
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome16 => Chromosome::Chromosome16,
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome17 => Chromosome::Chromosome17,
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome18 => Chromosome::Chromosome18,
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome19 => Chromosome::Chromosome19,
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome20 => Chromosome::Chromosome20,
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome21 => Chromosome::Chromosome21,
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome22 => Chromosome::Chromosome22,
            pbs::clinvar_data::clinvar_public::Chromosome::X => Chromosome::X,
            pbs::clinvar_data::clinvar_public::Chromosome::Y => Chromosome::Y,
            pbs::clinvar_data::clinvar_public::Chromosome::Mt => Chromosome::Mt,
            pbs::clinvar_data::clinvar_public::Chromosome::Par => Chromosome::Par,
            pbs::clinvar_data::clinvar_public::Chromosome::Un => Chromosome::Un,
            _ => anyhow::bail!("Unknown Chromosome: {:?}", value),
        })
    }
}

/// Enumeration describing comment type.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum CommentType {
    /// corresponds to "public"
    Public,
    /// corresponds to "ConvertedByNCBI"
    ConvertedByNcb,
    /// corresponds to "MissingFromAssembly"
    MissingFromAssembly,
    /// corresponds to "GenomicLocationNotEstablished"
    GenomicLocationNotEstablished,
    /// corresponds to "LocationOnGenomeAndProductNotAligned"
    LocationOnGenomeAndProductNotAligned,
    /// corresponds to "DeletionComment"
    DeletionComment,
    /// corresponds to "MergeComment"
    MergeComment,
    /// corresponds to "AssemblySpecificAlleleDefinition"
    AssemblySpecificAlleleDefinition,
    /// corresponds to "AlignmentGapMakesAppearInconsistent"
    AlignmentGapMakesAppearInconsistent,
    /// corresponds to "ExplanationOfClassification"
    ExplanationOfClassification,
    /// corresponds to "FlaggedComment"
    FlaggedComment,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::CommentType> for CommentType {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::CommentType,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::CommentType::Public => CommentType::Public,
            pbs::clinvar_data::clinvar_public::CommentType::ConvertedByNcb => CommentType::ConvertedByNcb,
            pbs::clinvar_data::clinvar_public::CommentType::MissingFromAssembly => CommentType::MissingFromAssembly,
            pbs::clinvar_data::clinvar_public::CommentType::GenomicLocationNotEstablished => {
                CommentType::GenomicLocationNotEstablished
            }
            pbs::clinvar_data::clinvar_public::CommentType::LocationOnGenomeAndProductNotAligned => {
                CommentType::LocationOnGenomeAndProductNotAligned
            }
            pbs::clinvar_data::clinvar_public::CommentType::DeletionComment => CommentType::DeletionComment,
            pbs::clinvar_data::clinvar_public::CommentType::MergeComment => CommentType::MergeComment,
            pbs::clinvar_data::clinvar_public::CommentType::AssemblySpecificAlleleDefinition => {
                CommentType::AssemblySpecificAlleleDefinition
            }
            pbs::clinvar_data::clinvar_public::CommentType::AlignmentGapMakesAppearInconsistent => {
                CommentType::AlignmentGapMakesAppearInconsistent
            }
            pbs::clinvar_data::clinvar_public::CommentType::ExplanationOfClassification => {
                CommentType::ExplanationOfClassification
            }
            pbs::clinvar_data::clinvar_public::CommentType::FlaggedComment => CommentType::FlaggedComment,
            _ => anyhow::bail!("Unknown CommentType: {:?}", value),
        })
    }
}

/// Enumeration describing nucleotide sequence.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum NucleotideSequence {
    /// corresponds to "genomic, top-level"
    GenomicTopLevel,
    /// corresponds to "genomic, RefSeqGene"
    GenomicRefSeqGene,
    /// corresponds to "genomic"
    Genomic,
    /// corresponds to "coding"
    Coding,
    /// corresponds to "non-coding"
    NonCoding,
    /// corresponds to "protein"
    Protein,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::NucleotideSequence> for NucleotideSequence {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::NucleotideSequence,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::NucleotideSequence::GenomicTopLevel => {
                NucleotideSequence::GenomicTopLevel
            }
            pbs::clinvar_data::clinvar_public::NucleotideSequence::GenomicRefSeqGene => {
                NucleotideSequence::GenomicRefSeqGene
            }
            pbs::clinvar_data::clinvar_public::NucleotideSequence::Genomic => {
                NucleotideSequence::Genomic
            }
            pbs::clinvar_data::clinvar_public::NucleotideSequence::Coding => {
                NucleotideSequence::Coding
            }
            pbs::clinvar_data::clinvar_public::NucleotideSequence::NonCoding => {
                NucleotideSequence::NonCoding
            }
            pbs::clinvar_data::clinvar_public::NucleotideSequence::Protein => {
                NucleotideSequence::Protein
            }
            _ => anyhow::bail!("Unknown NucleotideSequence: {:?}", value),
        })
    }
}

/// Enumeration describing protein sequence.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum ProteinSequence {
    /// corresponds to "protein"
    Protein,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::ProteinSequence> for ProteinSequence {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::ProteinSequence,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::ProteinSequence::Protein => ProteinSequence::Protein,
            _ => anyhow::bail!("Unknown ProteinSequence: {:?}", value),
        })
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum PhenotypeSetType {
    /// corresponds to "Disease"
    Disease,
    /// corresponds to "DrugResponse"
    DrugResponse,
    /// corresponds to "Finding"
    Finding,
    /// corresponds to "PhenotypeInstruction"
    PhenotypeInstruction,
    /// corresponds to "TraitChoice"
    TraitChoice,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::PhenotypeSetType> for PhenotypeSetType {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::PhenotypeSetType,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::PhenotypeSetType::Disease => {
                PhenotypeSetType::Disease
            }
            pbs::clinvar_data::clinvar_public::PhenotypeSetType::DrugResponse => {
                PhenotypeSetType::DrugResponse
            }
            pbs::clinvar_data::clinvar_public::PhenotypeSetType::Finding => {
                PhenotypeSetType::Finding
            }
            pbs::clinvar_data::clinvar_public::PhenotypeSetType::PhenotypeInstruction => {
                PhenotypeSetType::PhenotypeInstruction
            }
            pbs::clinvar_data::clinvar_public::PhenotypeSetType::TraitChoice => {
                PhenotypeSetType::TraitChoice
            }
            _ => anyhow::bail!("Unknown PhenotypeSetType: {:?}", value),
        })
    }
}

/// Enumeration describing variation type.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum VariationType {
    /// corresponds to "Diplotype"
    Diplotype,
    /// corresponds to "CompoundHeterozygote"
    CompoundHeterozygote,
    /// corresponds to "Distinct chromosomes"
    DistinctChromosomes,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::VariationType> for VariationType {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::VariationType,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::VariationType::Diplotype => VariationType::Diplotype,
            pbs::clinvar_data::clinvar_public::VariationType::CompoundHeterozygote => {
                VariationType::CompoundHeterozygote
            }
            pbs::clinvar_data::clinvar_public::VariationType::DistinctChromosomes => {
                VariationType::DistinctChromosomes
            }
            _ => anyhow::bail!("Unknown VariationType: {:?}", value),
        })
    }
}

/// Enumeration describing evidence type.
///
/// Corresponds to "EvidenceType" in XSD.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum EvidenceType {
    /// corresponds to "Genetic"
    Genetic,
    /// corresponds to "Experimental"
    Experimental,
    /// corresponds to "Population"
    Population,
    /// corresponds to "Computational"
    Computational,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::EvidenceType> for EvidenceType {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::EvidenceType,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::EvidenceType::Genetic => EvidenceType::Genetic,
            pbs::clinvar_data::clinvar_public::EvidenceType::Experimental => {
                EvidenceType::Experimental
            }
            pbs::clinvar_data::clinvar_public::EvidenceType::Population => EvidenceType::Population,
            pbs::clinvar_data::clinvar_public::EvidenceType::Computational => {
                EvidenceType::Computational
            }
            _ => anyhow::bail!("Unknown EvidenceType: {:?}", value),
        })
    }
}

/// Enumeration describing method list.
///
/// Corresponds to "MethodListType" in XSD.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum MethodListType {
    /// corresponds to "literature only"
    LiteratureOnly,
    /// corresponds to "reference population"
    ReferencePopulation,
    /// corresponds to "case-control"
    CaseControl,
    /// corresponds to "clinical testing"
    ClinicalTesting,
    /// corresponds to "in vitro"
    InVitro,
    /// corresponds to "in vivo"
    InVivo,
    /// corresponds to "research"
    Research,
    /// corresponds to "curation"
    Curation,
    /// corresponds to "not provided"
    NotProvided,
    /// corresponds to "provider interpretation"
    ProviderInterpretation,
    /// corresponds to "phenotyping only"
    PhenotypingOnly,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::MethodListType> for MethodListType {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::MethodListType,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::MethodListType::LiteratureOnly => {
                MethodListType::LiteratureOnly
            }
            pbs::clinvar_data::clinvar_public::MethodListType::ReferencePopulation => {
                MethodListType::ReferencePopulation
            }
            pbs::clinvar_data::clinvar_public::MethodListType::CaseControl => {
                MethodListType::CaseControl
            }
            pbs::clinvar_data::clinvar_public::MethodListType::ClinicalTesting => {
                MethodListType::ClinicalTesting
            }
            pbs::clinvar_data::clinvar_public::MethodListType::InVitro => MethodListType::InVitro,
            pbs::clinvar_data::clinvar_public::MethodListType::InVivo => MethodListType::InVivo,
            pbs::clinvar_data::clinvar_public::MethodListType::Research => MethodListType::Research,
            pbs::clinvar_data::clinvar_public::MethodListType::Curation => MethodListType::Curation,
            pbs::clinvar_data::clinvar_public::MethodListType::NotProvided => {
                MethodListType::NotProvided
            }
            pbs::clinvar_data::clinvar_public::MethodListType::ProviderInterpretation => {
                MethodListType::ProviderInterpretation
            }
            pbs::clinvar_data::clinvar_public::MethodListType::PhenotypingOnly => {
                MethodListType::PhenotypingOnly
            }
            _ => anyhow::bail!("Unknown MethodListType: {:?}", value),
        })
    }
}

/// Enumeration describing HGVS types.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum HgvsType {
    /// corresponds to "coding"
    Coding,
    /// corresponds to "genomic"
    Genomic,
    /// corresponds to "genomic, top-level"
    GenomicTopLevel,
    /// corresponds to "non-coding"
    NonCoding,
    /// corresponds to "protein"
    Protein,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::HgvsType> for HgvsType {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::HgvsType) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::HgvsType::Coding => HgvsType::Coding,
            pbs::clinvar_data::clinvar_public::HgvsType::Genomic => HgvsType::Genomic,
            pbs::clinvar_data::clinvar_public::HgvsType::GenomicTopLevel => {
                HgvsType::GenomicTopLevel
            }
            pbs::clinvar_data::clinvar_public::HgvsType::NonCoding => HgvsType::NonCoding,
            pbs::clinvar_data::clinvar_public::HgvsType::Protein => HgvsType::Protein,
            _ => anyhow::bail!("Unknown HgvsType: {:?}", value),
        })
    }
}

/// Enumeration describing clinical features affected status.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum ClinicalFeaturesAffectedStatusType {
    /// corresponds to "present"
    Present,
    /// corresponds to "absent"
    Absent,
    /// corresponds to "not tested"
    NotTested,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::ClinicalFeaturesAffectedStatusType>
    for ClinicalFeaturesAffectedStatusType
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::ClinicalFeaturesAffectedStatusType,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::ClinicalFeaturesAffectedStatusType::Present => {
                ClinicalFeaturesAffectedStatusType::Present
            }
            pbs::clinvar_data::clinvar_public::ClinicalFeaturesAffectedStatusType::Absent => {
                ClinicalFeaturesAffectedStatusType::Absent
            }
            pbs::clinvar_data::clinvar_public::ClinicalFeaturesAffectedStatusType::NotTested => {
                ClinicalFeaturesAffectedStatusType::NotTested
            }
            _ => anyhow::bail!("Unknown ClinicalFeaturesAffectedStatusType: {:?}", value),
        })
    }
}

/// Enumeration describing haplotype variation types.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum HaploVariationType {
    /// corresponds to "Haplotype"
    Haplotype,
    /// corresponds to "Haplotype, single variant"
    HaplotypeSingleVariant,
    /// corresponds to "Variation"
    Variation,
    /// corresponds to "Phase unknown"
    PhaseUnknown,
    /// corresponds to "Haplotype defined by a single variant"
    HaplotypeDefinedBySingleVariant,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::HaploVariationType> for HaploVariationType {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::HaploVariationType,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::HaploVariationType::Haplotype => HaploVariationType::Haplotype,
            pbs::clinvar_data::clinvar_public::HaploVariationType::HaplotypeSingleVariant => {
                HaploVariationType::HaplotypeSingleVariant
            }
            pbs::clinvar_data::clinvar_public::HaploVariationType::Variation => HaploVariationType::Variation,
            pbs::clinvar_data::clinvar_public::HaploVariationType::PhaseUnknown => HaploVariationType::PhaseUnknown,
            pbs::clinvar_data::clinvar_public::HaploVariationType::HaplotypeDefinedBySingleVariant => {
                HaploVariationType::HaplotypeDefinedBySingleVariant
            }
            _ => anyhow::bail!("Unknown HaploVariationType: {:?}", value),
        })
    }
}

/// Accession with version.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct VersionedAccession {
    /// The accession.
    pub accession: String,
    /// The version.
    pub version: i32,
}

impl From<pbs::clinvar_data::extracted_vars::VersionedAccession> for VersionedAccession {
    fn from(value: pbs::clinvar_data::extracted_vars::VersionedAccession) -> Self {
        Self {
            accession: value.accession,
            version: value.version,
        }
    }
}
/// Protocol buffer for storing essential information of one RCV.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ExtractedRcvRecord {
    /// The accession.
    pub accession: Option<VersionedAccession>,
    /// Title of RCV.
    pub title: String,
    /// Classifications (thinned out).
    pub classifications: Option<RcvClassifications>,
}

impl TryFrom<pbs::clinvar_data::extracted_vars::ExtractedRcvRecord> for ExtractedRcvRecord {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::extracted_vars::ExtractedRcvRecord,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            accession: value
                .accession
                .map(VersionedAccession::try_from)
                .transpose()?,
            title: value.title,
            classifications: value
                .classifications
                .map(RcvClassifications::try_from)
                .transpose()?,
        })
    }
}
/// Protocol buffer for storing essential information of one VCV.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ExtractedVcvRecord {
    /// The accession.
    pub accession: Option<VersionedAccession>,
    /// List of aggregated RCVs.
    pub rcvs: Vec<ExtractedRcvRecord>,
    /// Name of VCV.
    pub name: String,
    /// The type of the variant.
    pub variation_type: ExtractedVariationType,
    /// Classifications (thinned out).
    pub classifications: Option<AggregateClassificationSet>,
    /// Clinical assertions (thinned out),
    pub clinical_assertions: Vec<ClinicalAssertion>,
    /// The sequence location on one reference.
    pub sequence_location: Option<SequenceLocation>,
    /// List of HGNC IDs.
    pub hgnc_ids: Vec<String>,
}

impl TryFrom<pbs::clinvar_data::extracted_vars::ExtractedVcvRecord> for ExtractedVcvRecord {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::extracted_vars::ExtractedVcvRecord,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            accession: value
                .accession
                .map(VersionedAccession::try_from)
                .transpose()?,
            rcvs: value
                .rcvs
                .into_iter()
                .map(ExtractedRcvRecord::try_from)
                .collect::<Result<_, _>>()?,
            name: value.name,
            variation_type: ExtractedVariationType::try_from(
                pbs::clinvar_data::extracted_vars::VariationType::try_from(value.variation_type)?,
            )?,
            classifications: value
                .classifications
                .map(AggregateClassificationSet::try_from)
                .transpose()?,
            clinical_assertions: value
                .clinical_assertions
                .into_iter()
                .map(ClinicalAssertion::try_from)
                .collect::<Result<_, _>>()?,
            sequence_location: value
                .sequence_location
                .map(SequenceLocation::try_from)
                .transpose()?,
            hgnc_ids: value.hgnc_ids,
        })
    }
}

/// Enumeration for the type of the variant.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
pub enum ExtractedVariationType {
    /// Corresponds to "insertion".
    Insertion,
    /// Corresponds to "deletion".
    Deletion,
    /// Corresponds to "single nucleotide variant".
    Snv,
    /// Corresponds to "indel".
    Indel,
    /// Corresponds to "duplication".
    Duplication,
    /// Corresponds to "tandem duplication".
    TandemDuplication,
    /// Corresponds to "structural variant".
    StructuralVariant,
    /// Corresponds to "copy number gain".
    CopyNumberGain,
    /// Corresponds to "copy number loss".
    CopyNumberLoss,
    /// Corresponds to "protein only".
    ProteinOnly,
    /// Corresponds to "microsatellite".
    Microsatellite,
    /// Corresponds to "inversion".
    Inversion,
    /// Corresponds to "other".
    Other,
}

impl TryFrom<pbs::clinvar_data::extracted_vars::VariationType> for ExtractedVariationType {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::extracted_vars::VariationType,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::extracted_vars::VariationType::Insertion => {
                ExtractedVariationType::Insertion
            }
            pbs::clinvar_data::extracted_vars::VariationType::Deletion => {
                ExtractedVariationType::Deletion
            }
            pbs::clinvar_data::extracted_vars::VariationType::Snv => ExtractedVariationType::Snv,
            pbs::clinvar_data::extracted_vars::VariationType::Indel => {
                ExtractedVariationType::Indel
            }
            pbs::clinvar_data::extracted_vars::VariationType::Duplication => {
                ExtractedVariationType::Duplication
            }
            pbs::clinvar_data::extracted_vars::VariationType::TandemDuplication => {
                ExtractedVariationType::TandemDuplication
            }
            pbs::clinvar_data::extracted_vars::VariationType::StructuralVariant => {
                ExtractedVariationType::StructuralVariant
            }
            pbs::clinvar_data::extracted_vars::VariationType::CopyNumberGain => {
                ExtractedVariationType::CopyNumberGain
            }
            pbs::clinvar_data::extracted_vars::VariationType::CopyNumberLoss => {
                ExtractedVariationType::CopyNumberLoss
            }
            pbs::clinvar_data::extracted_vars::VariationType::ProteinOnly => {
                ExtractedVariationType::ProteinOnly
            }
            pbs::clinvar_data::extracted_vars::VariationType::Microsatellite => {
                ExtractedVariationType::Microsatellite
            }
            pbs::clinvar_data::extracted_vars::VariationType::Inversion => {
                ExtractedVariationType::Inversion
            }
            pbs::clinvar_data::extracted_vars::VariationType::Other => {
                ExtractedVariationType::Other
            }
            _ => anyhow::bail!("Invalid variation type {:?}", value),
        })
    }
}
