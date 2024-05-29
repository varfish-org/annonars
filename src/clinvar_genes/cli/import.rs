//! Import of minimal ClinVar data.

use std::{collections::HashSet, io::BufRead, sync::Arc};

use clap::Parser;
use prost::Message;

use crate::common;
use crate::pbs::clinvar::per_gene::{ClinvarPerGeneRecord, ExtractedVariantsPerRelease};
use crate::pbs::clinvar_data::class_by_freq::GeneCoarseClinsigFrequencyCounts;
use crate::pbs::clinvar_data::extracted_vars::ExtractedVcvRecord;
use crate::pbs::clinvar_data::gene_impact::GeneImpactCounts;

/// Command line arguments for `tsv import` sub command.
#[derive(Parser, Debug, Clone)]
#[command(about = "import ClinVar per-gene data into RocksDB", long_about = None)]
pub struct Args {
    /// Path to input per-impact JSONL file(s).
    #[arg(long, required = true)]
    pub path_per_impact_jsonl: String,
    /// Path to input per-frequency JSONL file(s).
    #[arg(long, required = true)]
    pub path_per_frequency_jsonl: String,
    /// Paths to variant JSONL files.
    #[arg(long, required = true)]
    pub paths_variant_jsonl: Vec<String>,
    /// Path to output RocksDB directory.
    #[arg(long)]
    pub path_out_rocksdb: String,

    /// Name of the column family to import into.
    #[arg(long, default_value = "clinvar-genes")]
    pub cf_name: String,
    /// Optional path to RocksDB WAL directory.
    #[arg(long)]
    pub path_wal_dir: Option<String>,
}

/// Load per-impact JSONL file.
fn load_per_impact_jsonl(
    path_per_impact_jsonl: &str,
) -> Result<indexmap::IndexMap<String, GeneImpactCounts>, anyhow::Error> {
    // Open reader, possibly decompressing gziped files.
    let reader: Box<dyn std::io::Read> = if path_per_impact_jsonl.ends_with(".gz") {
        Box::new(flate2::read::GzDecoder::new(std::fs::File::open(
            path_per_impact_jsonl,
        )?))
    } else {
        Box::new(std::fs::File::open(path_per_impact_jsonl)?)
    };

    let mut result = indexmap::IndexMap::new();

    let reader = std::io::BufReader::new(reader);
    for line in reader.lines() {
        let line = line?;
        let record = serde_json::from_str::<GeneImpactCounts>(&line)?;
        result.insert(record.hgnc_id.clone(), record);
    }

    Ok(result)
}

/// Load per-frequency JSONL file.
fn load_per_frequency_jsonl(
    path_per_impact_jsonl: &str,
) -> Result<indexmap::IndexMap<String, GeneCoarseClinsigFrequencyCounts>, anyhow::Error> {
    // Open reader, possibly decompressing gziped files.
    let reader: Box<dyn std::io::Read> = if path_per_impact_jsonl.ends_with(".gz") {
        Box::new(flate2::read::GzDecoder::new(std::fs::File::open(
            path_per_impact_jsonl,
        )?))
    } else {
        Box::new(std::fs::File::open(path_per_impact_jsonl)?)
    };

    let mut result = indexmap::IndexMap::new();

    let reader = std::io::BufReader::new(reader);
    for line in reader.lines() {
        let line = line?;
        let record = serde_json::from_str::<GeneCoarseClinsigFrequencyCounts>(&line)?;
        result.insert(record.hgnc_id.clone(), record);
    }

    Ok(result)
}

type PerRelease = indexmap::IndexMap<String, Vec<ExtractedVcvRecord>>;
type PerGene = indexmap::IndexMap<String, PerRelease>;

/// Load per-gene sequence variants.
fn load_variants_jsonl(
    variant_jsonls: &[String],
) -> Result<indexmap::IndexMap<String, Vec<ExtractedVariantsPerRelease>>, anyhow::Error> {
    // Build intermediate data structure using nested maps.
    let mut per_gene: PerGene = Default::default();

    for path_jsonl in variant_jsonls {
        let reader: Box<dyn std::io::Read> = if path_jsonl.ends_with(".gz") {
            Box::new(flate2::read::GzDecoder::new(std::fs::File::open(
                path_jsonl,
            )?))
        } else {
            Box::new(std::fs::File::open(path_jsonl)?)
        };

        let reader = std::io::BufReader::new(reader);

        for line in reader.lines() {
            let line = line?;
            let input_record = serde_json::from_str::<ExtractedVcvRecord>(&line);
            match input_record {
                Err(e) => {
                    tracing::warn!("skipping line because of error: {}", e);
                    continue;
                }
                Ok(input_record) => {
                    for hgnc_id in &input_record.hgnc_ids {
                        let per_gene_entry = per_gene.entry(hgnc_id.clone()).or_default();
                        let release = input_record
                            .sequence_location
                            .as_ref()
                            .expect("missing sequence_location")
                            .assembly
                            .clone();
                        let per_release_entry = per_gene_entry.entry(release.clone()).or_default();
                        per_release_entry.push(input_record.clone());
                    }
                }
            }
        }
    }

    // Convert into final data structure that uses lists of entry records rather than nested maps.
    let mut result = indexmap::IndexMap::new();
    for (hgnc_id, per_release) in per_gene {
        let mut per_gene_out = Vec::new();
        for (release, extracted_vars) in per_release {
            let mut variants = extracted_vars;
            variants.sort_by(|a, b| {
                a.accession
                    .as_ref()
                    .expect("no accession")
                    .accession
                    .cmp(&b.accession.as_ref().expect("no accession").accession)
            });
            per_gene_out.push(ExtractedVariantsPerRelease {
                release: Some(release),
                variants,
            });
        }
        result.insert(hgnc_id, per_gene_out);
    }

    Ok(result)
}

/// Perform import of the JSONL files.
fn jsonl_import(
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    args: &Args,
) -> Result<(), anyhow::Error> {
    let cf_data = db.cf_handle(&args.cf_name).unwrap();

    tracing::info!("Loading impact per gene ...");
    let before_per_impact = std::time::Instant::now();
    let counts_per_impact = load_per_impact_jsonl(&args.path_per_impact_jsonl)?;
    tracing::info!(
        "... done loading impact per gene in {:?}",
        &before_per_impact.elapsed()
    );
    tracing::info!("Loading frequency per significance and gene ...");
    let before_per_freq = std::time::Instant::now();
    let counts_per_freq = load_per_frequency_jsonl(&args.path_per_frequency_jsonl)?;
    tracing::info!(
        "... done loading impact frequency per significance gene in {:?}",
        &before_per_freq.elapsed()
    );
    tracing::info!("Loading variants per gene ...");
    let before_vars = std::time::Instant::now();
    let vars_per_gene = load_variants_jsonl(&args.paths_variant_jsonl)?;
    tracing::info!(
        "... done loading variants per gene in {:?}",
        &before_vars.elapsed()
    );

    tracing::info!("Writing to database ...");
    let before_write_to_db = std::time::Instant::now();
    let mut hgnc_ids = counts_per_impact
        .keys()
        .cloned()
        .chain(counts_per_freq.keys().cloned())
        .chain(vars_per_gene.keys().cloned())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    hgnc_ids.sort();

    // Read through all records and insert each into the database.
    for hgnc_id in hgnc_ids.iter() {
        let per_release_vars = vars_per_gene.get(hgnc_id).cloned().unwrap_or_default();
        let record = ClinvarPerGeneRecord {
            per_impact_counts: Some(counts_per_impact.get(hgnc_id).cloned().unwrap_or_default()),
            per_freq_counts: Some(counts_per_freq.get(hgnc_id).cloned().unwrap_or_default()),
            per_release_vars,
        };
        let buf = record.encode_to_vec();

        db.put_cf(&cf_data, hgnc_id, buf)?;
    }
    tracing::info!(
        "... done writing to database in {:?}",
        &before_write_to_db.elapsed()
    );

    Ok(())
}

/// Implementation of `clinvar-genes import` sub command.
pub fn run(common: &common::cli::Args, args: &Args) -> Result<(), anyhow::Error> {
    tracing::info!("Starting 'clinvar-genes import' command");
    tracing::info!("common = {:#?}", &common);
    tracing::info!("args = {:#?}", &args);

    // Open the RocksDB for writing.
    tracing::info!("Opening RocksDB for writing ...");
    let before_opening_rocksdb = std::time::Instant::now();
    let options = rocksdb_utils_lookup::tune_options(
        rocksdb::Options::default(),
        args.path_wal_dir.as_ref().map(|s| s.as_ref()),
    );
    let cf_names = &["meta", &args.cf_name];
    let db = Arc::new(rocksdb::DB::open_cf_with_opts(
        &options,
        common::readlink_f(&args.path_out_rocksdb)?,
        cf_names
            .iter()
            .map(|name| (name.to_string(), options.clone()))
            .collect::<Vec<_>>(),
    )?);
    tracing::info!("  writing meta information");
    let cf_meta = db.cf_handle("meta").unwrap();
    db.put_cf(&cf_meta, "annonars-version", crate::VERSION)?;
    db.put_cf(&cf_meta, "db-name", "clinvar-genes")?;
    tracing::info!(
        "... done opening RocksDB for writing in {:?}",
        before_opening_rocksdb.elapsed()
    );

    tracing::info!("Importing TSV files ...");
    let before_import = std::time::Instant::now();
    jsonl_import(&db, args)?;
    tracing::info!(
        "... done importing TSV files in {:?}",
        before_import.elapsed()
    );

    tracing::info!("Running RocksDB compaction ...");
    let before_compaction = std::time::Instant::now();
    rocksdb_utils_lookup::force_compaction_cf(&db, cf_names, Some("  "), true)?;
    tracing::info!(
        "... done compacting RocksDB in {:?}",
        before_compaction.elapsed()
    );

    tracing::info!("All done. Have a nice day!");
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    use clap_verbosity_flag::Verbosity;
    use temp_testdir::TempDir;

    #[test]
    fn smoke_test_import() {
        let tmp_dir = TempDir::default();
        let common = common::cli::Args {
            verbose: Verbosity::new(1, 0),
        };
        let args = Args {
            path_per_impact_jsonl: String::from("tests/clinvar-genes/gene-variant-report.jsonl"),
            path_per_frequency_jsonl: String::from(
                "tests/clinvar-genes/gene-frequency-report.jsonl",
            ),
            paths_variant_jsonl: vec![
                String::from("tests/clinvar-genes/clinvar-variants-grch37-seqvars.jsonl"),
                String::from("tests/clinvar-genes/clinvar-variants-grch38-seqvars.jsonl"),
            ],
            path_out_rocksdb: format!("{}", tmp_dir.join("out-rocksdb").display()),
            cf_name: String::from("clinvar"),
            path_wal_dir: None,
        };

        run(&common, &args).unwrap();
    }
}
