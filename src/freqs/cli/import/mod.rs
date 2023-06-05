//! Import of sequence variant frequencies.

pub mod auto;
pub mod mt;
pub mod xy;

use std::{collections::HashMap, sync::Arc};

use clap::Parser;
use hgvs::static_data::{Assembly, ASSEMBLY_INFOS};

use crate::{
    common::{
        self,
        cli::{GenomeRelease, CANONICAL},
    },
    freqs,
};

use super::reading::ContigMap;

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
    pub path_helix_mtdb: Option<String>,

    /// Optional path to WAL directory.
    #[arg(long)]
    pub path_wal_dir: Option<String>,
}

/// Guess the assembly from the given header.
///
/// If the header only contains chrM, for example, the result may be ambiguous. Use `ambiguous_ok`
/// to allow or disallow this.  You can specify an initial value for the assembly to overcome
/// issues.  If the result is incompatible with the `initial_assembly` then an error will
/// be returned.
pub fn guess_assembly(
    vcf_header: &noodles_vcf::Header,
    ambiguous_ok: bool,
    initial_assembly: Option<Assembly>,
) -> Result<Assembly, anyhow::Error> {
    let mut result = initial_assembly;

    let assembly_infos = vec![
        (Assembly::Grch37p10, &ASSEMBLY_INFOS[Assembly::Grch37p10]),
        (Assembly::Grch38, &ASSEMBLY_INFOS[Assembly::Grch38]),
    ];

    // Check each assembly.
    for (assembly, info) in assembly_infos.iter() {
        // Collect contig name / length pairs for the assembly.
        let contig_map = ContigMap::new(*assembly);
        let mut lengths = HashMap::new();
        for seq in &info.sequences {
            if CANONICAL.contains(&seq.name.as_str()) {
                lengths.insert(
                    contig_map.name_map.get(seq.name.as_str()).unwrap(),
                    seq.length,
                );
            }
        }

        // Count compatible and incompatible contigs.
        let mut incompatible = 0;
        let mut compatible = 0;
        for (name, data) in vcf_header.contigs() {
            if let Some(length) = data.length() {
                let idx = contig_map.name_map.get(name.as_ref());
                if let Some(idx) = idx {
                    let name = &info.sequences[*idx].name;
                    if CANONICAL.contains(&name.as_ref()) {
                        if *lengths.get(idx).unwrap() == length {
                            compatible += 1;
                        } else {
                            incompatible += 1;
                        }
                    }
                }
            } else {
                tracing::warn!(
                    "Cannot guess assembly because no length for contig {}",
                    &name
                );
                compatible = 0;
                break;
            }
        }

        if compatible > 0 && incompatible == 0 {
            // Found a compatible assembly.  Check if we already have one and bail out if
            // ambiguity is not allowed.  Anyway, we only keep the first found compatible
            // assembly.
            if let Some(result) = result {
                if result != *assembly && !ambiguous_ok {
                    return Err(anyhow::anyhow!(
                        "Found ambiguity;  initial={:?}, previous={:?}, current={:?}",
                        initial_assembly,
                        result,
                        assembly,
                    ));
                }
                // else: do not re-assign
            } else {
                result = Some(*assembly);
            }
        } else {
            // Found incompatible assembly, bail out if is the initial assembly.
            if let Some(initial_assembly) = initial_assembly {
                if initial_assembly == *assembly {
                    return Err(anyhow::anyhow!(
                        "Incompatible with initial assembly {:?}",
                        result.unwrap()
                    ));
                }
            }
        }
    }

    if let Some(result) = result {
        Ok(result)
    } else {
        Err(anyhow::anyhow!("No matching assembly found"))
    }
}

/// Return mapping from chromosome to path.
fn assign_to_chrom(
    paths: &Vec<String>,
    assembly: Assembly,
) -> Result<HashMap<usize, String>, anyhow::Error> {
    let contig_map = ContigMap::new(assembly);
    let mut res = HashMap::new();

    for path in paths {
        tracing::debug!("    path = {}", path);
        let mut reader = noodles_util::variant::reader::Builder::default().build_from_path(path)?;
        let header = Box::new(reader.read_header()?);
        guess_assembly(header.as_ref(), true, Some(assembly))?;
        let record = reader
            .records(header.as_ref())
            .next()
            .transpose()?
            .ok_or(anyhow::anyhow!("No records in VCF file {}", path))?;
        let k = contig_map.chrom_to_idx(record.chromosome());
        let v = path.clone();
        tracing::debug!("    k = {}, v = {}", &k, &v);
        res.insert(k, v);
    }
    tracing::debug!("    result = {:?}", &res);

    Ok(res)
}

/// Import of autosomal variant frequencies.
fn import_autosomal(
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    genome_release: Assembly,
    path_genome: Option<&String>,
    path_exome: Option<&String>,
) -> Result<(), anyhow::Error> {
    Ok(())
}

/// Import of gonomosomal variant frequencies.
fn import_gonomosomal(
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    genome_release: Assembly,
    path_genome: Option<&String>,
    path_exome: Option<&String>,
) -> Result<(), anyhow::Error> {
    Ok(())
}

/// Import of mitochondrial variant frequencies.
fn import_chrmt(
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    genome_release: Assembly,
    path_gnomad_mtdna: Option<&String>,
    path_helixmtdb: Option<&String>,
) -> Result<(), anyhow::Error> {
    let cf_mtdna = db.cf_handle("mitochondrial").unwrap();

    let mut chrmt_written = 0usize;
    let mut mt_reader = auto::Reader::new(
        path_gnomad_mtdna.map(|s| s.as_str()),
        path_helixmtdb.map(|s| s.as_str()),
        Some(genome_release),
    )?;

    let mut prev = std::time::Instant::now();
    let mut has_next = true;
    while has_next {
        has_next = mt_reader.run(|variant, gnomad_mtdna, helix_mtdb| {
            if prev.elapsed().as_secs() > 60 {
                tracing::info!("at {:?}", &variant);
                prev = std::time::Instant::now();
            }

            let key: Vec<u8> = variant.into();
            let mut value = [0u8; 24];
            freqs::serialized::mt::Record {
                gnomad_mtdna,
                helix_mtdb,
            }
            .to_buf(&mut value);
            db.put_cf(&cf_mtdna, key, value)?;

            chrmt_written += 1;

            Ok(())
        })?;
    }

    Ok(())
}

/// Implementation of `gnomad_nuclear import` sub command.
pub fn run(_common: &common::cli::Args, args: &Args) -> Result<(), anyhow::Error> {
    // Guess genome release from paths.
    let genome_release = match args.genome_release {
        GenomeRelease::Grch37 => Assembly::Grch37p10, // has chrMT!
        GenomeRelease::Grch38 => Assembly::Grch38,
    };

    // Open the RocksDB for writing.
    tracing::info!("Opening RocksDB for writing ...");
    let before_opening_rocksdb = std::time::Instant::now();
    let options = common::rocks_utils::tune_options(
        rocksdb::Options::default(),
        args.path_wal_dir.as_ref().map(|s| s.as_ref()),
    );
    let cf_names = ["meta", "autosomal", "gonosomal", "mitochondrial"];
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
        tracing::info!("  k={}; from:", k);
        tracing::info!("    - genomes: {:?}", path_genome);
        tracing::info!("    - exomes:  {:?}", path_exome);
        import_autosomal(&db, genome_release, path_genome, path_exome)?;
    }
    tracing::info!(
        "... done importing autosomal variants in {:?}",
        before_auto.elapsed()
    );
    tracing::info!("Importing gonomosomal variants...");
    let before_xy = std::time::Instant::now();
    for k in &xy_keys {
        let path_genome = genomes_xy_by_chrom.get(k);
        let path_exome = exomes_xy_by_chrom.get(k);
        tracing::info!("  k={}; from:", k);
        tracing::info!("    - genomes: {:?}", path_genome);
        tracing::info!("    - exomes:  {:?}", path_exome);
        import_gonomosomal(&db, genome_release, path_genome, path_exome)?;
    }
    tracing::info!(
        "... done importing gonomosomal variants in {:?}",
        before_xy.elapsed()
    );
    tracing::info!("Importing mitochondrial variants...");
    let before_mito = std::time::Instant::now();
    import_chrmt(
        &db,
        genome_release,
        args.path_gnomad_mtdna.as_ref(),
        args.path_helix_mtdb.as_ref(),
    )?;
    tracing::info!(
        "... done importing mitochondrial variants in {:?}",
        before_mito.elapsed()
    );

    tracing::info!("Running RocksDB compaction ...");
    let before_compaction = std::time::Instant::now();
    common::rocks_utils::force_compaction_cf(&db, cf_names, Some("  "), true)?;
    tracing::info!(
        "... done compacting RocksDB in {:?}",
        before_compaction.elapsed()
    );

    tracing::info!("All done. Have a nice day!");
    Ok(())
}

#[cfg(test)]
mod test {
    use hgvs::static_data::Assembly;

    use super::*;
    use noodles_util::variant::reader::Builder as VariantReaderBuilder;

    #[test]
    fn guess_assembly_helix_chrmt_ambiguous_ok_initial_none() -> Result<(), anyhow::Error> {
        let path = "tests/freqs/import/guess_assembly/helix.chrM.vcf";
        let mut reader = VariantReaderBuilder::default().build_from_path(path)?;
        let header = reader.read_header()?;

        let actual = guess_assembly(&header, true, None)?;
        assert_eq!(actual, Assembly::Grch37p10);

        Ok(())
    }

    #[test]
    fn guess_assembly_helix_chrmt_ambiguous_ok_initial_override() -> Result<(), anyhow::Error> {
        let path = "tests/freqs/import/guess_assembly/helix.chrM.vcf";
        let mut reader = VariantReaderBuilder::default().build_from_path(path)?;
        let header = reader.read_header()?;

        let actual = guess_assembly(&header, true, Some(Assembly::Grch37p10))?;
        assert_eq!(actual, Assembly::Grch37p10);

        Ok(())
    }

    #[test]
    fn guess_assembly_helix_chrmt_ambiguous_ok_initial_override_fails() -> Result<(), anyhow::Error>
    {
        let path = "tests/freqs/import/guess_assembly/helix.chrM.vcf";
        let mut reader = VariantReaderBuilder::default().build_from_path(path)?;
        let header = reader.read_header()?;

        assert!(guess_assembly(&header, false, Some(Assembly::Grch37)).is_err());

        Ok(())
    }

    #[test]
    fn guess_assembly_helix_chrmt_ambiguous_fail() -> Result<(), anyhow::Error> {
        let path = "tests/freqs/import/guess_assembly/helix.chrM.vcf";
        let mut reader = VariantReaderBuilder::default().build_from_path(path)?;
        let header = reader.read_header()?;

        assert!(guess_assembly(&header, false, None).is_err());

        Ok(())
    }
}
