//! Import of gene annotation data.

use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
    time::Instant,
};

use clap::Parser;
use indicatif::ProgressIterator;
use prost::Message;
use tracing::info;

use crate::{
    common::{self, version},
    genes::pbs,
};

use super::data::{
    self, acmg_sf, clingen_gene, dbnsfp_gene, gnomad_constraints, hgnc, ncbi, rcnv, shet,
};

/// Command line arguments for `genes import` sub command.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "Import gene annotation into database", long_about = None)]
pub struct Args {
    /// Path to the TSV file with ACMG secondary findings list.
    #[arg(long, required = true)]
    pub path_in_acmg: String,
    /// Path to the CSV file with ClinGen curations.
    #[arg(long, required = true)]
    pub path_in_clingen: String,
    /// Path to the TSV file with gnomAD gene constraints.
    #[arg(long, required = true)]
    pub path_in_gnomad_constraints: String,
    /// Path to the TSV file with dbNSFP gene information.
    #[arg(long, required = true)]
    pub path_in_dbnsfp: String,
    /// Path to the JSONL file with HGNC information.
    #[arg(long, required = true)]
    pub path_in_hgnc: String,
    /// Path to the JSONL file with NCBI information.
    #[arg(long, required = true)]
    pub path_in_ncbi: String,
    /// Path to the TSV file with rCNV information.
    #[arg(long, required = true)]
    pub path_in_rcnv: String,
    /// Path to the TSV file with sHet information.
    #[arg(long, required = true)]
    pub path_in_shet: String,

    /// Path to output RocksDB.
    #[arg(long, required = true)]
    pub path_out_rocksdb: String,
}

/// Load ACMG SF list.
///
/// # Result
///
/// A map from HGNC ID to ACMG SF record.
fn load_acmg(path: &str) -> Result<HashMap<String, acmg_sf::Record>, anyhow::Error> {
    info!("  loading ACMG SF list from {}", path);
    let mut result = HashMap::new();

    let mut reader = csv::ReaderBuilder::new().delimiter(b'\t').from_path(path)?;
    for record in reader.deserialize::<acmg_sf::Record>() {
        let record = record?;
        result.insert(record.hgnc_id.clone(), record);
    }

    Ok(result)
}

/// Load ClinGen CSV file.
///
/// # Result
///
/// A map from HGNC ID to ClinGen record.
fn load_clingen(path: &str) -> Result<HashMap<String, clingen_gene::Record>, anyhow::Error> {
    info!("  loading ClinGen curations from {}", path);
    let mut result = HashMap::new();

    let mut reader = csv::ReaderBuilder::new().from_path(path)?;
    for record in reader.deserialize::<clingen_gene::Record>() {
        let record = record.unwrap();
        result.insert(record.hgnc_id.clone(), record);
    }

    Ok(result)
}

/// Load gnomAD constraints.
///
/// # Result
///
/// A map from ENSEMBL gene ID to gnomAD constraints record.
fn load_gnomad_constraints(
    path: &str,
) -> Result<HashMap<String, gnomad_constraints::Record>, anyhow::Error> {
    info!("  loading gnomAD constraints from {}", path);
    let mut result = HashMap::new();

    let mut reader = csv::ReaderBuilder::new().delimiter(b'\t').from_path(path)?;
    for record in reader.deserialize::<gnomad_constraints::Record>() {
        let record = record?;
        result.insert(record.ensembl_gene_id.clone(), record);
    }

    Ok(result)
}

/// Load dbNSFP genes information.
///
/// # Result
///
/// A map from HGNC gene symbol to dbNSFP gene information.
fn load_dbnsfp(path: &str) -> Result<HashMap<String, dbnsfp_gene::Record>, anyhow::Error> {
    info!("  loading dbNSFP gene information from {}", path);
    let mut result = HashMap::new();

    let reader: Box<dyn Read> = if path.ends_with(".gz") {
        Box::new(flate2::bufread::MultiGzDecoder::new(BufReader::new(
            std::fs::File::open(path)?,
        )))
    } else {
        Box::new(BufReader::new(std::fs::File::open(path)?))
    };

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(reader);
    for record in reader.deserialize::<dbnsfp_gene::Record>() {
        let record = record?;
        result.insert(record.gene_name.clone(), record);
    }

    Ok(result)
}

/// Load HGNC information.
///
/// # Result
///
/// A map from HGNC ID to HGNC record.
fn load_hgnc(path: &str) -> Result<HashMap<String, hgnc::Record>, anyhow::Error> {
    info!("  loading HGNC information from {}", path);
    let mut result = HashMap::new();

    let reader = std::fs::File::open(path).map(std::io::BufReader::new)?;
    for line in reader.lines() {
        let line = line?;
        let record = serde_json::from_str::<hgnc::Record>(&line)?;
        result.insert(record.hgnc_id.clone(), record);
    }

    Ok(result)
}

/// Load NCBI information.
///
/// # Result
///
/// A map from NCBI gene ID to NCBI record.
fn load_ncbi(path: &str) -> Result<HashMap<String, ncbi::Record>, anyhow::Error> {
    info!("  loading NCBI information from {}", path);
    let mut result = HashMap::new();

    let reader = std::fs::File::open(path).map(std::io::BufReader::new)?;
    for line in reader.lines() {
        let line = line?;
        let record = serde_json::from_str::<ncbi::Record>(&line)?;
        result.insert(record.gene_id.clone(), record);
    }

    Ok(result)
}

/// Load rCNV (Collins et al., 2022) information.
///
/// # Result
///
/// A map from HGNC ID to rCNV record.
fn load_rcnv(path: &str) -> Result<HashMap<String, rcnv::Record>, anyhow::Error> {
    info!("  loading rCNV information from {}", path);
    let mut result = HashMap::new();

    let mut reader = csv::ReaderBuilder::new().delimiter(b'\t').from_path(path)?;
    for record in reader.deserialize::<rcnv::Record>() {
        let record = record?;
        result.insert(record.hgnc_id.clone(), record);
    }

    Ok(result)
}

/// Load sHet (Weghorn et al., 2019) information.
///
/// # Result
///
/// A map from HGNC ID to sHet record.
fn load_shet(path: &str) -> Result<HashMap<String, shet::Record>, anyhow::Error> {
    info!("  loading sHet information from {}", path);
    let mut result = HashMap::new();

    let mut reader = csv::ReaderBuilder::new().delimiter(b'\t').from_path(path)?;
    for record in reader.deserialize::<shet::Record>() {
        let record = record?;
        result.insert(record.hgnc_id.clone(), record);
    }

    Ok(result)
}

/// Convert from `data::*` records to `pbs::*` records.
fn convert_record(record: data::Record) -> pbs::Record {
    let data::Record {
        acmg_sf,
        clingen,
        dbnsfp,
        gnomad_constraints,
        hgnc,
        ncbi,
        rcnv,
        shet,
    } = record;

    let acmg_sf = acmg_sf.map(|acmg_sf| {
        let acmg_sf::Record {
            hgnc_id,
            ensembl_gene_id,
            ncbi_gene_id,
            gene_symbol,
            mim_gene_id,
            disease_phenotype,
            disorder_mim,
            phenotype_category,
            inheritance,
            sf_list_version,
            variants_to_report,
        } = acmg_sf;

        pbs::AcmgSecondaryFindingRecord {
            hgnc_id,
            ensembl_gene_id,
            ncbi_gene_id,
            gene_symbol,
            mim_gene_id,
            disease_phenotype,
            disorder_mim,
            phenotype_category,
            inheritance,
            sf_list_version,
            variants_to_report,
        }
    });

    let clingen = clingen.map(|clingen| {
        let clingen_gene::Record {
            gene_symbol,
            hgnc_id,
            gene_url,
            disease_label,
            mondo_id,
            disease_url,
            mode_of_inheritance,
            dosage_haploinsufficiency_assertion,
            dosage_triplosensitivity_assertion,
            dosage_report,
            dosage_group,
            gene_disease_validity_assertion_classifications,
            gene_disease_validity_assertion_reports,
            gene_disease_validity_gceps,
            actionability_assertion_classifications,
            actionability_assertion_reports,
            actionability_groups,
        } = clingen;

        pbs::ClingenCurationRecord {
            gene_symbol,
            hgnc_id,
            gene_url,
            disease_label,
            mondo_id,
            disease_url,
            mode_of_inheritance,
            dosage_haploinsufficiency_assertion,
            dosage_triplosensitivity_assertion,
            dosage_report,
            dosage_group,
            gene_disease_validity_assertion_classifications,
            gene_disease_validity_assertion_reports,
            gene_disease_validity_gceps,
            actionability_assertion_classifications,
            actionability_assertion_reports,
            actionability_groups,
        }
    });

    let dbnsfp = dbnsfp.map(|dbnsfp| {
        let dbnsfp_gene::Record {
            gene_name,
            ensembl_gene,
            chr,
            gene_old_names,
            gene_other_names,
            uniprot_acc,
            uniprot_id,
            entrez_gene_id,
            ccds_id,
            refseq_id,
            ucsc_id,
            mim_id,
            omim_id,
            gene_full_name,
            pathway_uniprot,
            pathway_biocarta_short,
            pathway_biocarta_full,
            pathway_consensus_path_db,
            pathway_kegg_id,
            pathway_kegg_full,
            function_description,
            disease_description,
            mim_phenotype_id,
            mim_disease,
            orphanet_disorder_id,
            orphanet_disorder,
            orphanet_association_type,
            trait_association_gwas,
            hpo_id,
            hpo_name,
            go_biological_process,
            go_cellular_component,
            go_molecular_function,
            tissue_specificity_uniprot,
            expression_egenetics,
            expression_gnf_atlas,
            interactions_intact,
            interactions_biogrid,
            interactions_consensus_path_db,
            haploinsufficiency,
            hipred_score,
            hipred,
            ghis,
            prec,
            known_rec_info,
            rvis_evs,
            rvis_percentile_evs,
            lof_fdr_exac,
            rvis_exac,
            rvis_percentile_exac,
            exac_pli,
            exac_prec,
            exac_pnull,
            exac_nontcga_pli,
            exac_nontcga_prec,
            exac_nontcga_pnull,
            exac_nonpsych_pli,
            exac_nonpsych_prec,
            exac_nonpsych_pnull,
            gnomad_pli,
            gnomad_prec,
            gnomad_pnull,
            exac_del_score,
            exac_dup_score,
            exac_cnv_score,
            exac_cnv_flag,
            gdi,
            gdi_phred,
            gdp_all_disease_causing,
            gdp_all_mendelian,
            gdp_all_mendelian_ad,
            gdp_mendelian_ar,
            gdp_pid,
            gdp_pid_ad,
            gdp_pid_ar,
            gdp_cancer,
            gdb_cancer_rec,
            gdp_cancer_dom,
            loftool_score,
            sorva_lof_maf_5_het_or_hom,
            sorva_lof_maf_5_hom_or_comphet,
            sorva_lof_maf_1_het_or_hom,
            sorva_lof_maf_1_hom_or_comphet,
            sorva_lof_or_mis_maf_5_het_or_hom,
            sorva_lof_or_mis_maf_5_hom_or_comphet,
            sorva_lof_or_mis_maf_1_het_or_hom,
            sorva_lof_or_mis_maf_1_hom_or_comphet,
            essential_gene,
            essential_gene_crispr,
            essential_gene_crispr2,
            essential_gene_gene_trap,
            gene_indispensability_score,
            gene_indispensability_pred,
            mgi_mouse_gene,
            mgi_mouse_phenotype,
            zfin_zebrafish_gene,
            zfin_zebrafish_structure,
            zfin_zebrafish_phenotype_quality,
            zfin_zebrafish_phenotype_tag,
        } = dbnsfp;

        pbs::DbnsfpRecord {
            gene_name,
            ensembl_gene,
            chr,
            gene_old_names,
            gene_other_names,
            uniprot_acc,
            uniprot_id,
            entrez_gene_id,
            ccds_id,
            refseq_id,
            ucsc_id,
            mim_id,
            omim_id,
            gene_full_name,
            pathway_uniprot,
            pathway_biocarta_short,
            pathway_biocarta_full,
            pathway_consensus_path_db,
            pathway_kegg_id,
            pathway_kegg_full,
            function_description,
            disease_description,
            mim_phenotype_id,
            mim_disease,
            orphanet_disorder_id,
            orphanet_disorder,
            orphanet_association_type,
            trait_association_gwas,
            hpo_id,
            hpo_name,
            go_biological_process,
            go_cellular_component,
            go_molecular_function,
            tissue_specificity_uniprot,
            expression_egenetics,
            expression_gnf_atlas,
            interactions_intact,
            interactions_biogrid,
            interactions_consensus_path_db,
            haploinsufficiency,
            hipred_score,
            hipred,
            ghis,
            prec,
            known_rec_info,
            rvis_evs,
            rvis_percentile_evs,
            lof_fdr_exac,
            rvis_exac,
            rvis_percentile_exac,
            exac_pli,
            exac_prec,
            exac_pnull,
            exac_nontcga_pli,
            exac_nontcga_prec,
            exac_nontcga_pnull,
            exac_nonpsych_pli,
            exac_nonpsych_prec,
            exac_nonpsych_pnull,
            gnomad_pli,
            gnomad_prec,
            gnomad_pnull,
            exac_del_score,
            exac_dup_score,
            exac_cnv_score,
            exac_cnv_flag,
            gdi,
            gdi_phred,
            gdp_all_disease_causing,
            gdp_all_mendelian,
            gdp_all_mendelian_ad,
            gdp_mendelian_ar,
            gdp_pid,
            gdp_pid_ad,
            gdp_pid_ar,
            gdp_cancer,
            gdb_cancer_rec,
            gdp_cancer_dom,
            loftool_score,
            sorva_lof_maf_5_het_or_hom,
            sorva_lof_maf_5_hom_or_comphet,
            sorva_lof_maf_1_het_or_hom,
            sorva_lof_maf_1_hom_or_comphet,
            sorva_lof_or_mis_maf_5_het_or_hom,
            sorva_lof_or_mis_maf_5_hom_or_comphet,
            sorva_lof_or_mis_maf_1_het_or_hom,
            sorva_lof_or_mis_maf_1_hom_or_comphet,
            essential_gene,
            essential_gene_crispr,
            essential_gene_crispr2,
            essential_gene_gene_trap,
            gene_indispensability_score,
            gene_indispensability_pred,
            mgi_mouse_gene,
            mgi_mouse_phenotype,
            zfin_zebrafish_gene,
            zfin_zebrafish_structure,
            zfin_zebrafish_phenotype_quality,
            zfin_zebrafish_phenotype_tag,
        }
    });

    let gnomad_constraints = gnomad_constraints.map(|gnomad_constraints| {
        let gnomad_constraints::Record {
            ensembl_gene_id,
            entrez_id,
            gene_symbol,
            exp_lof,
            exp_mis,
            exp_syn,
            mis_z,
            obs_lof,
            obs_mis,
            obs_syn,
            oe_lof,
            oe_lof_lower,
            oe_lof_upper,
            oe_mis,
            oe_mis_lower,
            oe_mis_upper,
            oe_syn,
            oe_syn_lower,
            oe_syn_upper,
            pli,
            syn_z,
            exac_pli,
            exac_obs_lof,
            exac_exp_lof,
            exac_oe_lof,
        } = gnomad_constraints;

        pbs::GnomadConstraintsRecord {
            ensembl_gene_id,
            entrez_id,
            gene_symbol,
            exp_lof,
            exp_mis,
            exp_syn,
            mis_z,
            obs_lof,
            obs_mis,
            obs_syn,
            oe_lof,
            oe_lof_lower,
            oe_lof_upper,
            oe_mis,
            oe_mis_lower,
            oe_mis_upper,
            oe_syn,
            oe_syn_lower,
            oe_syn_upper,
            pli,
            syn_z,
            exac_pli,
            exac_obs_lof,
            exac_exp_lof,
            exac_oe_lof,
        }
    });

    let hgnc = {
        let hgnc::Record {
            hgnc_id,
            symbol,
            name,
            locus_group,
            locus_type,
            status,
            location,
            location_sortable,
            alias_symbol,
            alias_name,
            prev_symbol,
            prev_name,
            gene_group,
            gene_group_id,
            date_approved_reserved,
            date_symbol_changed,
            date_name_changed,
            date_modified,
            entrez_id,
            ensembl_gene_id,
            vega_id,
            ucsc_id,
            ena,
            refseq_accession,
            ccds_id,
            uniprot_ids,
            pubmed_id,
            mgd_id,
            rgd_id,
            lsdb,
            cosmic,
            omim_id,
            mirbase,
            homeodb,
            snornabase,
            bioparadigms_slc,
            orphanet,
            pseudogene_org,
            horde_id,
            merops,
            imgt,
            iuphar,
            mamit_trnadb,
            cd,
            lncrnadb,
            enzyme_id,
            intermediate_filament_db,
            agr,
            mane_select,
        } = hgnc;

        Some(pbs::HgncRecord {
            hgnc_id,
            symbol,
            name,
            locus_group,
            locus_type,
            status: status as i32,
            location,
            location_sortable,
            alias_symbol: alias_symbol.unwrap_or_default(),
            alias_name: alias_name.unwrap_or_default(),
            prev_symbol: prev_symbol.unwrap_or_default(),
            prev_name: prev_name.unwrap_or_default(),
            gene_group: gene_group.unwrap_or_default(),
            gene_group_id: gene_group_id.unwrap_or_default(),
            date_approved_reserved: date_approved_reserved
                .map(|d| d.format("%Y-%m-%d").to_string()),
            date_symbol_changed: date_symbol_changed.map(|d| d.format("%Y-%m-%d").to_string()),
            date_name_changed: date_name_changed.map(|d| d.format("%Y-%m-%d").to_string()),
            date_modified: date_modified.map(|d| d.format("%Y-%m-%d").to_string()),
            entrez_id,
            ensembl_gene_id,
            vega_id,
            ucsc_id,
            ena: ena.unwrap_or_default(),
            refseq_accession: refseq_accession.unwrap_or_default(),
            ccds_id: ccds_id.unwrap_or_default(),
            uniprot_ids: uniprot_ids.unwrap_or_default(),
            pubmed_id: pubmed_id.unwrap_or_default(),
            mgd_id: mgd_id.unwrap_or_default(),
            rgd_id: rgd_id.unwrap_or_default(),
            lsdb: lsdb
                .map(|lsdb| {
                    lsdb.iter()
                        .map(|lsdb| pbs::HgncLsdb {
                            name: lsdb.name.clone(),
                            url: lsdb.url.clone(),
                        })
                        .collect()
                })
                .unwrap_or_default(),
            cosmic,
            omim_id: omim_id.unwrap_or_default(),
            mirbase,
            homeodb,
            snornabase,
            bioparadigms_slc,
            orphanet,
            pseudogene_org,
            horde_id,
            merops,
            imgt,
            iuphar,
            mamit_trnadb,
            cd,
            lncrnadb,
            enzyme_id: enzyme_id.unwrap_or_default(),
            intermediate_filament_db,
            agr,
            mane_select: mane_select.unwrap_or_default(),
        })
    };

    let ncbi = ncbi.map(|ncbi| {
        let ncbi::Record {
            gene_id,
            summary,
            rif_entries,
        } = ncbi;
        pbs::NcbiRecord {
            gene_id,
            summary,
            rif_entries: rif_entries
                .map(|rif_entries| {
                    rif_entries
                        .into_iter()
                        .map(|rif_entry| pbs::RifEntry {
                            pmids: rif_entry.pmids.unwrap_or_default(),
                            text: rif_entry.text,
                        })
                        .collect()
                })
                .unwrap_or_default(),
        }
    });

    let rcnv = rcnv.map(|rcnv| {
        let rcnv::Record {
            hgnc_id,
            p_haplo,
            p_triplo,
        } = rcnv;
        pbs::RcnvRecord {
            hgnc_id,
            p_haplo,
            p_triplo,
        }
    });

    let shet = shet.map(|shet| {
        let shet::Record { hgnc_id, s_het } = shet;
        pbs::ShetRecord { hgnc_id, s_het }
    });

    pbs::Record {
        acmg_sf,
        clingen,
        dbnsfp,
        gnomad_constraints,
        hgnc,
        ncbi,
        rcnv,
        shet,
    }
}

/// Write gene database to a RocksDB.
#[allow(clippy::too_many_arguments)]
fn write_rocksdb(
    acmg_by_hgnc_id: HashMap<String, acmg_sf::Record>,
    clingen_by_hgnc_id: HashMap<String, clingen_gene::Record>,
    dbnsfp_by_symbol: HashMap<String, dbnsfp_gene::Record>,
    constraints_by_ensembl_id: HashMap<String, gnomad_constraints::Record>,
    hgnc: HashMap<String, hgnc::Record>,
    ncbi_by_ncbi_id: HashMap<String, ncbi::Record>,
    rcnv_by_hgnc_id: HashMap<String, rcnv::Record>,
    shet_by_hgnc_id: HashMap<String, shet::Record>,
    args: &&Args,
) -> Result<(), anyhow::Error> {
    // Construct RocksDB options and open file for writing.
    let options = rocksdb_utils_lookup::tune_options(rocksdb::Options::default(), None);
    let db = rocksdb::DB::open_cf_with_opts(
        &options,
        &args.path_out_rocksdb,
        ["meta", "genes"]
            .iter()
            .map(|name| (name.to_string(), options.clone()))
            .collect::<Vec<_>>(),
    )?;

    let cf_meta = db.cf_handle("meta").unwrap();
    let cf_genes = db.cf_handle("genes").unwrap();

    tracing::info!("  writing meta data to database");
    db.put_cf(&cf_meta, "builder-version", version())?;
    // TODO: read meta information about input data and write out

    tracing::info!("  compose genes data into database");
    for hgnc_record in hgnc
        .values()
        .progress_with(common::cli::progress_bar(hgnc.len()))
    {
        let hgnc_id = hgnc_record.hgnc_id.clone();
        let record = convert_record(data::Record {
            acmg_sf: acmg_by_hgnc_id.get(&hgnc_id).cloned(),
            clingen: clingen_by_hgnc_id.get(&hgnc_id).cloned(),
            dbnsfp: dbnsfp_by_symbol.get(&hgnc_record.symbol).cloned(),
            gnomad_constraints: hgnc_record
                .ensembl_gene_id
                .as_ref()
                .map(|ensembl_gene_id| constraints_by_ensembl_id.get(ensembl_gene_id).cloned())
                .unwrap_or_default(),
            hgnc: hgnc_record.clone(),
            ncbi: hgnc_record
                .entrez_id
                .as_ref()
                .map(|entrez_id| ncbi_by_ncbi_id.get(entrez_id).cloned())
                .unwrap_or_default(),
            rcnv: rcnv_by_hgnc_id.get(&hgnc_id).cloned(),
            shet: shet_by_hgnc_id.get(&hgnc_id).cloned(),
        });
        tracing::debug!("writing {:?} -> {:?}", &hgnc, &record);
        db.put_cf(&cf_genes, hgnc_id, &record.encode_to_vec())?;
    }

    // Finally, compact manually.
    tracing::info!("  enforce manual compaction");
    rocksdb_utils_lookup::force_compaction_cf(&db, ["meta", "genes"], Some("  "), true)?;

    Ok(())
}

/// Main entry point for the `db gene build` command.
pub fn run(common_args: &common::cli::Args, args: &Args) -> Result<(), anyhow::Error> {
    info!("Starting `db gene build`");
    info!("  common_args = {:?}", &common_args);
    info!("  args = {:?}", &args);

    let before_loading = Instant::now();
    info!("Loading genes data files...");
    let acmg_by_hgnc_id = load_acmg(&args.path_in_acmg)?;
    let clingen_by_hgnc_id = load_clingen(&args.path_in_clingen)?;
    let constraints_by_ensembl_id = load_gnomad_constraints(&args.path_in_gnomad_constraints)?;
    let dbnsfp_by_symbol = load_dbnsfp(&args.path_in_dbnsfp)?;
    let hgnc = load_hgnc(&args.path_in_hgnc)?;
    let ncbi_by_ncbi_id = load_ncbi(&args.path_in_ncbi)?;
    let rcnv_by_hgnc_id = load_rcnv(&args.path_in_rcnv)?;
    let shet_by_hgnc_id = load_shet(&args.path_in_shet)?;
    info!(
        "... done loadin genes data files in {:?}",
        before_loading.elapsed()
    );

    let before_writing = Instant::now();
    info!("Writing genes database...");
    write_rocksdb(
        acmg_by_hgnc_id,
        clingen_by_hgnc_id,
        dbnsfp_by_symbol,
        constraints_by_ensembl_id,
        hgnc,
        ncbi_by_ncbi_id,
        rcnv_by_hgnc_id,
        shet_by_hgnc_id,
        &args,
    )?;
    info!(
        "... done writing genes database in {:?}",
        before_writing.elapsed()
    );

    Ok(())
}

#[cfg(test)]
pub mod test {
    use super::*;

    use crate::common;
    use clap_verbosity_flag::Verbosity;
    use temp_testdir::TempDir;

    #[test]
    fn smoke_test() -> Result<(), anyhow::Error> {
        let tmp_dir = TempDir::default();
        let common_args = common::cli::Args {
            verbose: Verbosity::new(1, 0),
        };
        let args = Args {
            path_in_acmg: String::from("tests/genes/acmg/acmg.tsv"),
            path_in_clingen: String::from("tests/genes/clingen/clingen.csv"),
            path_in_gnomad_constraints: String::from(
                "tests/genes/gnomad_constraints/gnomad_constraints.tsv",
            ),
            path_in_dbnsfp: String::from("tests/genes/dbnsfp/genes.tsv"),
            path_in_hgnc: String::from("tests/genes/hgnc/hgnc_info.jsonl"),
            path_in_ncbi: String::from("tests/genes/ncbi/gene_info.jsonl"),
            path_in_rcnv: String::from("tests/genes/rcnv/rcnv.tsv"),
            path_in_shet: String::from("tests/genes/shet/shet.tsv"),
            path_out_rocksdb: tmp_dir
                .to_path_buf()
                .into_os_string()
                .into_string()
                .unwrap(),
        };

        run(&common_args, &args)?;

        Ok(())
    }
}
