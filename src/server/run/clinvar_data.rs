//! Data structures for serde/Utoipa corresponding to the ones from clinvar protobufs.

use crate::pbs;

/// A structure to support reporting unformatted content, with type and
/// source specified.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarComment {
    /// The comment's value.
    pub value: String,
    /// The optional comment data source.
    pub data_source: Option<String>,
    /// The comment's type.
    pub r#type: Option<ClinvarCommentType>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Comment> for ClinvarComment {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Comment) -> Result<Self, Self::Error> {
        Ok(Self {
            value: value.value,
            data_source: value.data_source,
            r#type: value
                .r#type
                .map(|t| {
                    ClinvarCommentType::try_from(
                        pbs::clinvar_data::clinvar_public::CommentType::try_from(t)?,
                    )
                })
                .transpose()?,
        })
    }
}

/// This structure is used to represent how an object described in the
/// submission relates to objects in other databases.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarXref {
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
    pub status: Option<ClinvarStatus>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Xref> for ClinvarXref {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Xref) -> Result<Self, Self::Error> {
        Ok(ClinvarXref {
            db: value.db,
            id: value.id,
            r#type: value.r#type,
            url: value.url,
            status: value
                .status
                .map(|status| {
                    ClinvarStatus::try_from(pbs::clinvar_data::clinvar_public::Status::try_from(
                        status,
                    )?)
                })
                .transpose()?,
        })
    }
}

/// Description of a citation.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarCitation {
    /// Optional list of IDs.
    pub ids: Vec<ClinvarIdType>,
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

impl TryFrom<pbs::clinvar_data::clinvar_public::Citation> for ClinvarCitation {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Citation) -> Result<Self, Self::Error> {
        Ok(ClinvarCitation {
            ids: value
                .ids
                .into_iter()
                .map(ClinvarIdType::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            url: value.url,
            citation_text: value.citation_text,
            r#type: value.r#type,
            abbrev: value.abbrev,
        })
    }
}

/// Local ID with source.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarIdType {
    /// The citation's value.
    pub value: String,
    /// If there is an identifier, what database provides it.
    pub source: String,
}

impl From<pbs::clinvar_data::clinvar_public::citation::IdType> for ClinvarIdType {
    fn from(value: pbs::clinvar_data::clinvar_public::citation::IdType) -> Self {
        ClinvarIdType {
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
pub struct ClinvarBaseAttribute {
    /// The attribute's value; can be empty.
    pub value: Option<String>,
    /// The optional integer value.
    pub integer_value: Option<i64>,
    /// The optional date value.
    pub date_value: Option<chrono::DateTime<chrono::Utc>>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::BaseAttribute> for ClinvarBaseAttribute {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::BaseAttribute,
    ) -> Result<Self, Self::Error> {
        Ok(ClinvarBaseAttribute {
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
pub struct ClinvarHgvsNucleotideExpression {
    /// The expression values.
    pub expression: String,
    /// The type of the nucleotide sequence.
    pub sequence_type: Option<ClinvarNucleotideSequence>,
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
    for ClinvarHgvsNucleotideExpression
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::HgvsNucleotideExpression,
    ) -> Result<Self, Self::Error> {
        Ok(ClinvarHgvsNucleotideExpression {
            expression: value.expression,
            sequence_type: value
                .sequence_type
                .map(|sequence_type| {
                    ClinvarNucleotideSequence::try_from(
                        pbs::clinvar_data::clinvar_public::NucleotideSequence::try_from(
                            sequence_type,
                        )?,
                    )
                })
                .transpose()?,
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
pub struct ClinvarHgvsProteinExpression {
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

impl TryFrom<pbs::clinvar_data::clinvar_public::HgvsProteinExpression>
    for ClinvarHgvsProteinExpression
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::HgvsProteinExpression,
    ) -> Result<Self, Self::Error> {
        Ok(ClinvarHgvsProteinExpression {
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
pub struct ClinvarHgvsExpression {
    /// Optional nucleotide sequence expression.
    pub nucleotide_expression: Option<ClinvarHgvsNucleotideExpression>,
    /// Optional protein sequence expression.
    pub protein_expression: Option<ClinvarHgvsProteinExpression>,
    /// List of molecular consequences.
    pub molecular_consequences: Vec<ClinvarXref>,
    /// Type of HGVS expression.
    pub r#type: ClinvarHgvsType,
    /// Optional assembly.
    pub assembly: Option<String>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::HgvsExpression> for ClinvarHgvsExpression {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::HgvsExpression,
    ) -> Result<Self, Self::Error> {
        Ok(ClinvarHgvsExpression {
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
            r#type: ClinvarHgvsType::try_from(
                pbs::clinvar_data::clinvar_public::HgvsType::try_from(value.r#type)?,
            )?,
            assembly: value.assembly,
        })
    }
}

/// Description of a software.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarSoftware {
    /// Name of the software.
    pub name: String,
    /// Version of the software; optional.
    pub version: Option<String>,
    /// Purpose of the software; optional.
    pub purpose: Option<String>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Software> for ClinvarSoftware {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Software) -> Result<Self, Self::Error> {
        Ok(ClinvarSoftware {
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
pub struct ClinvarDescriptionHistory {
    /// The pathogenicity description.
    pub description: String,
    /// The date of the description.
    pub dated: Option<chrono::DateTime<chrono::Utc>>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::DescriptionHistory> for ClinvarDescriptionHistory {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::DescriptionHistory,
    ) -> Result<Self, Self::Error> {
        Ok(ClinvarDescriptionHistory {
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
pub struct ClinvarGenericSetElement {
    /// The element's value.
    pub value: String,
    /// The element's type.
    pub r#type: String,
    /// List of citations.
    pub citations: Vec<ClinvarCitation>,
    /// List of xrefs.
    pub xrefs: Vec<ClinvarXref>,
    /// List of comments.
    pub comments: Vec<ClinvarComment>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::GenericSetElement> for ClinvarGenericSetElement {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::GenericSetElement,
    ) -> Result<Self, Self::Error> {
        Ok(ClinvarGenericSetElement {
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
pub struct ClinvarAttributeSetElement {
    /// The attribute value.
    pub attribute: Option<ClinvarAttribute>,
    /// List of xrefs.
    pub xrefs: Vec<ClinvarXref>,
    /// List of citations.
    pub citations: Vec<ClinvarCitation>,
    /// List of comments.
    pub comments: Vec<ClinvarComment>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::AttributeSetElement>
    for ClinvarAttributeSetElement
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::AttributeSetElement,
    ) -> Result<Self, Self::Error> {
        Ok(ClinvarAttributeSetElement {
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
pub struct ClinvarAttribute {
    /// The base value.
    pub base: Option<ClinvarBaseAttribute>,
    /// The type of the attribute.
    pub r#type: String,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::attribute_set_element::Attribute>
    for ClinvarAttribute
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::attribute_set_element::Attribute,
    ) -> Result<Self, Self::Error> {
        Ok(ClinvarAttribute {
            base: value.base.map(|x| x.try_into()).transpose()?,
            r#type: value.r#type,
        })
    }
}

/// Type to describe traits in various places.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarTrait {
    /// names
    pub names: Vec<ClinvarGenericSetElement>,
    /// symbols
    pub symbols: Vec<ClinvarGenericSetElement>,
    /// attributes
    pub attributes: Vec<ClinvarAttributeSetElement>,
    /// Trait relationships
    pub trait_relationships: Vec<ClinvarTraitRelationship>,
    /// Citation list.
    pub citations: Vec<ClinvarCitation>,
    /// Xref list.
    pub xrefs: Vec<ClinvarXref>,
    /// Comment list.
    pub comments: Vec<ClinvarComment>,
    /// Sources
    pub sources: Vec<String>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Trait> for ClinvarTrait {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Trait) -> Result<Self, Self::Error> {
        Ok(ClinvarTrait {
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
pub struct ClinvarTraitRelationship {
    /// Name(s) of the trait.
    pub names: Vec<ClinvarGenericSetElement>,
    /// Citation list.
    pub citations: Vec<ClinvarCitation>,
    /// Xref list.
    pub xrefs: Vec<ClinvarXref>,
    /// Comment list.
    pub comments: Vec<ClinvarComment>,
    /// Sources
    pub sources: Vec<String>,
    /// Trait type.
    pub r#type: ClinvarTraitRelationshipType,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::r#trait::TraitRelationship>
    for ClinvarTraitRelationship
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::r#trait::TraitRelationship,
    ) -> Result<Self, Self::Error> {
        Ok(ClinvarTraitRelationship {
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
            r#type: ClinvarTraitRelationshipType::try_from(
                pbs::clinvar_data::clinvar_public::r#trait::r#trait_relationship::Type::try_from(
                    value.r#type,
                )?,
            )?,
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarTraitRelationshipType {
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
    for ClinvarTraitRelationshipType
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::r#trait::r#trait_relationship::Type,
    ) -> Result<Self, Self::Error> {
        match value {
            pbs::clinvar_data::clinvar_public::r#trait::r#trait_relationship::Type::Phenotype => {
                Ok(ClinvarTraitRelationshipType::Phenotype)
            }
            pbs::clinvar_data::clinvar_public::r#trait::r#trait_relationship::Type::Subphenotype => {
                Ok(ClinvarTraitRelationshipType::Subphenotype)
            }
            pbs::clinvar_data::clinvar_public::r#trait::r#trait_relationship::Type::DrugResponseAndDisease => {
                Ok(ClinvarTraitRelationshipType::DrugResponseAndDisease)
            }
            pbs::clinvar_data::clinvar_public::r#trait::r#trait_relationship::Type::CoOccuringCondition => {
                Ok(ClinvarTraitRelationshipType::CoOccuringCondition)
            }
            pbs::clinvar_data::clinvar_public::r#trait::r#trait_relationship::Type::FindingMember => {
                Ok(ClinvarTraitRelationshipType::FindingMember)
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
pub struct ClinvarIndication {
    /// Represents the value for the test indication as a name of a trait.
    pub traits: Vec<ClinvarTrait>,
    /// List of names.
    pub names: Vec<ClinvarGenericSetElement>,
    /// List of attributes.
    pub attributes: Vec<ClinvarAttributeSetElement>,
    /// List of xrefs.
    pub xrefs: Vec<ClinvarXref>,
    /// List of citations.
    pub citations: Vec<ClinvarCitation>,
    /// List of comments.
    pub comments: Vec<ClinvarComment>,
    /// The type of indication.
    pub r#type: ClinvarIndicationType,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Indication> for ClinvarIndication {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Indication) -> Result<Self, Self::Error> {
        Ok(ClinvarIndication {
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
            r#type: ClinvarIndicationType::try_from(
                pbs::clinvar_data::clinvar_public::indication::Type::try_from(value.r#type)?,
            )?,
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarIndicationType {
    /// corresponds to "Indication"
    Indication,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::indication::Type> for ClinvarIndicationType {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::indication::Type,
    ) -> Result<Self, Self::Error> {
        match value {
            pbs::clinvar_data::clinvar_public::indication::Type::Indication => {
                Ok(ClinvarIndicationType::Indication)
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
pub struct ClinvarTraitSet {
    /// The traits.
    pub traits: Vec<ClinvarTrait>,
    /// The names.
    pub names: Vec<ClinvarGenericSetElement>,
    /// The symbols.
    pub symbols: Vec<ClinvarGenericSetElement>,
    /// The attributes.
    pub attributes: Vec<ClinvarAttributeSetElement>,
    /// List of xrefs.
    pub xrefs: Vec<ClinvarXref>,
    /// List of citations.
    pub citations: Vec<ClinvarCitation>,
    /// List of comments.
    pub comments: Vec<ClinvarComment>,
    /// The type.
    pub r#type: ClinvarTraitSetType,
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

impl TryFrom<pbs::clinvar_data::clinvar_public::TraitSet> for ClinvarTraitSet {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::TraitSet) -> Result<Self, Self::Error> {
        Ok(ClinvarTraitSet {
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
            r#type: ClinvarTraitSetType::try_from(
                pbs::clinvar_data::clinvar_public::trait_set::Type::try_from(value.r#type)?,
            )?,
            date_last_evaluated: value.date_last_evaluated.map(|x| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(x.seconds, x.nanos as u32)
                    .unwrap_or_default()
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarTraitSetType {
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

impl TryFrom<pbs::clinvar_data::clinvar_public::trait_set::Type> for ClinvarTraitSetType {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::trait_set::Type,
    ) -> Result<Self, Self::Error> {
        match value {
            pbs::clinvar_data::clinvar_public::trait_set::Type::Disease => {
                Ok(ClinvarTraitSetType::Disease)
            }
            pbs::clinvar_data::clinvar_public::trait_set::Type::DrugResponse => {
                Ok(ClinvarTraitSetType::DrugResponse)
            }
            pbs::clinvar_data::clinvar_public::trait_set::Type::Finding => {
                Ok(ClinvarTraitSetType::Finding)
            }
            pbs::clinvar_data::clinvar_public::trait_set::Type::PhenotypeInstruction => {
                Ok(ClinvarTraitSetType::PhenotypeInstruction)
            }
            pbs::clinvar_data::clinvar_public::trait_set::Type::TraitChoice => {
                Ok(ClinvarTraitSetType::TraitChoice)
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
pub struct ClinvarAggregatedGermlineClassification {
    /// The aggregate review status based on all germline submissions
    /// for this record.
    pub review_status: ClinvarAggregateGermlineReviewStatus,
    /// We are not providing an enumeration for the values we report
    /// for germline classification within the xsd. Details are in
    /// <https://github.com/ncbi/clinvar/ClassificationOnClinVar.md>
    ///
    pub description: Option<String>,
    /// Explanation is used only when the description is 'conflicting
    /// data from submitters' The element summarizes the conflict.
    pub explanation: Option<ClinvarComment>,
    /// List of xrefs.
    pub xrefs: Vec<ClinvarXref>,
    /// List of citations.
    pub citations: Vec<ClinvarCitation>,
    /// List of comments.
    pub comments: Vec<ClinvarComment>,
    /// History information.
    pub history_records: Vec<ClinvarDescriptionHistory>,
    /// List of conditions.
    pub conditions: Vec<ClinvarTraitSet>,
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
    for ClinvarAggregatedGermlineClassification
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::AggregatedGermlineClassification,
    ) -> Result<Self, Self::Error> {
        Ok(ClinvarAggregatedGermlineClassification {
            review_status: ClinvarAggregateGermlineReviewStatus::try_from(
                pbs::clinvar_data::clinvar_public::AggregateGermlineReviewStatus::try_from(
                    value.review_status,
                )?,
            )?,
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
pub struct ClinvarAggregatedSomaticClinicalImpact {
    /// The aggregate review status based on all somatic clinical
    /// impact submissions for this record.
    pub review_status: ClinvarAggregateSomaticClinicalImpactReviewStatus,
    /// We are not providing an enumeration for the values we report
    /// for somatic clinical impact classification within the xsd. Details are in
    /// <https://github.com/ncbi/clinvar/ClassificationOnClinVar.md>
    pub description: Option<String>,
    /// List of xrefs.
    pub xrefs: Vec<ClinvarXref>,
    /// List of citations.
    pub citations: Vec<ClinvarCitation>,
    /// List of comments.
    pub comments: Vec<ClinvarComment>,
    /// History information.
    pub history_records: Vec<ClinvarDescriptionHistory>,
    /// List of conditions.
    pub conditions: Vec<ClinvarTraitSet>,
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
    for ClinvarAggregatedSomaticClinicalImpact
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::AggregatedSomaticClinicalImpact,
    ) -> Result<Self, Self::Error> {
        Ok(ClinvarAggregatedSomaticClinicalImpact {
            review_status: ClinvarAggregateSomaticClinicalImpactReviewStatus::try_from(
                pbs::clinvar_data::clinvar_public::AggregateSomaticClinicalImpactReviewStatus::try_from(
                    value.review_status,
                )?,
            )?,
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
pub struct ClinvarAggregatedOncogenicityClassification {
    /// The aggregate review status based on all somatic clinical
    /// impact submissions for this record.
    pub review_status: ClinvarAggregateOncogenicityReviewStatus,
    /// We are not providing an enumeration for the values we report
    /// for somatic clinical impact classification within the xsd. Details are in
    /// <https://github.com/ncbi/clinvar/ClassificationOnClinVar.md>
    pub description: Option<String>,
    /// List of xrefs.
    pub xrefs: Vec<ClinvarXref>,
    /// List of citations.
    pub citations: Vec<ClinvarCitation>,
    /// List of comments.
    pub comments: Vec<ClinvarComment>,
    /// History information.
    pub history_records: Vec<ClinvarDescriptionHistory>,
    /// List of conditions.
    pub conditions: Vec<ClinvarTraitSet>,
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
    for ClinvarAggregatedOncogenicityClassification
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::AggregatedOncogenicityClassification,
    ) -> Result<Self, Self::Error> {
        Ok(ClinvarAggregatedOncogenicityClassification {
            review_status: ClinvarAggregateOncogenicityReviewStatus::try_from(
                pbs::clinvar_data::clinvar_public::AggregateOncogenicityReviewStatus::try_from(
                    value.review_status,
                )?,
            )?,
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
pub struct ClinvarAggregateClassificationSet {
    /// The aggregate germline classification.
    pub germline_classification: Option<ClinvarAggregatedGermlineClassification>,
    /// The aggregate somatic clinical impact.
    pub somatic_clinical_impact: Option<ClinvarAggregatedSomaticClinicalImpact>,
    /// The aggregate oncogenicity classification.
    pub oncogenicity_classification: Option<ClinvarAggregatedOncogenicityClassification>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::AggregateClassificationSet>
    for ClinvarAggregateClassificationSet
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::AggregateClassificationSet,
    ) -> Result<Self, Self::Error> {
        Ok(ClinvarAggregateClassificationSet {
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
pub struct ClinvarClinicalSignificance {
    /// The optional review status.
    pub review_status: Option<ClinvarSubmitterReviewStatus>,
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
    pub explanation: Option<ClinvarComment>,
    /// Optional list of xrefs.
    pub xrefs: Vec<ClinvarXref>,
    /// Optional list of citations.
    pub citations: Vec<ClinvarCitation>,
    /// Optional list of comments.
    pub comments: Vec<ClinvarComment>,
    /// Date of last evaluation.
    ///
    /// NB: unused in XML
    pub date_last_evaluated: Option<chrono::DateTime<chrono::Utc>>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::ClinicalSignificance>
    for ClinvarClinicalSignificance
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::ClinicalSignificance,
    ) -> Result<Self, Self::Error> {
        Ok(ClinvarClinicalSignificance {
            review_status: value
                .review_status
                .map(|review_status| {
                    ClinvarSubmitterReviewStatus::try_from(
                        pbs::clinvar_data::clinvar_public::SubmitterReviewStatus::try_from(
                            review_status,
                        )?,
                    )
                })
                .transpose()?,
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
pub struct ClinvarAlleleDescription {
    /// The name of the allele.
    pub name: String,
    /// Optional relative orientation.
    ///
    /// NB: Unused in XML
    pub relative_orientation: Option<ClinvarRelativeOrientation>,
    /// Optional zygosity.
    pub zygosity: Option<ClinvarZygosity>,
    /// Optional clinical significance.
    ///
    /// Corresponds to `ClinicalSignificanceType` in XSD.
    pub clinical_significance: Option<ClinvarClinicalSignificance>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::AlleleDescription> for ClinvarAlleleDescription {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::AlleleDescription,
    ) -> Result<Self, Self::Error> {
        Ok(ClinvarAlleleDescription {
            name: value.name,
            relative_orientation: value
                .relative_orientation
                .map(|relative_orientation|
                    ClinvarRelativeOrientation::try_from(
                        pbs::clinvar_data::clinvar_public::allele_description::RelativeOrientation::try_from(relative_orientation)?
                    )
                )
                .transpose()?,
            zygosity: value.zygosity                .map(|zygosity| {
                ClinvarZygosity::try_from(pbs::clinvar_data::clinvar_public::Zygosity::try_from(
                    zygosity,
                )?)
            })
            .transpose()?,
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarRelativeOrientation {
    /// corresponds to "cis"
    Cis,
    /// corresponds to "trans"
    Trans,
    /// corresponds to "unknown"
    Unknown,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::allele_description::RelativeOrientation>
    for ClinvarRelativeOrientation
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::allele_description::RelativeOrientation,
    ) -> Result<Self, Self::Error> {
        match value {
            pbs::clinvar_data::clinvar_public::allele_description::RelativeOrientation::Cis => {
                Ok(ClinvarRelativeOrientation::Cis)
            }
            pbs::clinvar_data::clinvar_public::allele_description::RelativeOrientation::Trans => {
                Ok(ClinvarRelativeOrientation::Trans)
            }
            pbs::clinvar_data::clinvar_public::allele_description::RelativeOrientation::Unknown => {
                Ok(ClinvarRelativeOrientation::Unknown)
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
pub struct ClinvarRecordHistory {
    /// Optional comment on the history record.
    pub comment: Option<ClinvarComment>,
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

impl TryFrom<pbs::clinvar_data::clinvar_public::RecordHistory> for ClinvarRecordHistory {
    type Error = anyhow::Error;

    fn try_from(
        record_history: pbs::clinvar_data::clinvar_public::RecordHistory,
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            comment: record_history.comment.map(|x| x.try_into()).transpose()?,
            accession: record_history.accession,
            version: record_history.version,
            date_changed: record_history.date_changed.map(|x| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(x.seconds, x.nanos as u32)
                    .unwrap_or_default()
            }),
            variation_id: record_history.variation_id,
        })
    }
}

/// Report classification of a variant for a SCV.
///
/// Corresponds to `ClassificationTypeSCV` in XSD.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarClassificationScv {
    /// The field's review status.
    pub review_status: ClinvarSubmitterReviewStatus,
    /// The germline classification; mutually exlusive with `somatic_clinical_impact`
    /// and `oncogenicity_classification`.
    pub germline_classification: Option<String>,
    /// Information on the clinical impact; mutually exlusive with `germline_classification`
    /// and `oncogenicity_classification`.
    pub somatic_clinical_impact: Option<ClinvarClassificationScvSomaticClinicalImpact>,
    /// The oncogenicity classification; mutually exlusive with `germline_classification`
    /// and `oncogenicity_classification`.
    pub oncogenicity_classification: Option<String>,
    /// Optional explanation of classification.
    pub explanation_of_classification: Option<String>,
    /// List of classification scores.
    pub classification_scores: Vec<ClinvarClassificationScore>,
    /// List of xrefs.
    pub xrefs: Vec<ClinvarXref>,
    /// List of citations.
    pub citations: Vec<ClinvarCitation>,
    /// List of comments.
    pub comments: Vec<ClinvarComment>,
    /// Date of last evaluation.
    pub date_last_evaluated: Option<chrono::DateTime<chrono::Utc>>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::ClassificationScv> for ClinvarClassificationScv {
    type Error = anyhow::Error;

    fn try_from(
        classification_scv: pbs::clinvar_data::clinvar_public::ClassificationScv,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            review_status: ClinvarSubmitterReviewStatus::try_from(
                pbs::clinvar_data::clinvar_public::SubmitterReviewStatus::try_from(
                    classification_scv.review_status,
                )?,
            )?,
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
                .map(|x| x.try_into())
                .collect::<Result<Vec<_>, _>>()?,
            citations: classification_scv
                .citations
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<Vec<_>, _>>()?,
            comments: classification_scv
                .comments
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<Vec<_>, _>>()?,
            date_last_evaluated: classification_scv.date_last_evaluated.map(|x| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(x.seconds, x.nanos as u32)
                    .unwrap_or_default()
            }),
        })
    }
}

/// Clinical impact of a somatic variatn.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarClassificationScvSomaticClinicalImpact {
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
    for ClinvarClassificationScvSomaticClinicalImpact
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
pub struct ClinvarClassificationScore {
    /// The score's value.
    pub value: f64,
    /// The score's type; optional.
    pub r#type: Option<String>,
}

impl From<pbs::clinvar_data::clinvar_public::classification_scv::ClassificationScore>
    for ClinvarClassificationScore
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
pub struct ClinvarSubmitterIdentifiers {
    /// Name of submitter.
    pub submitter_name: String,
    /// Organization ID.
    pub org_id: i64,
    /// Organization category.
    pub org_category: String,
    /// Organization abbreviation; optional.
    pub org_abbreviation: Option<String>,
}

impl From<pbs::clinvar_data::clinvar_public::SubmitterIdentifiers> for ClinvarSubmitterIdentifiers {
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
pub struct ClinvarSpecies {
    /// Name of the species.
    pub name: String,
    /// Optional taxonomy ID.
    pub taxonomy_id: Option<i32>,
}

impl From<pbs::clinvar_data::clinvar_public::Species> for ClinvarSpecies {
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
pub struct ClinvarClassifiedCondition {
    /// Condition value.
    pub value: String,
    /// Database name.
    pub db: Option<String>,
    /// Identifier in database.
    pub id: Option<String>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::ClassifiedCondition>
    for ClinvarClassifiedCondition
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::ClassifiedCondition,
    ) -> Result<Self, Self::Error> {
        Ok(ClinvarClassifiedCondition {
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
pub struct ClinvarClinicalAssertionRecordHistory {
    /// Optional comment.
    pub comment: Option<ClinvarComment>,
    /// Accession.
    pub accession: String,
    /// Optional version.
    pub version: Option<i32>,
    /// Date of the record.
    pub date_changed: Option<chrono::DateTime<chrono::Utc>>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::ClinicalAssertionRecordHistory>
    for ClinvarClinicalAssertionRecordHistory
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::ClinicalAssertionRecordHistory,
    ) -> Result<Self, Self::Error> {
        Ok(ClinvarClinicalAssertionRecordHistory {
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
pub struct ClinvarFunctionalConsequence {
    /// List of xrefs.
    pub xrefs: Vec<ClinvarXref>,
    /// List of citations.
    pub citations: Vec<ClinvarCitation>,
    /// List of comments.
    pub comments: Vec<ClinvarComment>,
    /// Value of functional consequence.
    pub value: String,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::FunctionalConsequence>
    for ClinvarFunctionalConsequence
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::FunctionalConsequence,
    ) -> Result<Self, Self::Error> {
        Ok(ClinvarFunctionalConsequence {
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
pub struct ClinvarGeneralCitations {
    /// List of xrefs.
    pub xrefs: Vec<ClinvarXref>,
    /// List of citations.
    pub citations: Vec<ClinvarCitation>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::GeneralCitations> for ClinvarGeneralCitations {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::GeneralCitations,
    ) -> Result<Self, Self::Error> {
        Ok(ClinvarGeneralCitations {
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
pub struct ClinvarCooccurrence {
    /// Optional zygosity.
    pub zygosity: Option<ClinvarZygosity>,
    /// The allele descriptions.
    pub allele_descriptions: Vec<ClinvarAlleleDescription>,
    /// The optional count.
    pub count: Option<i32>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Cooccurrence> for ClinvarCooccurrence {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::Cooccurrence,
    ) -> Result<Self, Self::Error> {
        Ok(ClinvarCooccurrence {
            zygosity: value
                .zygosity
                .map(|zygosity| {
                    ClinvarZygosity::try_from(
                        pbs::clinvar_data::clinvar_public::Zygosity::try_from(zygosity)?,
                    )
                })
                .transpose()?,
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
pub struct ClinvarSubmitter {
    /// The submitter's identifier.
    pub submitter_identifiers: Option<ClinvarSubmitterIdentifiers>,
    /// The submitter type.
    pub r#type: ClinvarSubmitterType,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Submitter> for ClinvarSubmitter {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Submitter) -> Result<Self, Self::Error> {
        Ok(ClinvarSubmitter {
            submitter_identifiers: value.submitter_identifiers.map(|x| x.into()),
            r#type: ClinvarSubmitterType::try_from(
                pbs::clinvar_data::clinvar_public::submitter::Type::try_from(value.r#type)?,
            )?,
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarSubmitterType {
    /// corresponds to "primary"
    Primary,
    /// corresponds to "secondary"
    Secondary,
    /// corresponds to "behalf"
    Behalf,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::submitter::Type> for ClinvarSubmitterType {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::submitter::Type,
    ) -> Result<Self, Self::Error> {
        match value {
            pbs::clinvar_data::clinvar_public::submitter::Type::Primary => {
                Ok(ClinvarSubmitterType::Primary)
            }
            pbs::clinvar_data::clinvar_public::submitter::Type::Secondary => {
                Ok(ClinvarSubmitterType::Secondary)
            }
            pbs::clinvar_data::clinvar_public::submitter::Type::Behalf => {
                Ok(ClinvarSubmitterType::Behalf)
            }
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
pub struct ClinvarDosageSensitivity {
    /// Value.
    pub value: String,
    /// Optional last evaluated date.
    pub last_evaluated: Option<chrono::DateTime<chrono::Utc>>,
    /// URL to ClinGen.
    pub clingen: Option<String>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::DosageSensitivity> for ClinvarDosageSensitivity {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::DosageSensitivity,
    ) -> Result<Self, Self::Error> {
        Ok(ClinvarDosageSensitivity {
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
pub struct ClinvarOtherName {
    /// The name's value.
    pub value: String,
    /// The name's type.
    pub r#type: Option<String>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::OtherName> for ClinvarOtherName {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::OtherName) -> Result<Self, Self::Error> {
        Ok(ClinvarOtherName {
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
pub struct ClinvarDeletedScv {
    /// The accession.
    pub accession: String,
    /// The version.
    pub version: i32,
    /// The date of deletion.
    pub date_deleted: Option<chrono::DateTime<chrono::Utc>>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::DeletedScv> for ClinvarDeletedScv {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::DeletedScv) -> Result<Self, Self::Error> {
        Ok(ClinvarDeletedScv {
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
pub struct ClinvarLocation {
    /// Cytogenetic location is maintained independent of sequence
    /// location, and can be submitted or computed from the sequence location.
    ///
    /// Between 0 and 4 entries.
    pub cytogenetic_locations: Vec<String>,
    /// Location on a defined sequence, with reference and alternate
    /// allele, and start /stop values depending on the specificity with which the
    /// variant location is known. The number system of offset 1, and
    /// right-justified to be consistent with HGVS location data.
    pub sequence_locations: Vec<ClinvarSequenceLocation>,
    /// The location of the variant relative to features within the gene.
    pub gene_locations: Vec<String>,
    /// List of xrefs.
    pub xrefs: Vec<ClinvarXref>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Location> for ClinvarLocation {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Location) -> Result<Self, Self::Error> {
        Ok(ClinvarLocation {
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
pub struct ClinvarSequenceLocation {
    /// forDisplay value.
    pub for_display: Option<bool>,
    /// Name of assembly.
    pub assembly: String,
    /// Chromosomeof variant.
    pub chr: ClinvarChromosome,
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
    pub assembly_status: Option<ClinvarAssemblyStatus>,
    /// Position in VCF.
    pub position_vcf: Option<u32>,
    /// Reference allele in VCF.
    pub reference_allele_vcf: Option<String>,
    /// Alternate allele in VCF.
    pub alternate_allele_vcf: Option<String>,
    /// For display length.
    pub for_display_length: Option<u32>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::location::SequenceLocation>
    for ClinvarSequenceLocation
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::location::SequenceLocation,
    ) -> Result<Self, Self::Error> {
        Ok(ClinvarSequenceLocation {
            for_display: value.for_display,
            assembly: value.assembly,
            chr: ClinvarChromosome::try_from(
                pbs::clinvar_data::clinvar_public::Chromosome::try_from(
                    value.chr
                )?
            )?,
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
            assembly_status: value.assembly_status.map(|assembly_status|
                ClinvarAssemblyStatus::try_from(
                pbs::clinvar_data::clinvar_public::location::sequence_location::AssemblyStatus::try_from(assembly_status)?
                )
            ).transpose()?,
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarAssemblyStatus {
    /// corresponds to "current"
    Current,
    /// corresponds to "previous"
    Previous,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::location::sequence_location::AssemblyStatus>
    for ClinvarAssemblyStatus
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::location::sequence_location::AssemblyStatus,
    ) -> Result<Self, Self::Error> {
        match value {
            pbs::clinvar_data::clinvar_public::location::sequence_location::AssemblyStatus::Current => {
                Ok(ClinvarAssemblyStatus::Current)
            }
            pbs::clinvar_data::clinvar_public::location::sequence_location::AssemblyStatus::Previous => {
                Ok(ClinvarAssemblyStatus::Previous)
            }
            _ => Err(anyhow::anyhow!("Unknown AssemblyStatus value: {:?}", value)),
        }
    }
}

/// Description of a SCV.
///
/// Corresponds to "typeSCV" in XSD.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarScv {
    /// Optional title.
    pub title: Option<String>,
    /// Accession.
    pub accession: String,
    /// Version.
    pub version: i32,
}

impl From<pbs::clinvar_data::clinvar_public::Scv> for ClinvarScv {
    fn from(value: pbs::clinvar_data::clinvar_public::Scv) -> Self {
        ClinvarScv {
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
pub struct ClinvarFamilyData {
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

impl From<pbs::clinvar_data::clinvar_public::FamilyData> for ClinvarFamilyData {
    fn from(value: pbs::clinvar_data::clinvar_public::FamilyData) -> Self {
        ClinvarFamilyData {
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
pub struct ClinvarSample {
    /// The sample description.
    pub sample_description: Option<ClinvarSampleDescription>,
    /// The sample origin.
    pub origin: Option<ClinvarOrigin>,
    /// Sample ethnicity.
    pub ethnicity: Option<String>,
    /// Sample geographic origin.
    pub geographic_origin: Option<String>,
    /// Sample tissue.
    pub tissue: Option<String>,
    /// Presence of variant in normal tissue.
    pub somatic_variant_in_normal_tissue: Option<ClinvarSomaticVariantInNormalTissue>,
    /// Somatic variant allele fraction.
    pub somatic_variant_allele_fraction: Option<String>,
    /// Cell line name.
    pub cell_line: Option<String>,
    /// Species.
    pub species: Option<ClinvarSpecies>,
    /// Age (range), max. size of 2.
    pub ages: Vec<ClinvarAge>,
    /// Strain.
    pub strain: Option<String>,
    /// Affected status.
    pub affected_status: Option<ClinvarAffectedStatus>,
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
    pub gender: Option<ClinvarGender>,
    /// Family information.
    pub family_data: Option<ClinvarFamilyData>,
    /// Optional proband ID.
    pub proband: Option<String>,
    /// Optional indication.
    pub indication: Option<ClinvarIndication>,
    /// List of citations.
    pub citations: Vec<ClinvarCitation>,
    /// List of xrefs.
    pub xrefs: Vec<ClinvarXref>,
    /// List of comments.
    pub comments: Vec<ClinvarComment>,
    /// Source type.
    pub source_type: Option<ClinvarSampleSourceType>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Sample> for ClinvarSample {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Sample) -> Result<Self, Self::Error> {
        Ok(ClinvarSample {
            sample_description: value
                .sample_description
                .map(ClinvarSampleDescription::try_from)
                .transpose()?,
            origin: value.origin.map(|origin| ClinvarOrigin::try_from(
                pbs::clinvar_data::clinvar_public::Origin::try_from(origin)?
            )).transpose()?,
            ethnicity: value.ethnicity,
            geographic_origin: value.geographic_origin,
            tissue: value.tissue,
            somatic_variant_in_normal_tissue: value
                .somatic_variant_in_normal_tissue
                .map(|somatic_variant_in_normal_tissue|
                    ClinvarSomaticVariantInNormalTissue::try_from(
                    pbs::clinvar_data::clinvar_public::sample::SomaticVariantInNormalTissue::try_from(
                        somatic_variant_in_normal_tissue,
                    )?
                )
                )
                .transpose()?,
            somatic_variant_allele_fraction: value.somatic_variant_allele_fraction,
            cell_line: value.cell_line,
            species: value.species.map(ClinvarSpecies::try_from).transpose()?,
            ages: value
                .ages
                .into_iter()
                .map(ClinvarAge::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            strain: value.strain,
            affected_status: value
                .affected_status
                .map(|affected_status| {
                    ClinvarAffectedStatus::try_from(
                        pbs::clinvar_data::clinvar_public::sample::AffectedStatus::try_from(
                            affected_status,
                        )?,
                    )
                })
                .transpose()?,
            numer_tested: value.numer_tested,
            number_males: value.number_males,
            number_females: value.number_females,
            number_chr_tested: value.number_chr_tested,
            gender: value
                .gender
                .map(|gender| {
                    ClinvarGender::try_from(pbs::clinvar_data::clinvar_public::sample::Gender::try_from(
                        gender,
                    )?)
                })
                .transpose()?,
            family_data: value.family_data.map(ClinvarFamilyData::try_from).transpose()?,
            proband: value.proband,
            indication: value.indication.map(ClinvarIndication::try_from).transpose()?,
            citations: value
                .citations
                .into_iter()
                .map(ClinvarCitation::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            xrefs: value
                .xrefs
                .into_iter()
                .map(ClinvarXref::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            comments: value
                .comments
                .into_iter()
                .map(ClinvarComment::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            source_type: value
                .source_type
                .map(|source_type| {
                    ClinvarSampleSourceType::try_from(
                        pbs::clinvar_data::clinvar_public::sample::SourceType::try_from(
                            source_type,
                        )?,
                    )
                })
                .transpose()?,
        })
    }
}

/// Local type for sample description.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarSampleDescription {
    /// Description of sample.
    pub description: Option<ClinvarComment>,
    /// Citation.
    pub citation: Option<ClinvarCitation>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::sample::SampleDescription>
    for ClinvarSampleDescription
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::sample::SampleDescription,
    ) -> Result<Self, Self::Error> {
        Ok(ClinvarSampleDescription {
            description: value
                .description
                .map(ClinvarComment::try_from)
                .transpose()?,
            citation: value.citation.map(ClinvarCitation::try_from).transpose()?,
        })
    }
}

/// Local type for an age.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarAge {
    /// The age value.
    pub value: i32,
    /// The age unit.
    pub unit: ClinvarAgeUnit,
    /// The age type.
    pub r#type: ClinvarAgeType,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::sample::Age> for ClinvarAge {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::sample::Age,
    ) -> Result<Self, Self::Error> {
        Ok(ClinvarAge {
            value: value.value,
            unit: ClinvarAgeUnit::try_from(
                pbs::clinvar_data::clinvar_public::sample::AgeUnit::try_from(value.unit)?,
            )?,
            r#type: ClinvarAgeType::try_from(
                pbs::clinvar_data::clinvar_public::sample::AgeType::try_from(value.r#type)?,
            )?,
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarSomaticVariantInNormalTissue {
    /// corresponds to "present"
    Present,
    /// corresponds to "absent"
    Absent,
    /// corresponds to "not tested"
    NotTested,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::sample::SomaticVariantInNormalTissue>
    for ClinvarSomaticVariantInNormalTissue
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::sample::SomaticVariantInNormalTissue,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::sample::SomaticVariantInNormalTissue::Present => {
                ClinvarSomaticVariantInNormalTissue::Present
            }
            pbs::clinvar_data::clinvar_public::sample::SomaticVariantInNormalTissue::Absent => {
                ClinvarSomaticVariantInNormalTissue::Absent
            }
            pbs::clinvar_data::clinvar_public::sample::SomaticVariantInNormalTissue::NotTested => {
                ClinvarSomaticVariantInNormalTissue::NotTested
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarAgeUnit {
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

impl TryFrom<pbs::clinvar_data::clinvar_public::sample::AgeUnit> for ClinvarAgeUnit {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::sample::AgeUnit,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::sample::AgeUnit::Days => ClinvarAgeUnit::Days,
            pbs::clinvar_data::clinvar_public::sample::AgeUnit::Weeks => ClinvarAgeUnit::Weeks,
            pbs::clinvar_data::clinvar_public::sample::AgeUnit::Months => ClinvarAgeUnit::Months,
            pbs::clinvar_data::clinvar_public::sample::AgeUnit::Years => ClinvarAgeUnit::Years,
            pbs::clinvar_data::clinvar_public::sample::AgeUnit::WeeksGestation => {
                ClinvarAgeUnit::WeeksGestation
            }
            pbs::clinvar_data::clinvar_public::sample::AgeUnit::MonthsGestation => {
                ClinvarAgeUnit::MonthsGestation
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarAgeType {
    /// corresponds to "minimum"
    Minimum,
    /// corresponds to "maximum"
    Maximum,
    /// corresponds to "single"
    Single,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::sample::AgeType> for ClinvarAgeType {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::sample::AgeType,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::sample::AgeType::Minimum => ClinvarAgeType::Minimum,
            pbs::clinvar_data::clinvar_public::sample::AgeType::Maximum => ClinvarAgeType::Maximum,
            pbs::clinvar_data::clinvar_public::sample::AgeType::Single => ClinvarAgeType::Single,
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarAffectedStatus {
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

impl TryFrom<pbs::clinvar_data::clinvar_public::sample::AffectedStatus> for ClinvarAffectedStatus {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::sample::AffectedStatus,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::sample::AffectedStatus::Yes => {
                ClinvarAffectedStatus::Yes
            }
            pbs::clinvar_data::clinvar_public::sample::AffectedStatus::No => {
                ClinvarAffectedStatus::No
            }
            pbs::clinvar_data::clinvar_public::sample::AffectedStatus::NotProvided => {
                ClinvarAffectedStatus::NotProvided
            }
            pbs::clinvar_data::clinvar_public::sample::AffectedStatus::Unknown => {
                ClinvarAffectedStatus::Unknown
            }
            pbs::clinvar_data::clinvar_public::sample::AffectedStatus::NotApplicable => {
                ClinvarAffectedStatus::NotApplicable
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarGender {
    /// corresponds to "male"
    Male,
    /// corresponds to "female"
    Female,
    /// corresponds to "mixed"
    Mixed,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::sample::Gender> for ClinvarGender {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::sample::Gender,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::sample::Gender::Male => ClinvarGender::Male,
            pbs::clinvar_data::clinvar_public::sample::Gender::Female => ClinvarGender::Female,
            pbs::clinvar_data::clinvar_public::sample::Gender::Mixed => ClinvarGender::Mixed,
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarSampleSourceType {
    /// corresponds to "submitter-generated"
    SubmitterGenerated,
    /// corresponds to "data mining"
    DataMining,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::sample::SourceType> for ClinvarSampleSourceType {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::sample::SourceType,
    ) -> Result<Self, Self::Error> {
        match value {
            pbs::clinvar_data::clinvar_public::sample::SourceType::SubmitterGenerated => {
                Ok(ClinvarSampleSourceType::SubmitterGenerated)
            }
            pbs::clinvar_data::clinvar_public::sample::SourceType::DataMining => {
                Ok(ClinvarSampleSourceType::DataMining)
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
pub struct ClinvarMethod {
    /// Platform name.
    pub name_platform: Option<String>,
    /// Platform type.
    pub type_platform: Option<String>,
    /// Method purpose.
    pub purpose: Option<String>,
    /// Method result type.
    pub result_type: Option<ClinvarResultType>,
    /// Smallest reported.
    pub min_reported: Option<String>,
    /// Largest reported.
    pub max_reported: Option<String>,
    /// Reference standard.
    pub reference_standard: Option<String>,
    /// List of citations.
    pub citations: Vec<ClinvarCitation>,
    /// List of xrefs.
    pub xrefs: Vec<ClinvarXref>,
    /// Free text to enrich the description of the method and to
    /// provide information not captured in specific fields.
    pub description: Option<String>,
    /// List of softwares used.
    pub software: Vec<ClinvarSoftware>,
    /// Source type.
    pub source_type: Option<ClinvarMethodSourceType>,
    /// Method type.
    pub method_type: ClinvarMethodListType,
    /// Method attribute.
    pub method_attributes: Vec<ClinvarMethodAttribute>,
    /// ObsMethodAttribute is used to indicate an attribute specific
    /// to a particular method in conjunction with a particular observation .
    pub obs_method_attributes: Vec<ClinvarObsMethodAttribute>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Method> for ClinvarMethod {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Method) -> Result<Self, Self::Error> {
        Ok(Self {
            name_platform: value.name_platform,
            type_platform: value.type_platform,
            purpose: value.purpose,
            result_type: value
                .result_type
                .map(|result_type| {
                    ClinvarResultType::try_from(
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
            source_type: value
                .source_type
                .map(|source_type| {
                    ClinvarMethodSourceType::try_from(
                        pbs::clinvar_data::clinvar_public::method::SourceType::try_from(
                            source_type,
                        )?,
                    )
                })
                .transpose()?,
            method_type: ClinvarMethodListType::try_from(
                pbs::clinvar_data::clinvar_public::MethodListType::try_from(value.method_type)?,
            )?,
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
pub struct ClinvarMethodAttribute {
    /// The base value.
    pub base: Option<ClinvarBaseAttribute>,
    /// The attribute type.
    pub r#type: ClinvarMethodAttributeType,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::method::MethodAttribute>
    for ClinvarMethodAttribute
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::method::MethodAttribute,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            base: value.base.map(TryInto::try_into).transpose()?,
            r#type: ClinvarMethodAttributeType::try_from(
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarMethodAttributeType {
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
    for ClinvarMethodAttributeType
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::method::method_attribute::AttributeType,
    ) -> Result<Self, Self::Error> {
        match value {
            pbs::clinvar_data::clinvar_public::method::method_attribute::AttributeType::Location => Ok(ClinvarMethodAttributeType::Location),
            pbs::clinvar_data::clinvar_public::method::method_attribute::AttributeType::ControlsAppropriate => {
                Ok(ClinvarMethodAttributeType::ControlsAppropriate)
            }
            pbs::clinvar_data::clinvar_public::method::method_attribute::AttributeType::MethodAppropriate => {
                Ok(ClinvarMethodAttributeType::MethodAppropriate)
            }
            pbs::clinvar_data::clinvar_public::method::method_attribute::AttributeType::TestName => Ok(ClinvarMethodAttributeType::TestName),
            pbs::clinvar_data::clinvar_public::method::method_attribute::AttributeType::StructVarMethodType => {
                Ok(ClinvarMethodAttributeType::StructVarMethodType)
            }
            pbs::clinvar_data::clinvar_public::method::method_attribute::AttributeType::ProbeAccession => {
                Ok(ClinvarMethodAttributeType::ProbeAccession)
            }
            _ => anyhow::bail!("Invalid AttributeType {:?}", value)
        }
    }
}

/// Local type for observation method attribute.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarObsMethodAttribute {
    /// The base value.
    pub base: Option<ClinvarBaseAttribute>,
    /// The attribute type.
    pub r#type: ClinvarObsMethodAttributeType,
    /// Optional comments.
    pub comments: Vec<ClinvarComment>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::method::ObsMethodAttribute>
    for ClinvarObsMethodAttribute
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::method::ObsMethodAttribute,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            base: value.base.map(TryInto::try_into).transpose()?,
            r#type: ClinvarObsMethodAttributeType::try_from(
                pbs::clinvar_data::clinvar_public::method::obs_method_attribute::AttributeType::try_from(
                    value.r#type,
                )?,
            )?,
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarObsMethodAttributeType {
    /// corresponds to "MethodResult"
    MethodResult,
    /// corresponds to "TestingLaboratory"
    TestingLaboratory,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::method::obs_method_attribute::AttributeType>
    for ClinvarObsMethodAttributeType
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::method::obs_method_attribute::AttributeType,
    ) -> Result<Self, Self::Error> {
        match value {
            pbs::clinvar_data::clinvar_public::method::obs_method_attribute::AttributeType::MethodResult => Ok(ClinvarObsMethodAttributeType::MethodResult),
            pbs::clinvar_data::clinvar_public::method::obs_method_attribute::AttributeType::TestingLaboratory => Ok(ClinvarObsMethodAttributeType::TestingLaboratory),
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarResultType {
    /// corresponds to "number of occurrences"
    NumberOfOccurrences,
    /// corresponds to "p value"
    PValue,
    /// corresponds to "odds ratio"
    OddsRatio,
    /// corresponds to "variant call"
    VariantCall,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::method::ResultType> for ClinvarResultType {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::method::ResultType,
    ) -> Result<Self, Self::Error> {
        match value {
            pbs::clinvar_data::clinvar_public::method::ResultType::NumberOfOccurrences => {
                Ok(ClinvarResultType::NumberOfOccurrences)
            }
            pbs::clinvar_data::clinvar_public::method::ResultType::PValue => {
                Ok(ClinvarResultType::PValue)
            }
            pbs::clinvar_data::clinvar_public::method::ResultType::OddsRatio => {
                Ok(ClinvarResultType::OddsRatio)
            }
            pbs::clinvar_data::clinvar_public::method::ResultType::VariantCall => {
                Ok(ClinvarResultType::VariantCall)
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarMethodSourceType {
    /// corresponds to "submitter-generated"
    SubmitterGenerated,
    /// corresponds to "data mining"
    DataMining,
    /// corresponds to "data review"
    DataReview,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::method::SourceType> for ClinvarMethodSourceType {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::method::SourceType,
    ) -> Result<Self, Self::Error> {
        match value {
            pbs::clinvar_data::clinvar_public::method::SourceType::SubmitterGenerated => {
                Ok(ClinvarMethodSourceType::SubmitterGenerated)
            }
            pbs::clinvar_data::clinvar_public::method::SourceType::DataMining => {
                Ok(ClinvarMethodSourceType::DataMining)
            }
            pbs::clinvar_data::clinvar_public::method::SourceType::DataReview => {
                Ok(ClinvarMethodSourceType::DataReview)
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
pub struct ClinvarAlleleScv {
    /// 0 to many genes (and related data ) related to the allele
    /// being reported.
    pub genes: Vec<ClinvarAlleleScvGene>,
    /// Name provided by the submitter.
    pub name: Option<ClinvarOtherName>,
    /// Variant type.
    pub variant_type: Option<String>,
    /// Location.
    pub location: Option<ClinvarLocation>,
    /// List of other names.
    pub other_names: Vec<ClinvarOtherName>,
    /// Single letter representation of the amino acid change and its
    /// location.
    pub protein_changes: Vec<String>,
    /// List of xrefs.
    pub xrefs: Vec<ClinvarXref>,
    /// List of citations.
    pub citations: Vec<ClinvarCitation>,
    /// List of comments.
    pub comments: Vec<ClinvarComment>,
    /// Currently redundant with the MolecularConsequence element of
    /// the HGVS element?
    pub molecular_consequences: Vec<ClinvarMolecularConsequence>,
    /// Functional consequences.
    pub functional_consequences: Vec<ClinvarFunctionalConsequence>,
    /// Attributes.
    pub attributes: Vec<ClinvarAttributeSetElement>,
    /// Allele ID.
    pub allele_id: Option<i64>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::AlleleScv> for ClinvarAlleleScv {
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
pub struct ClinvarAlleleScvGene {
    /// Gene name.
    pub name: Option<String>,
    /// Used to set key words for retrieval or
    /// display about a gene, such as genes listed by the
    /// ACMG guidelines.
    pub properties: Vec<String>,
    /// Used for gene specific identifiers
    /// such as MIM number, Gene ID, HGNC ID, etc.
    pub xrefs: Vec<ClinvarXref>,
    /// Optional gene symbol.
    pub symbol: Option<String>,
    /// Relationship between gene and variant.
    pub relationship_type: Option<ClinvarGeneVariantRelationship>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::allele_scv::Gene> for ClinvarAlleleScvGene {
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
            relationship_type: value
                .relationship_type
                .map(|relationship_type| {
                    ClinvarGeneVariantRelationship::try_from(
                        pbs::clinvar_data::clinvar_public::GeneVariantRelationship::try_from(
                            relationship_type,
                        )?,
                    )
                })
                .transpose()?,
        })
    }
}

/// Local type for MolecularConsequence.
///
/// nested elements
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarMolecularConsequence {
    /// Xref list.
    pub xrefs: Vec<ClinvarXref>,
    /// Citation list.
    pub citations: Vec<ClinvarCitation>,
    /// Comment list.
    pub comments: Vec<ClinvarComment>,
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
    for ClinvarMolecularConsequence
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
pub struct ClinvarHaplotypeScv {
    /// The list of alleles in the haplotype.
    pub simple_alleles: Vec<ClinvarAlleleScv>,
    /// The preferred representation of the haplotype.
    pub name: Option<String>,
    /// Names other than 'preferred' used for the haplotype.
    pub other_names: Vec<ClinvarOtherName>,
    /// Classification of the variant.
    pub classifications: Option<ClinvarAggregateClassificationSet>,
    /// Functional consequences of the variant.
    pub functional_consequences: Vec<ClinvarFunctionalConsequence>,
    /// List of attributes.
    pub attributes: Vec<ClinvarAttributeSetElement>,
    /// List of citations.
    pub citations: Vec<ClinvarCitation>,
    /// List of cross-references.
    pub xrefs: Vec<ClinvarXref>,
    /// List of comments.
    pub comments: Vec<ClinvarComment>,
    /// Variation ID.
    pub variation_id: Option<i64>,
    /// Number of copies.
    pub number_of_copies: Option<i32>,
    /// Number of chromosomes.
    pub number_of_chromosomes: Option<i32>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::HaplotypeScv> for ClinvarHaplotypeScv {
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
pub struct ClinvarGenotypeScv {
    /// Simple alleles; mutually exclusive with `haplotypes`.
    pub simple_alleles: Vec<ClinvarAlleleScv>,
    /// Haplotype; mutually exclusive with `simple_alleles`.
    ///
    /// Allows more than 2 haplotypes per genotype to support
    /// representation of ploidy.
    pub haplotypes: Vec<ClinvarHaplotypeScv>,
    /// Optional name.
    pub name: Option<String>,
    /// Other names used for the genotype.
    pub other_names: Vec<ClinvarOtherName>,
    /// The variation type.
    pub variation_type: ClinvarVariationType,
    /// Functional consequences.
    pub functional_consequences: Vec<ClinvarFunctionalConsequence>,
    /// Attributes.
    pub attributes: Vec<ClinvarAttributeSetElement>,
    /// List of citations.
    pub citations: Vec<ClinvarCitation>,
    /// List of xrefs.
    pub xrefs: Vec<ClinvarXref>,
    /// List of comments.
    pub comments: Vec<ClinvarComment>,
    /// Variation ID.
    pub variation_id: Option<i64>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::GenotypeScv> for ClinvarGenotypeScv {
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
            variation_type: ClinvarVariationType::try_from(
                pbs::clinvar_data::clinvar_public::VariationType::try_from(value.variation_type)?,
            )?,
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
pub struct ClinvarObservedIn {
    /// Sample.
    pub sample: Option<ClinvarSample>,
    /// Observed data.
    pub observed_data: Vec<ClinvarObservedData>,
    /// Co-occurence set.
    pub cooccurrence_sets: Vec<ClinvarCooccurrence>,
    /// TraitSet.
    pub trait_set: Option<ClinvarTraitSet>,
    /// Citation list.
    pub citations: Vec<ClinvarCitation>,
    /// Xref list.
    pub xrefs: Vec<ClinvarXref>,
    /// Comment list.
    pub comments: Vec<ClinvarComment>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::ObservedIn> for ClinvarObservedIn {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::ObservedIn) -> Result<Self, Self::Error> {
        Ok(Self {
            sample: value.sample.map(TryInto::try_into).transpose()?,
            observed_data: value
                .observed_data
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            cooccurrence_sets: value
                .cooccurrence_sets
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            trait_set: value.trait_set.map(TryInto::try_into).transpose()?,
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
        })
    }
}

/// Local struct for attributes based on `BaseAttribute`.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarObservedDataAttribute {
    /// base
    pub base: Option<ClinvarBaseAttribute>,
    /// type
    pub r#type: ClinvarObservedDataAttributeType,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::observed_in::ObservedDataAttribute>
    for ClinvarObservedDataAttribute
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::observed_in::ObservedDataAttribute,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            base: value.base.map(TryInto::try_into).transpose()?,
            r#type: ClinvarObservedDataAttributeType::try_from(
                pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::try_from(
                    value.r#type
                )?
            )?,
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarObservedDataAttributeType {
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
    for ClinvarObservedDataAttributeType
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::Description => {
                ClinvarObservedDataAttributeType::Description
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::VariantAlleles => {
                ClinvarObservedDataAttributeType::VariantAlleles
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::SubjectsWithVariant => {
                ClinvarObservedDataAttributeType::SubjectsWithVariant
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::SubjectsWithDifferentCausativeVariant => {
                ClinvarObservedDataAttributeType::SubjectsWithDifferentCausativeVariant
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::VariantChromosomes => {
                ClinvarObservedDataAttributeType::VariantChromosomes
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::IndependentObservations => {
                ClinvarObservedDataAttributeType::IndependentObservations
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::SingleHeterozygous => {
                ClinvarObservedDataAttributeType::SingleHeterozygous
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::CompoundHeterozygous => {
                ClinvarObservedDataAttributeType::CompoundHeterozygous
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::Homozygous => {
                ClinvarObservedDataAttributeType::Homozygous
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::Hemizygous => {
                ClinvarObservedDataAttributeType::Hemizygous
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::NumberMosaic => {
                ClinvarObservedDataAttributeType::NumberMosaic
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::ObservedUnspecified => {
                ClinvarObservedDataAttributeType::ObservedUnspecified
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::AlleleFrequency => {
                ClinvarObservedDataAttributeType::AlleleFrequency
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::SecondaryFinding => {
                ClinvarObservedDataAttributeType::SecondaryFinding
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::GenotypeAndMoiConsistent => {
                ClinvarObservedDataAttributeType::GenotypeAndMoiConsistent
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::UnaffectedFamilyMemberWithCausativeVariant => {
                ClinvarObservedDataAttributeType::UnaffectedFamilyMemberWithCausativeVariant
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::HetParentTransmitNormalAllele => {
                ClinvarObservedDataAttributeType::HetParentTransmitNormalAllele
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::CosegregatingFamilies => {
                ClinvarObservedDataAttributeType::CosegregatingFamilies
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::InformativeMeioses => {
                ClinvarObservedDataAttributeType::InformativeMeioses
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::SampleLocalId => {
                ClinvarObservedDataAttributeType::SampleLocalId
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::SampleVariantId => {
                ClinvarObservedDataAttributeType::SampleVariantId
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::FamilyHistory => {
                ClinvarObservedDataAttributeType::FamilyHistory
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::NumFamiliesWithVariant => {
                ClinvarObservedDataAttributeType::NumFamiliesWithVariant
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::NumFamiliesWithSegregationObserved => {
                ClinvarObservedDataAttributeType::NumFamiliesWithSegregationObserved
            }
            pbs::clinvar_data::clinvar_public::observed_in::observed_data_attribute::Type::SegregationObserved => {
                ClinvarObservedDataAttributeType::SegregationObserved
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
pub struct ClinvarObservedData {
    /// Attributes.
    pub attributes: Vec<ClinvarObservedDataAttribute>,
    /// Severity.
    pub severity: Option<ClinvarSeverity>,
    /// Citation list.
    pub citations: Vec<ClinvarCitation>,
    /// Xref list.
    pub xrefs: Vec<ClinvarXref>,
    /// Comment list.
    pub comments: Vec<ClinvarComment>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::observed_in::ObservedData> for ClinvarObservedData {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::observed_in::ObservedData,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            attributes: value
                .attributes
                .into_iter()
                .map(ClinvarObservedDataAttribute::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            severity: value
                .severity
                .map(|severity| {
                    ClinvarSeverity::try_from(
                        pbs::clinvar_data::clinvar_public::Severity::try_from(severity)?,
                    )
                })
                .transpose()?,
            citations: value
                .citations
                .into_iter()
                .map(ClinvarCitation::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            xrefs: value
                .xrefs
                .into_iter()
                .map(ClinvarXref::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            comments: value
                .comments
                .into_iter()
                .map(ClinvarComment::try_from)
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarMethodType {
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

impl TryFrom<pbs::clinvar_data::clinvar_public::observed_in::MethodType> for ClinvarMethodType {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::observed_in::MethodType,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::observed_in::MethodType::LiteratureOnly => {
                ClinvarMethodType::LiteratureOnly
            }
            pbs::clinvar_data::clinvar_public::observed_in::MethodType::ReferencePopulation => {
                ClinvarMethodType::ReferencePopulation
            }
            pbs::clinvar_data::clinvar_public::observed_in::MethodType::CaseControl => {
                ClinvarMethodType::CaseControl
            }
            pbs::clinvar_data::clinvar_public::observed_in::MethodType::ClinicalTesting => {
                ClinvarMethodType::ClinicalTesting
            }
            pbs::clinvar_data::clinvar_public::observed_in::MethodType::InVitro => {
                ClinvarMethodType::InVitro
            }
            pbs::clinvar_data::clinvar_public::observed_in::MethodType::InVivo => {
                ClinvarMethodType::InVivo
            }
            pbs::clinvar_data::clinvar_public::observed_in::MethodType::InferredFromSource => {
                ClinvarMethodType::InferredFromSource
            }
            pbs::clinvar_data::clinvar_public::observed_in::MethodType::Research => {
                ClinvarMethodType::Research
            }
            _ => anyhow::bail!("Invalid observed_in::MethodType: {:?}", value),
        })
    }
}

/// A clinical assertion as submitted (SCV record).
///
/// Corresponds to `MeasureTraitType` in XSD and `<ClinicalAssertion>` in XML
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarClinicalAssertion {
    /// The ClinVar submission ID.
    pub clinvar_submission_id: Option<ClinvarSubmissionId>,
    /// The ClinVar SCV accessions.
    pub clinvar_accession: Option<ClinvarAccession>,
    /// Optional list of additional submitters.
    pub additional_submitters: Vec<ClinvarSubmitter>,
    /// Record status.
    pub record_status: ClinvarClinicalAssertionRecordStatus,
    /// Replaces; mutually exclusive with replaceds
    pub replaces: Vec<String>,
    /// Replaced list; mutually exclusive with replaces
    pub replaceds: Vec<ClinvarClinicalAssertionRecordHistory>,
    /// SCV classification.
    pub classifications: Option<ClinvarClassificationScv>,
    /// The assertion.
    pub assertion: ClinvarAssertion,
    /// Attributes.
    pub attributes: Vec<ClinvarClinicalAssertionAttributeSetElement>,
    /// Observed in.
    pub observed_ins: Vec<ClinvarObservedIn>,
    /// Allele in SCV; mutually exclusive with haplotype/genotype.
    pub simple_allele: Option<ClinvarAlleleScv>,
    /// Haplotype in SCV; mutually exclusive with allele/genotype.
    pub haplotype: Option<ClinvarHaplotypeScv>,
    /// Genotype in SCV; mutually exclusive with allele/haplotype.
    pub genotype: Option<ClinvarGenotypeScv>,
    /// The trait set.
    pub trait_set: Option<ClinvarTraitSet>,
    /// List of citations.
    pub citations: Vec<ClinvarCitation>,
    /// Optional study name.
    pub study_name: Option<String>,
    /// Optional study description.
    pub study_description: Option<String>,
    /// List of comments.
    pub comments: Vec<ClinvarComment>,
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

impl TryFrom<pbs::clinvar_data::clinvar_public::ClinicalAssertion> for ClinvarClinicalAssertion {
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
                .map(ClinvarSubmitter::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            record_status: ClinvarClinicalAssertionRecordStatus::try_from(
                pbs::clinvar_data::clinvar_public::clinical_assertion::RecordStatus::try_from(
                    value.record_status,
                )?,
            )?,
            replaces: value.replaces,
            replaceds: value
                .replaceds
                .into_iter()
                .map(ClinvarClinicalAssertionRecordHistory::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            classifications: value
                .classifications
                .map(ClinvarClassificationScv::try_from)
                .transpose()?,
            assertion: ClinvarAssertion::try_from(
                pbs::clinvar_data::clinvar_public::Assertion::try_from(value.assertion)?,
            )?,
            attributes: value
                .attributes
                .into_iter()
                .map(ClinvarClinicalAssertionAttributeSetElement::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            observed_ins: value
                .observed_ins
                .into_iter()
                .map(ClinvarObservedIn::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            simple_allele: value
                .simple_allele
                .map(ClinvarAlleleScv::try_from)
                .transpose()?,
            haplotype: value
                .haplotype
                .map(ClinvarHaplotypeScv::try_from)
                .transpose()?,
            genotype: value
                .genotype
                .map(ClinvarGenotypeScv::try_from)
                .transpose()?,
            trait_set: value.trait_set.map(ClinvarTraitSet::try_from).transpose()?,
            citations: value
                .citations
                .into_iter()
                .map(ClinvarCitation::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            study_name: value.study_name,
            study_description: value.study_description,
            comments: value
                .comments
                .into_iter()
                .map(ClinvarComment::try_from)
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
pub struct ClinvarClinicalAssertionAttributeSetElement {
    /// The base value.
    pub attribute: Option<ClinvarBaseAttribute>,
    /// The type of the attribute.
    pub r#type: ClinvarAttributeSetElementType,
    /// List of xrefs.
    pub xrefs: Vec<ClinvarXref>,
    /// List of citations.
    pub citations: Vec<ClinvarCitation>,
    /// List of comments.
    pub comments: Vec<ClinvarComment>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::clinical_assertion::AttributeSetElement>
    for ClinvarClinicalAssertionAttributeSetElement
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::clinical_assertion::AttributeSetElement,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            attribute: value.attribute.map(ClinvarBaseAttribute::try_from).transpose()?,
            r#type: ClinvarAttributeSetElementType::try_from(
                pbs::clinvar_data::clinvar_public::clinical_assertion::attribute_set_element::Type::try_from(
                    value.r#type
                )?
            )?,
            xrefs: value
                .xrefs
                .into_iter()
                .map(ClinvarXref::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            citations: value
                .citations
                .into_iter()
                .map(ClinvarCitation::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            comments: value
                .comments
                .into_iter()
                .map(ClinvarComment::try_from)
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarAttributeSetElementType {
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
    for ClinvarAttributeSetElementType
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
    pub submitter_identifiers: Option<ClinvarSubmitterIdentifiers>,
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
                .map(ClinvarSubmitterIdentifiers::try_from)
                .transpose()?,
            date_updated: value.date_updated.map(|ts| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(ts.seconds, ts.nanos as u32)
                    .unwrap_or_default()
            }),
            date_created: value.date_created.map(|ts| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(ts.seconds, ts.nanos as u32)
                    .unwrap_or_default()
            }),
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarClinicalAssertionRecordStatus {
    /// corresponds to "current"
    Current,
    /// corresponds to "replaced"
    Replaced,
    /// corresponds to "removed"
    Removed,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::clinical_assertion::RecordStatus>
    for ClinvarClinicalAssertionRecordStatus
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
pub struct ClinvarAllele {
    /// Gene list.
    pub genes: Vec<ClinvarAlleleGene>,
    /// Name.
    pub name: String,
    /// Canonical SPDI.
    pub canonical_spdi: Option<String>,
    /// Variant type(s).
    pub variant_types: Vec<String>,
    /// Location.
    pub locations: Vec<ClinvarLocation>,
    /// List of other names.
    pub other_names: Vec<ClinvarOtherName>,
    /// These are the single-letter representations of the protein change.
    pub protein_changes: Vec<String>,
    /// List of HGVS expressions.
    pub hgvs_expressions: Vec<ClinvarHgvsExpression>,
    /// Aggregated classifications.
    pub classifications: Option<ClinvarAggregateClassificationSet>,
    /// List of xrefs.
    pub xrefs: Vec<ClinvarXref>,
    /// List of comments.
    pub comments: Vec<ClinvarComment>,
    /// List of functional consequences.
    pub functional_consequences: Vec<ClinvarFunctionalConsequence>,
    /// Allele frequencies.
    pub allele_frequencies: Vec<ClinvarAlleleFrequency>,
    /// Global minor allele frequencies.
    pub global_minor_allele_frequency: Option<ClinvarGlobalMinorAlleleFrequency>,
    /// Allele ID.
    pub allele_id: i64,
    /// Variation ID.
    pub variation_id: i64,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Allele> for ClinvarAllele {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Allele) -> Result<Self, Self::Error> {
        Ok(Self {
            genes: value
                .genes
                .into_iter()
                .map(ClinvarAlleleGene::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            name: value.name,
            canonical_spdi: value.canonical_spdi,
            variant_types: value.variant_types,
            locations: value
                .locations
                .into_iter()
                .map(ClinvarLocation::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            other_names: value
                .other_names
                .into_iter()
                .map(ClinvarOtherName::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            protein_changes: value.protein_changes,
            hgvs_expressions: value
                .hgvs_expressions
                .into_iter()
                .map(ClinvarHgvsExpression::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            classifications: value
                .classifications
                .map(ClinvarAggregateClassificationSet::try_from)
                .transpose()?,
            xrefs: value
                .xrefs
                .into_iter()
                .map(ClinvarXref::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            comments: value
                .comments
                .into_iter()
                .map(ClinvarComment::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            functional_consequences: value
                .functional_consequences
                .into_iter()
                .map(ClinvarFunctionalConsequence::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            allele_frequencies: value
                .allele_frequencies
                .into_iter()
                .map(ClinvarAlleleFrequency::from)
                .collect(),
            global_minor_allele_frequency: value
                .global_minor_allele_frequency
                .map(ClinvarGlobalMinorAlleleFrequency::from),
            allele_id: value.allele_id,
            variation_id: value.variation_id,
        })
    }
}

/// Local type for Gene.
///
/// nested elements
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarAlleleGene {
    /// Gene's locations.
    pub locations: Vec<ClinvarLocation>,
    /// OMIM ID.
    pub omims: Vec<u64>,
    /// Haploinsuffiency.
    pub haploinsufficiency: Option<ClinvarDosageSensitivity>,
    /// Triplosensitivity.
    pub triplosensitivity: Option<ClinvarDosageSensitivity>,
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
    pub relationship_type: Option<ClinvarGeneVariantRelationship>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::allele::Gene> for ClinvarAlleleGene {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::allele::Gene,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            locations: value
                .locations
                .into_iter()
                .map(ClinvarLocation::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            omims: value.omims,
            haploinsufficiency: value
                .haploinsufficiency
                .map(ClinvarDosageSensitivity::try_from)
                .transpose()?,
            triplosensitivity: value
                .triplosensitivity
                .map(ClinvarDosageSensitivity::try_from)
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
                    crate::server::run::clinvar_data::ClinvarGeneVariantRelationship::try_from(
                        pbs::clinvar_data::clinvar_public::GeneVariantRelationship::try_from(x)?,
                    )
                })
                .transpose()?,
        })
    }
}
/// Local type for allele frequency.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarAlleleFrequency {
    /// Value.
    pub value: f64,
    /// Source.
    pub source: String,
    /// URL.
    pub url: Option<String>,
}

impl From<pbs::clinvar_data::clinvar_public::allele::AlleleFrequency> for ClinvarAlleleFrequency {
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
pub struct ClinvarGlobalMinorAlleleFrequency {
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
    for ClinvarGlobalMinorAlleleFrequency
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
pub struct ClinvarAlleleName {
    /// The name's value.
    pub value: String,
    /// The name's type.
    pub r#type: Option<String>,
}

impl From<pbs::clinvar_data::clinvar_public::allele::Name> for ClinvarAlleleName {
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
pub struct ClinvarHaplotype {
    /// The list of alleles in the haplotype.
    pub simple_alleles: Vec<ClinvarAllele>,
    /// The preferred representation of the haplotype.
    pub name: String,
    /// The type of the haplotype.
    pub variation_type: ClinvarVariationType,
    /// Names other than 'preferred' used for the haplotype.
    pub other_names: Vec<ClinvarOtherName>,
    /// List of all the HGVS expressions valid for, or used to submit,
    /// a variant.
    pub hgvs_expressions: Vec<ClinvarHgvsExpression>,
    /// Classifications of the variant.
    pub classifications: Option<ClinvarAggregateClassificationSet>,
    /// Functional consequences of the variant.
    pub functional_consequences: Vec<ClinvarFunctionalConsequence>,
    /// List of cross-references.
    pub xrefs: Vec<ClinvarXref>,
    /// List of comments.
    pub comments: Vec<ClinvarComment>,
    /// Variation ID.
    pub variation_id: i64,
    /// Number of copies.
    pub number_of_copies: Option<i32>,
    /// Number of chromosomes.
    pub number_of_chromosomes: Option<i32>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Haplotype> for ClinvarHaplotype {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Haplotype) -> Result<Self, Self::Error> {
        Ok(Self {
            simple_alleles: value
                .simple_alleles
                .into_iter()
                .map(ClinvarAllele::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            name: value.name,
            variation_type: ClinvarVariationType::try_from(
                pbs::clinvar_data::clinvar_public::VariationType::try_from(value.variation_type)?,
            )?,
            other_names: value
                .other_names
                .into_iter()
                .map(ClinvarOtherName::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            hgvs_expressions: value
                .hgvs_expressions
                .into_iter()
                .map(ClinvarHgvsExpression::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            classifications: value
                .classifications
                .map(ClinvarAggregateClassificationSet::try_from)
                .transpose()?,
            functional_consequences: value
                .functional_consequences
                .into_iter()
                .map(ClinvarFunctionalConsequence::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            xrefs: value
                .xrefs
                .into_iter()
                .map(ClinvarXref::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            comments: value
                .comments
                .into_iter()
                .map(ClinvarComment::try_from)
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
pub struct ClinvarIncludedRecord {
    /// Simple allele; mutually exclusive with haplotype.
    pub simple_allele: Option<ClinvarAllele>,
    /// Haplotype; mutually exclusive with simple_allele.
    pub haplotype: Option<ClinvarHaplotype>,
    /// Aggregate classification sets.
    pub classifications: Option<ClinvarAggregateClassificationSet>,
    /// List of submitted records.
    pub submitted_classifications: Vec<ClinvarScv>,
    /// Maintains the list of classified variants represented in
    /// this submission, although not submitted with an Classification
    /// independently.
    pub classified_variations: Vec<ClinvarClassifiedVariation>,
    /// List of general citations.
    pub general_citations: Vec<ClinvarGeneralCitations>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::IncludedRecord> for ClinvarIncludedRecord {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::IncludedRecord,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            simple_allele: value
                .simple_allele
                .map(ClinvarAllele::try_from)
                .transpose()?,
            haplotype: value
                .haplotype
                .map(ClinvarHaplotype::try_from)
                .transpose()?,
            classifications: value
                .classifications
                .map(ClinvarAggregateClassificationSet::try_from)
                .transpose()?,
            submitted_classifications: value
                .submitted_classifications
                .into_iter()
                .map(ClinvarScv::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            classified_variations: value
                .classified_variations
                .into_iter()
                .map(ClinvarClassifiedVariation::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            general_citations: value
                .general_citations
                .into_iter()
                .map(ClinvarGeneralCitations::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

/// Local type for tag `ClassifiedVariation`.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarClassifiedVariation {
    /// Variation ID.
    pub variation_id: i64,
    /// Optional accession.
    pub accession: Option<String>,
    /// Version.
    pub version: i32,
}

impl From<pbs::clinvar_data::clinvar_public::included_record::ClassifiedVariation>
    for ClinvarClassifiedVariation
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
pub struct ClinvarGenotype {
    /// Simple allele; mutually exclusive with `haplotype`.
    pub simple_alleles: Vec<ClinvarAllele>,
    /// Haplotype; mutually exclusive with `simple_allele`.
    ///
    /// Allows more than 2 haplotypes per genotype to support
    /// representation of ploidy.
    pub haplotypes: Vec<ClinvarHaplotype>,
    /// Optional name.
    pub name: String,
    /// The variation type.
    pub variation_type: ClinvarVariationType,
    /// Names other than 'preferred' used for the Genotype.
    pub other_names: Vec<ClinvarOtherName>,
    /// HGVS descriptions.
    pub hgvs_expressions: Vec<ClinvarHgvsExpression>,
    /// Functional consequences.
    pub functional_consequences: Vec<ClinvarFunctionalConsequence>,
    /// Aggregated classifications.
    pub classifications: Option<ClinvarAggregateClassificationSet>,
    /// List of xrefs.
    pub xrefs: Vec<ClinvarXref>,
    /// List of citations.
    pub citations: Vec<ClinvarCitation>,
    /// List of comments.
    pub comments: Vec<ClinvarComment>,
    /// Attributes.
    pub attributes: Vec<ClinvarAttributeSetElement>,
    /// Variation ID.
    pub variation_id: Option<i64>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Genotype> for ClinvarGenotype {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Genotype) -> Result<Self, Self::Error> {
        Ok(Self {
            simple_alleles: value
                .simple_alleles
                .into_iter()
                .map(ClinvarAllele::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            haplotypes: value
                .haplotypes
                .into_iter()
                .map(ClinvarHaplotype::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            name: value.name,
            variation_type: ClinvarVariationType::try_from(
                pbs::clinvar_data::clinvar_public::VariationType::try_from(value.variation_type)?,
            )?,
            other_names: value
                .other_names
                .into_iter()
                .map(ClinvarOtherName::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            hgvs_expressions: value
                .hgvs_expressions
                .into_iter()
                .map(ClinvarHgvsExpression::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            functional_consequences: value
                .functional_consequences
                .into_iter()
                .map(ClinvarFunctionalConsequence::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            classifications: value
                .classifications
                .map(ClinvarAggregateClassificationSet::try_from)
                .transpose()?,
            xrefs: value
                .xrefs
                .into_iter()
                .map(ClinvarXref::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            citations: value
                .citations
                .into_iter()
                .map(ClinvarCitation::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            comments: value
                .comments
                .into_iter()
                .map(ClinvarComment::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            attributes: value
                .attributes
                .into_iter()
                .map(ClinvarAttributeSetElement::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            variation_id: value.variation_id,
        })
    }
}
/// Corresponds to "typeRCV" in XSD.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarRcvAccession {
    /// The list of classified conditions.
    pub classified_condition_list: Option<ClinvarRcvClassifiedConditionList>,
    /// The list of RCV classifications.
    pub rcv_classifications: Option<ClinvarRcvClassifications>,
    /// The list of RCV accessions this record has replaced.
    pub replaceds: Vec<ClinvarRecordHistory>,
    /// Optional title.
    pub title: Option<String>,
    /// Accession.
    pub accession: String,
    /// Version.
    pub version: i32,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::RcvAccession> for ClinvarRcvAccession {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::RcvAccession,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            classified_condition_list: value
                .classified_condition_list
                .map(ClinvarRcvClassifiedConditionList::try_from)
                .transpose()?,
            rcv_classifications: value
                .rcv_classifications
                .map(ClinvarRcvClassifications::try_from)
                .transpose()?,
            replaceds: value
                .replaceds
                .into_iter()
                .map(ClinvarRecordHistory::try_from)
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
pub struct ClinvarRcvClassifiedConditionList {
    /// List of interpreted conditions.
    pub classified_conditions: Vec<ClinvarClassifiedCondition>,
    /// Trait set ID.
    pub trait_set_id: Option<i64>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::rcv_accession::ClassifiedConditionList>
    for ClinvarRcvClassifiedConditionList
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::rcv_accession::ClassifiedConditionList,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            classified_conditions: value
                .classified_conditions
                .into_iter()
                .map(ClinvarClassifiedCondition::try_from)
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
pub struct ClinvarRcvGermlineClassification {
    /// The aggregate review status based on
    /// all somatic clinical impact submissions for this
    /// record.
    pub review_status: ClinvarAggregateGermlineReviewStatus,
    /// The oncogenicity description.
    pub description: Option<ClinvarRcvGermlineClassificationDescription>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::rcv_accession::GermlineClassification>
    for ClinvarRcvGermlineClassification
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::rcv_accession::GermlineClassification,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            review_status: ClinvarAggregateGermlineReviewStatus::try_from(
                pbs::clinvar_data::clinvar_public::AggregateGermlineReviewStatus::try_from(
                    value.review_status,
                )?,
            )?,
            description: value
                .description
                .map(ClinvarRcvGermlineClassificationDescription::try_from)
                .transpose()?,
        })
    }
}

/// Local type for Description.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarRcvGermlineClassificationDescription {
    /// The description.
    pub value: String,
    /// The date of the description.
    pub date_last_evaluated: Option<chrono::DateTime<chrono::Utc>>,
    /// The number of submissions.
    pub submission_count: Option<u32>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::rcv_accession::germline_classification::Description>
    for ClinvarRcvGermlineClassificationDescription
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::rcv_accession::germline_classification::Description,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            value: value.value,
            date_last_evaluated: value.date_last_evaluated.map(|ts| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(ts.seconds, ts.nanos as u32)
                    .unwrap_or_default()
            }),
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
pub struct ClinvarRcvAccessionSomaticClinicalImpact {
    /// The aggregate review status based on
    /// all somatic clinical impact submissions for this
    /// record.
    pub review_status: ClinvarAggregateSomaticClinicalImpactReviewStatus,
    /// The oncogenicity description.
    pub descriptions: Vec<ClinvarRcvSomaticClinicalImpactDescription>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::rcv_accession::SomaticClinicalImpact>
    for ClinvarRcvAccessionSomaticClinicalImpact
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::rcv_accession::SomaticClinicalImpact,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            review_status: ClinvarAggregateSomaticClinicalImpactReviewStatus::try_from(
                pbs::clinvar_data::clinvar_public::AggregateSomaticClinicalImpactReviewStatus::try_from(
                    value.review_status
                )?
            )?,
            descriptions: value.descriptions.into_iter().map(ClinvarRcvSomaticClinicalImpactDescription::try_from).collect::<Result<Vec<_>, _>>()?,
        })
    }
}

/// Local type for Description.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarRcvSomaticClinicalImpactDescription {
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
    for ClinvarRcvSomaticClinicalImpactDescription
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::rcv_accession::somatic_clinical_impact::Description,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            value: value.value,
            clinical_impact_assertion_type: value.clinical_impact_assertion_type,
            clinical_impact_clinical_significance: value.clinical_impact_clinical_significance,
            date_last_evaluated: value.date_last_evaluated.map(|ts| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(ts.seconds, ts.nanos as u32)
                    .unwrap_or_default()
            }),
            submission_count: value.submission_count,
        })
    }
}

/// Local type for OncogenicityClassification.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarRcvOncogenicityClassification {
    /// The aggregate review status based on
    /// all oncogenic submissions for this record.
    pub review_status: ClinvarAggregateGermlineReviewStatus,
    /// The oncogenicity description.
    pub description: Option<ClinvarRcvOncogenicityDescription>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::rcv_accession::OncogenicityClassification>
    for ClinvarRcvOncogenicityClassification
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::rcv_accession::OncogenicityClassification,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            review_status: ClinvarAggregateGermlineReviewStatus::try_from(
                pbs::clinvar_data::clinvar_public::AggregateGermlineReviewStatus::try_from(
                    value.review_status,
                )?,
            )?,
            description: value
                .description
                .map(ClinvarRcvOncogenicityDescription::try_from)
                .transpose()?,
        })
    }
}
/// Local type for Description.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarRcvOncogenicityDescription {
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
    > for ClinvarRcvOncogenicityDescription
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::rcv_accession::oncogenicity_classification::Description,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            value: value.value,
            date_last_evaluated: value.date_last_evaluated.map(|ts| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(ts.seconds, ts.nanos as u32)
                    .unwrap_or_default()
            }),
            submission_count: value.submission_count,
        })
    }
}

/// Local type for RCV classifications.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarRcvClassifications {
    /// Germline classification.
    pub germline_classification: Option<ClinvarRcvGermlineClassification>,
    /// Somatic clinical impact.
    pub somatic_clinical_impact: Option<ClinvarRcvAccessionSomaticClinicalImpact>,
    /// Oncogenicity classification.
    pub oncogenicity_classification: Option<ClinvarRcvOncogenicityClassification>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::rcv_accession::RcvClassifications>
    for ClinvarRcvClassifications
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::rcv_accession::RcvClassifications,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            germline_classification: value
                .germline_classification
                .map(ClinvarRcvGermlineClassification::try_from)
                .transpose()?,
            somatic_clinical_impact: value
                .somatic_clinical_impact
                .map(ClinvarRcvAccessionSomaticClinicalImpact::try_from)
                .transpose()?,
            oncogenicity_classification: value
                .oncogenicity_classification
                .map(ClinvarRcvOncogenicityClassification::try_from)
                .transpose()?,
        })
    }
}

/// This element is restricted to variation records for which an explicit
/// classification was submitted.  Compare to IncludedRecord, which provides aggregate
/// information about variants that are part of another submission, but for which
/// ClinVar has *not* received a submission specific to that variant independently.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarClassifiedRecord {
    /// Describes a single sequence change relative to a
    /// contiguous region of a chromosome or the mitochondrion.
    ///
    /// Mutually exclusive with `haplotype` and `genotype`.
    pub simple_allele: Option<ClinvarAllele>,
    /// Describes multiple sequence changes on one of the
    /// chromosomes of a homologous pair or on the mitochondrion.
    ///
    /// Mutually exclusive with `simple_allele` and `genotype`.
    pub haplotype: Option<ClinvarHaplotype>,
    /// Describes the combination of sequence changes on each
    /// chromosome of a homologous pair.
    ///
    /// Mutually exclusive with `simple_allele` and `haplotype`.
    pub genotype: Option<ClinvarGenotype>,
    /// List of RCV records.
    pub rcv_list: Option<ClinvarRcvList>,
    /// List of classifications.
    pub classifications: Option<ClinvarAggregateClassificationSet>,
    /// List of clinical assertions.
    pub clinical_assertions: Vec<ClinvarClinicalAssertion>,
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
    pub trait_mappings: Vec<ClinvarRcvTraitMapping>,
    /// List of deleted SCVs.
    pub deleted_scvs: Vec<ClinvarDeletedScv>,
    /// List of general citations.
    pub general_citations: Vec<ClinvarGeneralCitations>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::ClassifiedRecord> for ClinvarClassifiedRecord {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::ClassifiedRecord,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            simple_allele: value
                .simple_allele
                .map(ClinvarAllele::try_from)
                .transpose()?,
            haplotype: value
                .haplotype
                .map(ClinvarHaplotype::try_from)
                .transpose()?,
            genotype: value.genotype.map(ClinvarGenotype::try_from).transpose()?,
            rcv_list: value.rcv_list.map(ClinvarRcvList::try_from).transpose()?,
            classifications: value
                .classifications
                .map(ClinvarAggregateClassificationSet::try_from)
                .transpose()?,
            clinical_assertions: value
                .clinical_assertions
                .into_iter()
                .map(ClinvarClinicalAssertion::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            trait_mappings: value
                .trait_mappings
                .into_iter()
                .map(ClinvarRcvTraitMapping::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            deleted_scvs: value
                .deleted_scvs
                .into_iter()
                .map(ClinvarDeletedScv::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            general_citations: value
                .general_citations
                .into_iter()
                .map(ClinvarGeneralCitations::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

/// Local type for tag `RCVList`.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarRcvList {
    /// The RCV record.
    pub rcv_accessions: Vec<ClinvarRcvAccession>,
    /// The number of submissions (SCV accessions) referencing the VariationID.
    pub submission_count: Option<i32>,
    /// The number of idependent observations.
    pub independent_observations: Option<i32>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::classified_record::RcvList> for ClinvarRcvList {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::classified_record::RcvList,
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            rcv_accessions: value
                .rcv_accessions
                .into_iter()
                .map(ClinvarRcvAccession::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            submission_count: value.submission_count,
            independent_observations: value.independent_observations,
        })
    }
}

/// Local type for the tag `TraitMapping`.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarRcvTraitMapping {
    /// nested elements
    pub medgens: Vec<ClinvarRcvTraitMappingMedgen>,
    /// ID of clinical assertion.
    pub clinical_assertion_id: i64,
    /// The trait type.
    pub trait_type: String,
    /// The mapping type.
    pub mapping_type: ClinvarRcvTraitMappingType,
    /// The mapping value.
    pub mapping_value: String,
    /// The mapping reference.
    pub mapping_ref: String,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::classified_record::TraitMapping>
    for ClinvarRcvTraitMapping
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::classified_record::TraitMapping,
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            medgens: value
                .medgens
                .into_iter()
                .map(ClinvarRcvTraitMappingMedgen::from)
                .collect(),
            clinical_assertion_id: value.clinical_assertion_id,
            trait_type: value.trait_type,
            mapping_type: ClinvarRcvTraitMappingType::try_from(
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
pub struct ClinvarRcvTraitMappingMedgen {
    /// Name.
    pub name: String,
    /// CUI.
    pub cui: String,
}

impl From<pbs::clinvar_data::clinvar_public::classified_record::trait_mapping::Medgen>
    for ClinvarRcvTraitMappingMedgen
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarRcvTraitMappingType {
    /// corresponds to "Name"
    Name,
    /// corresponds to "Xref"
    Xref,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::classified_record::MappingType>
    for ClinvarRcvTraitMappingType
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
pub struct ClinvarVariationArchive {
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
    pub record_type: ClinvarVariationArchiveRecordType,
    /// The record's status.
    pub record_status: ClinvarVariationArchiveRecordStatus,
    /// Pointer to the replacing record; optional.
    pub replaced_by: Option<ClinvarRecordHistory>,
    /// The list of VCV accessions this record has replaced.
    pub replaceds: Vec<ClinvarRecordHistory>,
    /// Comment on the record; optional.
    pub comment: Option<ClinvarComment>,
    /// Specification of the species.
    pub species: Option<ClinvarSpecies>,
    /// This element describes the classification of a single
    /// allele, haplotype, or genotype based on all submissions to ClinVar. This
    /// differs from the element IncludedRecord, which describes simple alleles
    /// or haplotypes, referenced in ClassifiedRecord, but for which no explicit
    /// classification was submitted. Once that variation is described, details
    /// are added about the phenotypes being classified, the classification, the
    /// submitters providing the classifications, and all supported evidence.
    ///
    /// NB: mutually exclusive with `included_record`.
    pub classified_record: Option<ClinvarClassifiedRecord>,
    /// This element describes a single allele or haplotype
    /// included in submissions to ClinVar, but for which no explicit
    /// classification was submitted. It also references the submissions and the
    /// Classified records that include them.
    ///
    /// NB: mutually exclusive with `classified_record`.
    pub included_record: Option<ClinvarIncludedRecord>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::VariationArchive> for ClinvarVariationArchive {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::VariationArchive,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            variation_id: value.variation_id,
            variation_name: value.variation_name,
            variation_type: value.variation_type,
            date_created: value.date_created.map(|ts| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(ts.seconds, ts.nanos as u32)
                    .unwrap_or_default()
            }),
            date_last_updated: value.date_last_updated.map(|ts| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(ts.seconds, ts.nanos as u32)
                    .unwrap_or_default()
            }),
            most_recent_submission: value.most_recent_submission.map(|ts| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(ts.seconds, ts.nanos as u32)
                    .unwrap_or_default()
            }),
            accession: value.accession,
            version: value.version,
            number_of_submitters: value.number_of_submitters,
            number_of_submissions: value.number_of_submissions,
            record_type: ClinvarVariationArchiveRecordType::try_from(
                pbs::clinvar_data::clinvar_public::variation_archive::RecordType::try_from(
                    value.record_type,
                )?,
            )?,
            record_status: ClinvarVariationArchiveRecordStatus::try_from(
                pbs::clinvar_data::clinvar_public::variation_archive::RecordStatus::try_from(
                    value.record_status,
                )?,
            )?,
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarVariationArchiveRecordType {
    /// corresponds to "included"
    Included,
    /// corresponds to "classified"
    Classified,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::variation_archive::RecordType>
    for ClinvarVariationArchiveRecordType
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::variation_archive::RecordType,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::variation_archive::RecordType::Included => {
                ClinvarVariationArchiveRecordType::Included
            }
            pbs::clinvar_data::clinvar_public::variation_archive::RecordType::Classified => {
                ClinvarVariationArchiveRecordType::Classified
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarVariationArchiveRecordStatus {
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
    for ClinvarVariationArchiveRecordStatus
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::variation_archive::RecordStatus,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::variation_archive::RecordStatus::Current => {
                ClinvarVariationArchiveRecordStatus::Current
            }
            pbs::clinvar_data::clinvar_public::variation_archive::RecordStatus::Previous => {
                ClinvarVariationArchiveRecordStatus::Previous
            }
            pbs::clinvar_data::clinvar_public::variation_archive::RecordStatus::Replaced => {
                ClinvarVariationArchiveRecordStatus::Replaced
            }
            pbs::clinvar_data::clinvar_public::variation_archive::RecordStatus::Deleted => {
                ClinvarVariationArchiveRecordStatus::Deleted
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
    pub variation_archives: Vec<ClinvarVariationArchive>,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::ClinvarVariationRelease>
    for ClinvarVariationRelease
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::ClinvarVariationRelease,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            release_date: value.release_date.map(|ts| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(ts.seconds, ts.nanos as u32)
                    .unwrap_or_default()
            }),
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarGeneVariantRelationship {
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
    for ClinvarGeneVariantRelationship
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::GeneVariantRelationship,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::GeneVariantRelationship::VariantWithinGene => {
                ClinvarGeneVariantRelationship::VariantWithinGene
            }
            pbs::clinvar_data::clinvar_public::GeneVariantRelationship::GeneOverlappedByVariant => {
                ClinvarGeneVariantRelationship::GeneOverlappedByVariant
            }
            pbs::clinvar_data::clinvar_public::GeneVariantRelationship::NearGeneUpstream => {
                ClinvarGeneVariantRelationship::NearGeneUpstream
            }
            pbs::clinvar_data::clinvar_public::GeneVariantRelationship::NearGeneDownstream => {
                ClinvarGeneVariantRelationship::NearGeneDownstream
            }
            pbs::clinvar_data::clinvar_public::GeneVariantRelationship::AssertedButNotComputed => {
                ClinvarGeneVariantRelationship::AssertedButNotComputed
            }
            pbs::clinvar_data::clinvar_public::GeneVariantRelationship::WithinMultipleGenesByOverlap => {
                ClinvarGeneVariantRelationship::WithinMultipleGenesByOverlap
            }
            pbs::clinvar_data::clinvar_public::GeneVariantRelationship::WithinSingleGene => {
                ClinvarGeneVariantRelationship::WithinSingleGene
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarSeverity {
    /// corresponds to "mild"
    Mild,
    /// corresponds to "moderate"
    Moderate,
    /// corresponds to "sever"
    Severe,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::Severity> for ClinvarSeverity {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Severity) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::Severity::Mild => ClinvarSeverity::Mild,
            pbs::clinvar_data::clinvar_public::Severity::Moderate => ClinvarSeverity::Moderate,
            pbs::clinvar_data::clinvar_public::Severity::Severe => ClinvarSeverity::Severe,
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarStatus {
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

impl TryFrom<pbs::clinvar_data::clinvar_public::Status> for ClinvarStatus {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Status) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::Status::Current => ClinvarStatus::Current,
            pbs::clinvar_data::clinvar_public::Status::CompletedAndRetired => {
                ClinvarStatus::CompletedAndRetired
            }
            pbs::clinvar_data::clinvar_public::Status::Delete => ClinvarStatus::Delete,
            pbs::clinvar_data::clinvar_public::Status::InDevelopment => {
                ClinvarStatus::InDevelopment
            }
            pbs::clinvar_data::clinvar_public::Status::Reclassified => ClinvarStatus::Reclassified,
            pbs::clinvar_data::clinvar_public::Status::Reject => ClinvarStatus::Reject,
            pbs::clinvar_data::clinvar_public::Status::Secondary => ClinvarStatus::Secondary,
            pbs::clinvar_data::clinvar_public::Status::Suppressed => ClinvarStatus::Suppressed,
            pbs::clinvar_data::clinvar_public::Status::UnderReview => ClinvarStatus::UnderReview,
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarSubmitterReviewStatus {
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

impl TryFrom<pbs::clinvar_data::clinvar_public::SubmitterReviewStatus>
    for ClinvarSubmitterReviewStatus
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::SubmitterReviewStatus,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::SubmitterReviewStatus::NoClassificationProvided => ClinvarSubmitterReviewStatus::NoClassificationProvided,
            pbs::clinvar_data::clinvar_public::SubmitterReviewStatus::NoAssertionCriteriaProvided => ClinvarSubmitterReviewStatus::NoAssertionCriteriaProvided,
            pbs::clinvar_data::clinvar_public::SubmitterReviewStatus::CriteriaProvidedSingleSubmitter => ClinvarSubmitterReviewStatus::CriteriaProvidedSingleSubmitter,
            pbs::clinvar_data::clinvar_public::SubmitterReviewStatus::ReviewedByExpertPanel => ClinvarSubmitterReviewStatus::ReviewedByExpertPanel,
            pbs::clinvar_data::clinvar_public::SubmitterReviewStatus::PracticeGuideline => ClinvarSubmitterReviewStatus::PracticeGuideline,
            pbs::clinvar_data::clinvar_public::SubmitterReviewStatus::FlaggedSubmission => ClinvarSubmitterReviewStatus::FlaggedSubmission,
            pbs::clinvar_data::clinvar_public::SubmitterReviewStatus::CriteriaProvidedMultipleSubmittersNoConflicts => ClinvarSubmitterReviewStatus::CriteriaProvidedMultipleSubmittersNoConflicts,
            pbs::clinvar_data::clinvar_public::SubmitterReviewStatus::CriteriaProvidedConflictingClassifications => ClinvarSubmitterReviewStatus::CriteriaProvidedConflictingClassifications,
            pbs::clinvar_data::clinvar_public::SubmitterReviewStatus::ClassifiedBySingleSubmitter => ClinvarSubmitterReviewStatus::ClassifiedBySingleSubmitter,
            pbs::clinvar_data::clinvar_public::SubmitterReviewStatus::ReviewedByProfessionalSociety => ClinvarSubmitterReviewStatus::ReviewedByProfessionalSociety,
            pbs::clinvar_data::clinvar_public::SubmitterReviewStatus::NotClassifiedBySubmitter => ClinvarSubmitterReviewStatus::NotClassifiedBySubmitter,
            pbs::clinvar_data::clinvar_public::SubmitterReviewStatus::ClassifiedByMultipleSubmitters => ClinvarSubmitterReviewStatus::ClassifiedByMultipleSubmitters,
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarZygosity {
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

impl TryFrom<pbs::clinvar_data::clinvar_public::Zygosity> for ClinvarZygosity {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Zygosity) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::Zygosity::Homozygote => ClinvarZygosity::Homozygote,
            pbs::clinvar_data::clinvar_public::Zygosity::SingleHeterozygote => {
                ClinvarZygosity::SingleHeterozygote
            }
            pbs::clinvar_data::clinvar_public::Zygosity::CompoundHeterozygote => {
                ClinvarZygosity::CompoundHeterozygote
            }
            pbs::clinvar_data::clinvar_public::Zygosity::Hemizygote => ClinvarZygosity::Hemizygote,
            pbs::clinvar_data::clinvar_public::Zygosity::NotProvided => {
                ClinvarZygosity::NotProvided
            }
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarAssertion {
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

impl TryFrom<pbs::clinvar_data::clinvar_public::Assertion> for ClinvarAssertion {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Assertion) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::Assertion::VariationToDisease => {
                ClinvarAssertion::VariationToDisease
            }
            pbs::clinvar_data::clinvar_public::Assertion::VariationToIncludedDisease => {
                ClinvarAssertion::VariationToIncludedDisease
            }
            pbs::clinvar_data::clinvar_public::Assertion::VariationInModifierGeneToDisease => {
                ClinvarAssertion::VariationInModifierGeneToDisease
            }
            pbs::clinvar_data::clinvar_public::Assertion::ConfersSensitivity => {
                ClinvarAssertion::ConfersSensitivity
            }
            pbs::clinvar_data::clinvar_public::Assertion::ConfersResistance => {
                ClinvarAssertion::ConfersResistance
            }
            pbs::clinvar_data::clinvar_public::Assertion::VariantToNamedProtein => {
                ClinvarAssertion::VariantToNamedProtein
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarAggregateGermlineReviewStatus {
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
    for ClinvarAggregateGermlineReviewStatus
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::AggregateGermlineReviewStatus,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::AggregateGermlineReviewStatus::NoClassificationProvided => {
                ClinvarAggregateGermlineReviewStatus::NoClassificationProvided
            }
            pbs::clinvar_data::clinvar_public::AggregateGermlineReviewStatus::NoAssertionCriteriaProvided => {
                ClinvarAggregateGermlineReviewStatus::NoAssertionCriteriaProvided
            }
            pbs::clinvar_data::clinvar_public::AggregateGermlineReviewStatus::CriteriaProvidedSingleSubmitter => {
                ClinvarAggregateGermlineReviewStatus::CriteriaProvidedSingleSubmitter
            }
            pbs::clinvar_data::clinvar_public::AggregateGermlineReviewStatus::CriteriaProvidedMultipleSubmittersNoConflicts => {
                ClinvarAggregateGermlineReviewStatus::CriteriaProvidedMultipleSubmittersNoConflicts
            }
            pbs::clinvar_data::clinvar_public::AggregateGermlineReviewStatus::CriteriaProvidedConflictingClassifications => {
                ClinvarAggregateGermlineReviewStatus::CriteriaProvidedConflictingClassifications
            }
            pbs::clinvar_data::clinvar_public::AggregateGermlineReviewStatus::ReviewedByExpertPanel => {
                ClinvarAggregateGermlineReviewStatus::ReviewedByExpertPanel
            }
            pbs::clinvar_data::clinvar_public::AggregateGermlineReviewStatus::PracticeGuideline => {
                ClinvarAggregateGermlineReviewStatus::PracticeGuideline
            }
            pbs::clinvar_data::clinvar_public::AggregateGermlineReviewStatus::NoClassificationsFromUnflaggedRecords => {
                ClinvarAggregateGermlineReviewStatus::NoClassificationsFromUnflaggedRecords
            }
            pbs::clinvar_data::clinvar_public::AggregateGermlineReviewStatus::NoClassificationForTheSingleVariant => {
                ClinvarAggregateGermlineReviewStatus::NoClassificationForTheSingleVariant
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarAggregateSomaticClinicalImpactReviewStatus {
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
    for ClinvarAggregateSomaticClinicalImpactReviewStatus
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::AggregateSomaticClinicalImpactReviewStatus,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::AggregateSomaticClinicalImpactReviewStatus::NoClassificationProvided => {
                ClinvarAggregateSomaticClinicalImpactReviewStatus::NoClassificationProvided
            }
            pbs::clinvar_data::clinvar_public::AggregateSomaticClinicalImpactReviewStatus::NoAssertionCriteriaProvided => {
                ClinvarAggregateSomaticClinicalImpactReviewStatus::NoAssertionCriteriaProvided
            }
            pbs::clinvar_data::clinvar_public::AggregateSomaticClinicalImpactReviewStatus::CriteriaProvidedSingleSubmitter => {
                ClinvarAggregateSomaticClinicalImpactReviewStatus::CriteriaProvidedSingleSubmitter
            }
            pbs::clinvar_data::clinvar_public::AggregateSomaticClinicalImpactReviewStatus::CriteriaProvidedMultipleSubmitters => {
                ClinvarAggregateSomaticClinicalImpactReviewStatus::CriteriaProvidedMultipleSubmitters
            }
            pbs::clinvar_data::clinvar_public::AggregateSomaticClinicalImpactReviewStatus::ReviewedByExpertPanel => {
                ClinvarAggregateSomaticClinicalImpactReviewStatus::ReviewedByExpertPanel
            }
            pbs::clinvar_data::clinvar_public::AggregateSomaticClinicalImpactReviewStatus::PracticeGuideline => {
                ClinvarAggregateSomaticClinicalImpactReviewStatus::PracticeGuideline
            }
            pbs::clinvar_data::clinvar_public::AggregateSomaticClinicalImpactReviewStatus::NoClassificationsFromUnflaggedRecords => {
                ClinvarAggregateSomaticClinicalImpactReviewStatus::NoClassificationsFromUnflaggedRecords
            }
            pbs::clinvar_data::clinvar_public::AggregateSomaticClinicalImpactReviewStatus::NoClassificationForTheSingleVariant => {
                ClinvarAggregateSomaticClinicalImpactReviewStatus::NoClassificationForTheSingleVariant
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarAggregateOncogenicityReviewStatus {
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
    for ClinvarAggregateOncogenicityReviewStatus
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::AggregateOncogenicityReviewStatus,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::AggregateOncogenicityReviewStatus::NoClassificationProvided => {
                ClinvarAggregateOncogenicityReviewStatus::NoClassificationProvided
            }
            pbs::clinvar_data::clinvar_public::AggregateOncogenicityReviewStatus::NoAssertionCriteriaProvided => {
                ClinvarAggregateOncogenicityReviewStatus::NoAssertionCriteriaProvided
            }
            pbs::clinvar_data::clinvar_public::AggregateOncogenicityReviewStatus::CriteriaProvidedSingleSubmitter => {
                ClinvarAggregateOncogenicityReviewStatus::CriteriaProvidedSingleSubmitter
            }
            pbs::clinvar_data::clinvar_public::AggregateOncogenicityReviewStatus::CriteriaProvidedMultipleSubmittersNoConflicts => {
                ClinvarAggregateOncogenicityReviewStatus::CriteriaProvidedMultipleSubmittersNoConflicts
            }
            pbs::clinvar_data::clinvar_public::AggregateOncogenicityReviewStatus::CriteriaProvidedConflictingClassifications => {
                ClinvarAggregateOncogenicityReviewStatus::CriteriaProvidedConflictingClassifications
            }
            pbs::clinvar_data::clinvar_public::AggregateOncogenicityReviewStatus::ReviewedByExpertPanel => {
                ClinvarAggregateOncogenicityReviewStatus::ReviewedByExpertPanel
            }
            pbs::clinvar_data::clinvar_public::AggregateOncogenicityReviewStatus::PracticeGuideline => {
                ClinvarAggregateOncogenicityReviewStatus::PracticeGuideline
            }
            pbs::clinvar_data::clinvar_public::AggregateOncogenicityReviewStatus::NoClassificationsFromUnflaggedRecords => {
                ClinvarAggregateOncogenicityReviewStatus::NoClassificationsFromUnflaggedRecords
            }
            pbs::clinvar_data::clinvar_public::AggregateOncogenicityReviewStatus::NoClassificationForTheSingleVariant => {
                ClinvarAggregateOncogenicityReviewStatus::NoClassificationForTheSingleVariant
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarOrigin {
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

impl TryFrom<pbs::clinvar_data::clinvar_public::Origin> for ClinvarOrigin {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Origin) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::Origin::Germline => ClinvarOrigin::Germline,
            pbs::clinvar_data::clinvar_public::Origin::Somatic => ClinvarOrigin::Somatic,
            pbs::clinvar_data::clinvar_public::Origin::DeNovo => ClinvarOrigin::DeNovo,
            pbs::clinvar_data::clinvar_public::Origin::NotProvided => ClinvarOrigin::NotProvided,
            pbs::clinvar_data::clinvar_public::Origin::Inherited => ClinvarOrigin::Inherited,
            pbs::clinvar_data::clinvar_public::Origin::Maternal => ClinvarOrigin::Maternal,
            pbs::clinvar_data::clinvar_public::Origin::Paternal => ClinvarOrigin::Paternal,
            pbs::clinvar_data::clinvar_public::Origin::Uniparental => ClinvarOrigin::Uniparental,
            pbs::clinvar_data::clinvar_public::Origin::Biparental => ClinvarOrigin::Biparental,
            pbs::clinvar_data::clinvar_public::Origin::NotReported => ClinvarOrigin::NotReported,
            pbs::clinvar_data::clinvar_public::Origin::TestedInconclusive => {
                ClinvarOrigin::TestedInconclusive
            }
            pbs::clinvar_data::clinvar_public::Origin::Unknown => ClinvarOrigin::Unknown,
            pbs::clinvar_data::clinvar_public::Origin::NotApplicable => {
                ClinvarOrigin::NotApplicable
            }
            pbs::clinvar_data::clinvar_public::Origin::ExperimentallyGenerated => {
                ClinvarOrigin::ExperimentallyGenerated
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarChromosome {
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

impl TryFrom<pbs::clinvar_data::clinvar_public::Chromosome> for ClinvarChromosome {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::Chromosome) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome1 => {
                ClinvarChromosome::Chromosome1
            }
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome2 => {
                ClinvarChromosome::Chromosome2
            }
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome3 => {
                ClinvarChromosome::Chromosome3
            }
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome4 => {
                ClinvarChromosome::Chromosome4
            }
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome5 => {
                ClinvarChromosome::Chromosome5
            }
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome6 => {
                ClinvarChromosome::Chromosome6
            }
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome7 => {
                ClinvarChromosome::Chromosome7
            }
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome8 => {
                ClinvarChromosome::Chromosome8
            }
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome9 => {
                ClinvarChromosome::Chromosome9
            }
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome10 => {
                ClinvarChromosome::Chromosome10
            }
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome11 => {
                ClinvarChromosome::Chromosome11
            }
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome12 => {
                ClinvarChromosome::Chromosome12
            }
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome13 => {
                ClinvarChromosome::Chromosome13
            }
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome14 => {
                ClinvarChromosome::Chromosome14
            }
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome15 => {
                ClinvarChromosome::Chromosome15
            }
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome16 => {
                ClinvarChromosome::Chromosome16
            }
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome17 => {
                ClinvarChromosome::Chromosome17
            }
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome18 => {
                ClinvarChromosome::Chromosome18
            }
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome19 => {
                ClinvarChromosome::Chromosome19
            }
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome20 => {
                ClinvarChromosome::Chromosome20
            }
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome21 => {
                ClinvarChromosome::Chromosome21
            }
            pbs::clinvar_data::clinvar_public::Chromosome::Chromosome22 => {
                ClinvarChromosome::Chromosome22
            }
            pbs::clinvar_data::clinvar_public::Chromosome::X => ClinvarChromosome::X,
            pbs::clinvar_data::clinvar_public::Chromosome::Y => ClinvarChromosome::Y,
            pbs::clinvar_data::clinvar_public::Chromosome::Mt => ClinvarChromosome::Mt,
            pbs::clinvar_data::clinvar_public::Chromosome::Par => ClinvarChromosome::Par,
            pbs::clinvar_data::clinvar_public::Chromosome::Un => ClinvarChromosome::Un,
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarCommentType {
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

impl TryFrom<pbs::clinvar_data::clinvar_public::CommentType> for ClinvarCommentType {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::CommentType,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::CommentType::Public => ClinvarCommentType::Public,
            pbs::clinvar_data::clinvar_public::CommentType::ConvertedByNcb => ClinvarCommentType::ConvertedByNcb,
            pbs::clinvar_data::clinvar_public::CommentType::MissingFromAssembly => ClinvarCommentType::MissingFromAssembly,
            pbs::clinvar_data::clinvar_public::CommentType::GenomicLocationNotEstablished => {
                ClinvarCommentType::GenomicLocationNotEstablished
            }
            pbs::clinvar_data::clinvar_public::CommentType::LocationOnGenomeAndProductNotAligned => {
                ClinvarCommentType::LocationOnGenomeAndProductNotAligned
            }
            pbs::clinvar_data::clinvar_public::CommentType::DeletionComment => ClinvarCommentType::DeletionComment,
            pbs::clinvar_data::clinvar_public::CommentType::MergeComment => ClinvarCommentType::MergeComment,
            pbs::clinvar_data::clinvar_public::CommentType::AssemblySpecificAlleleDefinition => {
                ClinvarCommentType::AssemblySpecificAlleleDefinition
            }
            pbs::clinvar_data::clinvar_public::CommentType::AlignmentGapMakesAppearInconsistent => {
                ClinvarCommentType::AlignmentGapMakesAppearInconsistent
            }
            pbs::clinvar_data::clinvar_public::CommentType::ExplanationOfClassification => {
                ClinvarCommentType::ExplanationOfClassification
            }
            pbs::clinvar_data::clinvar_public::CommentType::FlaggedComment => ClinvarCommentType::FlaggedComment,
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarNucleotideSequence {
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

impl TryFrom<pbs::clinvar_data::clinvar_public::NucleotideSequence> for ClinvarNucleotideSequence {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::NucleotideSequence,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::NucleotideSequence::GenomicTopLevel => {
                ClinvarNucleotideSequence::GenomicTopLevel
            }
            pbs::clinvar_data::clinvar_public::NucleotideSequence::GenomicRefSeqGene => {
                ClinvarNucleotideSequence::GenomicRefSeqGene
            }
            pbs::clinvar_data::clinvar_public::NucleotideSequence::Genomic => {
                ClinvarNucleotideSequence::Genomic
            }
            pbs::clinvar_data::clinvar_public::NucleotideSequence::Coding => {
                ClinvarNucleotideSequence::Coding
            }
            pbs::clinvar_data::clinvar_public::NucleotideSequence::NonCoding => {
                ClinvarNucleotideSequence::NonCoding
            }
            pbs::clinvar_data::clinvar_public::NucleotideSequence::Protein => {
                ClinvarNucleotideSequence::Protein
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarProteinSequence {
    /// corresponds to "protein"
    Protein,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::ProteinSequence> for ClinvarProteinSequence {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::ProteinSequence,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::ProteinSequence::Protein => {
                ClinvarProteinSequence::Protein
            }
            _ => anyhow::bail!("Unknown ProteinSequence: {:?}", value),
        })
    }
}

/// Enumeration describing phenotype set.
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarPhenotypeSetType {
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

impl TryFrom<pbs::clinvar_data::clinvar_public::PhenotypeSetType> for ClinvarPhenotypeSetType {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::PhenotypeSetType,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::PhenotypeSetType::Disease => {
                ClinvarPhenotypeSetType::Disease
            }
            pbs::clinvar_data::clinvar_public::PhenotypeSetType::DrugResponse => {
                ClinvarPhenotypeSetType::DrugResponse
            }
            pbs::clinvar_data::clinvar_public::PhenotypeSetType::Finding => {
                ClinvarPhenotypeSetType::Finding
            }
            pbs::clinvar_data::clinvar_public::PhenotypeSetType::PhenotypeInstruction => {
                ClinvarPhenotypeSetType::PhenotypeInstruction
            }
            pbs::clinvar_data::clinvar_public::PhenotypeSetType::TraitChoice => {
                ClinvarPhenotypeSetType::TraitChoice
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarVariationType {
    /// corresponds to "Diplotype"
    Diplotype,
    /// corresponds to "CompoundHeterozygote"
    CompoundHeterozygote,
    /// corresponds to "Distinct chromosomes"
    DistinctChromosomes,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::VariationType> for ClinvarVariationType {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::VariationType,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::VariationType::Diplotype => {
                ClinvarVariationType::Diplotype
            }
            pbs::clinvar_data::clinvar_public::VariationType::CompoundHeterozygote => {
                ClinvarVariationType::CompoundHeterozygote
            }
            pbs::clinvar_data::clinvar_public::VariationType::DistinctChromosomes => {
                ClinvarVariationType::DistinctChromosomes
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarEvidenceType {
    /// corresponds to "Genetic"
    Genetic,
    /// corresponds to "Experimental"
    Experimental,
    /// corresponds to "Population"
    Population,
    /// corresponds to "Computational"
    Computational,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::EvidenceType> for ClinvarEvidenceType {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::EvidenceType,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::EvidenceType::Genetic => {
                ClinvarEvidenceType::Genetic
            }
            pbs::clinvar_data::clinvar_public::EvidenceType::Experimental => {
                ClinvarEvidenceType::Experimental
            }
            pbs::clinvar_data::clinvar_public::EvidenceType::Population => {
                ClinvarEvidenceType::Population
            }
            pbs::clinvar_data::clinvar_public::EvidenceType::Computational => {
                ClinvarEvidenceType::Computational
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarMethodListType {
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

impl TryFrom<pbs::clinvar_data::clinvar_public::MethodListType> for ClinvarMethodListType {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::MethodListType,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::MethodListType::LiteratureOnly => {
                ClinvarMethodListType::LiteratureOnly
            }
            pbs::clinvar_data::clinvar_public::MethodListType::ReferencePopulation => {
                ClinvarMethodListType::ReferencePopulation
            }
            pbs::clinvar_data::clinvar_public::MethodListType::CaseControl => {
                ClinvarMethodListType::CaseControl
            }
            pbs::clinvar_data::clinvar_public::MethodListType::ClinicalTesting => {
                ClinvarMethodListType::ClinicalTesting
            }
            pbs::clinvar_data::clinvar_public::MethodListType::InVitro => {
                ClinvarMethodListType::InVitro
            }
            pbs::clinvar_data::clinvar_public::MethodListType::InVivo => {
                ClinvarMethodListType::InVivo
            }
            pbs::clinvar_data::clinvar_public::MethodListType::Research => {
                ClinvarMethodListType::Research
            }
            pbs::clinvar_data::clinvar_public::MethodListType::Curation => {
                ClinvarMethodListType::Curation
            }
            pbs::clinvar_data::clinvar_public::MethodListType::NotProvided => {
                ClinvarMethodListType::NotProvided
            }
            pbs::clinvar_data::clinvar_public::MethodListType::ProviderInterpretation => {
                ClinvarMethodListType::ProviderInterpretation
            }
            pbs::clinvar_data::clinvar_public::MethodListType::PhenotypingOnly => {
                ClinvarMethodListType::PhenotypingOnly
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarHgvsType {
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

impl TryFrom<pbs::clinvar_data::clinvar_public::HgvsType> for ClinvarHgvsType {
    type Error = anyhow::Error;

    fn try_from(value: pbs::clinvar_data::clinvar_public::HgvsType) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::HgvsType::Coding => ClinvarHgvsType::Coding,
            pbs::clinvar_data::clinvar_public::HgvsType::Genomic => ClinvarHgvsType::Genomic,
            pbs::clinvar_data::clinvar_public::HgvsType::GenomicTopLevel => {
                ClinvarHgvsType::GenomicTopLevel
            }
            pbs::clinvar_data::clinvar_public::HgvsType::NonCoding => ClinvarHgvsType::NonCoding,
            pbs::clinvar_data::clinvar_public::HgvsType::Protein => ClinvarHgvsType::Protein,
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarClinicalFeaturesAffectedStatusType {
    /// corresponds to "present"
    Present,
    /// corresponds to "absent"
    Absent,
    /// corresponds to "not tested"
    NotTested,
}

impl TryFrom<pbs::clinvar_data::clinvar_public::ClinicalFeaturesAffectedStatusType>
    for ClinvarClinicalFeaturesAffectedStatusType
{
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::ClinicalFeaturesAffectedStatusType,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::ClinicalFeaturesAffectedStatusType::Present => {
                ClinvarClinicalFeaturesAffectedStatusType::Present
            }
            pbs::clinvar_data::clinvar_public::ClinicalFeaturesAffectedStatusType::Absent => {
                ClinvarClinicalFeaturesAffectedStatusType::Absent
            }
            pbs::clinvar_data::clinvar_public::ClinicalFeaturesAffectedStatusType::NotTested => {
                ClinvarClinicalFeaturesAffectedStatusType::NotTested
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarHaploVariationType {
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

impl TryFrom<pbs::clinvar_data::clinvar_public::HaploVariationType> for ClinvarHaploVariationType {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::clinvar_public::HaploVariationType,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::clinvar_public::HaploVariationType::Haplotype => ClinvarHaploVariationType::Haplotype,
            pbs::clinvar_data::clinvar_public::HaploVariationType::HaplotypeSingleVariant => {
                ClinvarHaploVariationType::HaplotypeSingleVariant
            }
            pbs::clinvar_data::clinvar_public::HaploVariationType::Variation => ClinvarHaploVariationType::Variation,
            pbs::clinvar_data::clinvar_public::HaploVariationType::PhaseUnknown => ClinvarHaploVariationType::PhaseUnknown,
            pbs::clinvar_data::clinvar_public::HaploVariationType::HaplotypeDefinedBySingleVariant => {
                ClinvarHaploVariationType::HaplotypeDefinedBySingleVariant
            }
            _ => anyhow::bail!("Unknown HaploVariationType: {:?}", value),
        })
    }
}

/// Accession with version.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarVersionedAccession {
    /// The accession.
    pub accession: String,
    /// The version.
    pub version: i32,
}

impl From<pbs::clinvar_data::extracted_vars::VersionedAccession> for ClinvarVersionedAccession {
    fn from(value: pbs::clinvar_data::extracted_vars::VersionedAccession) -> Self {
        Self {
            accession: value.accession,
            version: value.version,
        }
    }
}
/// Protocol buffer for storing essential information of one RCV.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarExtractedRcvRecord {
    /// The accession.
    pub accession: Option<ClinvarVersionedAccession>,
    /// Title of RCV.
    pub title: String,
    /// Classifications (thinned out).
    pub classifications: Option<ClinvarRcvClassifications>,
}

impl TryFrom<pbs::clinvar_data::extracted_vars::ExtractedRcvRecord> for ClinvarExtractedRcvRecord {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::extracted_vars::ExtractedRcvRecord,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            accession: value
                .accession
                .map(ClinvarVersionedAccession::try_from)
                .transpose()?,
            title: value.title,
            classifications: value
                .classifications
                .map(ClinvarRcvClassifications::try_from)
                .transpose()?,
        })
    }
}
/// Protocol buffer for storing essential information of one VCV.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinvarExtractedVcvRecord {
    /// The accession.
    pub accession: Option<ClinvarVersionedAccession>,
    /// List of aggregated RCVs.
    pub rcvs: Vec<ClinvarExtractedRcvRecord>,
    /// Name of VCV.
    pub name: String,
    /// The type of the variant.
    pub variation_type: ClinvarExtractedVariationType,
    /// Classifications (thinned out).
    pub classifications: Option<ClinvarAggregateClassificationSet>,
    /// Clinical assertions (thinned out),
    pub clinical_assertions: Vec<ClinvarClinicalAssertion>,
    /// The sequence location on one reference.
    pub sequence_location: Option<ClinvarSequenceLocation>,
    /// List of HGNC IDs.
    pub hgnc_ids: Vec<String>,
}

impl TryFrom<pbs::clinvar_data::extracted_vars::ExtractedVcvRecord> for ClinvarExtractedVcvRecord {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::extracted_vars::ExtractedVcvRecord,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            accession: value
                .accession
                .map(ClinvarVersionedAccession::try_from)
                .transpose()?,
            rcvs: value
                .rcvs
                .into_iter()
                .map(ClinvarExtractedRcvRecord::try_from)
                .collect::<Result<_, _>>()?,
            name: value.name,
            variation_type: ClinvarExtractedVariationType::try_from(
                pbs::clinvar_data::extracted_vars::VariationType::try_from(value.variation_type)?,
            )?,
            classifications: value
                .classifications
                .map(ClinvarAggregateClassificationSet::try_from)
                .transpose()?,
            clinical_assertions: value
                .clinical_assertions
                .into_iter()
                .map(ClinvarClinicalAssertion::try_from)
                .collect::<Result<_, _>>()?,
            sequence_location: value
                .sequence_location
                .map(ClinvarSequenceLocation::try_from)
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
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinvarExtractedVariationType {
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

impl TryFrom<pbs::clinvar_data::extracted_vars::VariationType> for ClinvarExtractedVariationType {
    type Error = anyhow::Error;

    fn try_from(
        value: pbs::clinvar_data::extracted_vars::VariationType,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            pbs::clinvar_data::extracted_vars::VariationType::Insertion => {
                ClinvarExtractedVariationType::Insertion
            }
            pbs::clinvar_data::extracted_vars::VariationType::Deletion => {
                ClinvarExtractedVariationType::Deletion
            }
            pbs::clinvar_data::extracted_vars::VariationType::Snv => {
                ClinvarExtractedVariationType::Snv
            }
            pbs::clinvar_data::extracted_vars::VariationType::Indel => {
                ClinvarExtractedVariationType::Indel
            }
            pbs::clinvar_data::extracted_vars::VariationType::Duplication => {
                ClinvarExtractedVariationType::Duplication
            }
            pbs::clinvar_data::extracted_vars::VariationType::TandemDuplication => {
                ClinvarExtractedVariationType::TandemDuplication
            }
            pbs::clinvar_data::extracted_vars::VariationType::StructuralVariant => {
                ClinvarExtractedVariationType::StructuralVariant
            }
            pbs::clinvar_data::extracted_vars::VariationType::CopyNumberGain => {
                ClinvarExtractedVariationType::CopyNumberGain
            }
            pbs::clinvar_data::extracted_vars::VariationType::CopyNumberLoss => {
                ClinvarExtractedVariationType::CopyNumberLoss
            }
            pbs::clinvar_data::extracted_vars::VariationType::ProteinOnly => {
                ClinvarExtractedVariationType::ProteinOnly
            }
            pbs::clinvar_data::extracted_vars::VariationType::Microsatellite => {
                ClinvarExtractedVariationType::Microsatellite
            }
            pbs::clinvar_data::extracted_vars::VariationType::Inversion => {
                ClinvarExtractedVariationType::Inversion
            }
            pbs::clinvar_data::extracted_vars::VariationType::Other => {
                ClinvarExtractedVariationType::Other
            }
            _ => anyhow::bail!("Invalid variation type {:?}", value),
        })
    }
}

impl Into<pbs::clinvar_data::extracted_vars::VariationType> for ClinvarExtractedVariationType {
    fn into(self) -> pbs::clinvar_data::extracted_vars::VariationType {
        match self {
            ClinvarExtractedVariationType::Insertion => {
                pbs::clinvar_data::extracted_vars::VariationType::Insertion
            }
            ClinvarExtractedVariationType::Deletion => {
                pbs::clinvar_data::extracted_vars::VariationType::Deletion
            }
            ClinvarExtractedVariationType::Snv => {
                pbs::clinvar_data::extracted_vars::VariationType::Snv
            }
            ClinvarExtractedVariationType::Indel => {
                pbs::clinvar_data::extracted_vars::VariationType::Indel
            }
            ClinvarExtractedVariationType::Duplication => {
                pbs::clinvar_data::extracted_vars::VariationType::Duplication
            }
            ClinvarExtractedVariationType::TandemDuplication => {
                pbs::clinvar_data::extracted_vars::VariationType::TandemDuplication
            }
            ClinvarExtractedVariationType::StructuralVariant => {
                pbs::clinvar_data::extracted_vars::VariationType::StructuralVariant
            }
            ClinvarExtractedVariationType::CopyNumberGain => {
                pbs::clinvar_data::extracted_vars::VariationType::CopyNumberGain
            }
            ClinvarExtractedVariationType::CopyNumberLoss => {
                pbs::clinvar_data::extracted_vars::VariationType::CopyNumberLoss
            }
            ClinvarExtractedVariationType::ProteinOnly => {
                pbs::clinvar_data::extracted_vars::VariationType::ProteinOnly
            }
            ClinvarExtractedVariationType::Microsatellite => {
                pbs::clinvar_data::extracted_vars::VariationType::Microsatellite
            }
            ClinvarExtractedVariationType::Inversion => {
                pbs::clinvar_data::extracted_vars::VariationType::Inversion
            }
            ClinvarExtractedVariationType::Other => {
                pbs::clinvar_data::extracted_vars::VariationType::Other
            }
        }
    }
}
