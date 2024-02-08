//! Import gnomAD-exomes and genomes annotation data.

use std::{str::FromStr, sync::Arc};

use clap::Parser;
use indicatif::ParallelProgressIterator;
use noodles_csi::BinningIndex as _;
use noodles_vcf::header::record;
use prost::Message;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    common::{self},
    pbs::gnomad::{gnomad2, gnomad3},
};

/// Select the type of gnomAD data to import.
#[derive(strum::Display, clap::ValueEnum, Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum GnomadKind {
    /// gnomAD exomes / CNV (or: ExAC CNVs)
    #[strum(serialize = "exomes")]
    #[default]
    Exomes,
    /// gnomAD genomes / SVs
    #[strum(serialize = "genomes")]
    Genomes,
}

impl From<GnomadKind> for crate::pbs::gnomad::gnomad4::RecordType {
    fn from(val: GnomadKind) -> Self {
        match val {
            GnomadKind::Exomes => crate::pbs::gnomad::gnomad4::RecordType::Exomes,
            GnomadKind::Genomes => crate::pbs::gnomad::gnomad4::RecordType::Genomes,
        }
    }
}

/// Select the genomAD version (v2/v3; important for the field names).
#[derive(strum::Display, clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GnomadVersion {
    /// Version 1.x (ExAC)
    One,
    /// Version 2.x
    Two,
    /// Version 3.x
    Three,
    /// Version 4.x
    Four,
}

impl FromStr for GnomadVersion {
    type Err = anyhow::Error;

    /// Parse out the gnomAD version from the leading number.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if s.starts_with("1.") {
            GnomadVersion::One
        } else if s.starts_with("2.") {
            GnomadVersion::Two
        } else if s.starts_with("3.") {
            GnomadVersion::Three
        } else if s.starts_with("4.") {
            GnomadVersion::Four
        } else {
            anyhow::bail!("invalid GnomadVersion: {}", s)
        })
    }
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
    #[arg(long)]
    pub gnomad_version: String,
    /// Genome build to use in the build.
    #[arg(long, value_enum)]
    pub genome_release: common::cli::GenomeRelease,

    /// Windows size for TBI-based parallel import.
    #[arg(long, default_value = "100000")]
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
fn vcf_import(
    db: Arc<rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>>,
    args: &Args,
    path_in_vcf: &str,
    gnomad_version: GnomadVersion,
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

    windows
        .par_iter()
        .progress_with(common::cli::progress_bar(windows.len()))
        .map(|(chrom, begin, end)| {
            process_window(
                db.clone(),
                chrom,
                *begin,
                *end,
                args,
                path_in_vcf,
                gnomad_version,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

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
    gnomad_version: GnomadVersion,
) -> Result<(), anyhow::Error> {
    let cf_gnomad = db.cf_handle(&args.cf_name).unwrap();
    let mut reader =
        noodles_vcf::indexed_reader::Builder::default().build_from_path(path_in_vcf)?;
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
            for allele_no in 0..vcf_record.alternate_bases().len() {
                let key_buf: Vec<u8> =
                    common::keys::Var::from_vcf_allele(&vcf_record, allele_no).into();
                let record_buf = match gnomad_version {
                    GnomadVersion::Two => {
                        let details_options = serde_json::from_str(
                            args.import_fields_json
                                .as_ref()
                                .expect("has been set earlier"),
                        )?;
                        crate::pbs::gnomad::gnomad2::Record::from_vcf_allele(
                            &vcf_record,
                            allele_no,
                            &details_options,
                        )?
                        .encode_to_vec()
                    }
                    GnomadVersion::Three => {
                        let details_options = serde_json::from_str(
                            args.import_fields_json
                                .as_ref()
                                .expect("has been set earlier"),
                        )?;
                        crate::pbs::gnomad::gnomad3::Record::from_vcf_allele(
                            &vcf_record,
                            allele_no,
                            &details_options,
                        )?
                        .encode_to_vec()
                    }
                    GnomadVersion::Four => {
                        let details_options = serde_json::from_str(
                            args.import_fields_json
                                .as_ref()
                                .expect("has been set earlier"),
                        )?;
                        crate::pbs::gnomad::gnomad4::Record::from_vcf_allele(
                            &vcf_record,
                            allele_no,
                            &details_options,
                            args.gnomad_kind.into(),
                        )?
                        .encode_to_vec()
                    }
                    _ => anyhow::bail!("gnomAD version must be either 2, 3, or 4"),
                };
                db.put_cf(&cf_gnomad, &key_buf, &record_buf)?;
            }
        }
    }

    Ok(())
}

/// Some header fields to write to RocksDB meta data (gnomAD v4).
static MORE_HEADER_FIELDS_V4: &[&str] = &[
    "cadd_version",
    "gencode_version",
    "mane_select_version",
    "pangolin_version",
    "phylop_version",
    "polyphen_version",
    "revel_version",
    "seqrepo_version",
    "sift_version",
    "spliceai_version",
    "vrs_python_version",
    "vrs_schema_version",
];

/// Implementation of `gnomad_nuclear import` sub command.
pub fn run(common: &common::cli::Args, args: &Args) -> Result<(), anyhow::Error> {
    let gnomad_version: GnomadVersion = args.gnomad_version.parse()?;
    if !matches!(
        gnomad_version,
        GnomadVersion::Two | GnomadVersion::Three | GnomadVersion::Four
    ) {
        anyhow::bail!("gnomAD version must be either 2, 3, or 4");
    }

    // Put defaults for fields to serialize into args.
    let args = match gnomad_version {
        GnomadVersion::Two => Args {
            import_fields_json: args
                .import_fields_json
                .clone()
                .map(|v| {
                    serde_json::to_string(&serde_json::from_str::<gnomad2::DetailsOptions>(&v)?)
                })
                .or_else(|| Some(serde_json::to_string(&gnomad2::DetailsOptions::default())))
                .transpose()?,
            ..args.clone()
        },
        GnomadVersion::Three => Args {
            import_fields_json: args
                .import_fields_json
                .clone()
                .map(|v| {
                    serde_json::to_string(&serde_json::from_str::<gnomad3::DetailsOptions>(&v)?)
                })
                .or_else(|| Some(serde_json::to_string(&gnomad3::DetailsOptions::default())))
                .transpose()?,
            ..args.clone()
        },
        GnomadVersion::Four => Args {
            import_fields_json: args
                .import_fields_json
                .clone()
                .map(|v| {
                    serde_json::to_string(&serde_json::from_str::<gnomad3::DetailsOptions>(&v)?)
                })
                .or_else(|| Some(serde_json::to_string(&gnomad3::DetailsOptions::default())))
                .transpose()?,
            ..args.clone()
        },
        _ => anyhow::bail!("gnomAD version must be either 2, 3, or"),
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
    // Load additional metadata fields (v4).
    let mut more_header_values = Vec::new();
    for header_field in MORE_HEADER_FIELDS_V4 {
        if let Some(record::value::Collection::Unstructured(values)) = header
            .other_records()
            .get(&record::key::Other::from_str(header_field)?)
        {
            let val = values.first().expect("missing header value").to_owned();
            tracing::info!("  {}: {}", header_field, &val);
            more_header_values.push((header_field.to_string(), val));
        }
    }
    tracing::info!(
        "...done opening gnomAD-nuclear VCF file in {:?}",
        before_loading.elapsed()
    );

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
    db.put_cf(
        &cf_meta,
        "gnomad-kind",
        args.gnomad_kind.to_string().to_lowercase(),
    )?;
    db.put_cf(&cf_meta, "gnomad-version", &args.gnomad_version)?;
    if let Some(vep_version) = vep_version {
        db.put_cf(&cf_meta, "gnomad-vep-version", vep_version)?;
    }
    if let Some(dbsnp_version) = dbsnp_version {
        db.put_cf(&cf_meta, "gnomad-dbsnp-version", dbsnp_version)?;
    }
    if let Some(age_distributions) = age_distributions {
        db.put_cf(&cf_meta, "gnomad-age-distributions", age_distributions)?;
    }
    // Write additional metadata fields (v4).
    for (header_field, val) in more_header_values {
        db.put_cf(
            &cf_meta,
            &format!("gnomad-{}", header_field.replace('_', "-")),
            &val,
        )?;
    }
    tracing::info!(
        "... done opening RocksDB for writing in {:?}",
        before_opening_rocksdb.elapsed()
    );

    tracing::info!("Loading gnomad_nuclear VCF file into RocksDB...");
    let before_loading = std::time::Instant::now();
    for path_in_tsv in &args.path_in_vcf {
        tracing::info!("  importing file {} ...", &path_in_tsv);
        vcf_import(db.clone(), &args, path_in_tsv, gnomad_version)?;
    }
    tracing::info!(
        "... done loading gnomad_nuclear VCF file into RocksDB in {:?}",
        before_loading.elapsed()
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
    fn smoke_test_import_gnomad_exomes_grch37() -> Result<(), anyhow::Error> {
        let tmp_dir = TempDir::default();
        let common = common::cli::Args {
            verbose: Verbosity::new(1, 0),
        };
        let args = Args {
            genome_release: common::cli::GenomeRelease::Grch37,
            path_in_vcf: vec![String::from(
                "tests/gnomad-nuclear/example-exomes-grch37/v2.1/gnomad-exomes.vcf.bgz",
            )],
            path_out_rocksdb: format!("{}", tmp_dir.join("out-rocksdb").display()),
            cf_name: String::from("gnomad_nuclear_data"),
            path_wal_dir: None,
            tbi_window_size: 1_000_000,
            import_fields_json: Some(serde_json::to_string(
                &gnomad2::DetailsOptions::with_all_enabled(),
            )?),
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
                "tests/gnomad-nuclear/example-genomes-grch37/v2.1/gnomad-genomes.vcf.bgz",
            )],
            path_out_rocksdb: format!("{}", tmp_dir.join("out-rocksdb").display()),
            cf_name: String::from("gnomad_nuclear_data"),
            path_wal_dir: None,
            tbi_window_size: 1_000_000,
            import_fields_json: Some(serde_json::to_string(
                &gnomad2::DetailsOptions::with_all_enabled(),
            )?),
            gnomad_kind: GnomadKind::Genomes,
            gnomad_version: String::from("2.1"),
        };

        run(&common, &args)
    }

    #[test]
    fn smoke_test_import_gnomad_exomes_grch38_v2_1() -> Result<(), anyhow::Error> {
        let tmp_dir = TempDir::default();
        let common = common::cli::Args {
            verbose: Verbosity::new(1, 0),
        };
        let args = Args {
            genome_release: common::cli::GenomeRelease::Grch38,
            path_in_vcf: vec![String::from(
                "tests/gnomad-nuclear/example-exomes-grch38/v2.1/gnomad-exomes.vcf.bgz",
            )],
            path_out_rocksdb: format!("{}", tmp_dir.join("out-rocksdb").display()),
            cf_name: String::from("gnomad_nuclear_data"),
            path_wal_dir: None,
            tbi_window_size: 1_000_000,
            import_fields_json: Some(serde_json::to_string(
                &gnomad2::DetailsOptions::with_all_enabled(),
            )?),
            gnomad_kind: GnomadKind::Exomes,
            gnomad_version: String::from("2.1"),
        };

        run(&common, &args)
    }

    #[test]
    fn smoke_test_import_gnomad_genomes_grch38_v3_1() -> Result<(), anyhow::Error> {
        let tmp_dir = TempDir::default();
        let common = common::cli::Args {
            verbose: Verbosity::new(1, 0),
        };
        let args = Args {
            genome_release: common::cli::GenomeRelease::Grch38,
            path_in_vcf: vec![String::from(
                "tests/gnomad-nuclear/example-genomes-grch38/v3.1/gnomad-genomes.vcf.bgz",
            )],
            path_out_rocksdb: format!("{}", tmp_dir.join("out-rocksdb").display()),
            cf_name: String::from("gnomad_nuclear_data"),
            path_wal_dir: None,
            tbi_window_size: 1_000_000,
            import_fields_json: Some(serde_json::to_string(
                &gnomad3::DetailsOptions::with_all_enabled(),
            )?),
            gnomad_kind: GnomadKind::Genomes,
            gnomad_version: String::from("3.1"),
        };

        run(&common, &args)
    }

    #[test]
    fn smoke_test_import_gnomad_exomes_grch38_v4_0() -> Result<(), anyhow::Error> {
        let tmp_dir = TempDir::default();
        let common = common::cli::Args {
            verbose: Verbosity::new(1, 0),
        };
        let args = Args {
            genome_release: common::cli::GenomeRelease::Grch38,
            path_in_vcf: vec![String::from(
                "tests/gnomad-nuclear/example-exomes-grch38/v4.0/gnomad-exomes.vcf.bgz",
            )],
            path_out_rocksdb: format!("{}", tmp_dir.join("out-rocksdb").display()),
            cf_name: String::from("gnomad_nuclear_data"),
            path_wal_dir: None,
            tbi_window_size: 1_000_000,
            import_fields_json: Some(serde_json::to_string(
                &gnomad3::DetailsOptions::with_all_enabled(),
            )?),
            gnomad_kind: GnomadKind::Exomes,
            gnomad_version: String::from("4.0"),
        };

        run(&common, &args)
    }

    #[test]
    fn smoke_test_import_gnomad_genomes_grch38_v4_0() -> Result<(), anyhow::Error> {
        let tmp_dir = TempDir::default();
        let common = common::cli::Args {
            verbose: Verbosity::new(1, 0),
        };
        let args = Args {
            genome_release: common::cli::GenomeRelease::Grch38,
            path_in_vcf: vec![String::from(
                "tests/gnomad-nuclear/example-genomes-grch38/v4.0/gnomad-genomes.vcf.bgz",
            )],
            path_out_rocksdb: format!("{}", tmp_dir.join("out-rocksdb").display()),
            cf_name: String::from("gnomad_nuclear_data"),
            path_wal_dir: None,
            tbi_window_size: 1_000_000,
            import_fields_json: Some(serde_json::to_string(
                &gnomad3::DetailsOptions::with_all_enabled(),
            )?),
            gnomad_kind: GnomadKind::Genomes,
            gnomad_version: String::from("4.0"),
        };

        run(&common, &args)
    }
}
