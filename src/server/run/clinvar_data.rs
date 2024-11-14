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
    pub r#type: Option<i32>,
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
    pub status: Option<i32>,
}

/// Description of a citation.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Citation {
    /// Optional list of IDs.
    pub ids: Vec<citation::IdType>,
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
/// Nested message and enum types in `Citation`.
pub mod citation {
    /// Local ID with source.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct IdType {
        /// The citation's value.
        pub value: String,
        /// If there is an identifier, what database provides it.
        pub source: String,
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
    pub date_value: Option<chrono::NaiveDateTime>,
}
/// Description of a nucleotide sequence expression.
///
/// Corresponds to `typeNucleotideSequenceExpression`
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct HgvsNucleotideExpression {
    /// The expression values.
    pub expression: String,
    /// The type of the nucleotide sequence.
    pub sequence_type: Option<i32>,
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
    pub r#type: i32,
    /// Optional assembly.
    pub assembly: Option<String>,
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
/// Description of the history of a record.
///
/// Called ``typeDescriptionHistory`` in XSD.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct DescriptionHistory {
    /// The pathogenicity description.
    pub description: String,
    /// The date of the description.
    pub dated: Option<chrono::NaiveDateTime>,
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
/// Common type for an entry in a set of attributes.
///
/// Called ``typeAttributeSet`` in XSD.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AttributeSetElement {
    /// The attribute value.
    pub attribute: Option<attribute_set_element::Attribute>,
    /// List of xrefs.
    pub xrefs: Vec<Xref>,
    /// List of citations.
    pub citations: Vec<Citation>,
    /// List of comments.
    pub comments: Vec<Comment>,
}
/// Nested message and enum types in `AttributeSetElement`.
pub mod attribute_set_element {
    /// Extend the BaseAttribute with a `type` field.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct Attribute {
        /// The base value.
        pub base: Option<super::BaseAttribute>,
        /// The type of the attribute.
        pub r#type: String,
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
    pub trait_relationships: Vec<r#trait::TraitRelationship>,
    /// Citation list.
    pub citations: Vec<Citation>,
    /// Xref list.
    pub xrefs: Vec<Xref>,
    /// Comment list.
    pub comments: Vec<Comment>,
    /// Sources
    pub sources: Vec<String>,
}
/// Nested message and enum types in `Trait`.
pub mod r#trait {
    /// Local type for trait relationship.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct TraitRelationship {
        /// Name(s) of the trait.
        pub names: Vec<super::GenericSetElement>,
        /// Citation list.
        pub citations: Vec<super::Citation>,
        /// Xref list.
        pub xrefs: Vec<super::Xref>,
        /// Comment list.
        pub comments: Vec<super::Comment>,
        /// Sources
        pub sources: Vec<String>,
        /// Trait type.
        pub r#type: i32,
    }
    /// Nested message and enum types in `TraitRelationship`.
    pub mod trait_relationship {
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
        pub enum Type {
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
    pub r#type: i32,
}
/// Nested message and enum types in `Indication`.
pub mod indication {
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
    pub enum Type {
        /// corresponds to "Indication"
        Indication,
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
    pub r#type: i32,
    /// Date of last evaluation.
    pub date_last_evaluated: Option<chrono::NaiveDateTime>,
    /// ID.
    pub id: Option<i64>,
    /// Whether contributes to aggregate classification.
    pub contributes_to_aggregate_classification: Option<bool>,
    /// Lower level of evidence.
    pub lower_level_of_evidence: Option<bool>,
    /// Explanation of or multiple conditions.
    pub multiple_condition_explanation: Option<String>,
}
/// Nested message and enum types in `TraitSet`.
pub mod trait_set {
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
    pub enum Type {
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
    pub review_status: i32,
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
    pub date_last_evaluated: Option<chrono::NaiveDateTime>,
    /// Date of creation.
    pub date_created: Option<chrono::NaiveDateTime>,
    /// Date of most recent submission.
    pub most_recent_submission: Option<chrono::NaiveDateTime>,
    /// Number of submitters.
    pub number_of_submitters: Option<i32>,
    /// Number of submissions.
    pub number_of_submissions: Option<i32>,
}
/// Aggregated somatic clinical impact info.
///
/// nested elements
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AggregatedSomaticClinicalImpact {
    /// The aggregate review status based on all somatic clinical
    /// impact submissions for this record.
    pub review_status: i32,
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
    pub date_last_evaluated: Option<chrono::NaiveDateTime>,
    /// Date of creation.
    pub date_created: Option<chrono::NaiveDateTime>,
    /// Date of most recent submission.
    pub most_recent_submission: Option<chrono::NaiveDateTime>,
    /// Number of submitters.
    pub number_of_submitters: Option<i32>,
    /// Number of submissions.
    pub number_of_submissions: Option<i32>,
}
/// Aggregated oncogenicity classification info.
///
/// nested elements
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AggregatedOncogenicityClassification {
    /// The aggregate review status based on all somatic clinical
    /// impact submissions for this record.
    pub review_status: i32,
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
    pub date_last_evaluated: Option<chrono::NaiveDateTime>,
    /// Date of creation.
    pub date_created: Option<chrono::NaiveDateTime>,
    /// Date of most recent submission.
    pub most_recent_submission: Option<chrono::NaiveDateTime>,
    /// Number of submitters.
    pub number_of_submitters: Option<i32>,
    /// Number of submissions.
    pub number_of_submissions: Option<i32>,
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
/// Describes the clinical significance of a variant.
///
/// Corresponds to `ClinicalSignificanceType` in XSD.
///
/// contained elements
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinicalSignificance {
    /// The optional review status.
    pub review_status: Option<i32>,
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
    pub date_last_evaluated: Option<chrono::NaiveDateTime>,
}
/// This is to be used within co-occurrence set.
///
/// Corresponds to `typeAlleleDescr` in XSD.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AlleleDescription {
    /// The name of the allele.
    pub name: ::prost::alloc::string::String,
    /// Optional relative orientation.
    ///
    /// NB: Unused in XML
    pub relative_orientation: ::core::option::Option<i32>,
    /// Optional zygosity.
    pub zygosity: ::core::option::Option<i32>,
    /// Optional clinical significance.
    ///
    /// Corresponds to `ClinicalSignificanceType` in XSD.
    pub clinical_significance:
        ::core::option::Option<crate::pbs::clinvar_data::clinvar_public::ClinicalSignificance>,
}

/// Nested message and enum types in `AlleleDescription`.
pub mod allele_description {
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
    pub date_changed: Option<chrono::NaiveDateTime>,
    /// Attribute @VaritionID is only populated for VCV, where @Accession
    /// is like VCV000000009
    pub variation_id: Option<i64>,
}
/// Report classification of a variant for a SCV.
///
/// Corresponds to `ClassificationTypeSCV` in XSD.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClassificationScv {
    /// The field's review status.
    pub review_status: i32,
    /// The germline classification; mutually exlusive with `somatic_clinical_impact`
    /// and `oncogenicity_classification`.
    pub germline_classification: Option<String>,
    /// Information on the clinical impact; mutually exlusive with `germline_classification`
    /// and `oncogenicity_classification`.
    pub somatic_clinical_impact: Option<classification_scv::SomaticClinicalImpact>,
    /// The oncogenicity classification; mutually exlusive with `germline_classification`
    /// and `oncogenicity_classification`.
    pub oncogenicity_classification: Option<String>,
    /// Optional explanation of classification.
    pub explanation_of_classification: Option<String>,
    /// List of classification scores.
    pub classification_scores: Vec<classification_scv::ClassificationScore>,
    /// List of xrefs.
    pub xrefs: Vec<Xref>,
    /// List of citations.
    pub citations: Vec<Citation>,
    /// List of comments.
    pub comments: Vec<Comment>,
    /// Date of last evaluation.
    pub date_last_evaluated: Option<chrono::NaiveDateTime>,
}
/// Nested message and enum types in `ClassificationScv`.
pub mod classification_scv {
    /// Clinical impact of a somatic variatn.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct SomaticClinicalImpact {
        /// The somatic clinical impact value.
        pub value: String,
        /// Type of the clinical impact assertion.
        pub clinical_impact_assertion_type: Option<String>,
        /// Clinical impact significance.
        pub clinical_impact_clinical_significance: Option<String>,
        /// Name of the drug for the therapeutic assertion.
        pub drug_for_therapeutic_assertion: Option<String>,
    }
    /// Classification score description.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct ClassificationScore {
        /// The score's value.
        pub value: f64,
        /// The score's type; optional.
        pub r#type: Option<String>,
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
/// Definition of a species.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Species {
    /// Name of the species.
    pub name: String,
    /// Optional taxonomy ID.
    pub taxonomy_id: Option<i32>,
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
    pub date_changed: Option<chrono::NaiveDateTime>,
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
/// Type for the tag `GeneralCitations`.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct GeneralCitations {
    /// List of xrefs.
    pub xrefs: Vec<Xref>,
    /// List of citations.
    pub citations: Vec<Citation>,
}
/// This refers to the zygosity of the variant being asserted.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Cooccurrence {
    /// Optional zygosity.
    pub zygosity: Option<i32>,
    /// The allele descriptions.
    pub allele_descriptions: Vec<AlleleDescription>,
    /// The optional count.
    pub count: Option<i32>,
}
/// A structure to support reporting the name of a submitter, its
/// organization id, and its abbreviation and type.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Submitter {
    /// The submitter's identifier.
    pub submitter_identifiers: Option<SubmitterIdentifiers>,
    /// The submitter type.
    pub r#type: i32,
}
/// Nested message and enum types in `Submitter`.
pub mod submitter {
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
    pub enum Type {
        /// corresponds to "primary"
        Primary,
        /// corresponds to "secondary"
        Secondary,
        /// corresponds to "behalf"
        Behalf,
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
    pub last_evaluated: Option<chrono::NaiveDateTime>,
    /// URL to ClinGen.
    pub clingen: Option<String>,
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
    pub date_deleted: Option<chrono::NaiveDateTime>,
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
    pub sequence_locations: Vec<location::SequenceLocation>,
    /// The location of the variant relative to features within the gene.
    pub gene_locations: Vec<String>,
    /// List of xrefs.
    pub xrefs: Vec<Xref>,
}
/// Nested message and enum types in `Location`.
pub mod location {
    /// Local type for sequence location.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct SequenceLocation {
        /// forDisplay value.
        pub for_display: Option<bool>,
        /// Name of assembly.
        pub assembly: String,
        /// Chromosomeof variant.
        pub chr: i32,
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
        pub assembly_status: Option<i32>,
        /// Position in VCF.
        pub position_vcf: Option<u32>,
        /// Reference allele in VCF.
        pub reference_allele_vcf: Option<String>,
        /// Alternate allele in VCF.
        pub alternate_allele_vcf: Option<String>,
        /// For display length.
        pub for_display_length: Option<u32>,
    }
    /// Nested message and enum types in `SequenceLocation`.
    pub mod sequence_location {
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
/// Description of a sample.
///
/// Corresponds to `typeSample` in XSD.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Sample {
    /// The sample description.
    pub sample_description: Option<sample::SampleDescription>,
    /// The sample origin.
    pub origin: Option<i32>,
    /// Sample ethnicity.
    pub ethnicity: Option<String>,
    /// Sample geographic origin.
    pub geographic_origin: Option<String>,
    /// Sample tissue.
    pub tissue: Option<String>,
    /// Presence of variant in normal tissue.
    pub somatic_variant_in_normal_tissue: Option<i32>,
    /// Somatic variant allele fraction.
    pub somatic_variant_allele_fraction: Option<String>,
    /// Cell line name.
    pub cell_line: Option<String>,
    /// Species.
    pub species: Option<Species>,
    /// Age (range), max. size of 2.
    pub ages: Vec<sample::Age>,
    /// Strain.
    pub strain: Option<String>,
    /// Affected status.
    pub affected_status: Option<i32>,
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
    pub gender: Option<i32>,
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
    pub source_type: Option<i32>,
}
/// Nested message and enum types in `Sample`.
pub mod sample {
    /// Local type for sample description.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct SampleDescription {
        /// Description of sample.
        pub description: Option<super::Comment>,
        /// Citation.
        pub citation: Option<super::Citation>,
    }
    /// Local type for an age.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct Age {
        /// The age value.
        pub value: i32,
        /// The age unit.
        pub unit: i32,
        /// The age type.
        pub r#type: i32,
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
    pub enum SourceType {
        /// corresponds to "submitter-generated"
        SubmitterGenerated,
        /// corresponds to "data mining"
        DataMining,
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
    pub result_type: Option<i32>,
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
    pub source_type: Option<i32>,
    /// Method type.
    pub method_type: i32,
    /// Method attribute.
    pub method_attributes: Vec<method::MethodAttribute>,
    /// ObsMethodAttribute is used to indicate an attribute specific
    /// to a particular method in conjunction with a particular observation .
    pub obs_method_attributes: Vec<method::ObsMethodAttribute>,
}
/// Nested message and enum types in `Method`.
pub mod method {
    /// Local type for method attribute.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct MethodAttribute {
        /// The base value.
        pub base: Option<super::BaseAttribute>,
        /// The attribute type.
        pub r#type: i32,
    }
    /// Nested message and enum types in `MethodAttribute`.
    pub mod method_attribute {
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
        pub enum AttributeType {
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
    }
    /// Local type for observation method attribute.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct ObsMethodAttribute {
        /// The base value.
        pub base: Option<super::BaseAttribute>,
        /// The attribute type.
        pub r#type: i32,
        /// Optional comments.
        pub comments: Vec<super::Comment>,
    }
    /// Nested message and enum types in `ObsMethodAttribute`.
    pub mod obs_method_attribute {
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
        pub enum AttributeType {
            /// corresponds to "MethodResult"
            MethodResult,
            /// corresponds to "TestingLaboratory"
            TestingLaboratory,
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
    pub enum SourceType {
        /// corresponds to "submitter-generated"
        SubmitterGenerated,
        /// corresponds to "data mining"
        DataMining,
        /// corresponds to "data review"
        DataReview,
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
    pub genes: Vec<allele_scv::Gene>,
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
    pub molecular_consequences: Vec<allele_scv::MolecularConsequence>,
    /// Functional consequences.
    pub functional_consequences: Vec<FunctionalConsequence>,
    /// Attributes.
    pub attributes: Vec<AttributeSetElement>,
    /// Allele ID.
    pub allele_id: Option<i64>,
}
/// Nested message and enum types in `AlleleScv`.
pub mod allele_scv {
    /// Local type for Gene.
    ///
    /// nested elements
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct Gene {
        /// Gene name.
        pub name: Option<String>,
        /// Used to set key words for retrieval or
        /// display about a gene, such as genes listed by the
        /// ACMG guidelines.
        pub properties: Vec<String>,
        /// Used for gene specific identifiers
        /// such as MIM number, Gene ID, HGNC ID, etc.
        pub xrefs: Vec<super::Xref>,
        /// Optional gene symbol.
        pub symbol: Option<String>,
        /// Relationship between gene and variant.
        pub relationship_type: Option<i32>,
    }
    /// Local type for MolecularConsequence.
    ///
    /// nested elements
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct MolecularConsequence {
        /// Xref list.
        pub xrefs: Vec<super::Xref>,
        /// Citation list.
        pub citations: Vec<super::Citation>,
        /// Comment list.
        pub comments: Vec<super::Comment>,
        /// RS id.
        pub rs: Option<i64>,
        /// Optional HGVS expression.
        pub hgvs: Option<String>,
        /// Optional SO id.
        pub so_id: Option<String>,
        /// Function.
        pub function: String,
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
    pub variation_type: i32,
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
    pub observed_data: Vec<observed_in::ObservedData>,
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
/// Nested message and enum types in `ObservedIn`.
pub mod observed_in {
    /// Local struct for attributes based on `BaseAttribute`.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct ObservedDataAttribute {
        /// base
        pub base: Option<super::BaseAttribute>,
        /// type
        pub r#type: i32,
    }
    /// Nested message and enum types in `ObservedDataAttribute`.
    pub mod observed_data_attribute {
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
        pub enum Type {
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
        pub severity: Option<i32>,
        /// Citation list.
        pub citations: Vec<super::Citation>,
        /// Xref list.
        pub xrefs: Vec<super::Xref>,
        /// Comment list.
        pub comments: Vec<super::Comment>,
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
}
/// A clinical assertion as submitted (SCV record).
///
/// Corresponds to `MeasureTraitType` in XSD and `<ClinicalAssertion>` in XML
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ClinicalAssertion {
    /// The ClinVar submission ID.
    pub clinvar_submission_id: Option<clinical_assertion::ClinvarSubmissionId>,
    /// The ClinVar SCV accessions.
    pub clinvar_accession: Option<clinical_assertion::ClinvarAccession>,
    /// Optional list of additional submitters.
    pub additional_submitters: Vec<Submitter>,
    /// Record status.
    pub record_status: i32,
    /// Replaces; mutually exclusive with replaceds
    pub replaces: Vec<String>,
    /// Replaced list; mutually exclusive with replaces
    pub replaceds: Vec<ClinicalAssertionRecordHistory>,
    /// SCV classification.
    pub classifications: Option<ClassificationScv>,
    /// The assertion.
    pub assertion: i32,
    /// Attributes.
    pub attributes: Vec<clinical_assertion::AttributeSetElement>,
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
    pub date_created: Option<chrono::NaiveDateTime>,
    /// Date of creation.
    pub date_last_updated: Option<chrono::NaiveDateTime>,
    /// Date of creation.
    pub submission_date: Option<chrono::NaiveDateTime>,
    /// ID.
    pub id: Option<u64>,
    /// Whether it is an FDA recognized database.
    pub fda_recognized_database: Option<bool>,
}
/// Nested message and enum types in `ClinicalAssertion`.
pub mod clinical_assertion {
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
    /// Local type for attribute set.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct AttributeSetElement {
        /// The base value.
        pub attribute: Option<super::BaseAttribute>,
        /// The type of the attribute.
        pub r#type: i32,
        /// List of xrefs.
        pub xrefs: Vec<super::Xref>,
        /// List of citations.
        pub citations: Vec<super::Citation>,
        /// List of comments.
        pub comments: Vec<super::Comment>,
    }
    /// Nested message and enum types in `AttributeSetElement`.
    pub mod attribute_set_element {
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
        pub enum Type {
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
    }
    /// Local type for `ClinVarAccession`
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct ClinvarAccession {
        /// Accession.
        pub accession: String,
        /// Version.
        pub version: i32,
        /// The submitter's identifier.
        pub submitter_identifiers: Option<super::SubmitterIdentifiers>,
        /// The date that the latest update to the submitted
        /// record (SCV) became public in ClinVar.
        pub date_updated: Option<chrono::NaiveDateTime>,
        /// DateCreated is the date when the record first became
        /// public in ClinVar.
        pub date_created: Option<chrono::NaiveDateTime>,
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
    pub enum RecordStatus {
        /// corresponds to "current"
        Current,
        /// corresponds to "replaced"
        Replaced,
        /// corresponds to "removed"
        Removed,
    }
}
/// This is a record per variant (Measure/@ID,AlleleID).
///
/// Corresponds to "typeAllele" in XSD.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Allele {
    /// Gene list.
    pub genes: Vec<allele::Gene>,
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
    pub allele_frequencies: Vec<allele::AlleleFrequency>,
    /// Global minor allele frequencies.
    pub global_minor_allele_frequency: Option<allele::GlobalMinorAlleleFrequency>,
    /// Allele ID.
    pub allele_id: i64,
    /// Variation ID.
    pub variation_id: i64,
}
/// Nested message and enum types in `Allele`.
pub mod allele {
    /// Local type for Gene.
    ///
    /// nested elements
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct Gene {
        /// Gene's locations.
        pub locations: Vec<super::Location>,
        /// OMIM ID.
        pub omims: Vec<u64>,
        /// Haploinsuffiency.
        pub haploinsufficiency: Option<super::DosageSensitivity>,
        /// Triplosensitivity.
        pub triplosensitivity: Option<super::DosageSensitivity>,
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
        pub relationship_type: Option<i32>,
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
    /// Local type for allele name.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct Name {
        /// The name's value.
        pub value: String,
        /// The name's type.
        pub r#type: Option<String>,
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
    pub variation_type: i32,
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
    pub classified_variations: Vec<included_record::ClassifiedVariation>,
    /// List of general citations.
    pub general_citations: Vec<GeneralCitations>,
}
/// Nested message and enum types in `IncludedRecord`.
pub mod included_record {
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
    pub variation_type: i32,
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
/// Corresponds to "typeRCV" in XSD.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct RcvAccession {
    /// The list of classified conditions.
    pub classified_condition_list: Option<rcv_accession::ClassifiedConditionList>,
    /// The list of RCV classifications.
    pub rcv_classifications: Option<rcv_accession::RcvClassifications>,
    /// The list of RCV accessions this record has replaced.
    pub replaceds: Vec<RecordHistory>,
    /// Optional title.
    pub title: Option<String>,
    /// Accession.
    pub accession: String,
    /// Version.
    pub version: i32,
}
/// Nested message and enum types in `RcvAccession`.
pub mod rcv_accession {
    /// Local type for ClassifiedConditionList.
    ///
    /// nested elements
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct ClassifiedConditionList {
        /// List of interpreted conditions.
        pub classified_conditions: Vec<super::ClassifiedCondition>,
        /// Trait set ID.
        pub trait_set_id: Option<i64>,
    }
    /// Local type for GermlineClassification.
    ///
    /// The aggregate review status based on
    /// all germline submissions for this record.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GermlineClassification {
        /// The aggregate review status based on
        /// all somatic clinical impact submissions for this
        /// record.
        pub review_status: i32,
        /// The oncogenicity description.
        pub description: Option<germline_classification::Description>,
    }
    /// Nested message and enum types in `GermlineClassification`.
    pub mod germline_classification {
        /// Local type for Description.
        #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
        pub struct Description {
            /// The description.
            pub value: String,
            /// The date of the description.
            pub date_last_evaluated: Option<chrono::NaiveDateTime>,
            /// The number of submissions.
            pub submission_count: Option<u32>,
        }
    }
    /// Local type for SomaticClinicalImpact.
    ///
    /// The aggregate review status based on
    /// all somatic clinical impact submissions for this
    /// record.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct SomaticClinicalImpact {
        /// The aggregate review status based on
        /// all somatic clinical impact submissions for this
        /// record.
        pub review_status: i32,
        /// The oncogenicity description.
        pub descriptions: Vec<crate::pbs::clinvar_data::clinvar_public::rcv_accession::somatic_clinical_impact::Description>,
    }
    /// Nested message and enum types in `SomaticClinicalImpact`.
    pub mod somatic_clinical_impact {
        /// Local type for Description.
        #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
        pub struct Description {
            /// The description.
            pub value: String,
            /// Clinical impact assertion type.
            pub clinical_impact_assertion_type: Option<String>,
            /// Clinical impact significance
            pub clinical_impact_clinical_significance: Option<String>,
            /// The date of the description.
            pub date_last_evaluated: Option<chrono::NaiveDateTime>,
            /// The number of submissions.
            pub submission_count: Option<u32>,
        }
    }
    /// Local type for OncogenicityClassification.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct OncogenicityClassification {
        /// The aggregate review status based on
        /// all oncogenic submissions for this record.
        pub review_status: i32,
        /// The oncogenicity description.
        pub description: Option<oncogenicity_classification::Description>,
    }
    /// Nested message and enum types in `OncogenicityClassification`.
    pub mod oncogenicity_classification {
        /// Local type for Description.
        #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
        pub struct Description {
            /// The description.
            pub value: String,
            /// The date of the description.
            pub date_last_evaluated: Option<chrono::NaiveDateTime>,
            /// The number of submissions.
            pub submission_count: Option<u32>,
        }
    }
    /// Local type for RCV classifications.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct RcvClassifications {
        /// Germline classification.
        pub germline_classification: Option<GermlineClassification>,
        /// Somatic clinical impact.
        pub somatic_clinical_impact: Option<SomaticClinicalImpact>,
        /// Oncogenicity classification.
        pub oncogenicity_classification: Option<OncogenicityClassification>,
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
    pub rcv_list: Option<classified_record::RcvList>,
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
    pub trait_mappings: Vec<classified_record::TraitMapping>,
    /// List of deleted SCVs.
    pub deleted_scvs: Vec<DeletedScv>,
    /// List of general citations.
    pub general_citations: Vec<GeneralCitations>,
}
/// Nested message and enum types in `ClassifiedRecord`.
pub mod classified_record {
    /// Local type for tag `RCVList`.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct RcvList {
        /// The RCV record.
        pub rcv_accessions: Vec<super::RcvAccession>,
        /// The number of submissions (SCV accessions) referencing the VariationID.
        pub submission_count: Option<i32>,
        /// The number of idependent observations.
        pub independent_observations: Option<i32>,
    }
    
    /// Local type for the tag `TraitMapping`.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct TraitMapping {
        /// nested elements
        pub medgens: Vec<trait_mapping::Medgen>,
        /// ID of clinical assertion.
        pub clinical_assertion_id: i64,
        /// The trait type.
        pub trait_type: String,
        /// The mapping type.
        pub mapping_type: i32,
        /// The mapping value.
        pub mapping_value: String,
        /// The mapping reference.
        pub mapping_ref: String,
    }
    
    /// Nested message and enum types in `TraitMapping`.
    pub mod trait_mapping {
        /// Local type for the tag "MedGen"
        #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
        pub struct Medgen {
            /// Name.
            pub name: ::prost::alloc::string::String,
            /// CUI.
            pub cui: ::prost::alloc::string::String,
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
    pub enum MappingType {
        /// corresponds to "Name"
        Name,
        /// corresponds to "Xref"
        Xref,
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
    pub date_created: Option<chrono::NaiveDateTime>,
    /// The date the record was last updated in the public database. The
    /// update may be a change to one of the submitted records (SCVs) or
    /// annotation added to the aggregate record by NCBI staff. This date
    /// is independent of a version change; annotated added by NCBI may
    /// change without representing a change in the version.
    pub date_last_updated: Option<chrono::NaiveDateTime>,
    /// This date is of the most recent submitted record (SCV) for the
    /// VCV; it may reflect a new submitted record or an update to a submitted record.
    pub most_recent_submission: Option<chrono::NaiveDateTime>,
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
    pub record_type: i32,
    /// The record's status.
    pub record_status: i32,
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
/// Nested message and enum types in `VariationArchive`.
pub mod variation_archive {
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
    pub enum RecordType {
        /// corresponds to "included"
        Included,
        /// corresponds to "classified"
        Classified,
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
    pub enum RecordStatus {
        /// corresponds to "current"
        Current,
        /// corresponds to "previous"
        Previous,
        /// corresponds to "replaced"
        Replaced,
        /// correspodns to "deleted"
        Deleted,
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
    pub release_date: Option<chrono::NaiveDateTime>,
    /// List of `<VariationArchive>` tags.
    pub variation_archives: Vec<VariationArchive>,
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

/// Accession with version.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct VersionedAccession {
    /// The accession.
    pub accession: String,
    /// The version.
    pub version: i32,
}

/// Protocol buffer for storing essential information of one RCV.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ExtractedRcvRecord {
    /// The accession.
    pub accession: Option<VersionedAccession>,
    /// Title of RCV.
    pub title: String,
    /// Classifications (thinned out).
    pub classifications: Option<super::clinvar_public::rcv_accession::RcvClassifications>,
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
    pub classifications: Option<super::clinvar_public::AggregateClassificationSet>,
    /// Clinical assertions (thinned out),
    pub clinical_assertions: Vec<super::clinvar_public::ClinicalAssertion>,
    /// The sequence location on one reference.
    pub sequence_location: Option<super::clinvar_public::location::SequenceLocation>,
    /// List of HGNC IDs.
    pub hgnc_ids: Vec<String>,
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
