//! CLI for importing gnomad-SV data.

pub mod exac_cnv;
pub mod gnomad_cnv4;
pub mod gnomad_sv2;
pub mod gnomad_sv4;

use std::sync::Arc;

use crate::{
    common::{self},
    gnomad_nuclear::cli::import::{GnomadKind, GnomadVersion},
};
use clap::Parser;

/// Command line arguments for `gnomad_nuclear import` sub command.
#[derive(Parser, Debug, Clone, Default)]
#[command(about = "import gnomAD-SV data into RocksDB", long_about = None)]
pub struct Args {
    /// Path to input VCF file(s) -- or TSV in case of ExAC.
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

    /// Data column family to import into.
    #[arg(long, default_value = "gnomad_sv")]
    pub cf_name: String,
    /// Optional path to RocksDB WAL directory.
    #[arg(long)]
    pub path_wal_dir: Option<String>,
}

/// Implementation of `gnomad-sv import` sub command.
pub fn run(common: &common::cli::Args, args: &Args) -> Result<(), anyhow::Error> {
    let gnomad_version: GnomadVersion = args.gnomad_version.parse()?;

    tracing::info!("Starting 'gnomad-sv import' command");
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
    db.put_cf(
        &cf_meta,
        "gnomad-kind",
        args.gnomad_kind.to_string().to_lowercase(),
    )?;
    db.put_cf(&cf_meta, "gnomad-version", &args.gnomad_version)?;
    tracing::info!(
        "... done opening RocksDB for writing in {:?}",
        before_opening_rocksdb.elapsed()
    );

    tracing::info!("Loading gnomad-SV file into RocksDB...");
    let before_loading = std::time::Instant::now();
    let cf_data = db.cf_handle(&args.cf_name).unwrap();
    match (gnomad_version, args.gnomad_kind, args.genome_release) {
        (GnomadVersion::One, GnomadKind::Exomes, common::cli::GenomeRelease::Grch37) => {
            if args.path_in_vcf.len() != 1 {
                anyhow::bail!("ExAC CNV import requires exactly one input file");
            }
            exac_cnv::import(&db, &cf_data, &args.path_in_vcf[0])?;
        }
        (GnomadVersion::Two, GnomadKind::Genomes, common::cli::GenomeRelease::Grch37) => {
            tracing::info!("- selected gnomAD SV import for GRCh37");
            for path_in_vcf in &args.path_in_vcf {
                tracing::info!("  - file {}", &path_in_vcf);
                gnomad_sv2::import(&db, &cf_data, path_in_vcf)?;
            }
        }
        (GnomadVersion::Four, GnomadKind::Exomes, common::cli::GenomeRelease::Grch38) => {
            tracing::info!("- selected gnomAD CNV v4 import for GRCh38");
            for path_in_vcf in &args.path_in_vcf {
                tracing::info!("  - file {}", &path_in_vcf);
                gnomad_cnv4::import(&db, &cf_data, path_in_vcf)?;
            }
        }
        (GnomadVersion::Four, GnomadKind::Genomes, common::cli::GenomeRelease::Grch38) => {
            tracing::info!("- selected gnomAD SV v4 import for GRCh38");
            gnomad_sv4::import(&db, &args.cf_name, &args.path_in_vcf)?;
        }
        _ => anyhow::bail!(
            "invalid combination of gnomAD version, kind and genome release, valid ones \
            are v1 (ExAC) for exomes and GRCh37, v2 (gnomAD) for genomes and GRCh37, \
            v4 (gnomAD) for genomes/exomes and GRCh38"
        ),
    }
    tracing::info!(
        "... done loading gnomAD-SV file into RocksDB in {:?}",
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
    use clap_verbosity_flag::Verbosity;
    use temp_testdir::TempDir;

    /// Fixture with temporary directory.
    #[rstest::fixture]
    fn tmp_dir() -> TempDir {
        TempDir::default()
    }

    /// Fixture that takes the temporary directory and returns base arguments which
    /// point to the temporary directory.  Must return the `TempDir` object as well
    /// because we need to keep the directory alive.
    #[rstest::fixture]
    fn args_tmpdir(tmp_dir: TempDir) -> (super::Args, TempDir) {
        (
            super::Args {
                path_out_rocksdb: format!("{}", tmp_dir.join("out-rocksdb").display()),
                cf_name: String::from("gnomad_sv"),
                path_wal_dir: None,
                ..Default::default()
            },
            tmp_dir,
        )
    }

    /// Run smoke test of importing ExAC CNVs.
    #[tracing_test::traced_test]
    #[rstest::rstest]
    #[test]
    fn smoke_test_import_exac_cnv(
        args_tmpdir: (super::Args, TempDir),
    ) -> Result<(), anyhow::Error> {
        let common = crate::common::cli::Args {
            verbose: Verbosity::new(1, 0),
        };
        let args = super::Args {
            genome_release: crate::common::cli::GenomeRelease::Grch37,
            gnomad_kind: crate::gnomad_nuclear::cli::import::GnomadKind::Exomes,
            gnomad_version: String::from("1.0"),
            path_in_vcf: vec![String::from(
                "tests/gnomad-sv/exac-cnv/exac-final.autosome-1pct-sq60-qc-prot-coding.cnv.bed",
            )],
            ..args_tmpdir.0
        };

        super::run(&common, &args)
    }

    /// Run smoke test of importing gnomAD SV v2.
    #[tracing_test::traced_test]
    #[rstest::rstest]
    #[test]
    fn smoke_test_import_gnomad_sv2(
        args_tmpdir: (super::Args, TempDir),
    ) -> Result<(), anyhow::Error> {
        let common = crate::common::cli::Args {
            verbose: Verbosity::new(1, 0),
        };
        let args = super::Args {
            genome_release: crate::common::cli::GenomeRelease::Grch37,
            gnomad_kind: crate::gnomad_nuclear::cli::import::GnomadKind::Genomes,
            gnomad_version: String::from("2.1"),
            path_in_vcf: vec![
                String::from("tests/gnomad-sv/gnomad-sv2/gnomad_v2.1_sv.sites.vcf"),
                String::from("tests/gnomad-sv/gnomad-sv2/gnomad_v2.1_sv.controls_only.sites.vcf"),
                String::from("tests/gnomad-sv/gnomad-sv2/gnomad_v2.1_sv.nonneuro.sites.vcf"),
            ],
            ..args_tmpdir.0
        };

        super::run(&common, &args)
    }

    /// Run smoke test of importing gnomAD CNV v4.
    #[tracing_test::traced_test]
    #[rstest::rstest]
    #[test]
    fn smoke_test_import_gnomad_cnv4(
        args_tmpdir: (super::Args, TempDir),
    ) -> Result<(), anyhow::Error> {
        let common = crate::common::cli::Args {
            verbose: Verbosity::new(1, 0),
        };
        let args = super::Args {
            genome_release: crate::common::cli::GenomeRelease::Grch38,
            gnomad_kind: crate::gnomad_nuclear::cli::import::GnomadKind::Genomes,
            gnomad_version: String::from("4.0"),
            path_in_vcf: vec![String::from(
                "tests/gnomad-sv/gnomad-cnv4/gnomad.v4.0.cnv.all.vcf.gz",
            )],
            ..args_tmpdir.0
        };

        super::run(&common, &args)
    }

    /// Run smoke test of importing gnomAD SV v4.
    #[tracing_test::traced_test]
    #[rstest::rstest]
    #[test]
    fn smoke_test_import_gnomad_sv4(
        args_tmpdir: (super::Args, TempDir),
    ) -> Result<(), anyhow::Error> {
        let common = crate::common::cli::Args {
            verbose: Verbosity::new(1, 0),
        };
        let args = super::Args {
            genome_release: crate::common::cli::GenomeRelease::Grch38,
            gnomad_kind: crate::gnomad_nuclear::cli::import::GnomadKind::Genomes,
            gnomad_version: String::from("4.0"),
            path_in_vcf: vec![
                String::from("tests/gnomad-sv/gnomad-sv4/gnomad.v4.0.sv.chr1.vcf.gz"),
                String::from("tests/gnomad-sv/gnomad-sv4/gnomad.v4.0.sv.chr2.vcf.gz"),
            ],
            ..args_tmpdir.0
        };

        super::run(&common, &args)
    }
}
