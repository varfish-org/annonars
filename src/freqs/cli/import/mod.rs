//! Import of sequence variant frequencies.

pub mod auto;
pub mod mt;
pub mod reading;
pub mod xy;

use std::{collections::HashMap, sync::Arc};

use clap::Parser;
use indicatif::ParallelProgressIterator;
use noodles::csi::BinningIndex as _;
use rayon::prelude::*;

use crate::{common, freqs};

use reading::ContigMap;

/// Command line arguments for `db create freqs` sub command.
#[derive(Parser, Debug, Clone)]
#[command(about = "Construct sequence variant frequencies database", long_about = None)]
pub struct Args {
    /// Genome build to use in the build.
    #[arg(long, value_enum)]
    pub genome_release: common::cli::GenomeRelease,
    /// Path to the output database to build.
    #[arg(long)]
    pub path_out_rocksdb: String,

    /// Path(s) to the autosomal gnomAD exomes VCF file(s).
    #[arg(long)]
    pub path_gnomad_exomes_auto: Vec<String>,
    /// Path(s) to the autosomal gnomAD genomes VCF file(s).
    #[arg(long)]
    pub path_gnomad_genomes_auto: Vec<String>,
    /// Path(s) to the gonosomal gnomAD exomes VCF file(s).
    #[arg(long)]
    pub path_gnomad_exomes_xy: Vec<String>,
    /// Path(s) to the gonosomal gnomAD genomes VCF file(s).
    #[arg(long)]
    pub path_gnomad_genomes_xy: Vec<String>,
    /// Path(s) to the gnomAD mtDNA VCF file(s).
    #[arg(long)]
    pub path_gnomad_mtdna: Option<String>,
    /// Path(s) to the HelixMtDb TSV file.
    #[arg(long)]
    pub path_helixmtdb: Option<String>,

    /// Optional path to WAL directory.
    #[arg(long)]
    pub path_wal_dir: Option<String>,
    /// Windows size for TBI-based parallel import.
    #[arg(long, default_value = "100000")]
    pub tbi_window_size: usize,

    /// Version of gnomAD genomes.
    #[arg(long)]
    pub gnomad_genomes_version: String,
    /// Version of gnomAD exomes.
    #[arg(long)]
    pub gnomad_exomes_version: String,
    /// Version of gnomAD mtDNA.
    #[arg(long)]
    pub gnomad_mtdna_version: String,
    /// Version of HelixMtDb.
    #[arg(long)]
    pub helixmtdb_version: String,
}

/// Return mapping from chromosome to path.
fn assign_to_chrom(
    paths: &Vec<String>,
    assembly: biocommons_bioutils::assemblies::Assembly,
) -> Result<HashMap<usize, String>, anyhow::Error> {
    let contig_map = ContigMap::new(assembly);
    let mut res = HashMap::new();

    for path in paths {
        let mut reader =
            noodles::vcf::io::indexed_reader::Builder::default().build_from_path(path)?;
        let header = Box::new(reader.read_header()?);
        freqs::cli::import::reading::guess_assembly(header.as_ref(), true, Some(assembly))?;
        let record = reader
            .record_bufs(header.as_ref())
            .next()
            .transpose()?
            .ok_or(anyhow::anyhow!("No records in VCF file {}", path))?;
        let k = contig_map
            .chrom_to_idx(record.reference_sequence_name())
            .map_err(|e| {
                anyhow::anyhow!(
                    "Error mapping chromosome {} to index: {}",
                    record.reference_sequence_name(),
                    e
                )
            })?;
        let v = path.clone();
        res.insert(k, v);
    }

    Ok(res)
}

/// Get windows for the up to two given paths.
pub fn build_windows(
    genome_release: biocommons_bioutils::assemblies::Assembly,
    tbi_window_size: usize,
    paths: &[String],
) -> Result<Vec<(String, usize, usize)>, anyhow::Error> {
    let mut result = Vec::new();

    for path in paths.iter() {
        // Load tabix header and create BGZF reader with tabix index.
        let tabix_src = format!("{}.tbi", path);
        let index = noodles::tabix::read(tabix_src)?;
        let header = index.header().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "missing tabix header")
        })?;

        // Build list of canonical chromosome names from header.
        let canonical_header_chroms = header
            .reference_sequence_names()
            .iter()
            .filter_map(|chrom| {
                let chrom = &chrom.to_string();
                let canon_chrom = chrom.strip_prefix("chr").unwrap_or(chrom);
                if common::cli::is_canonical(canon_chrom) {
                    Some((common::cli::canonicalize(canon_chrom), chrom.clone()))
                } else {
                    None
                }
            })
            .collect::<std::collections::HashMap<String, String>>();

        // Generate list of regions on canonical chromosomes, limited to those present in header.
        result.append(
            &mut common::cli::build_genome_windows(genome_release, Some(tbi_window_size))?
                .into_iter()
                .filter_map(|(window_chrom, begin, end)| {
                    let canon_chrom = common::cli::canonicalize(&window_chrom);
                    canonical_header_chroms
                        .get(&canon_chrom)
                        .map(|header_chrom| (header_chrom.clone(), begin, end))
                })
                .collect::<Vec<_>>(),
        );
    }

    result.sort();
    result.dedup();

    Ok(result)
}

/// Implementation of `gnomad_nuclear import` sub command.
pub fn run(common: &common::cli::Args, args: &Args) -> Result<(), anyhow::Error> {
    tracing::info!("Starting mehari frequency import ...");
    tracing::info!("  common = {:#?}", &common);
    tracing::info!("  args =   {:#?}", &args);

    // Guess genome release from paths.
    let genome_release = match args.genome_release {
        common::cli::GenomeRelease::Grch37 => biocommons_bioutils::assemblies::Assembly::Grch37p10, // has chrMT!
        common::cli::GenomeRelease::Grch38 => biocommons_bioutils::assemblies::Assembly::Grch38,
    };

    // Open the RocksDB for writing.
    tracing::info!("Opening RocksDB for writing ...");
    let before_opening_rocksdb = std::time::Instant::now();
    let options = rocksdb_utils_lookup::tune_options(
        rocksdb::Options::default(),
        args.path_wal_dir.as_ref().map(|s| s.as_ref()),
    );
    let cf_names = ["meta", "autosomal", "gonosomal", "mitochondrial"];
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
        "gnomad-exomes-version",
        &args.gnomad_exomes_version,
    )?;
    db.put_cf(
        &cf_meta,
        "gnomad-genomoes-version",
        &args.gnomad_genomes_version,
    )?;
    db.put_cf(&cf_meta, "gnomad-mtdna-version", &args.gnomad_mtdna_version)?;
    db.put_cf(&cf_meta, "helixmtdb-version", &args.helixmtdb_version)?;
    db.put_cf(
        &cf_meta,
        "genome-release",
        format!("{}", args.genome_release),
    )?;
    tracing::info!(
        "... done opening RocksDB for writing in {:?}",
        before_opening_rocksdb.elapsed()
    );

    tracing::info!("Determine each file's chromosome (assuming one chrom per file)...");
    let before_chroms = std::time::Instant::now();
    let genomes_auto_by_chrom = assign_to_chrom(&args.path_gnomad_genomes_auto, genome_release)?;
    let exomes_auto_by_chrom = assign_to_chrom(&args.path_gnomad_exomes_auto, genome_release)?;
    let auto_keys = {
        let mut auto_keys = Vec::new();
        genomes_auto_by_chrom
            .keys()
            .for_each(|k| auto_keys.push(*k));
        exomes_auto_by_chrom.keys().for_each(|k| auto_keys.push(*k));
        auto_keys.sort();
        auto_keys.dedup();
        auto_keys
    };
    let genomes_xy_by_chrom = assign_to_chrom(&args.path_gnomad_genomes_xy, genome_release)?;
    let exomes_xy_by_chrom = assign_to_chrom(&args.path_gnomad_exomes_xy, genome_release)?;
    let xy_keys = {
        let mut xy_keys = Vec::new();
        genomes_xy_by_chrom.keys().for_each(|k| xy_keys.push(*k));
        exomes_xy_by_chrom.keys().for_each(|k| xy_keys.push(*k));
        xy_keys.sort();
        xy_keys.dedup();
        xy_keys
    };
    tracing::info!(
        "... done getting chromosomes in {:?}",
        before_chroms.elapsed()
    );

    tracing::info!("Importing autosomal variants...");
    let before_auto = std::time::Instant::now();
    for k in &auto_keys {
        let path_genome = genomes_auto_by_chrom.get(k);
        let path_exome = exomes_auto_by_chrom.get(k);
        tracing::info!("  contig {} from:", common::cli::CANONICAL[*k]);
        tracing::info!("    - genomes: {:?}", path_genome);
        tracing::info!("    - exomes:  {:?}", path_exome);

        let paths = {
            let mut paths = Vec::new();
            if let Some(path_genome) = path_genome {
                paths.push(path_genome.clone());
            }
            if let Some(path_exome) = path_exome {
                paths.push(path_exome.clone());
            }
            paths
        };
        let windows = build_windows(genome_release, args.tbi_window_size, &paths)?;
        windows
            .par_iter()
            .progress_with(common::cli::progress_bar(windows.len()))
            .map(|(chrom, begin, end)| {
                let start = noodles::core::position::Position::try_from(begin + 1)?;
                let stop = noodles::core::position::Position::try_from(*end)?;
                let region = noodles::core::region::Region::new(chrom.as_bytes(), start..=stop);
                auto::import_region(&db, path_genome, path_exome, &region)
            })
            .collect::<Result<Vec<_>, _>>()?;
    }
    tracing::info!(
        "... done importing autosomal variants in {:?}",
        before_auto.elapsed()
    );
    tracing::info!("Importing gonosomal variants...");
    let before_xy = std::time::Instant::now();
    for k in &xy_keys {
        let path_genome = genomes_xy_by_chrom.get(k);
        let path_exome = exomes_xy_by_chrom.get(k);
        tracing::info!("  contig {} from:", common::cli::CANONICAL[*k]);
        tracing::info!("    - genomes: {:?}", path_genome);
        tracing::info!("    - exomes:  {:?}", path_exome);

        let paths = {
            let mut paths = Vec::new();
            if let Some(path_genome) = path_genome {
                paths.push(path_genome.clone());
            }
            if let Some(path_exome) = path_exome {
                paths.push(path_exome.clone());
            }
            paths
        };
        let windows = build_windows(genome_release, args.tbi_window_size, &paths)?;
        windows
            .par_iter()
            .progress_with(common::cli::progress_bar(windows.len()))
            .map(|(chrom, begin, end)| {
                let start = noodles::core::position::Position::try_from(begin + 1)?;
                let stop = noodles::core::position::Position::try_from(*end)?;
                let region = noodles::core::region::Region::new(chrom.as_bytes(), start..=stop);
                xy::import_region(&db, path_genome, path_exome, &region)
            })
            .collect::<Result<Vec<_>, _>>()?;
    }
    tracing::info!(
        "... done importing gonosomal variants in {:?}",
        before_xy.elapsed()
    );
    tracing::info!("Importing mitochondrial variants...");
    let before_mito = std::time::Instant::now();

    let path_gnomad = args.path_gnomad_mtdna.as_ref();
    let path_helix = args.path_helixmtdb.as_ref();
    tracing::info!("  contig MT from:");
    tracing::info!("    - gnomAD:     {:?}", &path_gnomad);
    tracing::info!("    - HelixMtDb:  {:?}", &path_helix);

    let paths = {
        let mut paths = Vec::new();
        if let Some(path_gnomad) = path_gnomad {
            paths.push(path_gnomad.clone());
        }
        if let Some(path_helix) = path_helix {
            paths.push(path_helix.clone());
        }
        paths
    };
    let windows = build_windows(genome_release, args.tbi_window_size, &paths)?;
    windows
        .par_iter()
        .progress_with(common::cli::progress_bar(windows.len()))
        .map(|(chrom, begin, end)| {
            let start = noodles::core::position::Position::try_from(begin + 1)?;
            let stop = noodles::core::position::Position::try_from(*end)?;
            let region = noodles::core::region::Region::new(chrom.as_bytes(), start..=stop);
            mt::import_region(&db, path_gnomad, path_helix, &region)
        })
        .collect::<Result<Vec<_>, _>>()?;

    tracing::info!(
        "... done importing mitochondrial variants in {:?}",
        before_mito.elapsed()
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
