//! Import gnomAD-mtDNA annotation data.

use std::sync::Arc;

use clap::Parser;
use indicatif::ParallelProgressIterator;
use prost::Message;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    common::{self, cli::indicatif_style},
    gnomad_pbs,
};

/// Command line arguments for `gnomad_mtdna import` sub command.
#[derive(Parser, Debug, Clone)]
#[command(about = "import gnomAD-mtDNA data into RocksDB", long_about = None)]
pub struct Args {
    /// Genome build to use in the build.
    #[arg(long, value_enum)]
    pub genome_release: common::cli::GenomeRelease,
    /// Path to input VCF file(s).
    #[arg(long, required = true)]
    pub path_in_vcf: String,
    /// Path to output RocksDB directory.
    #[arg(long)]
    pub path_out_rocksdb: String,

    /// Windows size for TBI-based parallel import.
    #[arg(long, default_value = "1000000")]
    pub tbi_window_size: usize,

    /// Name of the column family to import into.
    #[arg(long, default_value = "gnomad_mtdna_data")]
    pub cf_name: String,
    /// Optional path to RocksDB WAL directory.
    #[arg(long)]
    pub path_wal_dir: Option<String>,
    /// JSON formatted configuration of which fields to import from gnomAD-mtDNA.  If not
    /// specified, the default fields are configured.
    #[arg(long)]
    pub import_fields_json: Option<String>,
}

/// Perform TBI-parallel import of the data.
fn tsv_import(
    db: Arc<rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>>,
    args: &Args,
) -> Result<(), anyhow::Error> {
    // Load tabix header and create BGZF reader with tabix index.
    let tabix_src = format!("{}.tbi", args.path_in_vcf);
    let index = noodles_tabix::read(tabix_src)?;
    let header = index.header().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "missing tabix header")
    })?;
    // Build list of canonical chromosome names from header.
    let canonical_header_chroms = header
        .reference_sequence_names()
        .iter()
        .filter_map(|chrom| {
            let canon_chrom = chrom.strip_prefix("chr").unwrap_or(chrom);
            if common::cli::is_canonical(canon_chrom) {
                Some((common::cli::canonicalize(canon_chrom), chrom.clone()))
            } else {
                None
            }
        })
        .collect::<std::collections::HashMap<String, String>>();

    // Generate list of regions on canonical chromosomes, limited to those present in header.
    let windows =
        common::cli::build_genome_windows(args.genome_release.into(), Some(args.tbi_window_size))?
            .into_iter()
            .filter_map(|(window_chrom, begin, end)| {
                let canon_chrom = common::cli::canonicalize(&window_chrom);
                canonical_header_chroms
                    .get(&canon_chrom)
                    .map(|header_chrom| (header_chrom.clone(), begin, end))
            })
            .collect::<Vec<_>>();

    tracing::info!("Loading gnomad_mtdna VCF file into RocksDB...");
    let before_loading = std::time::Instant::now();
    let style = indicatif_style();
    windows
        .par_iter()
        .progress_with_style(style)
        .for_each(|(chrom, begin, end)| {
            process_window(db.clone(), chrom, *begin, *end, args).unwrap_or_else(|e| {
                panic!(
                    "failed to process window {}:{}-{}: {}",
                    chrom, begin, end, e
                )
            });
        });
    tracing::info!(
        "... done loading gnomad_mtdna VCF file into RocksDB in {:?}",
        before_loading.elapsed()
    );

    Ok(())
}

/// Process one window.
fn process_window(
    db: Arc<rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>>,
    chrom: &str,
    begin: usize,
    end: usize,
    args: &Args,
) -> Result<(), anyhow::Error> {
    let cf_gnomad = db.cf_handle(&args.cf_name).unwrap();
    let mut reader =
        noodles_vcf::indexed_reader::Builder::default().build_from_path(&args.path_in_vcf)?;
    let header = reader.read_header()?;

    let raw_region = format!("{}:{}-{}", chrom, begin + 1, end);
    tracing::debug!("  processing region: {}", raw_region);
    let region = raw_region.parse()?;

    // Jump to the selected region.  In the case of errors, allow for the window not
    // to exist in the reference sequence (just return).  Otherwise, fail on
    // errors.
    let query = match reader.query(&header, &region) {
        Ok(result) => Ok(Some(result)),
        Err(e) => {
            let needle = "region reference sequence does not exist in reference sequences";
            if e.to_string().contains(needle) {
                Ok(None)
            } else {
                Err(e)
            }
        }
    }?;

    // Process the result (skip if determined above that the sequence does not
    // exist).
    if let Some(query) = query {
        for result in query {
            let vcf_record = result?;

            // Process each alternate allele into one record.
            let details_options = serde_json::from_str(
                args.import_fields_json
                    .as_ref()
                    .expect("has been set earlier"),
            )?;
            for allele_no in 0..vcf_record.alternate_bases().len() {
                let key_buf: Vec<u8> =
                    common::keys::Var::from_vcf_allele(&vcf_record, allele_no).into();
                let record = gnomad_pbs::mtdna::Record::from_vcf_allele(
                    &vcf_record,
                    allele_no,
                    &details_options,
                )?;
                tracing::trace!("  record: {:?}", &record);
                let record_buf = record.encode_to_vec();
                db.put_cf(&cf_gnomad, &key_buf, &record_buf)?;
            }
        }
    }

    Ok(())
}

/// Implementation of `gnomad_mtdna import` sub command.
pub fn run(common: &common::cli::Args, args: &Args) -> Result<(), anyhow::Error> {
    // Put defaults for fields to serialize into args.
    let args = Args {
        import_fields_json: args
            .import_fields_json
            .clone()
            .map(|v| {
                serde_json::to_string(&serde_json::from_str::<gnomad_pbs::mtdna::DetailsOptions>(
                    &v,
                )?)
            })
            .or_else(|| {
                Some(serde_json::to_string(
                    &gnomad_pbs::mtdna::DetailsOptions::default(),
                ))
            })
            .transpose()?,
        ..args.clone()
    };

    tracing::info!("Starting 'gnomad-mtdna import' command");
    tracing::info!("common = {:#?}", &common);
    tracing::info!("args = {:#?}", &args);

    // Open the RocksDB for writing.
    tracing::info!("Opening RocksDB for writing ...");
    let before_opening_rocksdb = std::time::Instant::now();
    let options = common::rocks_utils::tune_options(
        rocksdb::Options::default(),
        args.path_wal_dir.as_ref().map(|s| s.as_ref()),
    );
    let cf_names = &["meta", &args.cf_name];
    let db = Arc::new(rocksdb::DB::open_cf_with_opts(
        &options,
        &args.path_out_rocksdb,
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
    tracing::info!(
        "... done opening RocksDB for writing in {:?}",
        before_opening_rocksdb.elapsed()
    );

    tsv_import(db.clone(), &args)?;

    tracing::info!("Running RocksDB compaction ...");
    let before_compaction = std::time::Instant::now();
    common::rocks_utils::force_compaction_cf(&db, cf_names, Some("  "))?;
    tracing::info!(
        "... done compacting RocksDB in {:?}",
        before_compaction.elapsed()
    );

    tracing::info!("All done. Have a nice day!");
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::gnomad_pbs::mtdna::DetailsOptions;

    use super::*;

    use clap_verbosity_flag::Verbosity;
    use temp_testdir::TempDir;

    #[test]
    fn smoke_test_import_gnomad_mtdna() -> Result<(), anyhow::Error> {
        let tmp_dir = TempDir::default();
        let common = common::cli::Args {
            verbose: Verbosity::new(1, 0),
        };
        let args = Args {
            genome_release: common::cli::GenomeRelease::Grch37,
            path_in_vcf: String::from("tests/gnomad-mtdna/example/gnomad-mtdna.vcf.bgz"),
            path_out_rocksdb: format!("{}", tmp_dir.join("out-rocksdb").display()),
            cf_name: String::from("gnomad_mtdna_data"),
            path_wal_dir: None,
            tbi_window_size: 1_000_000,
            import_fields_json: Some(serde_json::to_string(&DetailsOptions::with_all_enabled())?),
        };

        run(&common, &args)
    }
}
