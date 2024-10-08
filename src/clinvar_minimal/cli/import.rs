//! Import of minimal ClinVar data.

use std::{io::BufRead, sync::Arc};

use clap::Parser;
use prost::Message;

use crate::common::{self, keys};

/// Command line arguments for `clinvar-minimal import` sub command.
#[derive(Parser, Debug, Clone)]
#[command(about = "import extracted seqvars ClinVar data into RocksDB", long_about = None)]
pub struct Args {
    /// Genome build to use in the build.
    #[arg(long, value_enum)]
    pub genome_release: common::cli::GenomeRelease,
    /// Path to input JSONL file(s).
    #[arg(long, required = true)]
    pub path_in_jsonl: String,
    /// Path to output RocksDB directory.
    #[arg(long)]
    pub path_out_rocksdb: String,

    /// Name of the column family to import into.
    #[arg(long, default_value = "clinvar")]
    pub cf_name: String,
    /// Name of the column family for accession lookup.
    #[arg(long, default_value = "clinvar_by_accession")]
    pub cf_name_by_accession: String,
    /// Optional path to RocksDB WAL directory.
    #[arg(long)]
    pub path_wal_dir: Option<String>,
}

/// Perform import of the JSONL file.
fn jsonl_import(
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    args: &Args,
) -> Result<(), anyhow::Error> {
    let cf_data = db.cf_handle(&args.cf_name).unwrap();
    let cf_by_accession = db.cf_handle(&args.cf_name_by_accession).unwrap();

    // Open reader, possibly decompressing gziped files.
    let reader: Box<dyn std::io::Read> = if args.path_in_jsonl.ends_with(".gz") {
        Box::new(flate2::read::GzDecoder::new(std::fs::File::open(
            &args.path_in_jsonl,
        )?))
    } else {
        Box::new(std::fs::File::open(&args.path_in_jsonl)?)
    };

    let reader = std::io::BufReader::new(reader);

    for line in reader.lines() {
        let line = line?;
        let vcv_record = match serde_json::from_str::<
            crate::pbs::clinvar_data::extracted_vars::ExtractedVcvRecord,
        >(&line)
        {
            Ok(record) => record,
            Err(e) => {
                tracing::warn!("skipping line because of error: {}", e);
                continue;
            }
        };

        let crate::pbs::clinvar_data::extracted_vars::ExtractedVcvRecord {
            accession,
            rcvs: rcv_records,
            sequence_location,
            ..
        } = vcv_record.clone();
        let accession = accession.expect("accession is required");
        let vcv = format!("{}.{}", accession.accession, accession.version);
        let sequence_location = sequence_location.expect("sequence_location is required");
        let crate::pbs::clinvar_data::clinvar_public::location::SequenceLocation {
            chr,
            position_vcf,
            reference_allele_vcf,
            alternate_allele_vcf,
            ..
        } = sequence_location;
        let chr_pb =
            crate::pbs::clinvar_data::clinvar_public::Chromosome::try_from(chr).map_err(|e| {
                anyhow::anyhow!("problem converting chromosome {} to Chromosome: {}", chr, e)
            })?;
        if let (Some(position_vcf), Some(reference_allele_vcf), Some(alternate_allele_vcf)) =
            (position_vcf, reference_allele_vcf, alternate_allele_vcf)
        {
            let var = keys::Var::from(
                &chr_pb.as_chr_name(),
                position_vcf as i32,
                &reference_allele_vcf,
                &alternate_allele_vcf,
            );
            let key: Vec<u8> = var.into();

            let data = db
                .get_cf(&cf_data, key.clone())
                .map_err(|e| anyhow::anyhow!("problem querying database: {}", e));
            match data {
                Err(e) => {
                    tracing::warn!("skipping line because of error: {}", e);
                    continue;
                }
                Ok(data) => {
                    db.put_cf(&cf_by_accession, vcv.as_bytes(), &key)?;
                    for rcv_record in &rcv_records {
                        let accession = rcv_record
                            .accession
                            .as_ref()
                            .expect("rcv.accession is required");
                        let rcv = format!("{}.{}", accession.accession, accession.version);
                        db.put_cf(&cf_by_accession, rcv.as_bytes(), &key)?;
                    }

                    let new_record = if let Some(data) = data {
                        let mut record =
                            crate::pbs::clinvar::minimal::ExtractedVcvRecordList::decode(
                                &data[..],
                            )?;
                        record.records.push(vcv_record);
                        record.records.sort_by_key(|a| {
                            a.accession
                                .as_ref()
                                .expect("accession is required")
                                .accession
                                .clone()
                        });
                        record
                    } else {
                        crate::pbs::clinvar::minimal::ExtractedVcvRecordList {
                            records: vec![vcv_record],
                        }
                    };
                    let buf = new_record.encode_to_vec();
                    db.put_cf(&cf_data, &key, &buf)?;
                }
            }
        }
    }

    Ok(())
}

/// Implementation of `clinvar-minimal import` sub command.
pub fn run(common: &common::cli::Args, args: &Args) -> Result<(), anyhow::Error> {
    tracing::info!("Starting 'clinvar-minimal import' command");
    tracing::info!("common = {:#?}", &common);
    tracing::info!("args = {:#?}", &args);

    // Open the RocksDB for writing.
    tracing::info!("Opening RocksDB for writing ...");
    let before_opening_rocksdb = std::time::Instant::now();
    let options = rocksdb_utils_lookup::tune_options(
        rocksdb::Options::default(),
        args.path_wal_dir.as_ref().map(|s| s.as_ref()),
    );
    let cf_names = &["meta", &args.cf_name, &args.cf_name_by_accession];
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
    db.put_cf(&cf_meta, "db-name", "clinvar-minimal")?;
    tracing::info!(
        "... done opening RocksDB for writing in {:?}",
        before_opening_rocksdb.elapsed()
    );

    tracing::info!("Importing JSONL file ...");
    let before_import = std::time::Instant::now();
    jsonl_import(&db, args)?;
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

    #[tracing_test::traced_test]
    #[rstest::rstest]
    #[case("tests/clinvar-minimal/clinvar-seqvars-grch37-tgds.jsonl")]
    #[case("tests/clinvar-minimal/clinvar-seqvars-grch37-flagged.jsonl")]
    #[case("tests/clinvar-minimal/clinvar-seqvars-grch37-no-unflagged.jsonl")]
    fn smoke_test_import_jsonl(#[case] path_in_jsonl: &str) {
        let tmp_dir = TempDir::default();
        let common = common::cli::Args {
            verbose: Verbosity::new(1, 0),
        };
        let args = Args {
            genome_release: common::cli::GenomeRelease::Grch37,
            path_in_jsonl: path_in_jsonl.into(),
            path_out_rocksdb: format!("{}", tmp_dir.join("out-rocksdb").display()),
            cf_name: String::from("clinvar"),
            cf_name_by_accession: String::from("clinvar_by_accession"),
            path_wal_dir: None,
        };

        run(&common, &args).unwrap();
    }
}
