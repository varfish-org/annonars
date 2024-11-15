//! Implementation of endpoint `/api/v1/genes/info`.
//!
//! Also includes the implementation of the `/genes/info` endpoint (deprecated).

use actix_web::{
    get,
    web::{self, Data, Json, Path},
};
use prost::Message;

use crate::pbs::genes;

use super::error::CustomError;
use serde_with::{formats::CommaSeparator, StringWithSeparator};

/// Parameters for `handle`.
#[serde_with::skip_serializing_none]
#[serde_with::serde_as]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, utoipa::IntoParams)]
struct GenesInfoQuery {
    /// The HGNC IDs to search for.
    #[serde_as(as = "Option<StringWithSeparator::<CommaSeparator, String>>")]
    pub hgnc_id: Option<Vec<String>>,
}

/// Result for `handle`.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde_with::skip_serializing_none]
struct Container {
    /// The resulting gene information.
    pub genes: indexmap::IndexMap<String, genes::base::Record>,
}

/// Implementation of both endpoints.
async fn handle_impl(
    data: Data<crate::server::run::WebServerData>,
    _path: Path<()>,
    query: web::Query<GenesInfoQuery>,
) -> actix_web::Result<Container, CustomError> {
    let genes_db = data.genes.as_ref().ok_or(CustomError::new(anyhow::anyhow!(
        "genes database not available"
    )))?;
    let cf_genes = genes_db
        .data
        .db
        .cf_handle("genes")
        .expect("no 'genes' column family");
    let mut genes = indexmap::IndexMap::new();
    if let Some(hgnc_id) = query.hgnc_id.as_ref() {
        for hgnc_id in hgnc_id {
            if let Some(raw_buf) = genes_db.data.db.get_cf(&cf_genes, hgnc_id).map_err(|e| {
                CustomError::new(anyhow::anyhow!("problem querying database: {}", e))
            })? {
                let record =
                    genes::base::Record::decode(std::io::Cursor::new(raw_buf)).map_err(|e| {
                        CustomError::new(anyhow::anyhow!("problem decoding value: {}", e))
                    })?;
                genes.insert(hgnc_id.to_string(), record);
            } else {
                tracing::debug!("no such gene: {}", hgnc_id);
            }
        }
    }

    let cf_meta = genes_db
        .data
        .db
        .cf_handle("meta")
        .expect("no 'meta' column family");
    let raw_builder_version = &genes_db
        .data
        .db
        .get_cf(&cf_meta, "builder-version")
        .map_err(|e| CustomError::new(anyhow::anyhow!("problem querying database: {}", e)))?
        .expect("database missing 'builder-version' key?");
    let _builder_version = std::str::from_utf8(raw_builder_version)
        .map_err(|e| CustomError::new(anyhow::anyhow!("problem decoding value: {}", e)))?
        .to_string();

    Ok(Container { genes })
}

/// Query for annotations for one or more genes.
#[get("/genes/info")]
async fn handle(
    data: Data<crate::server::run::WebServerData>,
    _path: Path<()>,
    query: web::Query<GenesInfoQuery>,
) -> actix_web::Result<Json<Container>, CustomError> {
    Ok(Json(handle_impl(data, _path, query).await?))
}

/// Query parameters for `handle_with_openapi()`.
#[serde_with::skip_serializing_none]
#[serde_with::serde_as]
#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::IntoParams, utoipa::ToSchema,
)]
pub struct VersionsInfoQuery {
    /// The HGNC IDs to search for.
    #[serde_as(as = "Option<StringWithSeparator::<CommaSeparator, String>>")]
    pub hgnc_id: Option<Vec<String>>,
}

/// `GenesInfoResponse` and related types.
pub mod response {
    use crate::pbs;

    /// Information from ACMG secondary findings list.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesAcmgSecondaryFindingRecord {
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

    impl From<pbs::genes::base::AcmgSecondaryFindingRecord> for GenesAcmgSecondaryFindingRecord {
        fn from(record: pbs::genes::base::AcmgSecondaryFindingRecord) -> Self {
            Self {
                hgnc_id: record.hgnc_id,
                ensembl_gene_id: record.ensembl_gene_id,
                ncbi_gene_id: record.ncbi_gene_id,
                gene_symbol: record.gene_symbol,
                mim_gene_id: record.mim_gene_id,
                disease_phenotype: record.disease_phenotype,
                disorder_mim: record.disorder_mim,
                phenotype_category: record.phenotype_category,
                inheritance: record.inheritance,
                sf_list_version: record.sf_list_version,
                variants_to_report: record.variants_to_report,
            }
        }
    }

    /// Enumeration for Haploinsufficiency / Triplosensitivity scores.
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
    pub enum GenesClingenDosageScore {
        /// Sufficient evidence for dosage pathogenicity
        SufficientEvidenceAvailable,
        /// Some evidence for dosage pathogenicity
        SomeEvidenceAvailable,
        /// Little evidence for dosage pathogenicity
        LittleEvidence,
        /// No evidence available
        NoEvidenceAvailable,
        /// Gene associated with autosomal recessive phenotype
        Recessive,
        /// Dosage sensitivity unlikely
        Unlikely,
    }

    impl From<pbs::genes::base::ClingenDosageScore> for Option<GenesClingenDosageScore> {
        fn from(score: pbs::genes::base::ClingenDosageScore) -> Self {
            match score {
                pbs::genes::base::ClingenDosageScore::Unknown => None,
                pbs::genes::base::ClingenDosageScore::SufficientEvidenceAvailable => {
                    Some(GenesClingenDosageScore::SufficientEvidenceAvailable)
                }
                pbs::genes::base::ClingenDosageScore::SomeEvidenceAvailable => {
                    Some(GenesClingenDosageScore::SomeEvidenceAvailable)
                }
                pbs::genes::base::ClingenDosageScore::LittleEvidence => {
                    Some(GenesClingenDosageScore::LittleEvidence)
                }
                pbs::genes::base::ClingenDosageScore::NoEvidenceAvailable => {
                    Some(GenesClingenDosageScore::NoEvidenceAvailable)
                }
                pbs::genes::base::ClingenDosageScore::Recessive => {
                    Some(GenesClingenDosageScore::Recessive)
                }
                pbs::genes::base::ClingenDosageScore::Unlikely => {
                    Some(GenesClingenDosageScore::Unlikely)
                }
            }
        }
    }

    /// `ClinGen` gene dosage sensitivity record.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesClingenDosageRecord {
        /// Gene symbol.
        pub gene_symbol: String,
        /// NCBI gene ID.
        pub ncbi_gene_id: String,
        /// Genomic location GRCh37.
        pub genomic_location_37: String,
        /// Genomic location GRCh38.
        pub genomic_location_38: String,
        /// Haploinsufficiency score.
        pub haploinsufficiency_score: Option<GenesClingenDosageScore>,
        /// Triplosensitivity score.
        pub triplosensitivity_score: Option<GenesClingenDosageScore>,
        /// Haploinsufficiency Disease ID.
        pub haploinsufficiency_disease_id: Option<String>,
        /// Haploinsufficiency Disease ID.
        pub triplosensitivity_disease_id: Option<String>,
    }

    impl TryFrom<pbs::genes::base::ClingenDosageRecord> for GenesClingenDosageRecord {
        type Error = anyhow::Error;

        fn try_from(record: pbs::genes::base::ClingenDosageRecord) -> Result<Self, Self::Error> {
            Ok(Self {
                gene_symbol: record.gene_symbol,
                ncbi_gene_id: record.ncbi_gene_id,
                genomic_location_37: record.genomic_location_37,
                genomic_location_38: record.genomic_location_38,
                haploinsufficiency_score: pbs::genes::base::ClingenDosageScore::try_from(
                    record.haploinsufficiency_score,
                )?
                .into(),
                triplosensitivity_score: pbs::genes::base::ClingenDosageScore::try_from(
                    record.triplosensitivity_score,
                )?
                .into(),
                haploinsufficiency_disease_id: record.haploinsufficiency_disease_id,
                triplosensitivity_disease_id: record.triplosensitivity_disease_id,
            })
        }
    }

    /// Decipher HI Predictions
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesDecipherHiRecord {
        /// HGNC identifier.
        pub hgnc_id: String,
        /// Official HGNC gene symbol.
        pub hgnc_symbol: String,
        /// P(HI) prediction from DECIPHER HI.
        pub p_hi: f64,
        /// Percent HI index.
        pub hi_index: f64,
    }

    impl From<pbs::genes::base::DecipherHiRecord> for GenesDecipherHiRecord {
        fn from(record: pbs::genes::base::DecipherHiRecord) -> Self {
            Self {
                hgnc_id: record.hgnc_id,
                hgnc_symbol: record.hgnc_symbol,
                p_hi: record.p_hi,
                hi_index: record.hi_index,
            }
        }
    }

    /// Information from DOMINO.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesDominoRecord {
        /// Gene symbol.
        pub gene_symbol: String,
        /// The score.
        pub score: f64,
    }

    impl From<pbs::genes::base::DominoRecord> for GenesDominoRecord {
        fn from(record: pbs::genes::base::DominoRecord) -> Self {
            Self {
                gene_symbol: record.gene_symbol,
                score: record.score,
            }
        }
    }

    /// Code for data from the dbNSFP database.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesDbnsfpRecord {
        /// Gene symbol from HGNC.
        pub gene_name: String,
        /// Ensembl gene id (from HGNC).
        pub ensembl_gene: Option<String>,
        /// Chromosome number (from HGNC).
        pub chr: Option<String>,
        /// Old gene symbol (from HGNC).
        pub gene_old_names: Vec<String>,
        /// Other gene names (from HGNC).
        pub gene_other_names: Vec<String>,
        /// Uniprot acc (from HGNC).
        pub uniprot_acc: Option<String>,
        /// Uniprot id (from HGNC).
        pub uniprot_id: Option<String>,
        /// Uniprot id (from HGNC).
        pub entrez_gene_id: Option<String>,
        /// CCDS id (from HGNC).
        pub ccds_id: Vec<String>,
        /// Refseq gene id (from HGNC).
        pub refseq_id: Vec<String>,
        /// UCSC gene id (from HGNC).
        pub ucsc_id: Option<String>,
        /// MIM gene id (from OMIM).
        pub mim_id: Vec<String>,
        /// MIM gene id from OMIM.
        pub omim_id: Vec<String>,
        /// Gene full name (from HGNC).
        pub gene_full_name: Option<String>,
        /// Pathway description from Uniprot.
        pub pathway_uniprot: Option<String>,
        /// Short name of the Pathway(s) the gene belongs to (from BioCarta).
        pub pathway_biocarta_short: Vec<String>,
        /// Full name(s) of the Pathway(s) the gene belongs to (from BioCarta).
        pub pathway_biocarta_full: Vec<String>,
        /// Pathway(s) the gene belongs to (from ConsensusPathDB).
        pub pathway_consensus_path_db: Vec<String>,
        /// ID(s) of the Pathway(s) the gene belongs to (from KEGG).
        pub pathway_kegg_id: Vec<String>,
        /// Full name(s) of the Pathway(s) the gene belongs to (from KEGG).
        pub pathway_kegg_full: Vec<String>,
        /// Function description of the gene (from Uniprot).
        pub function_description: Vec<String>,
        /// Disease(s) the gene caused or associated with (from Uniprot).
        pub disease_description: Vec<String>,
        /// MIM id(s) of the phenotype the gene caused or associated with (from Uniprot).
        pub mim_phenotype_id: Vec<String>,
        /// MIM disease name(s) with MIM id(s) in \[\] (from Uniprot).
        pub mim_disease: Vec<String>,
        /// Orphanet Number of the disorder the gene caused or associated with.
        pub orphanet_disorder_id: Vec<String>,
        /// Disorder name from Orphanet.
        pub orphanet_disorder: Vec<String>,
        /// The type of association beteen the gene and the disorder in Orphanet.
        pub orphanet_association_type: Vec<String>,
        /// Trait(s) the gene associated with (from GWAS catalog).
        pub trait_association_gwas: Vec<String>,
        /// ID of the mapped Human Phenotype Ontology.
        pub hpo_id: Vec<String>,
        /// Name of the mapped Human Phenotype Ontology.
        pub hpo_name: Vec<String>,
        /// GO terms for biological process.
        pub go_biological_process: Vec<String>,
        /// GO terms for cellular component.
        pub go_cellular_component: Vec<String>,
        /// GO terms for molecular function.
        pub go_molecular_function: Vec<String>,
        /// Tissue specificity description from Uniprot.
        pub tissue_specificity_uniprot: Vec<String>,
        /// Tissues/organs the gene expressed in (egenetics data from BioMart).
        pub expression_egenetics: Vec<String>,
        /// Tissues/organs the gene expressed in (GNF/Atlas data from BioMart).
        pub expression_gnf_atlas: Vec<String>,
        /// The interacting genes from IntAct.
        pub interactions_intact: Vec<String>,
        /// The interacting genes from BioGRID.
        pub interactions_biogrid: Vec<String>,
        /// The interacting genes from ConsensusPathDB.
        pub interactions_consensus_path_db: Vec<String>,
        /// Estimated probability of haploinsufficiency of the gene (from
        /// doi:10.1371/journal.pgen.1001154).
        pub haploinsufficiency: Option<f64>,
        /// Estimated probability of haploinsufficiency of the gene (from
        /// doi:10.1093/bioinformatics/btx028).
        pub hipred_score: Option<f64>,
        /// HIPred prediction of haploinsufficiency of the gene. Y(es) or N(o). (from
        /// doi:10.1093/bioinformatics/btx028).
        pub hipred: Option<String>,
        /// A score predicting the gene haploinsufficiency. The higher the score the more likely the
        /// gene is haploinsufficient (from doi: 10.1093/nar/gkv474).
        pub ghis: Option<f64>,
        /// Estimated probability that gene is a recessive disease gene (from
        /// DOI:10.1126/science.1215040).
        pub prec: Option<f64>,
        /// Known recessive status of the gene (from DOI:10.1126/science.1215040) "lof-tolerant =
        /// seen in homozygous state in at least one 1000G individual" "recessive = known OMIM
        /// recessive disease" (original annotations from DOI:10.1126/science.1215040).
        pub known_rec_info: Option<String>,
        /// Residual Variation Intolerance Score, a measure of intolerance of mutational burden, the
        /// higher the score the more tolerant to mutational burden the gene is. Based on EVS
        /// (ESP6500) data.  from doi:10.1371/journal.pgen.1003709.
        pub rvis_evs: Option<f64>,
        /// The percentile rank of the gene based on RVIS, the higher the percentile the more
        /// tolerant to mutational burden the gene is. Based on EVS (ESP6500) data.
        pub rvis_percentile_evs: Option<f64>,
        /// "A gene's corresponding FDR p-value for preferential LoF depletion among the ExAC
        /// population.  Lower FDR corresponds with genes that are increasingly depleted of LoF
        /// variants." cited from RVIS document.
        pub lof_fdr_exac: Option<f64>,
        /// "ExAC-based RVIS; setting 'common' MAF filter at 0.05% in at least one of the six
        /// individual ethnic strata from ExAC." cited from RVIS document.
        pub rvis_exac: Option<f64>,
        /// "Genome-Wide percentile for the new ExAC-based RVIS; setting 'common' MAF filter at 0.05%
        /// in at least one of the six individual ethnic strata from ExAC." cited from RVIS document.
        pub rvis_percentile_exac: Option<f64>,
        /// "the probability of being loss-of-function intolerant (intolerant of both heterozygous
        /// and homozygous lof variants)" based on ExAC r0.3 data.
        pub exac_pli: Option<f64>,
        /// "the probability of being intolerant of homozygous, but not heterozygous lof variants"
        /// based on ExAC r0.3 data.
        pub exac_prec: Option<f64>,
        /// "the probability of being tolerant of both heterozygous and homozygous lof variants"
        /// based on ExAC r0.3 data.
        pub exac_pnull: Option<f64>,
        /// "the probability of being loss-of-function intolerant (intolerant of both heterozygous
        /// and homozygous lof variants)" based on ExAC r0.3 nonTCGA subset.
        pub exac_nontcga_pli: Option<f64>,
        /// "the probability of being intolerant of homozygous, but not heterozygous lof variants"
        /// based on ExAC r0.3 nonTCGA subset.
        pub exac_nontcga_prec: Option<f64>,
        /// "the probability of being tolerant of both heterozygous and homozygous lof variants"
        /// based on ExAC r0.3 nonTCGA subset.
        pub exac_nontcga_pnull: Option<f64>,
        /// "the probability of being loss-of-function intolerant (intolerant of both heterozygous
        /// and homozygous lof variants)" based on ExAC r0.3 nonpsych subset.
        pub exac_nonpsych_pli: Option<f64>,
        /// "the probability of being intolerant of homozygous, but not heterozygous lof variants"
        /// based on ExAC r0.3 nonpsych subset.
        pub exac_nonpsych_prec: Option<f64>,
        /// "the probability of being tolerant of both heterozygous and homozygous lof variants"
        /// based on ExAC r0.3 nonpsych subset/
        pub exac_nonpsych_pnull: Option<f64>,
        /// "the probability of being loss-of-function intolerant (intolerant of both heterozygous
        /// and homozygous lof variants)" based on gnomAD 2.1 data.
        pub gnomad_pli: Option<f64>,
        /// "the probability of being intolerant of homozygous, but not heterozygous lof variants"
        /// based on gnomAD 2.1 data.
        pub gnomad_prec: Option<f64>,
        /// "the probability of being tolerant of both heterozygous and homozygous lof variants"
        /// based on gnomAD 2.1 data.
        pub gnomad_pnull: Option<f64>,
        /// "Winsorised deletion intolerance z-score" based on ExAC r0.3.1 CNV data.
        pub exac_del_score: Option<f64>,
        /// "Winsorised duplication intolerance z-score" based on ExAC r0.3.1 CNV data.
        pub exac_dup_score: Option<f64>,
        /// "Winsorised cnv intolerance z-score" based on ExAC r0.3.1 CNV data.
        pub exac_cnv_score: Option<f64>,
        /// "Gene is in a known region of recurrent CNVs mediated by tandem segmental duplications
        /// and intolerance scores are more likely to be biased or noisy." from ExAC r0.3.1 CNV
        /// release.
        pub exac_cnv_flag: Option<String>,
        /// gene damage index score, "a genome-wide, gene-level metric of the mutational damage that
        /// has accumulated in the general population" from doi: 10.1073/pnas.1518646112. The higher
        /// the score the less likely the gene is to be responsible for monogenic diseases.
        pub gdi: Option<f64>,
        /// Phred-scaled GDI scores.
        pub gdi_phred: Option<f64>,
        /// gene damage prediction (low/medium/high) by GDI for all diseases.,
        pub gdp_all_disease_causing: Option<String>,
        /// gene damage prediction (low/medium/high) by GDI for all Mendelian diseases.
        pub gdp_all_mendelian: Option<String>,
        /// gene damage prediction (low/medium/high) by GDI for Mendelian autosomal dominant
        /// diseases.
        pub gdp_all_mendelian_ad: Option<String>,
        /// gene damage prediction (low/medium/high) by GDI for Mendelian autosomal recessive
        /// diseases.
        pub gdp_mendelian_ar: Option<String>,
        /// gene damage prediction (low/medium/high) by GDI for all primary immunodeficiency
        /// diseases.
        pub gdp_pid: Option<String>,
        /// gene damage prediction (low/medium/high) by GDI for primary immunodeficiency autosomal
        /// dominant diseases.
        pub gdp_pid_ad: Option<String>,
        /// gene damage prediction (low/medium/high) by GDI for primary immunodeficiency autosomal
        /// recessive diseases.
        pub gdp_pid_ar: Option<String>,
        /// gene damage prediction (low/medium/high) by GDI for all cancer disease.
        pub gdp_cancer: Option<String>,
        /// gene damage prediction (low/medium/high) by GDI for cancer recessive disease.
        pub gdb_cancer_rec: Option<String>,
        /// gene damage prediction (low/medium/high) by GDI for cancer dominant disease.
        pub gdp_cancer_dom: Option<String>,
        /// A percentile score for gene intolerance to functional change. The lower the score the
        /// higher gene intolerance to functional change. For details see doi:
        /// 10.1093/bioinformatics/btv602.
        pub loftool_score: Option<f64>,
        /// The fraction of individuals in the 1000 Genomes Project data (N=2504) who are either
        /// Heterozygote or Homozygote of LOF SNVs whose MAF<0.005. This fraction is from a method
        /// for ranking genes based on mutational burden called SORVA (Significance Of Rare
        /// VAriants). Please see doi: 10.1101/103218 for details.
        pub sorva_lof_maf_5_het_or_hom: Option<f64>,
        /// The fraction of individuals in the 1000 Genomes Project data (N=2504) who are either
        /// Compound Heterozygote or Homozygote of LOF SNVs whose MAF<0.005. This fraction is from a
        /// method for ranking genes based on mutational burden called SORVA (Significance Of Rare
        /// VAriants). Please see doi: 10.1101/103218 for details.
        pub sorva_lof_maf_5_hom_or_comphet: Option<f64>,
        /// The fraction of individuals in the 1000 Genomes Project data (N=2504) who are either
        /// Heterozygote or Homozygote of LOF SNVs whose MAF<0.001. This fraction is from a method
        /// for ranking genes based on mutational burden called SORVA (Significance Of Rare
        /// VAriants). Please see doi: 10.1101/103218 for details.
        pub sorva_lof_maf_1_het_or_hom: Option<f64>,
        /// The fraction of individuals in the 1000 Genomes Project data (N=2504) who are either
        /// Compound Heterozygote or Homozygote of LOF SNVs whose MAF<0.001. This fraction is from a
        /// method for ranking genes based on mutational burden called SORVA (Significance Of Rare
        /// VAriants). Please see doi: 10.1101/103218 for details.
        pub sorva_lof_maf_1_hom_or_comphet: Option<f64>,
        /// The fraction of individuals in the 1000 Genomes Project data (N=2504) who are either
        /// Heterozygote or Homozygote of LOF or missense SNVs whose MAF<0.005. This fraction is from
        /// a method for ranking genes based on mutational burden called SORVA (Significance Of Rare
        /// VAriants).  Please see doi: 10.1101/103218 for details.
        pub sorva_lof_or_mis_maf_5_het_or_hom: Option<f64>,
        /// The fraction of individuals in the 1000 Genomes Project data (N=2504) who are either
        /// Compound Heterozygote or Homozygote of LOF or missense SNVs whose MAF<0.005. This
        /// fraction is from a method for ranking genes based on mutational burden called SORVA
        /// (Significance Of Rare VAriants).  Please see doi: 10.1101/103218 for details.
        pub sorva_lof_or_mis_maf_5_hom_or_comphet: Option<f64>,
        /// The fraction of individuals in the 1000 Genomes Project data (N=2504) who are either
        /// Heterozygote or Homozygote of LOF or missense SNVs whose MAF<0.001. This fraction is from
        /// a method for ranking genes based on mutational burden called SORVA (Significance Of Rare
        /// VAriants).  Please see doi: 10.1101/103218 for details.
        pub sorva_lof_or_mis_maf_1_het_or_hom: Option<f64>,
        /// The fraction of individuals in the 1000 Genomes Project data (N=2504) who are either
        /// Compound Heterozygote or Homozygote of LOF or missense SNVs whose MAF<0.001. This
        /// fraction is from a method for ranking genes based on mutational burden called SORVA
        /// (Significance Of Rare VAriants).  Please see doi: 10.1101/103218 for details.
        pub sorva_lof_or_mis_maf_1_hom_or_comphet: Option<f64>,
        /// Essential ("E") or Non-essential phenotype-changing ("N") based on Mouse Genome
        /// Informatics database. from doi:10.1371/journal.pgen.1003484.
        pub essential_gene: Option<String>,
        /// Essential ("E") or Non-essential phenotype-changing ("N") based on large scale CRISPR
        /// experiments. from doi: 10.1126/science.aac7041.
        pub essential_gene_crispr: Option<String>,
        /// Essential ("E"), context-Specific essential ("S"), or Non-essential phenotype-changing
        /// ("N") based on large scale CRISPR experiments. from
        /// <http://dx.doi.org/10.1016/j.cell.2015.11.015.>
        pub essential_gene_crispr2: Option<String>,
        /// Essential ("E"), HAP1-Specific essential ("H"), KBM7-Specific essential ("K"), or
        /// Non-essential phenotype-changing ("N"), based on large scale mutagenesis experiments.
        /// from doi: 10.1126/science.aac7557.
        pub essential_gene_gene_trap: Option<String>,
        /// A probability prediction of the gene being essential. From
        /// doi:10.1371/journal.pcbi.1002886.
        pub gene_indispensability_score: Option<f64>,
        /// Essential ("E") or loss-of-function tolerant ("N") based on Gene_indispensability_score.
        pub gene_indispensability_pred: Option<String>,
        /// Homolog mouse gene name from MGI.
        pub mgi_mouse_gene: Option<String>,
        /// Phenotype description for the homolog mouse gene from MGI.
        pub mgi_mouse_phenotype: Option<String>,
        /// Homolog zebrafish gene name from ZFIN.
        pub zfin_zebrafish_gene: Option<String>,
        /// Affected structure of the homolog zebrafish gene from ZFIN.
        pub zfin_zebrafish_structure: Option<String>,
        /// Phenotype description for the homolog zebrafish gene from ZFIN.
        pub zfin_zebrafish_phenotype_quality: Option<String>,
        /// Phenotype tag for the homolog zebrafish gene from ZFIN"
        pub zfin_zebrafish_phenotype_tag: Option<String>,
    }

    impl From<pbs::genes::base::DbnsfpRecord> for GenesDbnsfpRecord {
        fn from(record: pbs::genes::base::DbnsfpRecord) -> Self {
            Self {
                gene_name: record.gene_name,
                ensembl_gene: record.ensembl_gene,
                chr: record.chr,
                gene_old_names: record.gene_old_names,
                gene_other_names: record.gene_other_names,
                uniprot_acc: record.uniprot_acc,
                uniprot_id: record.uniprot_id,
                entrez_gene_id: record.entrez_gene_id,
                ccds_id: record.ccds_id,
                refseq_id: record.refseq_id,
                ucsc_id: record.ucsc_id,
                mim_id: record.mim_id,
                omim_id: record.omim_id,
                gene_full_name: record.gene_full_name,
                pathway_uniprot: record.pathway_uniprot,
                pathway_biocarta_short: record.pathway_biocarta_short,
                pathway_biocarta_full: record.pathway_biocarta_full,
                pathway_consensus_path_db: record.pathway_consensus_path_db,
                pathway_kegg_id: record.pathway_kegg_id,
                pathway_kegg_full: record.pathway_kegg_full,
                function_description: record.function_description,
                disease_description: record.disease_description,
                mim_phenotype_id: record.mim_phenotype_id,
                mim_disease: record.mim_disease,
                orphanet_disorder_id: record.orphanet_disorder_id,
                orphanet_disorder: record.orphanet_disorder,
                orphanet_association_type: record.orphanet_association_type,
                trait_association_gwas: record.trait_association_gwas,
                hpo_id: record.hpo_id,
                hpo_name: record.hpo_name,
                go_biological_process: record.go_biological_process,
                go_cellular_component: record.go_cellular_component,
                go_molecular_function: record.go_molecular_function,
                tissue_specificity_uniprot: record.tissue_specificity_uniprot,
                expression_egenetics: record.expression_egenetics,
                expression_gnf_atlas: record.expression_gnf_atlas,
                interactions_intact: record.interactions_intact,
                interactions_biogrid: record.interactions_biogrid,
                interactions_consensus_path_db: record.interactions_consensus_path_db,
                haploinsufficiency: record.haploinsufficiency,
                hipred_score: record.hipred_score,
                hipred: record.hipred,
                ghis: record.ghis,
                prec: record.prec,
                known_rec_info: record.known_rec_info,
                rvis_evs: record.rvis_evs,
                rvis_percentile_evs: record.rvis_percentile_evs,
                lof_fdr_exac: record.lof_fdr_exac,
                rvis_exac: record.rvis_exac,
                rvis_percentile_exac: record.rvis_percentile_exac,
                exac_pli: record.exac_pli,
                exac_prec: record.exac_prec,
                exac_pnull: record.exac_pnull,
                exac_nontcga_pli: record.exac_nontcga_pli,
                exac_nontcga_prec: record.exac_nontcga_prec,
                exac_nontcga_pnull: record.exac_nontcga_pnull,
                exac_nonpsych_pli: record.exac_nonpsych_pli,
                exac_nonpsych_prec: record.exac_nonpsych_prec,
                exac_nonpsych_pnull: record.exac_nonpsych_pnull,
                gnomad_pli: record.gnomad_pli,
                gnomad_prec: record.gnomad_prec,
                gnomad_pnull: record.gnomad_pnull,
                exac_del_score: record.exac_del_score,
                exac_dup_score: record.exac_dup_score,
                exac_cnv_score: record.exac_cnv_score,
                exac_cnv_flag: record.exac_cnv_flag,
                gdi: record.gdi,
                gdi_phred: record.gdi_phred,
                gdp_all_disease_causing: record.gdp_all_disease_causing,
                gdp_all_mendelian: record.gdp_all_mendelian,
                gdp_all_mendelian_ad: record.gdp_all_mendelian_ad,
                gdp_mendelian_ar: record.gdp_mendelian_ar,
                gdp_pid: record.gdp_pid,
                gdp_pid_ad: record.gdp_pid_ad,
                gdp_pid_ar: record.gdp_pid_ar,
                gdp_cancer: record.gdp_cancer,
                gdb_cancer_rec: record.gdb_cancer_rec,
                gdp_cancer_dom: record.gdp_cancer_dom,
                loftool_score: record.loftool_score,
                sorva_lof_maf_5_het_or_hom: record.sorva_lof_maf_5_het_or_hom,
                sorva_lof_maf_5_hom_or_comphet: record.sorva_lof_maf_5_hom_or_comphet,
                sorva_lof_maf_1_het_or_hom: record.sorva_lof_maf_1_het_or_hom,
                sorva_lof_maf_1_hom_or_comphet: record.sorva_lof_maf_1_hom_or_comphet,
                sorva_lof_or_mis_maf_5_het_or_hom: record.sorva_lof_or_mis_maf_5_het_or_hom,
                sorva_lof_or_mis_maf_5_hom_or_comphet: record.sorva_lof_or_mis_maf_5_hom_or_comphet,
                sorva_lof_or_mis_maf_1_het_or_hom: record.sorva_lof_or_mis_maf_1_het_or_hom,
                sorva_lof_or_mis_maf_1_hom_or_comphet: record.sorva_lof_or_mis_maf_1_hom_or_comphet,
                essential_gene: record.essential_gene,
                essential_gene_crispr: record.essential_gene_crispr,
                essential_gene_crispr2: record.essential_gene_crispr2,
                essential_gene_gene_trap: record.essential_gene_gene_trap,
                gene_indispensability_score: record.gene_indispensability_score,
                gene_indispensability_pred: record.gene_indispensability_pred,
                mgi_mouse_gene: record.mgi_mouse_gene,
                mgi_mouse_phenotype: record.mgi_mouse_phenotype,
                zfin_zebrafish_gene: record.zfin_zebrafish_gene,
                zfin_zebrafish_structure: record.zfin_zebrafish_structure,
                zfin_zebrafish_phenotype_quality: record.zfin_zebrafish_phenotype_quality,
                zfin_zebrafish_phenotype_tag: record.zfin_zebrafish_phenotype_tag,
            }
        }
    }

    /// Code for data from the gnomAD constraints.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesGnomadConstraintsRecord {
        /// The Ensembl gene ID.
        pub ensembl_gene_id: String,
        /// The NCBI gene ID.
        pub entrez_id: String,
        /// The HGNC gene symbol.
        pub gene_symbol: String,
        /// The expected number of loss-of-function variants.
        pub exp_lof: Option<f64>,
        /// The expected number of missense variants.
        pub exp_mis: Option<f64>,
        /// The expected number of synonymous variants.
        pub exp_syn: Option<f64>,
        /// The missense-related Z-score.
        pub mis_z: Option<f64>,
        /// The observed number of loss-of-function variants.
        pub obs_lof: Option<u32>,
        /// The observed number of missense variants.
        pub obs_mis: Option<u32>,
        /// The observed number of synonymous variants.
        pub obs_syn: Option<u32>,
        /// The loss-of-function observed/expected ratio.
        pub oe_lof: Option<f64>,
        /// The lower bound of the loss-of-function observed/expected ratio.
        pub oe_lof_lower: Option<f64>,
        /// The upper bound of the loss-of-function observed/expected ratio.
        pub oe_lof_upper: Option<f64>,
        /// The missense observed/expected ratio.
        pub oe_mis: Option<f64>,
        /// The lower bound of the missense observed/expected ratio.
        pub oe_mis_lower: Option<f64>,
        /// The upper bound of the missense observed/expected ratio.
        pub oe_mis_upper: Option<f64>,
        /// The synonymous observed/expected ratio.
        pub oe_syn: Option<f64>,
        /// The lower bound of the synonymous observed/expected ratio.
        pub oe_syn_lower: Option<f64>,
        /// The upper bound of the synonymous observed/expected ratio.
        pub oe_syn_upper: Option<f64>,
        /// The probability of loss-of-function intolerance (pLI score).
        pub pli: Option<f64>,
        /// The synonymous-related Z-score.
        pub syn_z: Option<f64>,
        /// The probability of loss-of-function intolerance (pLI score) from ExAC.
        pub exac_pli: Option<f64>,
        /// The observed number of loss-of-function variants from ExAC.
        pub exac_obs_lof: Option<f64>,
        /// The expected number of loss-of-function variants from ExAC.
        pub exac_exp_lof: Option<f64>,
        /// The loss-of-function observed/expected ratio from ExAC.
        pub exac_oe_lof: Option<f64>,
    }

    impl From<pbs::genes::base::GnomadConstraintsRecord> for GenesGnomadConstraintsRecord {
        fn from(record: pbs::genes::base::GnomadConstraintsRecord) -> Self {
            Self {
                ensembl_gene_id: record.ensembl_gene_id,
                entrez_id: record.entrez_id,
                gene_symbol: record.gene_symbol,
                exp_lof: record.exp_lof,
                exp_mis: record.exp_mis,
                exp_syn: record.exp_syn,
                mis_z: record.mis_z,
                obs_lof: record.obs_lof,
                obs_mis: record.obs_mis,
                obs_syn: record.obs_syn,
                oe_lof: record.oe_lof,
                oe_lof_lower: record.oe_lof_lower,
                oe_lof_upper: record.oe_lof_upper,
                oe_mis: record.oe_mis,
                oe_mis_lower: record.oe_mis_lower,
                oe_mis_upper: record.oe_mis_upper,
                oe_syn: record.oe_syn,
                oe_syn_lower: record.oe_syn_lower,
                oe_syn_upper: record.oe_syn_upper,
                pli: record.pli,
                syn_z: record.syn_z,
                exac_pli: record.exac_pli,
                exac_obs_lof: record.exac_obs_lof,
                exac_exp_lof: record.exac_exp_lof,
                exac_oe_lof: record.exac_oe_lof,
            }
        }
    }

    /// Information from the locus-specific dabase.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesHgncLsdb {
        /// The name of the Locus Specific Mutation Database.
        pub name: String,
        /// The URL for the gene.
        pub url: String,
    }

    impl From<pbs::genes::base::HgncLsdb> for GenesHgncLsdb {
        fn from(record: pbs::genes::base::HgncLsdb) -> Self {
            Self {
                name: record.name,
                url: record.url,
            }
        }
    }

    /// A record from the HGNC database.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesHgncRecord {
        /// HGNC ID. A unique ID created by the HGNC for every approved symbol.
        pub hgnc_id: String,
        /// The HGNC approved gene symbol.
        pub symbol: String,
        /// HGNC approved name for the gene.
        pub name: String,
        /// A group name for a set of related locus types as defined by the HGNC
        /// (e.g. non-coding RNA).
        pub locus_group: Option<String>,
        /// The locus type as defined by the HGNC (e.g. RNA, transfer).
        pub locus_type: Option<String>,
        /// Status of the symbol report.
        pub status: i32,
        /// Cytogenetic location of the gene (e.g. 2q34).
        pub location: Option<String>,
        /// Sortable cytogenic location of the gene (e.g. 02q34).
        pub location_sortable: Option<String>,
        /// Other symbols used to refer to this gene.
        pub alias_symbol: Vec<String>,
        /// Other names used to refer to this gene.
        pub alias_name: Vec<String>,
        /// Prevous symbols used to refer to this gene.
        pub prev_symbol: Vec<String>,
        /// Previous names used to refer to this gene.
        pub prev_name: Vec<String>,
        /// Name given to a gene group.
        pub gene_group: Vec<String>,
        /// ID used to designate a gene group.
        pub gene_group_id: Vec<u32>,
        /// The date the entry was first approved.
        pub date_approved_reserved: Option<String>,
        /// The date the gene symbol was last changed.
        pub date_symbol_changed: Option<String>,
        /// The date the gene name was last changed.
        pub date_name_changed: Option<String>,
        /// Date the entry was last modified.
        pub date_modified: Option<String>,
        /// Entrez gene id.
        pub entrez_id: Option<String>,
        /// Ensembl gene id.
        pub ensembl_gene_id: Option<String>,
        /// Vega gene id.
        pub vega_id: Option<String>,
        /// UCSC gene id.
        pub ucsc_id: Option<String>,
        /// ENA accession number(s).
        pub ena: Vec<String>,
        /// RefSeq nucleotide accession(s).
        pub refseq_accession: Vec<String>,
        /// Consensus CDS ID(ds).
        pub ccds_id: Vec<String>,
        /// Uniprot IDs.
        pub uniprot_ids: Vec<String>,
        /// Pubmed IDs.
        pub pubmed_id: Vec<u32>,
        /// Mouse genome informatics database ID(s).
        pub mgd_id: Vec<String>,
        /// Rat genome database gene ID(s).
        pub rgd_id: Vec<String>,
        /// The name of the Locus Specific Mutation Database and URL for the gene.
        pub lsdb: Vec<GenesHgncLsdb>,
        /// Symbol used within COSMIC.
        pub cosmic: Option<String>,
        /// OMIM ID(s).
        pub omim_id: Vec<String>,
        /// miRBase ID.
        pub mirbase: Option<String>,
        /// Homeobox Database ID.
        pub homeodb: Option<u32>,
        /// snoRNABase ID.
        pub snornabase: Option<String>,
        /// Symbol used to link to the SLC tables database at bioparadigms.org
        /// for the gene.
        pub bioparadigms_slc: Option<String>,
        /// Orphanet ID.
        pub orphanet: Option<u32>,
        /// Pseudogene.org.
        pub pseudogene_org: Option<String>,
        /// Symbol used within HORDE for the gene.
        pub horde_id: Option<String>,
        /// ID used to link to the MEROPS peptidase database.
        pub merops: Option<String>,
        /// Symbol used within international ImMunoGeneTics information system.
        pub imgt: Option<String>,
        /// The objectId used to link to the IUPHAR/BPS Guide to PHARMACOLOGY
        /// database.
        pub iuphar: Option<String>,
        /// Symbol used within the Human Cell Differentiation Molecule database.
        pub cd: Option<String>,
        /// ID to link to the Mamit-tRNA database
        pub mamit_trnadb: Option<u32>,
        /// lncRNA Database ID.
        pub lncrnadb: Option<String>,
        /// ENZYME EC accession number.
        pub enzyme_id: Vec<String>,
        /// ID used to link to the Human Intermediate Filament Database.
        pub intermediate_filament_db: Option<String>,
        /// The HGNC ID that the Alliance of Genome Resources (AGR) use.
        pub agr: Option<String>,
        /// NCBI and Ensembl transcript IDs/acessions including the version
        /// number.
        pub mane_select: Vec<String>,
    }

    impl TryFrom<pbs::genes::base::HgncRecord> for GenesHgncRecord {
        type Error = anyhow::Error;

        fn try_from(record: pbs::genes::base::HgncRecord) -> Result<Self, Self::Error> {
            Ok(Self {
                hgnc_id: record.hgnc_id,
                symbol: record.symbol,
                name: record.name,
                locus_group: record.locus_group,
                locus_type: record.locus_type,
                status: record.status,
                location: record.location,
                location_sortable: record.location_sortable,
                alias_symbol: record.alias_symbol,
                alias_name: record.alias_name,
                prev_symbol: record.prev_symbol,
                prev_name: record.prev_name,
                gene_group: record.gene_group,
                gene_group_id: record.gene_group_id,
                date_approved_reserved: record.date_approved_reserved,
                date_symbol_changed: record.date_symbol_changed,
                date_name_changed: record.date_name_changed,
                date_modified: record.date_modified,
                entrez_id: record.entrez_id,
                ensembl_gene_id: record.ensembl_gene_id,
                vega_id: record.vega_id,
                ucsc_id: record.ucsc_id,
                ena: record.ena,
                refseq_accession: record.refseq_accession,
                ccds_id: record.ccds_id,
                uniprot_ids: record.uniprot_ids,
                pubmed_id: record.pubmed_id,
                mgd_id: record.mgd_id,
                rgd_id: record.rgd_id,
                lsdb: record
                    .lsdb
                    .into_iter()
                    .map(GenesHgncLsdb::try_from)
                    .collect::<Result<Vec<_>, _>>()?,
                cosmic: record.cosmic,
                omim_id: record.omim_id,
                mirbase: record.mirbase,
                homeodb: record.homeodb,
                snornabase: record.snornabase,
                bioparadigms_slc: record.bioparadigms_slc,
                orphanet: record.orphanet,
                pseudogene_org: record.pseudogene_org,
                horde_id: record.horde_id,
                merops: record.merops,
                imgt: record.imgt,
                iuphar: record.iuphar,
                cd: record.cd,
                mamit_trnadb: record.mamit_trnadb,
                lncrnadb: record.lncrnadb,
                enzyme_id: record.enzyme_id,
                intermediate_filament_db: record.intermediate_filament_db,
                agr: record.agr,
                mane_select: record.mane_select,
            })
        }
    }

    /// Reference into function record.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesRifEntry {
        /// The RIF text.
        pub text: String,
        /// PubMed IDs.
        pub pmids: Vec<u32>,
    }

    impl TryFrom<pbs::genes::base::RifEntry> for GenesRifEntry {
        type Error = anyhow::Error;

        fn try_from(value: pbs::genes::base::RifEntry) -> Result<Self, Self::Error> {
            Ok(Self {
                text: value.text,
                pmids: value.pmids,
            })
        }
    }

    /// A record from the NCBI gene database.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesNcbiRecord {
        /// NCBI Gene ID.
        pub gene_id: String,
        /// Gene summary.
        pub summary: Option<String>,
        /// "Reference Into Function" entry.
        pub rif_entries: Vec<GenesRifEntry>,
    }

    impl From<pbs::genes::base::NcbiRecord> for GenesNcbiRecord {
        fn from(record: pbs::genes::base::NcbiRecord) -> Self {
            Self {
                gene_id: record.gene_id,
                summary: record.summary,
                rif_entries: record
                    .rif_entries
                    .into_iter()
                    .map(GenesRifEntry::try_from)
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap_or_default(),
            }
        }
    }

    /// Description of an OMIM record.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesOmimTerm {
        /// The OMIM ID.
        pub omim_id: String,
        /// The OMIM label.
        pub label: String,
    }

    impl From<pbs::genes::base::OmimTerm> for GenesOmimTerm {
        fn from(value: pbs::genes::base::OmimTerm) -> Self {
            Self {
                omim_id: value.omim_id,
                label: value.label,
            }
        }
    }

    /// A record from the OMIM gene association.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesOmimRecord {
        /// The HGNC gene ID.
        pub hgnc_id: String,
        /// The associated OMIM records.
        pub omim_diseases: Vec<GenesOmimTerm>,
    }

    impl From<pbs::genes::base::OmimRecord> for GenesOmimRecord {
        fn from(value: pbs::genes::base::OmimRecord) -> Self {
            Self {
                hgnc_id: value.hgnc_id,
                omim_diseases: value
                    .omim_diseases
                    .into_iter()
                    .map(GenesOmimTerm::from)
                    .collect(),
            }
        }
    }

    /// Description of an ORDO record.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesOrphaTerm {
        /// The ORPHA ID.
        pub orpha_id: String,
        /// The disease name.
        pub label: String,
    }

    impl From<pbs::genes::base::OrphaTerm> for GenesOrphaTerm {
        fn from(value: pbs::genes::base::OrphaTerm) -> Self {
            Self {
                orpha_id: value.orpha_id,
                label: value.label,
            }
        }
    }

    /// A record from the ORDO gene association.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesOrphaRecord {
        /// The HGNC gene ID.
        pub hgnc_id: String,
        /// The associated ORPHA diseases.
        pub orpha_diseases: Vec<GenesOrphaTerm>,
    }

    impl From<pbs::genes::base::OrphaRecord> for GenesOrphaRecord {
        fn from(value: pbs::genes::base::OrphaRecord) -> Self {
            Self {
                hgnc_id: value.hgnc_id,
                orpha_diseases: value
                    .orpha_diseases
                    .into_iter()
                    .map(GenesOrphaTerm::from)
                    .collect(),
            }
        }
    }

    /// Entry in the rCNV dosage sensitivity scores (Collins et al., 2022).
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesRcnvRecord {
        /// The HGNC ID.
        pub hgnc_id: String,
        /// The pHaplo value.
        pub p_haplo: f64,
        /// The pTriplo value.
        pub p_triplo: f64,
    }

    impl From<pbs::genes::base::RcnvRecord> for GenesRcnvRecord {
        fn from(value: pbs::genes::base::RcnvRecord) -> Self {
            Self {
                hgnc_id: value.hgnc_id,
                p_haplo: value.p_haplo,
                p_triplo: value.p_triplo,
            }
        }
    }

    /// Entry with sHet information (Weghorn et al., 2019).
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesShetRecord {
        /// The HGNC ID.
        pub hgnc_id: String,
        /// The sHet value.
        pub s_het: f64,
    }

    impl From<pbs::genes::base::ShetRecord> for GenesShetRecord {
        fn from(value: pbs::genes::base::ShetRecord) -> Self {
            Self {
                hgnc_id: value.hgnc_id,
                s_het: value.s_het,
            }
        }
    }

    /// Entry with the tissue-specific information for a gene.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesGtexTissueRecord {
        /// The tissue type
        pub tissue: GenesGtexTissue,
        /// The detailed tissue type
        pub tissue_detailed: GenesGtexTissueDetailed,
        /// TPM counts
        pub tpms: Vec<f32>,
    }

    impl TryFrom<pbs::genes::base::GtexTissueRecord> for GenesGtexTissueRecord {
        type Error = anyhow::Error;

        fn try_from(value: pbs::genes::base::GtexTissueRecord) -> Result<Self, Self::Error> {
            Ok(Self {
                tissue: GenesGtexTissue::try_from(pbs::genes::base::GtexTissue::try_from(
                    value.tissue,
                )?)?,
                tissue_detailed: GenesGtexTissueDetailed::try_from(
                    pbs::genes::base::GtexTissueDetailed::try_from(value.tissue_detailed)?,
                )?,
                tpms: value.tpms,
            })
        }
    }

    /// Entry with the GTEx information.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesGtexRecord {
        /// The HGNC ID.
        pub hgnc_id: String,
        /// ENSEMBL gene ID.
        pub ensembl_gene_id: String,
        /// ENSEMBL gene version.
        pub ensembl_gene_version: String,
        /// Counts per tissue
        pub records: Vec<GenesGtexTissueRecord>,
    }

    impl TryFrom<pbs::genes::base::GtexRecord> for GenesGtexRecord {
        type Error = anyhow::Error;

        fn try_from(value: pbs::genes::base::GtexRecord) -> Result<Self, Self::Error> {
            Ok(Self {
                hgnc_id: value.hgnc_id,
                ensembl_gene_id: value.ensembl_gene_id,
                ensembl_gene_version: value.ensembl_gene_version,
                records: value
                    .records
                    .into_iter()
                    .map(GenesGtexTissueRecord::try_from)
                    .collect::<Result<Vec<_>, _>>()?,
            })
        }
    }

    /// Entry in PanelApp.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesPanelAppRecord {
        /// Gene identity information.
        pub gene_data: Option<GenesGeneData>,
        /// Entity type.
        pub entity_type: GenesEntityType,
        /// Entity name.
        pub entity_name: String,
        /// Confidence level.
        pub confidence_level: GenesPanelappRecordConfidenceLevel,
        /// Penetrance.
        pub penetrance: Option<GenesPenetrance>,
        /// Publications.
        pub publications: Vec<String>,
        /// Evidence.
        pub evidence: Vec<String>,
        /// Phenotypes.
        pub phenotypes: Vec<String>,
        /// Mode of inheritance.
        pub mode_of_inheritance: String,
        /// Panel.
        pub panel: Option<GenesPanel>,
    }

    impl TryFrom<pbs::genes::base::PanelAppRecord> for GenesPanelAppRecord {
        type Error = anyhow::Error;

        fn try_from(value: pbs::genes::base::PanelAppRecord) -> Result<Self, Self::Error> {
            Ok(Self {
                gene_data: value.gene_data.map(GenesGeneData::from),
                entity_type: GenesEntityType::try_from(
                    pbs::genes::base::panel_app_record::EntityType::try_from(value.entity_type)?,
                )?,
                entity_name: value.entity_name,
                confidence_level: GenesPanelappRecordConfidenceLevel::try_from(
                    pbs::genes::base::panel_app_record::ConfidenceLevel::try_from(
                        value.entity_type,
                    )?,
                )?,
                penetrance: Option::<GenesPenetrance>::from(
                    pbs::genes::base::panel_app_record::Penetrance::try_from(value.penetrance)?,
                ),
                publications: value.publications,
                evidence: value.evidence,
                phenotypes: value.phenotypes,
                mode_of_inheritance: value.mode_of_inheritance,
                panel: value.panel.map(GenesPanel::from),
            })
        }
    }

    /// Gene identity information.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesGeneData {
        /// HGNC ID.
        pub hgnc_id: Option<String>,
        /// HGNC gene symbol.
        pub hgnc_symbol: Option<String>,
        /// Gene symbol.
        pub gene_symbol: Option<String>,
    }

    impl From<pbs::genes::base::panel_app_record::GeneData> for GenesGeneData {
        fn from(value: pbs::genes::base::panel_app_record::GeneData) -> Self {
            Self {
                hgnc_id: value.hgnc_id,
                hgnc_symbol: value.hgnc_symbol,
                gene_symbol: value.gene_symbol,
            }
        }
    }

    /// Message for panel statistics.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesPanelStats {
        /// Number of genes.
        pub number_of_genes: u32,
        /// Number of STRs.
        pub number_of_strs: u32,
        /// Number of regions.
        pub number_of_regions: u32,
    }

    impl From<pbs::genes::base::panel_app_record::PanelStats> for GenesPanelStats {
        fn from(value: pbs::genes::base::panel_app_record::PanelStats) -> Self {
            Self {
                number_of_genes: value.number_of_genes,
                number_of_strs: value.number_of_strs,
                number_of_regions: value.number_of_regions,
            }
        }
    }

    /// Message for panel types.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesPanelType {
        /// Type name.
        pub name: String,
        /// Slug.
        pub slug: String,
        /// Description.
        pub description: String,
    }

    impl From<pbs::genes::base::panel_app_record::PanelType> for GenesPanelType {
        fn from(value: pbs::genes::base::panel_app_record::PanelType) -> Self {
            Self {
                name: value.name,
                slug: value.slug,
                description: value.description,
            }
        }
    }

    /// Message for panel information.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesPanel {
        /// Panel ID.
        pub id: u32,
        /// Panel hash ID.
        pub hash_id: Option<String>,
        /// Panel name.
        pub name: String,
        /// Disease group.
        pub disease_group: String,
        /// Disease subgroup.
        pub disease_sub_group: String,
        /// Version
        pub version: String,
        /// Creation date of version.
        pub version_created: String,
        /// Relevant disorders.
        pub relevant_disorders: Vec<String>,
        /// Stats.
        pub stats: Option<GenesPanelStats>,
        /// Panel types.
        pub types: Vec<GenesPanelType>,
    }

    impl From<pbs::genes::base::panel_app_record::Panel> for GenesPanel {
        fn from(value: pbs::genes::base::panel_app_record::Panel) -> Self {
            Self {
                id: value.id,
                hash_id: value.hash_id,
                name: value.name,
                disease_group: value.disease_group,
                disease_sub_group: value.disease_sub_group,
                version: value.version,
                version_created: value.version_created,
                relevant_disorders: value.relevant_disorders,
                stats: value.stats.map(GenesPanelStats::from),
                types: value.types.into_iter().map(GenesPanelType::from).collect(),
            }
        }
    }

    /// Enumeration for entity types.
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
    pub enum GenesEntityType {
        /// Gene
        Gene,
        /// Short Tandem Repeat
        Str,
        /// Region
        Region,
    }

    impl TryFrom<pbs::genes::base::panel_app_record::EntityType> for GenesEntityType {
        type Error = anyhow::Error;

        fn try_from(
            value: pbs::genes::base::panel_app_record::EntityType,
        ) -> Result<Self, Self::Error> {
            match value {
                pbs::genes::base::panel_app_record::EntityType::Unknown => {
                    Err(anyhow::anyhow!("Unknown entity type"))
                }
                pbs::genes::base::panel_app_record::EntityType::Gene => Ok(GenesEntityType::Gene),
                pbs::genes::base::panel_app_record::EntityType::Str => Ok(GenesEntityType::Str),
                pbs::genes::base::panel_app_record::EntityType::Region => {
                    Ok(GenesEntityType::Region)
                }
            }
        }
    }

    /// Enumeration for confidence levels.
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
    pub enum GenesDiseaseAssociationEntryConfidenceLevel {
        /// High.
        High,
        /// Medium.
        Medium,
        /// Low.
        Low,
    }

    impl
        TryFrom<
            pbs::genes::base::conditions_record::gene_disease_association_entry::ConfidenceLevel,
        > for GenesDiseaseAssociationEntryConfidenceLevel
    {
        type Error = anyhow::Error;

        fn try_from(
            value: pbs::genes::base::conditions_record::gene_disease_association_entry::ConfidenceLevel,
        ) -> Result<Self, anyhow::Error> {
            Ok(match value {
                pbs::genes::base::conditions_record::gene_disease_association_entry::ConfidenceLevel::Unknown => anyhow::bail!("Unknown confidence level: {:?}", value),
                pbs::genes::base::conditions_record::gene_disease_association_entry::ConfidenceLevel::High => {
                    GenesDiseaseAssociationEntryConfidenceLevel::High
                }
                pbs::genes::base::conditions_record::gene_disease_association_entry::ConfidenceLevel::Medium => {
                    GenesDiseaseAssociationEntryConfidenceLevel::Medium
                }
                pbs::genes::base::conditions_record::gene_disease_association_entry::ConfidenceLevel::Low => {
                    GenesDiseaseAssociationEntryConfidenceLevel::Low
                }
            })
        }
    }

    /// Enumeration for penetrance.
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
    pub enum GenesPenetrance {
        /// Complete
        Complete,
        /// Incomplete
        Incomplete,
    }

    impl From<pbs::genes::base::panel_app_record::Penetrance> for Option<GenesPenetrance> {
        fn from(score: pbs::genes::base::panel_app_record::Penetrance) -> Self {
            match score {
                pbs::genes::base::panel_app_record::Penetrance::Unknown => None,
                pbs::genes::base::panel_app_record::Penetrance::Complete => {
                    Some(GenesPenetrance::Complete)
                }
                pbs::genes::base::panel_app_record::Penetrance::Incomplete => {
                    Some(GenesPenetrance::Incomplete)
                }
            }
        }
    }

    /// Record from the integrated conditions computation.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesConditionsRecord {
        /// The HGNC ID.
        pub hgnc_id: String,
        /// The gene-disease associations.
        pub disease_associations: Vec<GenesDiseaseAssociation>,
        /// The PanelApp associations.
        pub panelapp_associations: Vec<GenesPanelappAssociation>,
    }

    impl TryFrom<pbs::genes::base::ConditionsRecord> for GenesConditionsRecord {
        type Error = anyhow::Error;

        fn try_from(value: pbs::genes::base::ConditionsRecord) -> Result<Self, Self::Error> {
            Ok(Self {
                hgnc_id: value.hgnc_id,
                disease_associations: value
                    .disease_associations
                    .into_iter()
                    .map(GenesDiseaseAssociation::try_from)
                    .collect::<Result<Vec<_>, _>>()?,
                panelapp_associations: value
                    .panelapp_associations
                    .into_iter()
                    .map(GenesPanelappAssociation::try_from)
                    .collect::<Result<Vec<_>, _>>()?,
            })
        }
    }

    /// A gene-disease association entry.
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
    pub struct GenesGeneDiseaseAssociationEntry {
        /// The gene-disease association source.
        pub source: GenesDiseaseAssociationSource,
        /// The gene-disease association confidence level.
        pub confidence: GenesDiseaseAssociationEntryConfidenceLevel,
    }

    impl TryFrom<pbs::genes::base::conditions_record::GeneDiseaseAssociationEntry>
        for GenesGeneDiseaseAssociationEntry
    {
        type Error = anyhow::Error;

        fn try_from(
            value: pbs::genes::base::conditions_record::GeneDiseaseAssociationEntry,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                source: GenesDiseaseAssociationSource::try_from(
                    pbs::genes::base::conditions_record::gene_disease_association_entry::GeneDiseaseAssociationSource::try_from(
                    value.source
                    )?
                )?,
                confidence: GenesDiseaseAssociationEntryConfidenceLevel::try_from(
                    pbs::genes::base::conditions_record::gene_disease_association_entry::ConfidenceLevel::try_from(
                        value.confidence
                    )?
                )?,
            })
        }
    }

    /// Enumeration for sources.
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
    pub enum GenesDiseaseAssociationSource {
        /// OMIM
        Omim,
        /// Orphanet
        Orphanet,
        /// PanelApp
        Panelapp,
    }

    impl TryFrom<pbs::genes::base::conditions_record::gene_disease_association_entry::GeneDiseaseAssociationSource>
        for GenesDiseaseAssociationSource
    {
        type Error = anyhow::Error;

        fn try_from(
            value: pbs::genes::base::conditions_record::gene_disease_association_entry::GeneDiseaseAssociationSource,
        ) -> Result<Self, Self::Error> {
            match value {
                pbs::genes::base::conditions_record::gene_disease_association_entry::GeneDiseaseAssociationSource::Unknown => {
                    Err(anyhow::anyhow!("Unknown source"))
                }
                pbs::genes::base::conditions_record::gene_disease_association_entry::GeneDiseaseAssociationSource::Omim => Ok(GenesDiseaseAssociationSource::Omim),
                pbs::genes::base::conditions_record::gene_disease_association_entry::GeneDiseaseAssociationSource::Orphanet => Ok(GenesDiseaseAssociationSource::Orphanet),
                pbs::genes::base::conditions_record::gene_disease_association_entry::GeneDiseaseAssociationSource::Panelapp => Ok(GenesDiseaseAssociationSource::Panelapp),
            }
        }
    }

    /// Enumeration for confidence levels.
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
    pub enum GenesDiseaseAssociationConfidenceLevel {
        /// High confidence.
        High,
        /// Medium confidence.
        Medium,
        /// Low confidence.
        Low,
    }

    impl
        TryFrom<
            pbs::genes::base::conditions_record::gene_disease_association_entry::ConfidenceLevel,
        > for GenesDiseaseAssociationConfidenceLevel
    {
        type Error = anyhow::Error;

        fn try_from(
            value: pbs::genes::base::conditions_record::gene_disease_association_entry::ConfidenceLevel,
        ) -> Result<Self, Self::Error> {
            match value {
                pbs::genes::base::conditions_record::gene_disease_association_entry::ConfidenceLevel::Unknown => {
                    Err(anyhow::anyhow!("Unknown confidence level"))
                }
                pbs::genes::base::conditions_record::gene_disease_association_entry::ConfidenceLevel::High => Ok(GenesDiseaseAssociationConfidenceLevel::High),
                pbs::genes::base::conditions_record::gene_disease_association_entry::ConfidenceLevel::Medium => Ok(GenesDiseaseAssociationConfidenceLevel::Medium),
                pbs::genes::base::conditions_record::gene_disease_association_entry::ConfidenceLevel::Low => Ok(GenesDiseaseAssociationConfidenceLevel::Low),
            }
        }
    }

    /// A labeled disorder.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesLabeledDisorder {
        /// The disorder ID.
        pub term_id: String,
        /// The disorder name.
        pub title: Option<String>,
    }

    impl From<pbs::genes::base::conditions_record::LabeledDisorder> for GenesLabeledDisorder {
        fn from(value: pbs::genes::base::conditions_record::LabeledDisorder) -> Self {
            Self {
                term_id: value.term_id,
                title: value.title,
            }
        }
    }

    /// A gene-disease association.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesDiseaseAssociation {
        /// The HGNC ID.
        pub hgnc_id: String,
        /// The gene-disease association entries.
        pub labeled_disorders: Vec<GenesLabeledDisorder>,
        /// Overall disease name.
        pub disease_name: Option<String>,
        /// Disease definition.
        pub disease_definition: Option<String>,
        /// The gene-disease association sources.
        pub sources: Vec<GenesDiseaseAssociationSource>,
        /// Overall disease-gene association confidence level.
        pub confidence: GenesDiseaseAssociationConfidenceLevel,
    }

    impl TryFrom<pbs::genes::base::conditions_record::GeneDiseaseAssociation>
        for GenesDiseaseAssociation
    {
        type Error = anyhow::Error;

        fn try_from(
            value: pbs::genes::base::conditions_record::GeneDiseaseAssociation,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                hgnc_id: value.hgnc_id,
                labeled_disorders: value.labeled_disorders.into_iter().map(From::from).collect(),
                disease_name: value.disease_name,
                disease_definition: value.disease_definition,
                sources: value.sources.into_iter().map(|source| pbs::genes::base::conditions_record::gene_disease_association_entry::GeneDiseaseAssociationSource::try_from(
                    source
                ).map_err(|e| anyhow::anyhow!("invalid enum for gene disease association: {}", e)).and_then(GenesDiseaseAssociationSource::try_from)).collect::<Result<Vec<_>, _>>()?,
                confidence: GenesDiseaseAssociationConfidenceLevel::try_from(
                    pbs::genes::base::conditions_record::gene_disease_association_entry::ConfidenceLevel::try_from(
                        value.confidence
                    )?
                )?,
            })
        }
    }

    /// A panel from PanelApp.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesPanelappPanel {
        /// PanelApp panel ID.
        pub id: i32,
        /// PanelApp panel name.
        pub name: String,
        /// PanelApp panel version.
        pub version: String,
    }

    impl From<pbs::genes::base::conditions_record::PanelappPanel> for GenesPanelappPanel {
        fn from(value: pbs::genes::base::conditions_record::PanelappPanel) -> Self {
            Self {
                id: value.id,
                name: value.name,
                version: value.version,
            }
        }
    }

    /// An association of a gene by HGNC with a panel from PanelApp.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesPanelappAssociation {
        /// The HGNC ID.
        pub hgnc_id: String,
        /// The PanelApp confidence level.
        pub confidence_level: GenesPanelappAssociationConfidenceLevel,
        /// The PanelApp entity type.
        pub entity_type: GenesPanelappEntityType,
        /// The PanelApp entity name.
        pub mode_of_inheritance: Option<String>,
        /// The PanelApp publications.
        pub phenotypes: Vec<String>,
        /// The PanelApp panel.
        pub panel: Option<GenesPanelappPanel>,
    }

    impl TryFrom<pbs::genes::base::conditions_record::PanelappAssociation>
        for GenesPanelappAssociation
    {
        type Error = anyhow::Error;

        fn try_from(
            value: pbs::genes::base::conditions_record::PanelappAssociation,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                hgnc_id: value.hgnc_id,
                confidence_level: GenesPanelappAssociationConfidenceLevel::try_from(
                    pbs::genes::base::conditions_record::panelapp_association::PanelappConfidence::try_from(
                        value.confidence_level
                    )?
                )?,
                entity_type: GenesPanelappEntityType::try_from(pbs::genes::base::conditions_record::panelapp_association::PanelappEntityType::try_from(value.entity_type)?)?,
                mode_of_inheritance: value.mode_of_inheritance,
                phenotypes: value.phenotypes,
                panel: value.panel.map(|panel| GenesPanelappPanel {
                    id: panel.id,
                    name: panel.name,
                    version: panel.version,
                }),
            })
        }
    }

    /// Enumeration for PanelApp confidence level.
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
    pub enum GenesPanelappRecordConfidenceLevel {
        /// PanelApp green confidence.
        Green,
        /// PanelApp amber confidence.
        Amber,
        /// PanelApp red confidence.
        Red,
        /// PanelApp none confidence (when removed after expert review).
        None,
    }

    impl TryFrom<pbs::genes::base::panel_app_record::ConfidenceLevel>
        for GenesPanelappRecordConfidenceLevel
    {
        type Error = anyhow::Error;

        fn try_from(
            value: pbs::genes::base::panel_app_record::ConfidenceLevel,
        ) -> Result<Self, Self::Error> {
            match value {
                pbs::genes::base::panel_app_record::ConfidenceLevel::Unknown => {
                    Err(anyhow::anyhow!("Unknown confidence level"))
                }
                pbs::genes::base::panel_app_record::ConfidenceLevel::Green => {
                    Ok(GenesPanelappRecordConfidenceLevel::Green)
                }
                pbs::genes::base::panel_app_record::ConfidenceLevel::Amber => {
                    Ok(GenesPanelappRecordConfidenceLevel::Amber)
                }
                pbs::genes::base::panel_app_record::ConfidenceLevel::Red => {
                    Ok(GenesPanelappRecordConfidenceLevel::Red)
                }
                pbs::genes::base::panel_app_record::ConfidenceLevel::None => {
                    Ok(GenesPanelappRecordConfidenceLevel::None)
                }
            }
        }
    }

    /// Enumeration for PanelApp confidence level.
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
    pub enum GenesPanelappAssociationConfidenceLevel {
        /// PanelApp green confidence.
        Green,
        /// PanelApp amber confidence.
        Amber,
        /// PanelApp red confidence.
        Red,
        /// PanelApp none confidence (when removed after expert review).
        None,
    }

    impl TryFrom<pbs::genes::base::conditions_record::panelapp_association::PanelappConfidence>
        for GenesPanelappAssociationConfidenceLevel
    {
        type Error = anyhow::Error;

        fn try_from(
            value: pbs::genes::base::conditions_record::panelapp_association::PanelappConfidence,
        ) -> Result<Self, Self::Error> {
            match value {
                pbs::genes::base::conditions_record::panelapp_association::PanelappConfidence::Unknown => {
                    Err(anyhow::anyhow!("Unknown confidence level"))
                }
                pbs::genes::base::conditions_record::panelapp_association::PanelappConfidence::Green => {
                    Ok(GenesPanelappAssociationConfidenceLevel::Green)
                }
                pbs::genes::base::conditions_record::panelapp_association::PanelappConfidence::Amber => {
                    Ok(GenesPanelappAssociationConfidenceLevel::Amber)
                }
                pbs::genes::base::conditions_record::panelapp_association::PanelappConfidence::Red => {
                    Ok(GenesPanelappAssociationConfidenceLevel::Red)
                }
                pbs::genes::base::conditions_record::panelapp_association::PanelappConfidence::None => {
                    Ok(GenesPanelappAssociationConfidenceLevel::None)
                }
            }
        }
    }

    /// Enumeration for entity type.
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
    pub enum GenesPanelappEntityType {
        /// PanelApp gene entity type.
        Gene,
        /// PanelApp region entity type.
        Region,
        /// PanelApp short tandem repeat entity type.
        Str,
    }

    impl TryFrom<pbs::genes::base::conditions_record::panelapp_association::PanelappEntityType>
        for GenesPanelappEntityType
    {
        type Error = anyhow::Error;

        fn try_from(
            value: pbs::genes::base::conditions_record::panelapp_association::PanelappEntityType,
        ) -> Result<Self, Self::Error> {
            match value {
                pbs::genes::base::conditions_record::panelapp_association::PanelappEntityType::Unknown => {
                    Err(anyhow::anyhow!("Unknown entity type"))
                }
                pbs::genes::base::conditions_record::panelapp_association::PanelappEntityType::Gene => Ok(GenesPanelappEntityType::Gene),
                pbs::genes::base::conditions_record::panelapp_association::PanelappEntityType::Region => Ok(GenesPanelappEntityType::Region),
                pbs::genes::base::conditions_record::panelapp_association::PanelappEntityType::Str => Ok(GenesPanelappEntityType::Str),
            }
        }
    }

    /// Entry in the genes RocksDB database.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesGeneRecord {
        /// Information from the ACMG secondary finding list.
        pub acmg_sf: Option<GenesAcmgSecondaryFindingRecord>,
        /// Information from ClinGen dosage curation.
        pub clingen: Option<GenesClingenDosageRecord>,
        /// Information from dbNSFP.
        pub dbnsfp: Option<GenesDbnsfpRecord>,
        /// Information from the gnomAD constraints database.
        pub gnomad_constraints: Option<GenesGnomadConstraintsRecord>,
        /// Information from the HGNC database.
        pub hgnc: Option<GenesHgncRecord>,
        /// Information from the NCBI gene database (aka "Entrez").
        pub ncbi: Option<GenesNcbiRecord>,
        /// Information about gene to OMIM term annotation, composed from clingen and HPO.
        pub omim: Option<GenesOmimRecord>,
        /// Information about gene to Orphanet annotation, derived from Orphapacket.
        pub orpha: Option<GenesOrphaRecord>,
        /// Information from the rCNV dosage sensitivity scores (Collins et al., 2022).
        pub rcnv: Option<GenesRcnvRecord>,
        /// Information from the sHet score (Weghor et al., 2019)
        pub shet: Option<GenesShetRecord>,
        /// Information from GTEx data
        pub gtex: Option<GenesGtexRecord>,
        /// Information from DOMINO.
        pub domino: Option<GenesDominoRecord>,
        /// DECIPHER HI score.
        pub decipher_hi: Option<GenesDecipherHiRecord>,
        /// GenomicsEngland PanelApp gene information.
        pub panelapp: Vec<GenesPanelAppRecord>,
        /// Conditions record.
        pub conditions: Option<GenesConditionsRecord>,
    }

    impl TryFrom<pbs::genes::base::Record> for GenesGeneRecord {
        type Error = anyhow::Error;

        fn try_from(value: pbs::genes::base::Record) -> Result<Self, Self::Error> {
            Ok(Self {
                acmg_sf: value.acmg_sf.map(GenesAcmgSecondaryFindingRecord::from),
                clingen: value
                    .clingen
                    .map(GenesClingenDosageRecord::try_from)
                    .transpose()?,
                dbnsfp: value.dbnsfp.map(GenesDbnsfpRecord::from),
                gnomad_constraints: value
                    .gnomad_constraints
                    .map(GenesGnomadConstraintsRecord::from),
                hgnc: value.hgnc.map(GenesHgncRecord::try_from).transpose()?,
                ncbi: value.ncbi.map(GenesNcbiRecord::from),
                omim: value.omim.map(GenesOmimRecord::from),
                orpha: value.orpha.map(GenesOrphaRecord::from),
                rcnv: value.rcnv.map(GenesRcnvRecord::from),
                shet: value.shet.map(GenesShetRecord::from),
                gtex: value.gtex.map(GenesGtexRecord::try_from).transpose()?,
                domino: value.domino.map(GenesDominoRecord::from),
                decipher_hi: value.decipher_hi.map(GenesDecipherHiRecord::from),
                panelapp: value
                    .panelapp
                    .into_iter()
                    .map(GenesPanelAppRecord::try_from)
                    .collect::<Result<Vec<_>, _>>()?,
                conditions: value
                    .conditions
                    .map(GenesConditionsRecord::try_from)
                    .transpose()?,
            })
        }
    }

    /// Status of the symbol report, which can be either "Approved" or "Entry Withdrawn".
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
    pub enum GenesHgncStatus {
        /// Approved by HGNC.
        Approved,
        /// Withdrawn by HGNC.
        Withdrawn,
    }

    impl TryFrom<pbs::genes::base::HgncStatus> for GenesHgncStatus {
        type Error = anyhow::Error;

        fn try_from(value: pbs::genes::base::HgncStatus) -> Result<Self, Self::Error> {
            match value {
                pbs::genes::base::HgncStatus::Unknown => Err(anyhow::anyhow!("Unknown status")),
                pbs::genes::base::HgncStatus::Approved => Ok(GenesHgncStatus::Approved),
                pbs::genes::base::HgncStatus::Withdrawn => Ok(GenesHgncStatus::Withdrawn),
            }
        }
    }

    /// Enumeration for GTEx V8 tissue
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
    pub enum GenesGtexTissue {
        /// Adipose Tissue
        AdiposeTissue,
        /// Adrenal Gland
        AdrenalGland,
        /// Bladder
        Bladder,
        /// Blood
        Blood,
        /// Blood Vessel
        BloodVessel,
        /// Bone Marrow
        BoneMarrow,
        /// Brain
        Brain,
        /// Breast
        Breast,
        /// Cervix Uteri
        CervixUteri,
        /// Colon
        Colon,
        /// Esophagus
        Esophagus,
        /// Fallopian Tube
        FallopianTube,
        /// Heart
        Heart,
        /// Kidney
        Kidney,
        /// Liver
        Liver,
        /// Lung
        Lung,
        /// Muscle
        Muscle,
        /// Nerve
        Nerve,
        /// Ovary
        Ovary,
        /// Pancreas
        Pancreas,
        /// Pituitary
        Pituitary,
        /// Prostate
        Prostate,
        /// Salivary Gland
        SalivaryGland,
        /// Skin
        Skin,
        /// Small Intestine
        SmallIntestine,
        /// Spleen
        Spleen,
        /// Stomach
        Stomach,
        /// Testis
        Testis,
        /// Thyroid
        Thyroid,
        /// Uterus
        Uterus,
        /// Vagina
        Vagina,
    }

    impl TryFrom<pbs::genes::base::GtexTissue> for GenesGtexTissue {
        type Error = anyhow::Error;

        fn try_from(value: pbs::genes::base::GtexTissue) -> Result<Self, Self::Error> {
            Ok(match value {
                pbs::genes::base::GtexTissue::Unknown => {
                    anyhow::bail!("unknown gtex tissue: {:?}", value)
                }
                pbs::genes::base::GtexTissue::AdiposeTissue => GenesGtexTissue::AdiposeTissue,
                pbs::genes::base::GtexTissue::AdrenalGland => GenesGtexTissue::AdrenalGland,
                pbs::genes::base::GtexTissue::Bladder => GenesGtexTissue::Bladder,
                pbs::genes::base::GtexTissue::Blood => GenesGtexTissue::Blood,
                pbs::genes::base::GtexTissue::BloodVessel => GenesGtexTissue::BloodVessel,
                pbs::genes::base::GtexTissue::BoneMarrow => GenesGtexTissue::BoneMarrow,
                pbs::genes::base::GtexTissue::Brain => GenesGtexTissue::Brain,
                pbs::genes::base::GtexTissue::Breast => GenesGtexTissue::Breast,
                pbs::genes::base::GtexTissue::CervixUteri => GenesGtexTissue::CervixUteri,
                pbs::genes::base::GtexTissue::Colon => GenesGtexTissue::Colon,
                pbs::genes::base::GtexTissue::Esophagus => GenesGtexTissue::Esophagus,
                pbs::genes::base::GtexTissue::FallopianTube => GenesGtexTissue::FallopianTube,
                pbs::genes::base::GtexTissue::Heart => GenesGtexTissue::Heart,
                pbs::genes::base::GtexTissue::Kidney => GenesGtexTissue::Kidney,
                pbs::genes::base::GtexTissue::Liver => GenesGtexTissue::Liver,
                pbs::genes::base::GtexTissue::Lung => GenesGtexTissue::Lung,
                pbs::genes::base::GtexTissue::Muscle => GenesGtexTissue::Muscle,
                pbs::genes::base::GtexTissue::Nerve => GenesGtexTissue::Nerve,
                pbs::genes::base::GtexTissue::Ovary => GenesGtexTissue::Ovary,
                pbs::genes::base::GtexTissue::Pancreas => GenesGtexTissue::Pancreas,
                pbs::genes::base::GtexTissue::Pituitary => GenesGtexTissue::Pituitary,
                pbs::genes::base::GtexTissue::Prostate => GenesGtexTissue::Prostate,
                pbs::genes::base::GtexTissue::SalivaryGland => GenesGtexTissue::SalivaryGland,
                pbs::genes::base::GtexTissue::Skin => GenesGtexTissue::Skin,
                pbs::genes::base::GtexTissue::SmallIntestine => GenesGtexTissue::SmallIntestine,
                pbs::genes::base::GtexTissue::Spleen => GenesGtexTissue::Spleen,
                pbs::genes::base::GtexTissue::Stomach => GenesGtexTissue::Stomach,
                pbs::genes::base::GtexTissue::Testis => GenesGtexTissue::Testis,
                pbs::genes::base::GtexTissue::Thyroid => GenesGtexTissue::Thyroid,
                pbs::genes::base::GtexTissue::Uterus => GenesGtexTissue::Uterus,
                pbs::genes::base::GtexTissue::Vagina => GenesGtexTissue::Vagina,
            })
        }
    }

    /// Enumeration for GTEx V8 tissue details
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
    pub enum GenesGtexTissueDetailed {
        /// Adipose - Subcutaneous
        AdiposeSubcutaneous,
        /// Adipose - Visceral (Omentum)
        AdiposeVisceralOmentum,
        /// Adrenal Gland
        AdrenalGland,
        /// Artery - Aorta
        ArteryAorta,
        /// Artery - Coronary
        ArteryCoronary,
        /// Artery - Tibial
        ArteryTibial,
        /// Bladder
        Bladder,
        /// Brain - Amygdala
        BrainAmygdala,
        /// Brain - Anterior cingulate cortex (BA24)
        BrainAnteriorCingulateCortex,
        /// Brain - Caudate (basal ganglia)
        BrainCaudateBasalGanglia,
        /// Brain - Cerebellar Hemisphere
        BrainCerebellarHemisphere,
        /// Brain - Cerebellum
        BrainCerebellum,
        /// Brain - Cortex
        BrainCortex,
        /// Brain - Frontal Cortex (BA9)
        BrainFrontalCortex,
        /// Brain - Hippocampus
        BrainHippocampus,
        /// Brain - Hypothalamus
        BrainHypothalamus,
        /// Brain - Nucleus accumbens (basal ganglia)
        BrainNucleusAccumbens,
        /// Brain - Putamen (basal ganglia)
        BrainPutamenBasalGanglia,
        /// Brain - Spinal cord (cervical c-1)
        BrainSpinalCord,
        /// Brain - Substantia nigra
        BrainSubstantiaNigra,
        /// Breast - Mammary Tissue
        BreastMammaryTissue,
        /// Cells - Cultured fibroblasts
        CellsCulturedFibroblasts,
        /// Cells - EBV-transformed lymphocytes
        CellsEbvTransformedLymphocytes,
        /// Cells - Leukemia cell line (CML)
        CellsLeukemiaCellLine,
        /// Cervix - Ectocervix
        CervixEctocervix,
        /// Cervix - Endocervix
        CervixEndocervix,
        /// Colon - Sigmoid
        ColonSigmoid,
        /// Colon - Transverse
        ColonTransverse,
        /// Esophagus - Gastroesophageal Junction
        EsophagusGastroesophagealJunction,
        /// Esophagus - Mucosa
        EsophagusMucosa,
        /// Esophagus - Muscularis
        EsophagusMuscularis,
        /// Fallopian Tube
        FallopianTube,
        /// Heart - Atrial Appendage
        HeartAtrialAppendage,
        /// Heart - Left Ventricle
        HeartLeftVentricle,
        /// Kidney - Cortex
        KidneyCortex,
        /// Kidney - Medulla
        KidneyMedulla,
        /// Liver
        Liver,
        /// Lung
        Lung,
        /// Minor Salivary Gland
        MinorSalivaryGland,
        /// Muscle - Skeletal
        MuscleSkeletal,
        /// Nerve - Tibial
        NerveTibial,
        /// Ovary
        Ovary,
        /// Pancreas
        Pancreas,
        /// Pituitary
        Pituitary,
        /// Prostate
        Prostate,
        /// Salivary Gland
        SalivaryGland,
        /// Skin - Not Sun Exposed (Suprapubic)
        SkinNotSunExposedSuprapubic,
        /// Skin - Sun Exposed (Lower leg)
        SkinSunExposedLowerLeg,
        /// Small Intestine - Terminal Ileum
        SmallIntestineTerminalIleum,
        /// Spleen
        Spleen,
        /// Stomach
        Stomach,
        /// Testis
        Testis,
        /// Thyroid
        Thyroid,
        /// Uterus
        Uterus,
        /// Vagina
        Vagina,
        /// Whole Blood
        WholeBlood,
    }

    impl TryFrom<pbs::genes::base::GtexTissueDetailed> for GenesGtexTissueDetailed {
        type Error = anyhow::Error;

        fn try_from(value: pbs::genes::base::GtexTissueDetailed) -> Result<Self, Self::Error> {
            Ok(match value {
                pbs::genes::base::GtexTissueDetailed::Unknown => {
                    anyhow::bail!("unknown gtex tissue detailed: {:?}", value)
                }
                pbs::genes::base::GtexTissueDetailed::AdiposeSubcutaneous => {
                    GenesGtexTissueDetailed::AdiposeSubcutaneous
                }
                pbs::genes::base::GtexTissueDetailed::AdiposeVisceralOmentum => {
                    GenesGtexTissueDetailed::AdiposeVisceralOmentum
                }
                pbs::genes::base::GtexTissueDetailed::AdrenalGland => {
                    GenesGtexTissueDetailed::AdrenalGland
                }
                pbs::genes::base::GtexTissueDetailed::ArteryAorta => {
                    GenesGtexTissueDetailed::ArteryAorta
                }
                pbs::genes::base::GtexTissueDetailed::ArteryCoronary => {
                    GenesGtexTissueDetailed::ArteryCoronary
                }
                pbs::genes::base::GtexTissueDetailed::ArteryTibial => {
                    GenesGtexTissueDetailed::ArteryTibial
                }
                pbs::genes::base::GtexTissueDetailed::Bladder => GenesGtexTissueDetailed::Bladder,
                pbs::genes::base::GtexTissueDetailed::BrainAmygdala => {
                    GenesGtexTissueDetailed::BrainAmygdala
                }
                pbs::genes::base::GtexTissueDetailed::BrainAnteriorCingulateCortex => {
                    GenesGtexTissueDetailed::BrainAnteriorCingulateCortex
                }
                pbs::genes::base::GtexTissueDetailed::BrainCaudateBasalGanglia => {
                    GenesGtexTissueDetailed::BrainCaudateBasalGanglia
                }
                pbs::genes::base::GtexTissueDetailed::BrainCerebellarHemisphere => {
                    GenesGtexTissueDetailed::BrainCerebellarHemisphere
                }
                pbs::genes::base::GtexTissueDetailed::BrainCerebellum => {
                    GenesGtexTissueDetailed::BrainCerebellum
                }
                pbs::genes::base::GtexTissueDetailed::BrainCortex => {
                    GenesGtexTissueDetailed::BrainCortex
                }
                pbs::genes::base::GtexTissueDetailed::BrainFrontalCortex => {
                    GenesGtexTissueDetailed::BrainFrontalCortex
                }
                pbs::genes::base::GtexTissueDetailed::BrainHippocampus => {
                    GenesGtexTissueDetailed::BrainHippocampus
                }
                pbs::genes::base::GtexTissueDetailed::BrainHypothalamus => {
                    GenesGtexTissueDetailed::BrainHypothalamus
                }
                pbs::genes::base::GtexTissueDetailed::BrainNucleusAccumbens => {
                    GenesGtexTissueDetailed::BrainNucleusAccumbens
                }
                pbs::genes::base::GtexTissueDetailed::BrainPutamenBasalGanglia => {
                    GenesGtexTissueDetailed::BrainPutamenBasalGanglia
                }
                pbs::genes::base::GtexTissueDetailed::BrainSpinalCord => {
                    GenesGtexTissueDetailed::BrainSpinalCord
                }
                pbs::genes::base::GtexTissueDetailed::BrainSubstantiaNigra => {
                    GenesGtexTissueDetailed::BrainSubstantiaNigra
                }
                pbs::genes::base::GtexTissueDetailed::BreastMammaryTissue => {
                    GenesGtexTissueDetailed::BreastMammaryTissue
                }
                pbs::genes::base::GtexTissueDetailed::CellsCulturedFibroblasts => {
                    GenesGtexTissueDetailed::CellsCulturedFibroblasts
                }
                pbs::genes::base::GtexTissueDetailed::CellsEbvTransformedLymphocytes => {
                    GenesGtexTissueDetailed::CellsEbvTransformedLymphocytes
                }
                pbs::genes::base::GtexTissueDetailed::CellsLeukemiaCellLine => {
                    GenesGtexTissueDetailed::CellsLeukemiaCellLine
                }
                pbs::genes::base::GtexTissueDetailed::CervixEctocervix => {
                    GenesGtexTissueDetailed::CervixEctocervix
                }
                pbs::genes::base::GtexTissueDetailed::CervixEndocervix => {
                    GenesGtexTissueDetailed::CervixEndocervix
                }
                pbs::genes::base::GtexTissueDetailed::ColonSigmoid => {
                    GenesGtexTissueDetailed::ColonSigmoid
                }
                pbs::genes::base::GtexTissueDetailed::ColonTransverse => {
                    GenesGtexTissueDetailed::ColonTransverse
                }
                pbs::genes::base::GtexTissueDetailed::EsophagusGastroesophagealJunction => {
                    GenesGtexTissueDetailed::EsophagusGastroesophagealJunction
                }
                pbs::genes::base::GtexTissueDetailed::EsophagusMucosa => {
                    GenesGtexTissueDetailed::EsophagusMucosa
                }
                pbs::genes::base::GtexTissueDetailed::EsophagusMuscularis => {
                    GenesGtexTissueDetailed::EsophagusMuscularis
                }
                pbs::genes::base::GtexTissueDetailed::FallopianTube => {
                    GenesGtexTissueDetailed::FallopianTube
                }
                pbs::genes::base::GtexTissueDetailed::HeartAtrialAppendage => {
                    GenesGtexTissueDetailed::HeartAtrialAppendage
                }
                pbs::genes::base::GtexTissueDetailed::HeartLeftVentricle => {
                    GenesGtexTissueDetailed::HeartLeftVentricle
                }
                pbs::genes::base::GtexTissueDetailed::KidneyCortex => {
                    GenesGtexTissueDetailed::KidneyCortex
                }
                pbs::genes::base::GtexTissueDetailed::KidneyMedulla => {
                    GenesGtexTissueDetailed::KidneyMedulla
                }
                pbs::genes::base::GtexTissueDetailed::Liver => GenesGtexTissueDetailed::Liver,
                pbs::genes::base::GtexTissueDetailed::Lung => GenesGtexTissueDetailed::Lung,
                pbs::genes::base::GtexTissueDetailed::MinorSalivaryGland => {
                    GenesGtexTissueDetailed::MinorSalivaryGland
                }
                pbs::genes::base::GtexTissueDetailed::MuscleSkeletal => {
                    GenesGtexTissueDetailed::MuscleSkeletal
                }
                pbs::genes::base::GtexTissueDetailed::NerveTibial => {
                    GenesGtexTissueDetailed::NerveTibial
                }
                pbs::genes::base::GtexTissueDetailed::Ovary => GenesGtexTissueDetailed::Ovary,
                pbs::genes::base::GtexTissueDetailed::Pancreas => GenesGtexTissueDetailed::Pancreas,
                pbs::genes::base::GtexTissueDetailed::Pituitary => {
                    GenesGtexTissueDetailed::Pituitary
                }
                pbs::genes::base::GtexTissueDetailed::Prostate => GenesGtexTissueDetailed::Prostate,
                pbs::genes::base::GtexTissueDetailed::SalivaryGland => {
                    GenesGtexTissueDetailed::SalivaryGland
                }
                pbs::genes::base::GtexTissueDetailed::SkinNotSunExposedSuprapubic => {
                    GenesGtexTissueDetailed::SkinNotSunExposedSuprapubic
                }
                pbs::genes::base::GtexTissueDetailed::SkinSunExposedLowerLeg => {
                    GenesGtexTissueDetailed::SkinSunExposedLowerLeg
                }
                pbs::genes::base::GtexTissueDetailed::SmallIntestineTerminalIleum => {
                    GenesGtexTissueDetailed::SmallIntestineTerminalIleum
                }
                pbs::genes::base::GtexTissueDetailed::Spleen => GenesGtexTissueDetailed::Spleen,
                pbs::genes::base::GtexTissueDetailed::Stomach => GenesGtexTissueDetailed::Stomach,
                pbs::genes::base::GtexTissueDetailed::Testis => GenesGtexTissueDetailed::Testis,
                pbs::genes::base::GtexTissueDetailed::Thyroid => GenesGtexTissueDetailed::Thyroid,
                pbs::genes::base::GtexTissueDetailed::Uterus => GenesGtexTissueDetailed::Uterus,
                pbs::genes::base::GtexTissueDetailed::Vagina => GenesGtexTissueDetailed::Vagina,
                pbs::genes::base::GtexTissueDetailed::WholeBlood => {
                    GenesGtexTissueDetailed::WholeBlood
                }
            })
        }
    }

    /// Information about a gene.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesGeneInfoRecord {
        /// Information from the ACMG secondary finding list.
        pub acmg_sf: Option<GenesAcmgSecondaryFindingRecord>,
        /// Information from ClinGen dosage curation.
        pub clingen: Option<GenesClingenDosageRecord>,
        /// Information from dbNSFP.
        pub dbnsfp: Option<GenesDbnsfpRecord>,
        /// Information from the gnomAD constraints database.
        pub gnomad_constraints: Option<GenesGnomadConstraintsRecord>,
        /// Information from the HGNC database.
        pub hgnc: Option<GenesHgncRecord>,
        /// Information from the NCBI gene database (aka "Entrez").
        pub ncbi: Option<GenesNcbiRecord>,
        /// Information about gene to OMIM term annotation, composed from clingen and HPO.
        pub omim: Option<GenesOmimRecord>,
        /// Information about gene to Orphanet annotation, derived from Orphapacket.
        pub orpha: Option<GenesOrphaRecord>,
        /// Information from the rCNV dosage sensitivity scores (Collins et al., 2022).
        pub rcnv: Option<GenesRcnvRecord>,
        /// Information from the sHet score (Weghor et al., 2019)
        pub shet: Option<GenesShetRecord>,
        /// Information from GTEx data
        pub gtex: Option<GenesGtexRecord>,
        /// Information from DOMINO.
        pub domino: Option<GenesDominoRecord>,
        /// DECIPHER HI score.
        pub decipher_hi: Option<GenesDecipherHiRecord>,
        /// GenomicsEngland PanelApp gene information.
        pub panelapp: Vec<GenesPanelAppRecord>,
        /// Conditions record.
        pub conditions: Option<GenesConditionsRecord>,
    }

    impl TryFrom<pbs::genes::base::Record> for GenesGeneInfoRecord {
        type Error = anyhow::Error;

        fn try_from(value: pbs::genes::base::Record) -> Result<Self, Self::Error> {
            Ok(GenesGeneInfoRecord {
                acmg_sf: value.acmg_sf.map(TryInto::try_into).transpose()?,
                clingen: value.clingen.map(TryInto::try_into).transpose()?,
                dbnsfp: value.dbnsfp.map(TryInto::try_into).transpose()?,
                gnomad_constraints: value
                    .gnomad_constraints
                    .map(TryInto::try_into)
                    .transpose()?,
                hgnc: value.hgnc.map(TryInto::try_into).transpose()?,
                ncbi: value.ncbi.map(TryInto::try_into).transpose()?,
                omim: value.omim.map(TryInto::try_into).transpose()?,
                orpha: value.orpha.map(TryInto::try_into).transpose()?,
                rcnv: value.rcnv.map(TryInto::try_into).transpose()?,
                shet: value.shet.map(TryInto::try_into).transpose()?,
                gtex: value.gtex.map(TryInto::try_into).transpose()?,
                domino: value.domino.map(TryInto::try_into).transpose()?,
                decipher_hi: value.decipher_hi.map(TryInto::try_into).transpose()?,
                panelapp: value
                    .panelapp
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?,
                conditions: value.conditions.map(TryInto::try_into).transpose()?,
            })
        }
    }

    /// Query response for `handle_with_openapi()`.
    #[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct GenesInfoResponse {
        /// Version information of the genes.
        pub genes: Vec<GenesGeneInfoRecord>,
    }

    impl TryFrom<super::Container> for GenesInfoResponse {
        type Error = anyhow::Error;

        fn try_from(value: super::Container) -> Result<Self, Self::Error> {
            Ok(GenesInfoResponse {
                genes: value
                    .genes
                    .into_values()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?,
            })
        }
    }
}

use response::*;

/// Query for annotations for one or more genes.
#[utoipa::path(
    get,
    operation_id = "genesInfo",
    params(GenesInfoQuery),
    responses(
        (status = 200, description = "Per-gene information.", body = GenesInfoResponse),
        (status = 500, description = "Internal server error.", body = CustomError)
    )
)]
#[get("/api/v1/genes/info")]
async fn handle_with_openapi(
    data: Data<crate::server::run::WebServerData>,
    _path: Path<()>,
    query: web::Query<GenesInfoQuery>,
) -> actix_web::Result<Json<GenesInfoResponse>, CustomError> {
    let container = handle_impl(data, _path, query).await?;
    let response = container
        .try_into()
        .map_err(|e| CustomError::new(anyhow::anyhow!("Failed to convert response: {}", e)))?;
    Ok(Json(response))
}
