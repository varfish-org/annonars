//! Import gnomAD-exomes and genomes annotation data.

use std::{str::FromStr, sync::Arc};

use clap::Parser;
use indicatif::ParallelProgressIterator;
use noodles_vcf::header::record;
use prost::Message;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    common::{self, cli::indicatif_style},
    gnomad_pbs::{self, common::SexCoding, nuclear::DetailsOptions},
};

/// Select the type of gnomAD data to import.
#[derive(strum::Display, clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GnomadKind {
    /// gnomAD exomes
    #[strum(serialize = "exomes")]
    Exomes,
    /// gnomAD genomes
    #[strum(serialize = "genomes")]
    Genomes,
}

/// Command line arguments for `gnomad_nuclear import` sub command.
#[derive(Parser, Debug, Clone)]
#[command(about = "import gnomAD-mtDNA data into RocksDB", long_about = None)]
pub struct Args {
    /// Path to input VCF file(s).
    #[arg(long, required = true)]
    pub path_in_vcf: Vec<String>,
    /// Path to output RocksDB directory.
    #[arg(long)]
    pub path_out_rocksdb: String,

    /// Exomes or genomes.
    #[arg(long)]
    pub gnomad_kind: GnomadKind,
    /// The data version to write out.
    #[arg(long, default_value = "3.1")]
    pub gnomad_version: String,
    /// Genome build to use in the build.
    #[arg(long, value_enum)]
    pub genome_release: common::cli::GenomeRelease,

    /// Windows size for TBI-based parallel import.
    #[arg(long, default_value = "1000000")]
    pub tbi_window_size: usize,

    /// Name of the column family to import into.
    #[arg(long, default_value = "gnomad_nuclear_data")]
    pub cf_name: String,
    /// Optional path to RocksDB WAL directory.
    #[arg(long)]
    pub path_wal_dir: Option<String>,
    /// JSON formatted configuration of which fields to import from gnomAD-mtDNA.  If not
    /// specified, the default fields are configured.
    #[arg(long)]
    pub import_fields_json: Option<String>,
}

/// Perform TBI-parallel import of one file.
fn tsv_import(
    db: Arc<rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>>,
    args: &Args,
    path_in_vcf: &str,
) -> Result<(), anyhow::Error> {
    // Load tabix header and create BGZF reader with tabix index.
    let tabix_src = format!("{}.tbi", path_in_vcf);
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

    let style = indicatif_style();
    windows
        .par_iter()
        .progress_with_style(style)
        .for_each(|(chrom, begin, end)| {
            process_window(db.clone(), chrom, *begin, *end, args, path_in_vcf).unwrap_or_else(
                |e| {
                    panic!(
                        "failed to process window {}:{}-{} of {}: {}",
                        chrom, begin, end, path_in_vcf, e
                    )
                },
            );
        });

    Ok(())
}

/// Process one window.
fn process_window(
    db: Arc<rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>>,
    chrom: &str,
    begin: usize,
    end: usize,
    args: &Args,
    path_in_vcf: &str,
) -> Result<(), anyhow::Error> {
    let cf_gnomad = db.cf_handle(&args.cf_name).unwrap();
    let mut reader =
        noodles_vcf::indexed_reader::Builder::default().build_from_path(&path_in_vcf)?;
    let header = reader.read_header()?;
    // Determine coding of sex/XY karyotype from header keys.
    let sex_coding = if header
        .infos()
        .keys()
        .any(|x| x.to_string().contains("_XX") || x.to_string().contains("_XY"))
    {
        SexCoding::XxXy
    } else {
        SexCoding::FemaleMale
    };

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
                let record = gnomad_pbs::nuclear::Record::from_vcf_allele(
                    &vcf_record,
                    allele_no,
                    &details_options,
                    sex_coding,
                )?;
                tracing::trace!("  record: {:?}", &record);
                let record_buf = record.encode_to_vec();
                db.put_cf(&cf_gnomad, &key_buf, &record_buf)?;
            }
        }
    }

    Ok(())
}

/// Implementation of `gnomad_nuclear import` sub command.
pub fn run(common: &common::cli::Args, args: &Args) -> Result<(), anyhow::Error> {
    // Put defaults for fields to serialize into args.
    let args = Args {
        import_fields_json: args
            .import_fields_json
            .clone()
            .map(|v| serde_json::to_string(&serde_json::from_str::<DetailsOptions>(&v)?))
            .or_else(|| Some(serde_json::to_string(&DetailsOptions::default())))
            .transpose()?,
        ..args.clone()
    };

    tracing::info!("Starting 'gnomad-nuclear import' command");
    tracing::info!("common = {:#?}", &common);
    tracing::info!("args = {:#?}", &args);

    tracing::info!("Opening gnomAD-nuclear VCF file...");
    let before_loading = std::time::Instant::now();
    let mut reader_vcf =
        noodles_vcf::reader::Builder::default().build_from_path(&args.path_in_vcf[0])?;
    let header = reader_vcf.read_header()?;

    let vep_version = if let Some(record::value::Collection::Unstructured(values)) = header
        .other_records()
        .get(&record::key::Other::from_str("VEP version")?)
    {
        Some(values.first().expect("no VEP version value").to_owned())
    } else {
        None
    };
    let dbsnp_version = if let Some(record::value::Collection::Unstructured(values)) = header
        .other_records()
        .get(&record::key::Other::from_str("dbSNP version")?)
    {
        Some(values.first().expect("no reference value").to_owned())
    } else {
        None
    };
    let age_distributions = if let Some(record::value::Collection::Unstructured(values)) = header
        .other_records()
        .get(&record::key::Other::from_str("age distributions")?)
    {
        Some(values.first().expect("no reference value").to_owned())
    } else {
        None
    };
    tracing::info!(
        "...done opening gnomAD-nuclear VCF file in {:?}",
        before_loading.elapsed()
    );

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
    db.put_cf(
        &cf_meta,
        format!("gnomad-{}-version", args.gnomad_kind),
        &args.gnomad_version,
    )?;
    if let Some(vep_version) = vep_version {
        db.put_cf(
            &cf_meta,
            format!("gnomad-{}-vep-version", args.gnomad_kind),
            &vep_version,
        )?;
    }
    if let Some(dbsnp_version) = dbsnp_version {
        db.put_cf(
            &cf_meta,
            format!("gnomad-{}-dbsnp-version", args.gnomad_kind),
            &dbsnp_version,
        )?;
    }
    if let Some(age_distributions) = age_distributions {
        db.put_cf(
            &cf_meta,
            &format!("gnomad-{}-age-distributions", args.gnomad_kind),
            &age_distributions,
        )?;
    }
    tracing::info!(
        "... done opening RocksDB for writing in {:?}",
        before_opening_rocksdb.elapsed()
    );

    tracing::info!("Loading gnomad_nuclear VCF file into RocksDB...");
    let before_loading = std::time::Instant::now();
    for path_in_tsv in &args.path_in_vcf {
        tsv_import(db.clone(), &args, path_in_tsv)?;
    }
    tracing::info!(
        "... done loading gnomad_nuclear VCF file into RocksDB in {:?}",
        before_loading.elapsed()
    );

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
    use crate::gnomad_pbs::nuclear::DetailsOptions;

    use super::*;

    use clap_verbosity_flag::Verbosity;
    use temp_testdir::TempDir;

    #[test]
    fn smoke_test_import_gnomad_exomes_grch37() -> Result<(), anyhow::Error> {
        let tmp_dir = TempDir::default();
        let common = common::cli::Args {
            verbose: Verbosity::new(1, 0),
        };
        let args = Args {
            genome_release: common::cli::GenomeRelease::Grch37,
            path_in_vcf: vec![String::from(
                "tests/gnomad-nuclear/example-exomes-grch37/gnomad-exomes.vcf.bgz",
            )],
            path_out_rocksdb: format!("{}", tmp_dir.join("out-rocksdb").display()),
            cf_name: String::from("gnomad_nuclear_data"),
            path_wal_dir: None,
            tbi_window_size: 1_000_000,
            import_fields_json: Some(serde_json::to_string(&DetailsOptions::with_all_enabled())?),
            gnomad_kind: GnomadKind::Exomes,
            gnomad_version: String::from("2.1"),
        };

        run(&common, &args)
    }

    #[test]
    fn smoke_test_import_gnomad_genomes_grch37() -> Result<(), anyhow::Error> {
        let tmp_dir = TempDir::default();
        let common = common::cli::Args {
            verbose: Verbosity::new(1, 0),
        };
        let args = Args {
            genome_release: common::cli::GenomeRelease::Grch37,
            path_in_vcf: vec![String::from(
                "tests/gnomad-nuclear/example-genomes-grch37/gnomad-genomes.vcf.bgz",
            )],
            path_out_rocksdb: format!("{}", tmp_dir.join("out-rocksdb").display()),
            cf_name: String::from("gnomad_nuclear_data"),
            path_wal_dir: None,
            tbi_window_size: 1_000_000,
            import_fields_json: Some(serde_json::to_string(&DetailsOptions::with_all_enabled())?),
            gnomad_kind: GnomadKind::Genomes,
            gnomad_version: String::from("2.1"),
        };

        run(&common, &args)
    }

    #[test]
    fn smoke_test_import_gnomad_exomes_grch38() -> Result<(), anyhow::Error> {
        let tmp_dir = TempDir::default();
        let common = common::cli::Args {
            verbose: Verbosity::new(1, 0),
        };
        let args = Args {
            genome_release: common::cli::GenomeRelease::Grch38,
            path_in_vcf: vec![String::from(
                "tests/gnomad-nuclear/example-exomes-grch38/gnomad-exomes.vcf.bgz",
            )],
            path_out_rocksdb: format!("{}", tmp_dir.join("out-rocksdb").display()),
            cf_name: String::from("gnomad_nuclear_data"),
            path_wal_dir: None,
            tbi_window_size: 1_000_000,
            import_fields_json: Some(serde_json::to_string(&DetailsOptions::with_all_enabled())?),
            gnomad_kind: GnomadKind::Exomes,
            gnomad_version: String::from("2.1"),
        };

        run(&common, &args)
    }

    #[test]
    fn smoke_test_import_gnomad_genomes_grch38() -> Result<(), anyhow::Error> {
        let tmp_dir = TempDir::default();
        let common = common::cli::Args {
            verbose: Verbosity::new(1, 0),
        };
        let args = Args {
            genome_release: common::cli::GenomeRelease::Grch38,
            path_in_vcf: vec![String::from(
                "tests/gnomad-nuclear/example-genomes-grch38/gnomad-genomes.vcf.bgz",
            )],
            path_out_rocksdb: format!("{}", tmp_dir.join("out-rocksdb").display()),
            cf_name: String::from("gnomad_nuclear_data"),
            path_wal_dir: None,
            tbi_window_size: 1_000_000,
            import_fields_json: Some(serde_json::to_string(&DetailsOptions::with_all_enabled())?),
            gnomad_kind: GnomadKind::Genomes,
            gnomad_version: String::from("3.1"),
        };

        run(&common, &args)
    }
}
