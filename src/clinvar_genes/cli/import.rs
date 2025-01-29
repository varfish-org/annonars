//! Import of minimal ClinVar data.

use crate::common;
use crate::pbs::clinvar::per_gene::{ClinvarPerGeneRecord, ExtractedVariantsPerRelease};
use crate::pbs::clinvar_data::class_by_freq::GeneCoarseClinsigFrequencyCounts;
use crate::pbs::clinvar_data::extracted_vars::ExtractedVcvRecord;
use crate::pbs::clinvar_data::gene_impact::GeneImpactCounts;
use clap::Parser;
use extsort::ExternalSorter;
use itertools::Itertools;
use prost::Message;
use serde::{Deserialize, Serialize};
use std::io::{BufReader, Read, Write};
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

#[derive(Debug, Serialize, Deserialize)]
struct SortableVcvRecord {
    hgnc_id: String,
    record: ExtractedVcvRecord,
}

impl extsort::Sortable for SortableVcvRecord {
    fn encode<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        rmp_serde::encode::write_named(writer, self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        writer.flush()
    }

    fn decode<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        rmp_serde::decode::from_read(reader)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

struct ClinvarVariants {
    paths: Vec<String>,
}

impl ClinvarVariants {
    fn from_paths(variant_jsonls: &[String]) -> Self {
        Self {
            paths: variant_jsonls.to_vec(),
        }
    }

    fn _iter(&self) -> impl Iterator<Item = SortableVcvRecord> + use<'_> {
        self.paths
            .iter()
            .map(|path| {
                let reader: Box<dyn Read> = if path.ends_with(".gz") {
                    Box::new(flate2::read::GzDecoder::new(
                        std::fs::File::open(path).expect(&format!("failed to open file: {}", path)),
                    ))
                } else {
                    Box::new(
                        std::fs::File::open(path).expect(&format!("failed to open file: {}", path)),
                    )
                };
                reader
            })
            .map(BufReader::new)
            .flat_map(|reader| reader.lines())
            .map(Result::unwrap)
            .map(|line| serde_json::from_str::<ExtractedVcvRecord>(&line))
            .map(Result::unwrap)
            .flat_map(|record| {
                let hgnc_ids = record.hgnc_ids.clone();
                hgnc_ids.into_iter().map(move |hgnc_id| SortableVcvRecord {
                    hgnc_id,
                    record: record.clone(),
                })
            })
    }

    fn sorted_records(&self) -> Result<impl Iterator<Item = SortableVcvRecord>, anyhow::Error> {
        // We sort all records by hgnc_id and assembly (external memory sort),
        // such that we can group them by hgnc_id and release later.
        let sorter = ExternalSorter::new().with_segment_size(100_000);
        let sorted_records = sorter
            .sort_by(self._iter(), |a, b| {
                a.hgnc_id.cmp(&b.hgnc_id).then_with(|| {
                    a.record
                        .sequence_location
                        .as_ref()
                        .expect("Missing assembly in sequence_location")
                        .assembly
                        .cmp(
                            &b.record
                                .sequence_location
                                .as_ref()
                                .expect("Missing assembly in sequence_location")
                                .assembly,
                        )
                })
            })?
            .map(Result::unwrap);
        Ok(sorted_records)
    }

    fn hgnc_ids(&self) -> HashSet<String> {
        self._iter().map(|record| record.hgnc_id.clone()).collect()
    }
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
    let vars_per_gene = ClinvarVariants::from_paths(&args.paths_variant_jsonl);
    let vars_per_gene_hgnc_ids = vars_per_gene.hgnc_ids();
    tracing::info!("... done loading hgnc ids in {:?}", &before_vars.elapsed());

    let before_vars = std::time::Instant::now();
    let vars_per_gene_records = vars_per_gene.sorted_records()?;
    tracing::info!(
        "... done preparing reading variants per gene in {:?}",
        &before_vars.elapsed()
    );

    // tracing::info!(
    //     "... done loading variants per gene in {:?}",
    //     &before_vars.elapsed()
    // );

    tracing::info!("Writing to database ...");
    let before_write_to_db = std::time::Instant::now();
    let mut hgnc_ids = counts_per_impact
        .keys()
        .cloned()
        .chain(counts_per_freq.keys().cloned())
        .chain(vars_per_gene_hgnc_ids.iter().cloned())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    hgnc_ids.sort();

    // We need to check if there are any genes in the impact and frequency data that are not in the variants data,
    // such that we can skip advancing the iterator for the variants data in those cases.

    let hgnc_ids_not_in_vars_per_gene: HashSet<String> = hgnc_ids
        .iter()
        .filter(|hgnc_id| !vars_per_gene_hgnc_ids.contains(*hgnc_id))
        .cloned()
        .collect();
    dbg!(
        hgnc_ids.len(),
        vars_per_gene_hgnc_ids.len(),
        hgnc_ids_not_in_vars_per_gene.len()
    );

    let vars_per_gene_records_by_hgnc_id = vars_per_gene_records.chunk_by(|r| r.hgnc_id.clone());
    let mut vars_per_gene_records_by_hgnc_id = vars_per_gene_records_by_hgnc_id.into_iter();

    // Read through all records and insert each into the database.
    for (i, hgnc_id) in hgnc_ids.iter().enumerate() {
        eprint!(
            "Processing gene {:12}\t{:6}/{:6}\r",
            hgnc_id,
            i + 1,
            hgnc_ids.len()
        );
        let per_release_vars = if hgnc_ids_not_in_vars_per_gene.contains(hgnc_id) {
            tracing::warn!("No variants found for gene {}", hgnc_id);
            vec![]
        } else {
            if let Some((group_hgnc_id, records)) = vars_per_gene_records_by_hgnc_id.next() {
                if *hgnc_id != group_hgnc_id {
                    tracing::warn!(
                        "Iterators not in-lock step ({} vs {})",
                        hgnc_id,
                        &group_hgnc_id
                    );
                    vec![]
                } else {
                    let mut records = records.collect::<Vec<_>>();
                    let key = |r: &SortableVcvRecord| -> String {
                        r.record
                            .sequence_location
                            .as_ref()
                            .expect("Missing sequence location")
                            .assembly
                            .clone()
                    };
                    let sort_fn =
                        |a: &SortableVcvRecord, b: &SortableVcvRecord| key(a).cmp(&key(b));
                    records.sort_by(sort_fn);
                    let by_assembly = records.into_iter().chunk_by(|a| key(a));
                    by_assembly
                        .into_iter()
                        .map(
                            |(release, records_by_release)| ExtractedVariantsPerRelease {
                                release: Some(release),
                                variants: records_by_release.map(move |r| r.record).collect(),
                            },
                        )
                        .collect()
                }
            } else {
                panic!("No more records in vars_per_gene_records_by_hgnc_id");
            }
        };

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
