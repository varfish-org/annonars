//! Parallelized import via TBI.

use std::io::BufRead;

use indicatif::ParallelProgressIterator;
use rayon::prelude::*;

use crate::{common, tsv};

use super::Args;

/// Helper function to resolve regions.
fn resolve_region(
    header: &noodles_csi::index::Header,
    region: &noodles_core::Region,
) -> std::io::Result<usize> {
    header
        .reference_sequence_names()
        .get_index_of(region.name())
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "missing reference sequence name",
            )
        })
}

/// Helper function for parsing start positions.
pub fn parse_start_position(
    s: &str,
    coordinate_system: noodles_csi::index::header::format::CoordinateSystem,
) -> std::io::Result<noodles_core::Position> {
    fn invalid_position<E>(_: E) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid position")
    }

    match coordinate_system {
        noodles_csi::index::header::format::CoordinateSystem::Gff => {
            s.parse().map_err(invalid_position)
        }
        noodles_csi::index::header::format::CoordinateSystem::Bed => s
            .parse::<usize>()
            .map_err(invalid_position)
            .and_then(|n| noodles_core::Position::try_from(n + 1).map_err(invalid_position)),
    }
}

/// Helper function for region intersection.
pub fn intersects(
    header: &noodles_csi::index::Header,
    line: &str,
    region: &noodles_core::region::Region,
) -> Result<bool, anyhow::Error> {
    const DELIMITER: char = '\t';

    let fields: Vec<_> = line.split(DELIMITER).collect();

    let reference_sequence_name = fields[header.reference_sequence_name_index()];

    let raw_start = fields[header.start_position_index() - 1];
    let coordinate_system = header.format().coordinate_system();
    let start = parse_start_position(raw_start, coordinate_system)?;

    let end = if let Some(i) = header.end_position_index() {
        fields[i - 1].parse()?
    } else {
        start.checked_add(1).expect("attempt to add with overflow")
    };

    let interval = noodles_core::region::Interval::from(start..=end);

    Ok(reference_sequence_name == region.name() && interval.intersects(region.interval()))
}

/// Perform the import of a single region.
pub fn tsv_import_window(
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    args: &Args,
    config: &tsv::schema::infer::Config,
    schema: &tsv::schema::FileSchema,
    path_in_tsv: &str,
    window: &(usize, noodles_core::Region),
) -> Result<(), anyhow::Error> {
    // Get column family handle.
    let cf_data = db.cf_handle(&args.cf_name).unwrap();

    // Read tabix index.
    let tabix_src = format!("{}.tbi", path_in_tsv);
    let index = noodles_tabix::read(tabix_src)?;
    let header = index.header().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "missing tabix header")
    })?;
    let mut reader = std::fs::File::open(path_in_tsv).map(noodles_bgzf::Reader::new)?;

    // Query TBI for chunks.
    let (ref_id, region) = window;
    let chunks = index.query(*ref_id, region.interval())?;
    let query = noodles_csi::io::Query::new(&mut reader, chunks);

    // Read through the overlapping lines.
    let ctx = tsv::coding::Context::new(config.clone(), schema.clone());
    for result in query.lines() {
        let line = result?;

        if intersects(header, &line, region)? {
            super::process_tsv_line(&line, &ctx, db, &cf_data)?;
        }
    }

    Ok(())
}

/// Perform the import of multiple TSV files in parallel using region-based parallelism.
pub fn tsv_import(
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    args: &Args,
    config: &tsv::schema::infer::Config,
    schema: &tsv::schema::FileSchema,
    path_in_tsv: &str,
) -> Result<(), anyhow::Error> {
    // Load tabix header and create BGZF reader with tabix index.
    let tabix_src = format!("{}.tbi", path_in_tsv);
    let index = noodles_tabix::read(tabix_src)?;
    let header = index.header().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "missing tabix header")
    })?;

    // Generate list of regions on canonical chromosomes, limited to those present in tbi index.
    let regions =
        common::cli::build_genome_windows(args.genome_release.into(), Some(args.tbi_window_size))?
            .into_iter()
            .filter(|(chrom, _, _)| {
                header
                    .reference_sequence_names()
                    .get_index_of(chrom)
                    .is_some()
            })
            .map(|(chrom, begin, end)| {
                let start = noodles_core::Position::try_from(begin + 1)
                    .expect("could not convert to position");
                let stop = noodles_core::Position::try_from(std::cmp::max(begin + 1, end))
                    .expect("could not convert to position");
                let region = noodles_core::Region::new(chrom, start..=stop);
                let tid = resolve_region(header, &region)
                    .unwrap_or_else(|e| panic!("could not resolve region {:?}: {}", region, e));
                (tid, region)
            })
            .collect::<Vec<_>>();

    // Import each region in parallel.
    tracing::info!("  importing TBI-parallel: {}", path_in_tsv);
    regions
        .par_iter()
        .progress_with_style(common::cli::indicatif_style())
        .map(|region| tsv_import_window(db, args, config, schema, path_in_tsv, region))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}
