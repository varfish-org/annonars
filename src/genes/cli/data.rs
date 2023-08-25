//! Code for loading gene-related data from the TSV.

use serde::{Deserialize, Serialize};

/// Entry in the genes RocksDB database.
///
/// Note that the HGNC ID is used for the keys, e.g., `"HGNC:5"`.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    /// Information from the ACMG secondary finding list.
    pub acmg_sf: Option<acmg_sf::Record>,
    /// Information from the ClinGen gene curation.
    pub clingen: Option<clingen_gene::Record>,
    /// Information from dbNSFP genes.
    pub dbnsfp: Option<dbnsfp_gene::Record>,
    /// Information from the gnomAD constraints database.
    pub gnomad_constraints: Option<gnomad_constraints::Record>,
    /// Information from the HGNC database.
    pub hgnc: hgnc::Record,
    /// Information from the NCBI gene database (aka "Entrez").
    pub ncbi: Option<ncbi::Record>,
    /// Information about OMIM diseases for a gene.
    pub omim: Option<omim::Record>,
    /// Information from rCNV (Collins et al., 2022).
    pub rcnv: Option<rcnv::Record>,
    /// Information from sHet (Weghorn et al., 2019).
    pub shet: Option<shet::Record>,
}

/// Code for data from the ACMG secondary findings list.
pub mod acmg_sf {
    use serde::{Deserialize, Serialize};

    /// A record from the ACMG secondary findings list.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Record {
        /// The HGNC ID.
        pub hgnc_id: String,
        /// The Ensembl gene ID.
        pub ensembl_gene_id: String,
        /// The NCBI gene ID.
        pub ncbi_gene_id: String,
        /// The HGNC gene symbol.
        pub gene_symbol: String,
        /// The MIM gene ID.
        pub mim_gene_id: String,
        /// The disease phenotype.
        pub disease_phenotype: String,
        /// The disease MIM id.
        pub disorder_mim: String,
        /// The phenotype category.
        pub phenotype_category: String,
        /// The mode of inheritance.
        pub inheritance: String,
        /// The version of the ACMG SF list of first appearance.
        pub sf_list_version: String,
        /// The variants to report according to ACMG SF.
        pub variants_to_report: String,
    }
}

/// Code for deserializing data from ClinGen CSV file.
pub mod clingen_gene {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// A record from the ClinGen gene curation.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Record {
        /// HGNC gene symbol.
        pub gene_symbol: String,
        /// HGNC gene ID.
        pub hgnc_id: String,
        /// URL in clinicalgenome.org knowledge base for gene.
        pub gene_url: String,
        /// ClinGen disease label.
        pub disease_label: Option<String>,
        /// MONDO disease ID.
        pub mondo_id: Option<String>,
        /// URL in clinicalgenome.org knowledge base for disease.
        pub disease_url: Option<String>,
        /// Annotated mode of inheritance.
        #[serde(serialize_with = "serialize_vec", deserialize_with = "deserialize_vec")]
        pub mode_of_inheritance: Vec<String>,
        /// Dosage haploinsufficiency assertion.
        pub dosage_haploinsufficiency_assertion: Option<String>,
        /// Dosage triplosensitivity assertion.
        pub dosage_triplosensitivity_assertion: Option<String>,
        /// URL of dosage report in clinicalgenome.org knowledge base.
        pub dosage_report: Option<String>,
        /// Working group with dosage report (always "Dosage Working Group") or empty.
        pub dosage_group: Option<String>,
        /// Validity assertion classifications.
        #[serde(serialize_with = "serialize_vec", deserialize_with = "deserialize_vec")]
        pub gene_disease_validity_assertion_classifications: Vec<String>,
        /// Validity assertion report URLs.
        #[serde(serialize_with = "serialize_vec", deserialize_with = "deserialize_vec")]
        pub gene_disease_validity_assertion_reports: Vec<String>,
        /// Validity assertion Gene Curation Expert Panels.
        #[serde(serialize_with = "serialize_vec", deserialize_with = "deserialize_vec")]
        pub gene_disease_validity_gceps: Vec<String>,
        /// Actionability assertion classifications.
        #[serde(serialize_with = "serialize_vec", deserialize_with = "deserialize_vec")]
        pub actionability_assertion_classifications: Vec<String>,
        /// Actionability assertion report URLs.
        #[serde(serialize_with = "serialize_vec", deserialize_with = "deserialize_vec")]
        pub actionability_assertion_reports: Vec<String>,
        /// Actionability assertion Gene Curation Expert Panels.
        #[serde(serialize_with = "serialize_vec", deserialize_with = "deserialize_vec")]
        pub actionability_groups: Vec<String>,
    }

    /// Deserialize `Vec<String>` as " | "-separated string, empty is "".
    ///
    /// cf. https://stackoverflow.com/a/56384732/84349
    fn deserialize_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: String = Deserialize::deserialize(deserializer)?;
        if value.is_empty() {
            Ok(Vec::new())
        } else {
            Ok(value
                .split(" | ")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect())
        }
    }

    /// Serialize `Vec<String>`, counterpart to `deserialize_vec`.
    fn serialize_vec<S>(x: &Vec<String>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if x.is_empty() {
            s.serialize_str("")
        } else {
            s.serialize_str(&x.join(" | "))
        }
    }
}

/// Code for deserializing data from dbNSFP gene.
pub mod dbnsfp_gene {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// Deserialize `Option::None` as `"."` - String version.
    ///
    /// This also handles the case where a number is parsed yet a string is expected.
    ///
    /// cf. https://stackoverflow.com/a/56384732/84349
    fn deserialize_option_dot_str<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        /// We define a local enum type inside of the function because it is untagged, serde will
        /// deserialize as the first variant that it can.
        #[derive(Deserialize, Debug)]
        #[serde(untagged)]
        enum MaybeNA<U> {
            /// If it can be parsed as Option<T>, it will be..
            Value(Option<U>),
            /// ... otherwise try parsing as a string.
            NAString(String),
            /// (also handle case of i64).
            NAI64(i64),
            /// (also handle case of u64).
            NAU64(u64),
            /// (also handle case of f64).
            NAF64(f64),
        }

        // Deserialize into local enum.
        let value: MaybeNA<String> = Deserialize::deserialize(deserializer)?;
        match value {
            // If parsed as T or None, return that.
            MaybeNA::Value(value) => Ok(value),

            // Otherwise, if value is string an "n/a", return None (and fail if it is any other
            // string)
            MaybeNA::NAString(string) => {
                if string == "." {
                    Ok(None)
                } else {
                    Err(serde::de::Error::custom("Unexpected string"))
                }
            }
            MaybeNA::NAI64(v) => Ok(Some(format!("{}", v))),
            MaybeNA::NAU64(v) => Ok(Some(format!("{}", v))),
            MaybeNA::NAF64(v) => Ok(Some(format!("{}", v))),
        }
    }

    /// Deserialize `Option::None` as `"."`.
    ///
    /// cf. https://stackoverflow.com/a/56384732/84349
    fn deserialize_option_dot<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de> + std::fmt::Debug,
    {
        /// We define a local enum type inside of the function because it is untagged, serde will
        /// deserialize as the first variant that it can.
        #[derive(Deserialize, Debug)]
        #[serde(untagged)]
        enum MaybeNA<U> {
            /// If it can be parsed as Option<T>, it will be..
            Value(Option<U>),
            /// ... otherwise try parsing as a string.
            NAString(String),
        }

        // Deserialize into local enum.
        let value: MaybeNA<T> = Deserialize::deserialize(deserializer)?;
        match value {
            // If parsed as T or None, return that.
            MaybeNA::Value(value) => Ok(value),

            // Otherwise, if value is string an "n/a", return None (and fail if it is any other
            // string)
            MaybeNA::NAString(string) => {
                if string == "." {
                    Ok(None)
                } else {
                    Err(serde::de::Error::custom("Unexpected string"))
                }
            }
        }
    }

    /// Serialize `Option::None` as `"."`.
    fn serialize_option_dot<S, T>(x: &Option<T>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        match x {
            Some(x) => s.serialize_some(x),
            None => s.serialize_str("."),
        }
    }

    /// Deserialize `Vec<String>` as semicolon-separated string, empty is ".".
    ///
    /// cf. https://stackoverflow.com/a/56384732/84349
    fn deserialize_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: String = Deserialize::deserialize(deserializer)?;
        if value == "." {
            Ok(Vec::new())
        } else {
            Ok(value
                .split(';')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect())
        }
    }

    /// Serialize `Vec<String>`, counterpart to `deserialize_vec`.
    fn serialize_vec<S>(x: &Vec<String>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if x.is_empty() {
            s.serialize_str(".")
        } else {
            s.serialize_str(&x.join(";"))
        }
    }

    /// A record from the dbNSFP gene database.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Record {
        /// Gene symbol from HGNC.
        #[serde(alias = "Gene_name")]
        pub gene_name: String,
        /// Ensembl gene id (from HGNC).
        #[serde(
            alias = "Ensembl_gene",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub ensembl_gene: Option<String>,
        /// Chromosome number (from HGNC).
        #[serde(
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub chr: Option<String>,
        /// Old gene symbol (from HGNC).
        #[serde(
            alias = "Gene_old_names",
            serialize_with = "serialize_vec",
            deserialize_with = "deserialize_vec"
        )]
        pub gene_old_names: Vec<String>,
        /// Other gene names (from HGNC).
        #[serde(
            alias = "Gene_other_names",
            serialize_with = "serialize_vec",
            deserialize_with = "deserialize_vec"
        )]
        pub gene_other_names: Vec<String>,
        /// Uniprot acc (from HGNC).
        #[serde(
            alias = "Uniprot_acc(HGNC/Uniprot)",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub uniprot_acc: Option<String>,
        /// Uniprot id (from HGNC).
        #[serde(
            alias = "Uniprot_id(HGNC/Uniprot)",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub uniprot_id: Option<String>,
        /// Uniprot id (from HGNC).
        #[serde(
            alias = "Entrez_gene_id",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub entrez_gene_id: Option<String>,
        /// CCDS id (from HGNC).
        #[serde(
            alias = "CCDS_id",
            serialize_with = "serialize_vec",
            deserialize_with = "deserialize_vec"
        )]
        pub ccds_id: Vec<String>,
        /// Refseq gene id (from HGNC).
        #[serde(
            alias = "Refseq_id",
            serialize_with = "serialize_vec",
            deserialize_with = "deserialize_vec"
        )]
        pub refseq_id: Vec<String>,
        /// UCSC gene id (from HGNC).
        #[serde(
            alias = "ucsc_id",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub ucsc_id: Option<String>,
        /// MIM gene id (from OMIM).
        #[serde(
            alias = "MIM_id",
            serialize_with = "serialize_vec",
            deserialize_with = "deserialize_vec"
        )]
        pub mim_id: Vec<String>,
        /// MIM gene id from OMIM.
        #[serde(
            alias = "OMIM_id",
            serialize_with = "serialize_vec",
            deserialize_with = "deserialize_vec"
        )]
        pub omim_id: Vec<String>,
        /// Gene full name (from HGNC).
        #[serde(
            alias = "Gene_full_name",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub gene_full_name: Option<String>,

        /// Pathway description from Uniprot.
        #[serde(
            alias = "Pathway(Uniprot)",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub pathway_uniprot: Option<String>,
        /// Short name of the Pathway(s) the gene belongs to (from BioCarta).
        #[serde(
            alias = "Pathway(BioCarta)_short",
            serialize_with = "serialize_vec",
            deserialize_with = "deserialize_vec"
        )]
        pub pathway_biocarta_short: Vec<String>,
        /// Full name(s) of the Pathway(s) the gene belongs to (from BioCarta).
        #[serde(
            alias = "Pathway(BioCarta)_full",
            serialize_with = "serialize_vec",
            deserialize_with = "deserialize_vec"
        )]
        pub pathway_biocarta_full: Vec<String>,
        /// Pathway(s) the gene belongs to (from ConsensusPathDB).
        #[serde(
            alias = "Pathway(ConsensusPathDB)",
            serialize_with = "serialize_vec",
            deserialize_with = "deserialize_vec"
        )]
        pub pathway_consensus_path_db: Vec<String>,
        /// ID(s) of the Pathway(s) the gene belongs to (from KEGG).
        #[serde(
            alias = "Pathway(KEGG)_id",
            serialize_with = "serialize_vec",
            deserialize_with = "deserialize_vec"
        )]
        pub pathway_kegg_id: Vec<String>,
        /// Full name(s) of the Pathway(s) the gene belongs to (from KEGG).
        #[serde(
            alias = "Pathway(KEGG)_full",
            serialize_with = "serialize_vec",
            deserialize_with = "deserialize_vec"
        )]
        pub pathway_kegg_full: Vec<String>,

        /// Function description of the gene (from Uniprot).
        #[serde(
            alias = "Function_description",
            serialize_with = "serialize_vec",
            deserialize_with = "deserialize_vec"
        )]
        pub function_description: Vec<String>,
        /// Disease(s) the gene caused or associated with (from Uniprot).
        #[serde(
            alias = "Disease_description",
            serialize_with = "serialize_vec",
            deserialize_with = "deserialize_vec"
        )]
        pub disease_description: Vec<String>,
        /// MIM id(s) of the phenotype the gene caused or associated with (from Uniprot).
        #[serde(
            alias = "MIM_phenotype_id",
            serialize_with = "serialize_vec",
            deserialize_with = "deserialize_vec"
        )]
        pub mim_phenotype_id: Vec<String>,
        /// MIM disease name(s) with MIM id(s) in [] (from Uniprot).
        #[serde(
            alias = "MIM_disease",
            serialize_with = "serialize_vec",
            deserialize_with = "deserialize_vec"
        )]
        pub mim_disease: Vec<String>,
        /// Orphanet Number of the disorder the gene caused or associated with.
        #[serde(
            alias = "Orphanet_disorder_id",
            serialize_with = "serialize_vec",
            deserialize_with = "deserialize_vec"
        )]
        pub orphanet_disorder_id: Vec<String>,
        /// Disorder name from Orphanet.
        #[serde(
            alias = "Orphanet_disorder",
            serialize_with = "serialize_vec",
            deserialize_with = "deserialize_vec"
        )]
        pub orphanet_disorder: Vec<String>,
        /// The type of association beteen the gene and the disorder in Orphanet.
        #[serde(
            alias = "Orphanet_association_type",
            serialize_with = "serialize_vec",
            deserialize_with = "deserialize_vec"
        )]
        pub orphanet_association_type: Vec<String>,
        /// Trait(s) the gene associated with (from GWAS catalog).
        #[serde(
            alias = "Trait_association(GWAS)",
            serialize_with = "serialize_vec",
            deserialize_with = "deserialize_vec"
        )]
        pub trait_association_gwas: Vec<String>,
        /// ID of the mapped Human Phenotype Ontology.
        #[serde(
            alias = "HPO_id",
            serialize_with = "serialize_vec",
            deserialize_with = "deserialize_vec"
        )]
        pub hpo_id: Vec<String>,
        /// Name of the mapped Human Phenotype Ontology.
        #[serde(
            alias = "HPO_name",
            serialize_with = "serialize_vec",
            deserialize_with = "deserialize_vec"
        )]
        pub hpo_name: Vec<String>,
        /// GO terms for biological process.
        #[serde(
            alias = "GO_biological_process",
            serialize_with = "serialize_vec",
            deserialize_with = "deserialize_vec"
        )]
        pub go_biological_process: Vec<String>,
        /// GO terms for cellular component.
        #[serde(
            alias = "GO_cellular_component",
            serialize_with = "serialize_vec",
            deserialize_with = "deserialize_vec"
        )]
        pub go_cellular_component: Vec<String>,
        /// GO terms for molecular function.
        #[serde(
            alias = "GO_molecular_function",
            serialize_with = "serialize_vec",
            deserialize_with = "deserialize_vec"
        )]
        pub go_molecular_function: Vec<String>,
        /// Tissue specificity description from Uniprot.
        #[serde(
            alias = "Tissue_specificity(Uniprot)",
            serialize_with = "serialize_vec",
            deserialize_with = "deserialize_vec"
        )]
        pub tissue_specificity_uniprot: Vec<String>,
        /// Tissues/organs the gene expressed in (egenetics data from BioMart).
        #[serde(
            alias = "Expression(egenetics)",
            serialize_with = "serialize_vec",
            deserialize_with = "deserialize_vec"
        )]
        pub expression_egenetics: Vec<String>,
        /// Tissues/organs the gene expressed in (GNF/Atlas data from BioMart).
        #[serde(
            alias = "Expression(GNF/Atlas)",
            serialize_with = "serialize_vec",
            deserialize_with = "deserialize_vec"
        )]
        pub expression_gnf_atlas: Vec<String>,
        /// The interacting genes from IntAct.
        #[serde(
            alias = "Interactions(IntAct)",
            serialize_with = "serialize_vec",
            deserialize_with = "deserialize_vec"
        )]
        pub interactions_intact: Vec<String>,
        /// The interacting genes from BioGRID.
        #[serde(
            alias = "Interactions(BioGRID)",
            serialize_with = "serialize_vec",
            deserialize_with = "deserialize_vec"
        )]
        pub interactions_biogrid: Vec<String>,
        /// The interacting genes from ConsensusPathDB.
        #[serde(
            alias = "Interactions(ConsensusPathDB)",
            serialize_with = "serialize_vec",
            deserialize_with = "deserialize_vec"
        )]
        pub interactions_consensus_path_db: Vec<String>,

        /// Estimated probability of haploinsufficiency of the gene (from
        /// doi:10.1371/journal.pgen.1001154).
        #[serde(
            alias = "P(HI)",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub haploinsufficiency: Option<f64>,
        /// Estimated probability of haploinsufficiency of the gene (from
        /// doi:10.1093/bioinformatics/btx028).
        #[serde(
            alias = "HIPred_score",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub hipred_score: Option<f64>,
        /// HIPred prediction of haploinsufficiency of the gene. Y(es) or N(o). (from
        /// doi:10.1093/bioinformatics/btx028).
        #[serde(
            alias = "HIPred",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub hipred: Option<String>,
        /// A score predicting the gene haploinsufficiency. The higher the score the more likely the
        /// gene is haploinsufficient (from doi: 10.1093/nar/gkv474).
        #[serde(
            alias = "GHIS",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub ghis: Option<f64>,
        /// Estimated probability that gene is a recessive disease gene (from
        /// DOI:10.1126/science.1215040).
        #[serde(
            alias = "P(rec)",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub prec: Option<f64>,
        /// Known recessive status of the gene (from DOI:10.1126/science.1215040) "lof-tolerant =
        /// seen in homozygous state in at least one 1000G individual" "recessive = known OMIM
        /// recessive disease" (original annotations from DOI:10.1126/science.1215040).
        #[serde(
            alias = "Known_rec_info",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub known_rec_info: Option<String>,
        /// Residual Variation Intolerance Score, a measure of intolerance of mutational burden, the
        /// higher the score the more tolerant to mutational burden the gene is. Based on EVS
        /// (ESP6500) data.  from doi:10.1371/journal.pgen.1003709.
        #[serde(
            alias = "RVIS_EVS",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub rvis_evs: Option<f64>,
        /// The percentile rank of the gene based on RVIS, the higher the percentile the more
        /// tolerant to mutational burden the gene is. Based on EVS (ESP6500) data.
        #[serde(
            alias = "RVIS_percentile_EVS",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub rvis_percentile_evs: Option<f64>,
        /// "A gene's corresponding FDR p-value for preferential LoF depletion among the ExAC
        /// population.  Lower FDR corresponds with genes that are increasingly depleted of LoF
        /// variants." cited from RVIS document.
        #[serde(
            alias = "LoF-FDR_ExAC",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub lof_fdr_exac: Option<f64>,
        /// "ExAC-based RVIS; setting 'common' MAF filter at 0.05% in at least one of the six
        /// individual ethnic strata from ExAC." cited from RVIS document.
        #[serde(
            alias = "RVIS_ExAC",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub rvis_exac: Option<f64>,
        /// "Genome-Wide percentile for the new ExAC-based RVIS; setting 'common' MAF filter at 0.05%
        /// in at least one of the six individual ethnic strata from ExAC." cited from RVIS document.
        #[serde(
            alias = "RVIS_percentile_ExAC",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub rvis_percentile_exac: Option<f64>,
        /// "the probability of being loss-of-function intolerant (intolerant of both heterozygous
        /// and homozygous lof variants)" based on ExAC r0.3 data.
        #[serde(
            alias = "ExAC_pLI",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub exac_pli: Option<f64>,
        /// "the probability of being intolerant of homozygous, but not heterozygous lof variants"
        /// based on ExAC r0.3 data.
        #[serde(
            alias = "ExAC_pRec",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub exac_prec: Option<f64>,
        /// "the probability of being tolerant of both heterozygous and homozygous lof variants"
        /// based on ExAC r0.3 data.
        #[serde(
            alias = "ExAC_pNull",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub exac_pnull: Option<f64>,
        /// "the probability of being loss-of-function intolerant (intolerant of both heterozygous
        /// and homozygous lof variants)" based on ExAC r0.3 nonTCGA subset.
        #[serde(
            alias = "ExAC_nonTCGA_pLI",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub exac_nontcga_pli: Option<f64>,
        /// "the probability of being intolerant of homozygous, but not heterozygous lof variants"
        /// based on ExAC r0.3 nonTCGA subset.
        #[serde(
            alias = "ExAC_nonTCGA_pRec",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub exac_nontcga_prec: Option<f64>,
        /// "the probability of being tolerant of both heterozygous and homozygous lof variants"
        /// based on ExAC r0.3 nonTCGA subset.
        #[serde(
            alias = "ExAC_nonTCGA_pNull",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub exac_nontcga_pnull: Option<f64>,
        /// "the probability of being loss-of-function intolerant (intolerant of both heterozygous
        /// and homozygous lof variants)" based on ExAC r0.3 nonpsych subset.
        #[serde(
            alias = "ExAC_nonpsych_pLI",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub exac_nonpsych_pli: Option<f64>,
        /// "the probability of being intolerant of homozygous, but not heterozygous lof variants"
        /// based on ExAC r0.3 nonpsych subset.
        #[serde(
            alias = "ExAC_nonpsych_pRec",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub exac_nonpsych_prec: Option<f64>,
        /// "the probability of being tolerant of both heterozygous and homozygous lof variants"
        /// based on ExAC r0.3 nonpsych subset/
        #[serde(
            alias = "ExAC_nonpsych_pNull",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub exac_nonpsych_pnull: Option<f64>,
        /// "the probability of being loss-of-function intolerant (intolerant of both heterozygous
        /// and homozygous lof variants)" based on gnomAD 2.1 data.
        #[serde(
            alias = "gnomAD_pLI",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub gnomad_pli: Option<f64>,
        /// "the probability of being intolerant of homozygous, but not heterozygous lof variants"
        /// based on gnomAD 2.1 data.
        #[serde(
            alias = "gnomAD_pRec",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub gnomad_prec: Option<f64>,
        /// "the probability of being tolerant of both heterozygous and homozygous lof variants"
        /// based on gnomAD 2.1 data.
        #[serde(
            alias = "gnomAD_pNull",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub gnomad_pnull: Option<f64>,
        /// "Winsorised deletion intolerance z-score" based on ExAC r0.3.1 CNV data.
        #[serde(
            alias = "ExAC_del.score",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub exac_del_score: Option<f64>,
        /// "Winsorised duplication intolerance z-score" based on ExAC r0.3.1 CNV data.
        #[serde(
            alias = "ExAC_dup.score",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub exac_dup_score: Option<f64>,
        /// "Winsorised cnv intolerance z-score" based on ExAC r0.3.1 CNV data.
        #[serde(
            alias = "ExAC_cnv.score",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub exac_cnv_score: Option<f64>,
        /// "Gene is in a known region of recurrent CNVs mediated by tandem segmental duplications
        /// and intolerance scores are more likely to be biased or noisy." from ExAC r0.3.1 CNV
        /// release.
        #[serde(
            alias = "ExAC_cnv_flag",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub exac_cnv_flag: Option<String>,
        /// gene damage index score, "a genome-wide, gene-level metric of the mutational damage that
        /// has accumulated in the general population" from doi: 10.1073/pnas.1518646112. The higher
        /// the score the less likely the gene is to be responsible for monogenic diseases.
        #[serde(
            alias = "GDI",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub gdi: Option<f64>,
        /// Phred-scaled GDI scores.
        #[serde(
            alias = "GDI-Phred",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub gdi_phred: Option<f64>,
        /// gene damage prediction (low/medium/high) by GDI for all diseases.,
        #[serde(
            alias = "Gene damage prediction (all disease-causing genes)",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub gdp_all_disease_causing: Option<String>,
        /// gene damage prediction (low/medium/high) by GDI for all Mendelian diseases.
        #[serde(
            alias = "Gene damage prediction (all Mendelian disease-causing genes)",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub gdp_all_mendelian: Option<String>,
        /// gene damage prediction (low/medium/high) by GDI for Mendelian autosomal dominant
        /// diseases.
        #[serde(
            alias = "Gene damage prediction (Mendelian AD disease-causing genes)",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub gdp_all_mendelian_ad: Option<String>,
        /// gene damage prediction (low/medium/high) by GDI for Mendelian autosomal recessive
        /// diseases.
        #[serde(
            alias = "Gene damage prediction (Mendelian AR disease-causing genes)",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub gdp_mendelian_ar: Option<String>,
        /// gene damage prediction (low/medium/high) by GDI for all primary immunodeficiency
        /// diseases.
        #[serde(
            alias = "Gene damage prediction (all PID disease-causing genes)",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub gdp_pid: Option<String>,
        /// gene damage prediction (low/medium/high) by GDI for primary immunodeficiency autosomal
        /// dominant diseases.
        #[serde(
            alias = "Gene damage prediction (PID AD disease-causing genes)",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub gdp_pid_ad: Option<String>,
        /// gene damage prediction (low/medium/high) by GDI for primary immunodeficiency autosomal
        /// recessive diseases.
        #[serde(
            alias = "Gene damage prediction (PID AR disease-causing genes)",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub gdp_pid_ar: Option<String>,
        /// gene damage prediction (low/medium/high) by GDI for all cancer disease.
        #[serde(
            alias = "Gene damage prediction (all cancer disease-causing genes)",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub gdp_cancer: Option<String>,
        /// gene damage prediction (low/medium/high) by GDI for cancer recessive disease.
        #[serde(
            alias = "Gene damage prediction (cancer recessive disease-causing genes)",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub gdb_cancer_rec: Option<String>,
        /// gene damage prediction (low/medium/high) by GDI for cancer dominant disease.
        #[serde(
            alias = "Gene damage prediction (cancer dominant disease-causing genes)",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub gdp_cancer_dom: Option<String>,
        /// A percentile score for gene intolerance to functional change. The lower the score the
        /// higher gene intolerance to functional change. For details see doi:
        /// 10.1093/bioinformatics/btv602.
        #[serde(
            alias = "LoFtool_score",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub loftool_score: Option<f64>,
        /// The fraction of individuals in the 1000 Genomes Project data (N=2504) who are either
        /// Heterozygote or Homozygote of LOF SNVs whose MAF<0.005. This fraction is from a method
        /// for ranking genes based on mutational burden called SORVA (Significance Of Rare
        /// VAriants). Please see doi: 10.1101/103218 for details.
        #[serde(
            alias = "SORVA_LOF_MAF0.005_HetOrHom",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub sorva_lof_maf_5_het_or_hom: Option<f64>,
        /// The fraction of individuals in the 1000 Genomes Project data (N=2504) who are either
        /// Compound Heterozygote or Homozygote of LOF SNVs whose MAF<0.005. This fraction is from a
        /// method for ranking genes based on mutational burden called SORVA (Significance Of Rare
        /// VAriants). Please see doi: 10.1101/103218 for details.
        #[serde(
            alias = "SORVA_LOF_MAF0.005_HomOrCompoundHet",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub sorva_lof_maf_5_hom_or_comphet: Option<f64>,
        /// The fraction of individuals in the 1000 Genomes Project data (N=2504) who are either
        /// Heterozygote or Homozygote of LOF SNVs whose MAF<0.001. This fraction is from a method
        /// for ranking genes based on mutational burden called SORVA (Significance Of Rare
        /// VAriants). Please see doi: 10.1101/103218 for details.
        #[serde(
            alias = "SORVA_LOF_MAF0.001_HetOrHom",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub sorva_lof_maf_1_het_or_hom: Option<f64>,
        /// The fraction of individuals in the 1000 Genomes Project data (N=2504) who are either
        /// Compound Heterozygote or Homozygote of LOF SNVs whose MAF<0.001. This fraction is from a
        /// method for ranking genes based on mutational burden called SORVA (Significance Of Rare
        /// VAriants). Please see doi: 10.1101/103218 for details.
        #[serde(
            alias = "SORVA_LOF_MAF0.001_HomOrCompoundHet",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub sorva_lof_maf_1_hom_or_comphet: Option<f64>,
        /// The fraction of individuals in the 1000 Genomes Project data (N=2504) who are either
        /// Heterozygote or Homozygote of LOF or missense SNVs whose MAF<0.005. This fraction is from
        /// a method for ranking genes based on mutational burden called SORVA (Significance Of Rare
        /// VAriants).  Please see doi: 10.1101/103218 for details.
        #[serde(
            alias = "SORVA_LOForMissense_MAF0.005_HetOrHom",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub sorva_lof_or_mis_maf_5_het_or_hom: Option<f64>,
        /// The fraction of individuals in the 1000 Genomes Project data (N=2504) who are either
        /// Compound Heterozygote or Homozygote of LOF or missense SNVs whose MAF<0.005. This
        /// fraction is from a method for ranking genes based on mutational burden called SORVA
        /// (Significance Of Rare VAriants).  Please see doi: 10.1101/103218 for details.
        #[serde(
            alias = "SORVA_LOForMissense_MAF0.005_HomOrCompoundHet",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub sorva_lof_or_mis_maf_5_hom_or_comphet: Option<f64>,
        /// The fraction of individuals in the 1000 Genomes Project data (N=2504) who are either
        /// Heterozygote or Homozygote of LOF or missense SNVs whose MAF<0.001. This fraction is from
        /// a method for ranking genes based on mutational burden called SORVA (Significance Of Rare
        /// VAriants).  Please see doi: 10.1101/103218 for details.
        #[serde(
            alias = "SORVA_LOForMissense_MAF0.001_HetOrHom",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub sorva_lof_or_mis_maf_1_het_or_hom: Option<f64>,
        /// The fraction of individuals in the 1000 Genomes Project data (N=2504) who are either
        /// Compound Heterozygote or Homozygote of LOF or missense SNVs whose MAF<0.001. This
        /// fraction is from a method for ranking genes based on mutational burden called SORVA
        /// (Significance Of Rare VAriants).  Please see doi: 10.1101/103218 for details.
        #[serde(
            alias = "SORVA_LOForMissense_MAF0.001_HomOrCompoundHet",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub sorva_lof_or_mis_maf_1_hom_or_comphet: Option<f64>,
        /// Essential ("E") or Non-essential phenotype-changing ("N") based on Mouse Genome
        /// Informatics database. from doi:10.1371/journal.pgen.1003484.
        #[serde(
            alias = "Essential_gene",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub essential_gene: Option<String>,
        /// Essential ("E") or Non-essential phenotype-changing ("N") based on large scale CRISPR
        /// experiments. from doi: 10.1126/science.aac7041.
        #[serde(
            alias = "Essential_gene_CRISPR",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub essential_gene_crispr: Option<String>,
        /// Essential ("E"), context-Specific essential ("S"), or Non-essential phenotype-changing
        /// ("N") based on large scale CRISPR experiments. from
        /// http://dx.doi.org/10.1016/j.cell.2015.11.015.
        #[serde(
            alias = "Essential_gene_CRISPR2",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub essential_gene_crispr2: Option<String>,
        /// Essential ("E"), HAP1-Specific essential ("H"), KBM7-Specific essential ("K"), or
        /// Non-essential phenotype-changing ("N"), based on large scale mutagenesis experiments.
        /// from doi: 10.1126/science.aac7557.
        #[serde(
            alias = "Essential_gene_gene-trap",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub essential_gene_gene_trap: Option<String>,
        /// A probability prediction of the gene being essential. From
        /// doi:10.1371/journal.pcbi.1002886.
        #[serde(
            alias = "Gene_indispensability_score",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot"
        )]
        pub gene_indispensability_score: Option<f64>,
        /// Essential ("E") or loss-of-function tolerant ("N") based on Gene_indispensability_score.
        #[serde(
            alias = "Gene_indispensability_pred",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub gene_indispensability_pred: Option<String>,
        /// Homolog mouse gene name from MGI.
        #[serde(
            alias = "MGI_mouse_gene",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub mgi_mouse_gene: Option<String>,
        /// Phenotype description for the homolog mouse gene from MGI.
        #[serde(
            alias = "MGI_mouse_phenotype",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub mgi_mouse_phenotype: Option<String>,
        /// Homolog zebrafish gene name from ZFIN.
        #[serde(
            alias = "ZFIN_zebrafish_gene",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub zfin_zebrafish_gene: Option<String>,
        /// Affected structure of the homolog zebrafish gene from ZFIN.
        #[serde(
            alias = "ZFIN_zebrafish_structure",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub zfin_zebrafish_structure: Option<String>,
        /// Phenotype description for the homolog zebrafish gene from ZFIN.
        #[serde(
            alias = "ZFIN_zebrafish_phenotype_quality",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub zfin_zebrafish_phenotype_quality: Option<String>,
        /// Phenotype tag for the homolog zebrafish gene from ZFIN"
        #[serde(
            alias = "ZFIN_zebrafish_phenotype_tag",
            serialize_with = "serialize_option_dot",
            deserialize_with = "deserialize_option_dot_str"
        )]
        pub zfin_zebrafish_phenotype_tag: Option<String>,
    }
}

/// Code for data from the gnomAD constraints.
pub mod gnomad_constraints {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// Deserialize `Option::None` as `"NA"`.
    ///
    /// cf. https://stackoverflow.com/a/56384732/84349
    fn deserialize_option_na<'de, D, T: Deserialize<'de>>(
        deserializer: D,
    ) -> Result<Option<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // We define a local enum type inside of the function because it is untagged, serde will
        // deserialize as the first variant that it can.
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum MaybeNA<U> {
            // If it can be parsed as Option<T>, it will be..
            Value(Option<U>),
            // ... otherwise try parsing as a string.
            NAString(String),
        }

        // Deserialize into local enum.
        let value: MaybeNA<T> = Deserialize::deserialize(deserializer)?;
        match value {
            // If parsed as T or None, return that.
            MaybeNA::Value(value) => Ok(value),

            // Otherwise, if value is string an "n/a", return None (and fail if it is any other
            // string)
            MaybeNA::NAString(string) => {
                if string == "NA" {
                    Ok(None)
                } else {
                    Err(serde::de::Error::custom("Unexpected string"))
                }
            }
        }
    }

    /// Serialize `Option::None` as `"NA"`.
    fn serialize_option_na<S, T>(x: &Option<T>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        match x {
            Some(x) => s.serialize_some(x),
            None => s.serialize_str("NA"),
        }
    }

    /// A record from the gnomAD constraints database.
    #[serde_with::skip_serializing_none]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Record {
        /// The Ensembl gene ID.
        pub ensembl_gene_id: String,
        /// The NCBI gene ID.
        pub entrez_id: String,
        /// The HGNC gene symbol.
        pub gene_symbol: String,
        /// The expected number of loss-of-function variants.
        #[serde(
            serialize_with = "serialize_option_na",
            deserialize_with = "deserialize_option_na"
        )]
        pub exp_lof: Option<f64>,
        /// The expected number of missense variants.
        #[serde(
            serialize_with = "serialize_option_na",
            deserialize_with = "deserialize_option_na"
        )]
        pub exp_mis: Option<f64>,
        /// The expected number of synonymous variants.
        #[serde(
            serialize_with = "serialize_option_na",
            deserialize_with = "deserialize_option_na"
        )]
        pub exp_syn: Option<f64>,
        /// The missense-related Z-score.
        #[serde(
            serialize_with = "serialize_option_na",
            deserialize_with = "deserialize_option_na"
        )]
        pub mis_z: Option<f64>,
        /// The observed number of loss-of-function variants.
        #[serde(
            serialize_with = "serialize_option_na",
            deserialize_with = "deserialize_option_na"
        )]
        pub obs_lof: Option<u32>,
        /// The observed number of missense variants.
        #[serde(
            serialize_with = "serialize_option_na",
            deserialize_with = "deserialize_option_na"
        )]
        pub obs_mis: Option<u32>,
        /// The observed number of synonymous variants.
        #[serde(
            serialize_with = "serialize_option_na",
            deserialize_with = "deserialize_option_na"
        )]
        pub obs_syn: Option<u32>,
        /// The loss-of-function observed/expected ratio.
        #[serde(
            serialize_with = "serialize_option_na",
            deserialize_with = "deserialize_option_na"
        )]
        pub oe_lof: Option<f64>,
        /// The lower bound of the loss-of-function observed/expected ratio.
        #[serde(
            serialize_with = "serialize_option_na",
            deserialize_with = "deserialize_option_na"
        )]
        pub oe_lof_lower: Option<f64>,
        /// The upper bound of the loss-of-function observed/expected ratio.
        #[serde(
            serialize_with = "serialize_option_na",
            deserialize_with = "deserialize_option_na"
        )]
        pub oe_lof_upper: Option<f64>,
        /// The missense observed/expected ratio.
        #[serde(
            serialize_with = "serialize_option_na",
            deserialize_with = "deserialize_option_na"
        )]
        pub oe_mis: Option<f64>,
        /// The lower bound of the missense observed/expected ratio.
        #[serde(
            serialize_with = "serialize_option_na",
            deserialize_with = "deserialize_option_na"
        )]
        pub oe_mis_lower: Option<f64>,
        /// The upper bound of the missense observed/expected ratio.
        #[serde(
            serialize_with = "serialize_option_na",
            deserialize_with = "deserialize_option_na"
        )]
        pub oe_mis_upper: Option<f64>,
        /// The synonymous observed/expected ratio.
        #[serde(
            serialize_with = "serialize_option_na",
            deserialize_with = "deserialize_option_na"
        )]
        pub oe_syn: Option<f64>,
        /// The lower bound of the synonymous observed/expected ratio.
        #[serde(
            serialize_with = "serialize_option_na",
            deserialize_with = "deserialize_option_na"
        )]
        pub oe_syn_lower: Option<f64>,
        /// The upper bound of the synonymous observed/expected ratio.
        #[serde(
            serialize_with = "serialize_option_na",
            deserialize_with = "deserialize_option_na"
        )]
        pub oe_syn_upper: Option<f64>,
        /// The probability of loss-of-function intolerance (pLI score).
        #[serde(
            alias = "pLI",
            serialize_with = "serialize_option_na",
            deserialize_with = "deserialize_option_na"
        )]
        pub pli: Option<f64>,
        /// The synonymous-related Z-score.
        #[serde(
            serialize_with = "serialize_option_na",
            deserialize_with = "deserialize_option_na"
        )]
        pub syn_z: Option<f64>,
        /// The probability of loss-of-function intolerance (pLI score) from ExAC.
        #[serde(
            alias = "exac_pLI",
            serialize_with = "serialize_option_na",
            deserialize_with = "deserialize_option_na"
        )]
        pub exac_pli: Option<f64>,
        /// The observed number of loss-of-function variants from ExAC.
        #[serde(
            serialize_with = "serialize_option_na",
            deserialize_with = "deserialize_option_na"
        )]
        pub exac_obs_lof: Option<f64>,
        /// The expected number of loss-of-function variants from ExAC.
        #[serde(
            serialize_with = "serialize_option_na",
            deserialize_with = "deserialize_option_na"
        )]
        pub exac_exp_lof: Option<f64>,
        /// The loss-of-function observed/expected ratio from ExAC.
        #[serde(
            serialize_with = "serialize_option_na",
            deserialize_with = "deserialize_option_na"
        )]
        pub exac_oe_lof: Option<f64>,
    }
}

/// Code for data from the HGNC database.
pub mod hgnc {
    use std::{fmt::Display, str::FromStr};

    use chrono::naive::NaiveDate;
    use serde::{Deserialize, Serialize};
    use serde_with::DisplayFromStr;

    /// Status of the symbol report, which can be either "Approved" or "Entry Withdrawn".
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Status {
        /// Approved symbol.
        #[serde(rename = "Approved")]
        Approve,
        /// Withdrawn symbol.
        #[serde(rename = "Entry Withdrawn")]
        Withdrawn,
    }

    /// Information from the locus-specific dabase.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Lsdb {
        /// The name of the Locus Specific Mutation Database.
        pub name: String,
        /// The URL for the gene.
        pub url: String,
    }

    impl Display for Lsdb {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}|{}", &self.name, &self.url)
        }
    }

    impl FromStr for Lsdb {
        type Err = anyhow::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let mut vals: Vec<String> = s.splitn(2, '|').map(|s| s.to_string()).collect();
            if vals.len() != 2 {
                anyhow::bail!("invalid LSDB string: {}", s);
            } else {
                let name = vals.pop().unwrap();
                let url = vals.pop().unwrap();
                Ok(Lsdb { name, url })
            }
        }
    }

    /// A record from the HGNC database.
    ///
    /// Also see the [HGNC website](https://www.genenames.org/download/archive/).
    #[serde_with::skip_serializing_none]
    #[serde_with::serde_as]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Record {
        /// HGNC ID. A unique ID created by the HGNC for every approved symbol.
        pub hgnc_id: String,
        /// The HGNC approved gene symbol.
        pub symbol: String,
        /// HGNC approved name for the gene.
        pub name: String,
        /// A group name for a set of related locus types as defined by the HGNC (e.g. non-coding
        /// RNA).
        pub locus_group: Option<String>,
        /// The locus type as defined by the HGNC (e.g. RNA, transfer).
        pub locus_type: Option<String>,
        /// Status of the symbol report.
        pub status: Status,
        /// Cytogenetic location of the gene (e.g. 2q34).
        pub location: Option<String>,
        /// Sortable cytogenic location of the gene (e.g. 02q34).
        pub location_sortable: Option<String>,
        /// Other symbols used to refer to this gene.
        pub alias_symbol: Option<Vec<String>>,
        /// Other names used to refer to this gene.
        pub alias_name: Option<Vec<String>>,
        /// Prevous symbols used to refer to this gene.
        pub prev_symbol: Option<Vec<String>>,
        /// Previous names used to refer to this gene.
        pub prev_name: Option<Vec<String>>,
        /// Name given to a gene group.
        pub gene_group: Option<Vec<String>>,
        /// ID used to designate a gene group.
        pub gene_group_id: Option<Vec<u32>>,
        /// The date the entry was first approved.
        pub date_approved_reserved: Option<NaiveDate>,
        /// The date the gene symbol was last changed.
        pub date_symbol_changed: Option<NaiveDate>,
        /// The date the gene name was last changed.
        pub date_name_changed: Option<NaiveDate>,
        /// Date the entry was last modified.
        pub date_modified: Option<NaiveDate>,
        /// Entrez gene id.
        pub entrez_id: Option<String>,
        /// Ensembl gene id.
        pub ensembl_gene_id: Option<String>,
        /// Vega gene id.
        pub vega_id: Option<String>,
        /// UCSC gene id.
        pub ucsc_id: Option<String>,
        /// ENA accession number(s).
        pub ena: Option<Vec<String>>,
        /// RefSeq nucleotide accession(s).
        pub refseq_accession: Option<Vec<String>>,
        /// Consensus CDS ID(ds).
        pub ccds_id: Option<Vec<String>>,
        /// Uniprot IDs.
        pub uniprot_ids: Option<Vec<String>>,
        /// Pubmed IDs.
        pub pubmed_id: Option<Vec<u32>>,
        /// Mouse genome informatics database ID(s).
        pub mgd_id: Option<Vec<String>>,
        /// Rat genome database gene ID(s).
        pub rgd_id: Option<Vec<String>>,
        /// The name of the Locus Specific Mutation Database and URL for the gene.
        #[serde_as(as = "Option<Vec<DisplayFromStr>>")]
        pub lsdb: Option<Vec<Lsdb>>,
        /// Symbol used within COSMIC.
        pub cosmic: Option<String>,
        /// OMIM ID(s).
        pub omim_id: Option<Vec<String>>,
        /// miRBase ID.
        pub mirbase: Option<String>,
        /// Homeobox Database ID.
        pub homeodb: Option<u32>,
        /// snoRNABase ID.
        pub snornabase: Option<String>,
        /// Symbol used to link to the SLC tables database at bioparadigms.org for the gene.
        pub bioparadigms_slc: Option<String>,
        /// Orphanet ID.
        pub orphanet: Option<u32>,
        /// Pseudogene.org.
        #[serde(rename = "pseudogene.org")]
        pub pseudogene_org: Option<String>,
        /// Symbol used within HORDE for the gene.
        pub horde_id: Option<String>,
        /// ID used to link to the MEROPS peptidase database.
        pub merops: Option<String>,
        /// Symbol used within international ImMunoGeneTics information system.
        pub imgt: Option<String>,
        /// The objectId used to link to the IUPHAR/BPS Guide to PHARMACOLOGY database.
        pub iuphar: Option<String>,
        /// ID to link to the Mamit-tRNA database
        #[serde(rename = "mamit-trnadb")]
        pub mamit_trnadb: Option<u32>,
        /// Symbol used within the Human Cell Differentiation Molecule database.
        pub cd: Option<String>,
        /// lncRNA Database ID.
        pub lncrnadb: Option<String>,
        /// ENZYME EC accession number.
        pub enzyme_id: Option<Vec<String>>,
        /// ID used to link to the Human Intermediate Filament Database.
        pub intermediate_filament_db: Option<String>,
        /// The HGNC ID that the Alliance of Genome Resources (AGR) use.
        pub agr: Option<String>,
        /// NCBI and Ensembl transcript IDs/acessions including the version number.
        pub mane_select: Option<Vec<String>>,
    }
}

/// Code for data from the NCBI gene database (aka "Entrez").
pub mod ncbi {
    use serde::{Deserialize, Serialize};
    use serde_with::DisplayFromStr;

    /// Reference into function record.
    #[serde_with::skip_serializing_none]
    #[serde_with::serde_as]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RifEntry {
        /// The RIF text.
        pub text: String,
        /// PubMed IDs.
        #[serde_as(as = "Option<Vec<DisplayFromStr>>")]
        pub pmids: Option<Vec<u32>>,
    }

    /// A record from the NCBI gene database.
    #[serde_with::skip_serializing_none]
    #[serde_with::serde_as]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Record {
        /// NCBI Gene ID.
        pub gene_id: String,
        /// Gene summary.
        pub summary: Option<String>,
        /// "Reference into Function" entries.
        pub rif_entries: Option<Vec<RifEntry>>,
    }
}

/// Code for reading gene to OMIM disease associations.
pub mod omim {
    use serde::{Deserialize, Serialize};

    /// A single OMIM disease.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct OmimTerm {
        /// The OMIM disease ID.
        pub omim_id: String,
        /// The OMIM disease label.
        pub label: String,
    }

    /// Multiple omim terms for one gene.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Record {
        /// HGNC gene ID.
        pub hgnc_id: String,
        /// The OMIM diseases.
        pub diseases: Vec<OmimTerm>,
    }

    /// A record from the OMIM disease table.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RawRecord {
        /// HGNC gene ID.
        pub hgnc_id: String,
        /// The OMIM disease ID.
        pub omim_id: String,
        /// The OMIM disease label.
        pub disease_name: String,
    }
}

/// Code for data from rCNV (Collins et al., 2022).
pub mod rcnv {
    use serde::{Deserialize, Serialize};

    /// A record from the rCNV table.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Record {
        /// HGNC gene ID.
        pub hgnc_id: String,
        /// The pHaplo value.
        pub p_haplo: f64,
        /// The pTriplo value.
        pub p_triplo: f64,
    }
}

/// Code for data from sHet (Weghorn et al., 2019).
pub mod shet {
    use serde::{Deserialize, Serialize};

    /// A record from the sHet table.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Record {
        /// HGNC gene ID.
        pub hgnc_id: String,
        /// The sHet value.
        pub s_het: f64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_acmg_sf_record() -> Result<(), anyhow::Error> {
        let path_json = "tests/genes/ncbi/gene_info.jsonl";
        let str_json = std::fs::read_to_string(path_json)?;
        let records = str_json
            .lines()
            .map(|s| serde_json::from_str::<ncbi::Record>(s).unwrap())
            .collect::<Vec<_>>();

        insta::assert_yaml_snapshot!(records);

        Ok(())
    }

    #[test]
    fn deserialize_clingen_record() -> Result<(), anyhow::Error> {
        let path_csv = "tests/genes/clingen/clingen.csv";
        let str_csv = std::fs::read_to_string(path_csv)?;
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(str_csv.as_bytes());
        let records = rdr
            .deserialize()
            .collect::<Result<Vec<clingen_gene::Record>, csv::Error>>()?;
        insta::assert_yaml_snapshot!(records);

        Ok(())
    }

    #[test]
    fn deserialize_dbnsfp_record() -> Result<(), anyhow::Error> {
        let path_tsv = "tests/genes/dbnsfp/genes.tsv";
        let str_tsv = std::fs::read_to_string(path_tsv)?;
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(true)
            .from_reader(str_tsv.as_bytes());
        let records = rdr
            .deserialize()
            .collect::<Result<Vec<dbnsfp_gene::Record>, csv::Error>>()?;
        insta::assert_yaml_snapshot!(records);
        Ok(())
    }

    #[test]
    fn deserialize_hgnc_record() -> Result<(), anyhow::Error> {
        let path_json = "tests/genes/hgnc/hgnc_info.jsonl";
        let str_json = std::fs::read_to_string(path_json)?;
        let records = str_json
            .lines()
            .map(|s| serde_json::from_str::<hgnc::Record>(s).unwrap())
            .collect::<Vec<_>>();

        insta::assert_yaml_snapshot!(records);

        Ok(())
    }

    #[test]
    fn deserialize_gnomad_constraints() -> Result<(), anyhow::Error> {
        let path_tsv = "tests/genes/gnomad_constraints/gnomad_constraints.tsv";
        let str_tsv = std::fs::read_to_string(path_tsv)?;
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(true)
            .from_reader(str_tsv.as_bytes());
        let records = rdr
            .deserialize()
            .collect::<Result<Vec<gnomad_constraints::Record>, csv::Error>>()?;
        insta::assert_yaml_snapshot!(records);
        Ok(())
    }

    #[test]
    fn deserialize_omim_record() -> Result<(), anyhow::Error> {
        let path_tsv = "tests/genes/omim/omim_diseases.tsv";
        let str_tsv = std::fs::read_to_string(path_tsv).unwrap();
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(true)
            .from_reader(str_tsv.as_bytes());
        let records = rdr
            .deserialize()
            .collect::<Result<Vec<omim::RawRecord>, csv::Error>>()?;
        insta::assert_yaml_snapshot!(records);

        Ok(())
    }

    #[test]
    fn deserialize_ncbi_record() -> Result<(), anyhow::Error> {
        let path_tsv = "tests/genes/acmg/acmg.tsv";
        let str_tsv = std::fs::read_to_string(path_tsv).unwrap();
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(true)
            .from_reader(str_tsv.as_bytes());
        let records = rdr
            .deserialize()
            .collect::<Result<Vec<acmg_sf::Record>, csv::Error>>()?;
        insta::assert_yaml_snapshot!(records);

        Ok(())
    }

    #[test]
    fn deserialize_rcnv_record() -> Result<(), anyhow::Error> {
        let path_tsv = "tests/genes/rcnv/rcnv.tsv";
        let str_tsv = std::fs::read_to_string(path_tsv).unwrap();
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(true)
            .from_reader(str_tsv.as_bytes());
        let records = rdr
            .deserialize()
            .collect::<Result<Vec<rcnv::Record>, csv::Error>>()?;
        insta::assert_yaml_snapshot!(records);

        Ok(())
    }

    #[test]
    fn deserialize_shet_record() -> Result<(), anyhow::Error> {
        let path_tsv = "tests/genes/shet/shet.tsv";
        let str_tsv = std::fs::read_to_string(path_tsv).unwrap();
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(true)
            .from_reader(str_tsv.as_bytes());
        let records = rdr
            .deserialize()
            .collect::<Result<Vec<shet::Record>, csv::Error>>()?;
        insta::assert_yaml_snapshot!(records);

        Ok(())
    }
}
