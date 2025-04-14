//! Import of functional elements.

use std::{str::FromStr, sync::Arc};

use clap::Parser;
use noodles::gff;
use prost::Message;

use crate::pbs::functional::refseq::{
    Category as RefseqCategory, Record as RefseqRecord, RegulatoryClass as RefseqRegulatoryClass,
};
use crate::{
    common::{self, cli::is_canonical},
    freqs::cli::import::reading::ContigMap,
};

impl FromStr for RefseqCategory {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "misc_feature" => Self::MiscFeature,
            "misc_recomb" => Self::MiscRecomb,
            "misc_structure" => Self::MiscStructure,
            "mobile_element" => Self::MobileElement,
            "protein_bind" => Self::ProteinBind,
            "Region" => Self::Region,
            "regulatory" => Self::Regulatory,
            "repeat_region" => Self::RepeatRegion,
            "rep_origin" => Self::RepOrigin,
            _ => anyhow::bail!("unknown category: {}", s),
        })
    }
}

impl FromStr for RefseqRegulatoryClass {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "CAAT_signal" => Self::CaatSignal,
            "DNase_I_hypersensitive_site" => Self::DnaseIHypersensitiveSite,
            "enhancer" => Self::Enhancer,
            "enhancer_blocking_element" => Self::EnhancerBlockingElement,
            "epigenetically_modified_region" => Self::EpigeneticallyModifiedRegion,
            "GC_signal" => Self::GcSignal,
            "imprinting_control_region" => Self::ImprintingControlRegion,
            "insulator" => Self::Insulator,
            "locus_control_region" => Self::LocusControlRegion,
            "matrix_attachment_region" => Self::MatrixAttachmentRegion,
            "micrococcal_nuclease_hypersensitive_site" => {
                Self::MicrococcalNucleaseHypersensitiveSite
            }
            "promoter" => Self::Promoter,
            "replication_regulatory_region" => Self::ReplicationRegulatoryRegion,
            "response_element" => Self::ResponseElement,
            "silencer" => Self::Silencer,
            "TATA_box" => Self::TataBox,
            "transcriptional_cis_regulatory_region" => Self::TranscriptionalCisRegulatoryRegion,
            _ => anyhow::bail!("unknown category: {}", s),
        })
    }
}

/// Command line arguments for `functional-elements import` sub command.
#[derive(Parser, Debug, Clone)]
#[command(about = "import functional elements data into RocksDB", long_about = None)]
pub struct Args {
    /// Genome build to use in the build.
    #[arg(long, value_enum)]
    pub genome_release: common::cli::GenomeRelease,
    /// Path to input GFF file(s).
    #[arg(long, required = true)]
    pub path_in_gff: Vec<String>,
    /// Path to output RocksDB directory.
    #[arg(long)]
    pub path_out_rocksdb: String,

    /// Name of the column family to import into.
    #[arg(long, default_value = "functional")]
    pub cf_name: String,
    /// Optional path to RocksDB WAL directory.
    #[arg(long)]
    pub path_wal_dir: Option<String>,
}

/// Perform import of the GFF files.
fn gff_import(
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    args: &Args,
    path_in_gff: &str,
) -> Result<(), anyhow::Error> {
    let cf_data = db.cf_handle(&args.cf_name).unwrap();

    // Open reader, possibly decompressing gziped files.
    let reader: Box<dyn std::io::Read> = if path_in_gff.ends_with(".gz") {
        Box::new(flate2::read::GzDecoder::new(std::fs::File::open(
            path_in_gff,
        )?))
    } else {
        Box::new(std::fs::File::open(path_in_gff)?)
    };

    let mut skipped_seq = indexmap::IndexSet::new();

    // Import of RefSeq GFF data.
    let contig_map = ContigMap::new(args.genome_release.into());
    let mut reader = gff::Reader::new(std::io::BufReader::new(reader));
    for result in reader.record_bufs() {
        let record = result?;

        // Resolve reference sequence name to contig name (for canonical ones).
        let seq_name = &record.reference_sequence_name().to_string();
        let chromosome = match contig_map.chrom_name_to_seq(seq_name) {
            Ok(sequence) => {
                if is_canonical(&sequence.name) {
                    sequence.name.clone()
                } else {
                    tracing::debug!("reference not canonical: {}", seq_name);
                    continue;
                }
            }
            Err(e) => {
                if skipped_seq.insert(seq_name.to_string()) {
                    tracing::debug!("cannot map reference name: {}; skipping ({})", seq_name, e);
                }
                continue;
            }
        };

        /// Helper function to extract a key from the attributes of a record.
        fn extract(record: &gff::feature::RecordBuf, key: &str) -> Result<String, anyhow::Error> {
            let key_ = key.as_bytes();
            let value = record
                .attributes()
                .get(key_)
                .ok_or_else(|| anyhow::anyhow!("problem with {} attribute: {:?}", key, record))?;
            match value {
                gff::feature::record_buf::attributes::field::Value::String(s) => Ok(s.to_string()),
                gff::feature::record_buf::attributes::field::Value::Array(arr) => {
                    if arr.is_empty() {
                        Err(anyhow::anyhow!(
                            "problem with {} attribute: {:?}",
                            key,
                            record
                        ))
                    } else {
                        if arr.len() > 1 {
                            tracing::warn!("multiple values for {} attribute: {:?}", key, record);
                        }
                        Ok(arr[0].clone().to_string())
                    }
                }
            }
        }

        tracing::debug!("record = {:?}", &record);

        let record = RefseqRecord {
            chromosome,
            start: Into::<usize>::into(record.start()) as i32,
            stop: Into::<usize>::into(record.end()) as i32,
            id: extract(&record, "ID")
                .map_err(|e| anyhow::anyhow!("problem with ID attribute: {:?} ({})", record, e))?,
            dbxref: extract(&record, "Dbxref").map_err(|e| {
                anyhow::anyhow!("problem with Dbxref attribute: {:?} ({})", record, e)
            })?,
            category: extract(&record, "gbkey")?
                .parse::<RefseqCategory>()
                .map(|c| c as i32)
                .map_err(|e| {
                    anyhow::anyhow!("problem with gbkey attribute: {:?} ({})", record, e)
                })?,
            regulatory_class: extract(&record, "regulatory_class")
                .ok()
                .map(|s| s.parse::<RefseqRegulatoryClass>().map(|c| c as i32))
                .transpose()?,
            note: extract(&record, "note").ok(),
            experiment: extract(&record, "experiment").ok(),
            function: extract(&record, "function").ok(),
        };

        let buf = record.encode_to_vec();
        db.put_cf(&cf_data, record.id.as_bytes(), buf)?;
    }

    Ok(())
}

/// Implementation of `clinvar-minimal import` sub command.
pub fn run(common: &common::cli::Args, args: &Args) -> Result<(), anyhow::Error> {
    tracing::info!("Starting 'functional import' command");
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
    db.put_cf(
        &cf_meta,
        "genome-release",
        format!("{}", args.genome_release),
    )?;
    db.put_cf(&cf_meta, "db-name", "functional")?;
    tracing::info!(
        "... done opening RocksDB for writing in {:?}",
        before_opening_rocksdb.elapsed()
    );

    tracing::info!("Importing GFF files ...");
    let before_import = std::time::Instant::now();
    for path in &args.path_in_gff {
        tracing::info!("  - {}", &path);
        gff_import(&db, args, path)?;
    }
    tracing::info!(
        "... done importing JSONL file in {:?}",
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
    fn smoke_test_import_gff_37() {
        let tmp_dir = TempDir::default();
        let common = common::cli::Args {
            verbose: Verbosity::new(1, 0),
        };
        let args = Args {
            genome_release: common::cli::GenomeRelease::Grch37,
            path_in_gff: vec![String::from(
                "tests/functional/GCF_000001405.25_GRCh37.p13_genomic.functional.gff",
            )],
            path_out_rocksdb: format!("{}", tmp_dir.join("out-rocksdb").display()),
            cf_name: String::from("functional"),
            path_wal_dir: None,
        };

        run(&common, &args).unwrap();
    }

    #[test]
    fn smoke_test_import_gff_38() {
        let tmp_dir = TempDir::default();
        let common = common::cli::Args {
            verbose: Verbosity::new(1, 0),
        };
        let args = Args {
            genome_release: common::cli::GenomeRelease::Grch38,
            path_in_gff: vec![String::from(
                "tests/functional/GCF_000001405.40_GRCh38.p14_genomic.functional.gff",
            )],
            path_out_rocksdb: format!("{}", tmp_dir.join("out-rocksdb").display()),
            cf_name: String::from("functional"),
            path_wal_dir: None,
        };

        run(&common, &args).unwrap();
    }
}
