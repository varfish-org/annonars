//! Import of minimal ClinVar data.

use crate::common;
use crate::pbs::clinvar::per_gene::{ClinvarPerGeneRecord, ExtractedVariantsPerRelease};
use crate::pbs::clinvar_data::class_by_freq::GeneCoarseClinsigFrequencyCounts;
use crate::pbs::clinvar_data::extracted_vars::ExtractedVcvRecord;
use crate::pbs::clinvar_data::gene_impact::GeneImpactCounts;
use clap::Parser;
use prost::Message;
use rocksdb::{DBWithThreadMode, MultiThreaded};
use std::path::Path;
use std::{collections::HashSet, io::BufRead, sync::Arc};

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

type VariantsPerGeneDb = DBWithThreadMode<MultiThreaded>;
type Releases = HashSet<String>;

/// Load per-gene sequence variants.

fn load_variants_jsonl(
    variant_jsonls: &[String],
    db_path: impl AsRef<Path>,
    options: &rocksdb::Options,
) -> Result<(VariantsPerGeneDb, Releases), anyhow::Error> {
    tracing::info!("creating temporary RocksDB at {:?}", db_path.as_ref());

    let db: VariantsPerGeneDb = rocksdb::DB::open(options, db_path)?;
    let mut releases = Releases::default();

    for path_jsonl in variant_jsonls {
        tracing::info!("loading variants from {}", path_jsonl);
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
            let input_record: ExtractedVcvRecord = serde_json::from_str(&line).map_err(|e| {
                tracing::warn!("skipping line because of error: {}", e);
                e
            })?;

            for hgnc_id in &input_record.hgnc_ids {
                let release = input_record
                    .sequence_location
                    .as_ref()
                    .expect("missing sequence_location")
                    .assembly
                    .clone();

                // Add the release to the set, so we can later iterate over it
                releases.insert(release.clone());

                // Create a key for RocksDB
                let key = format!("{}:{}", hgnc_id, release);
                let mut variants: Vec<ExtractedVcvRecord> = vec![];

                // Retrieve existing data and deserialize it
                if let Some(existing_data) = db.get(&key)? {
                    variants = serde_json::from_slice(&existing_data)?;
                }

                // Add the new variant
                variants.push(input_record.clone());

                // Sort the variants by accession
                variants.sort_by(|a, b| {
                    a.accession
                        .as_ref()
                        .expect("no accession")
                        .accession
                        .cmp(&b.accession.as_ref().expect("no accession").accession)
                });

                // Serialize and store back in RocksDB
                let serialized_data = serde_json::to_vec(&variants)?;
                db.put(key, serialized_data)?;
            }
        }
    }

    Ok((db, releases))
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
    let options = rocksdb_utils_lookup::tune_options(
        rocksdb::Options::default(),
        args.path_wal_dir.as_ref().map(|s| s.as_ref()),
    );
    let tempdir = tempfile::TempDir::new_in(&args.path_out_rocksdb)?;
    let (vars_per_gene_db, releases) =
        load_variants_jsonl(&args.paths_variant_jsonl, &tempdir, &options)?;
    tracing::info!(
        "... done loading variants per gene in {:?}",
        &before_vars.elapsed()
    );

    tracing::info!("Writing to database ...");
    let before_write_to_db = std::time::Instant::now();
    let db_key_iter = vars_per_gene_db.iterator(rocksdb::IteratorMode::Start);
    let mut hgnc_ids = counts_per_impact
        .keys()
        .cloned()
        .chain(counts_per_freq.keys().cloned())
        .chain(db_key_iter.filter_map(|item| {
            if let Ok((key, _value)) = item {
                Some(
                    String::from_utf8(Vec::from(key))
                        .expect("Failed to convert to string")
                        .split_once(":")
                        .expect("Expected hgnc_id:release key format")
                        .0
                        .to_string(),
                )
            } else {
                None
            }
        }))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    hgnc_ids.sort();

    // Read through all records and insert each into the database.
    for hgnc_id in hgnc_ids.iter() {
        let mut per_release_vars = vec![];
        for release in &releases {
            let key = format!("{}:{}", hgnc_id, release);
            if let Some(buf) = vars_per_gene_db.get(key)? {
                let extracted_vcvs: Vec<ExtractedVcvRecord> = serde_json::from_slice(&buf)?;
                let per_release_var = ExtractedVariantsPerRelease {
                    release: Some(release.clone()),
                    variants: extracted_vcvs,
                };
                per_release_vars.push(per_release_var);
            }
        }
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
