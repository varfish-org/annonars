//! Import of ClinVar SV data.

use std::{io::BufRead, sync::Arc};

use clap::Parser;
use prost::Message as _;

use crate::common;

/// Command line arguments for `clinvar-sv import` sub command.
#[derive(Parser, Debug, Clone)]
#[command(about = "import ClinVar SV data into RocksDB", long_about = None)]
pub struct Args {
    /// Genome build to use in the build.
    #[arg(long, value_enum)]
    pub genome_release: common::cli::GenomeRelease,
    /// Path to input JSONL file(s).
    #[arg(long, required = true)]
    pub path_in_jsonl: Vec<String>,
    /// Path to output RocksDB directory.
    #[arg(long)]
    pub path_out_rocksdb: String,

    /// Minimal VCF REF/ALT length to consider as SV.
    #[arg(long, default_value_t = 50)]
    pub min_var_size: u32,
    /// Name of the column family to import into.
    #[arg(long, default_value = "clinvar_sv")]
    pub cf_name: String,
    /// Mapping from ClinVar RCV to ClinVar VCV.
    #[arg(long, default_value = "clinvar_sv_by_rcv")]
    pub cf_name_by_rcv: String,
    /// Optional path to RocksDB WAL directory.
    #[arg(long)]
    pub path_wal_dir: Option<String>,
}

/// Perform import of the JSONL file.
fn jsonl_import(
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    args: &Args,
    path_in_jsonl: &str,
) -> Result<(), anyhow::Error> {
    let cf_data = db.cf_handle(&args.cf_name).unwrap();
    let cf_by_rcv = db.cf_handle(&args.cf_name_by_rcv).unwrap();

    // Open reader, possibly decompressing gziped files.
    let reader: Box<dyn std::io::Read> = if path_in_jsonl.ends_with(".gz") {
        Box::new(flate2::read::GzDecoder::new(std::fs::File::open(
            path_in_jsonl,
        )?))
    } else {
        Box::new(std::fs::File::open(path_in_jsonl)?)
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
            start,
            stop,
            reference_allele_vcf,
            alternate_allele_vcf,
            inner_start,
            inner_stop,
            outer_start,
            outer_stop,
            ..
        } = sequence_location;
        if let (Some(reference_allele_vcf), Some(alternate_allee_vcf)) =
            (reference_allele_vcf.as_ref(), alternate_allele_vcf.as_ref())
        {
            if reference_allele_vcf.len() < args.min_var_size as usize
                && alternate_allee_vcf.len() < args.min_var_size as usize
            {
                tracing::debug!(
                    "skipping line because of short REF/ALT: {}: {}>{}",
                    &vcv,
                    reference_allele_vcf,
                    alternate_allee_vcf,
                );
                continue;
            }
        }

        if (start.is_none() || stop.is_none())
            && (inner_start.is_none() || inner_stop.is_none())
            && (outer_start.is_none() || outer_stop.is_none())
        {
            tracing::warn!("skipping line because no start/stop: {}", &vcv,);
            continue;
        };

        let key: Vec<u8> = vcv.clone().into();

        let buf = vcv_record.encode_to_vec();
        db.put_cf(&cf_data, &key, &buf)?;

        for rcv_record in &rcv_records {
            let accession = rcv_record
                .accession
                .as_ref()
                .expect("rcv.accession is required");
            let rcv = format!("{}.{}", accession.accession, accession.version);
            db.put_cf(&cf_by_rcv, rcv.as_bytes(), &key)?;
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
    let cf_names = &["meta", &args.cf_name, &args.cf_name_by_rcv];
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
    for path in &args.path_in_jsonl {
        tracing::info!("  - {}", &path);
        jsonl_import(&db, args, path)?;
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
    fn smoke_test_import_jsonl() {
        let tmp_dir = TempDir::default();
        let common = common::cli::Args {
            verbose: Verbosity::new(1, 0),
        };
        let args = Args {
            genome_release: common::cli::GenomeRelease::Grch37,
            path_in_jsonl: vec![
                String::from("tests/clinvar-sv/clinvar-variants-grch37-seqvars.jsonl"),
                String::from("tests/clinvar-sv/clinvar-variants-grch37-strucvars.jsonl"),
            ],
            path_out_rocksdb: format!("{}", tmp_dir.join("out-rocksdb").display()),
            cf_name: String::from("clinvar_sv"),
            cf_name_by_rcv: String::from("clinvar_sv_by_rcv"),
            min_var_size: 50,
            path_wal_dir: None,
        };

        run(&common, &args).unwrap();
    }
}
