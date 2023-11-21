//! Import of genomic region data.

use std::sync::Arc;

use clap::Parser;
use prost::Message;

use crate::{common, genes::cli::data::clingen_gene};

/// Helper data structures for reading CSV files.
pub mod clingen {
    use std::{
        collections::HashMap,
        io::{BufRead as _, BufReader},
    };

    /// `ClinGen` region dosage sensitivity entry.
    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct Region {
        /// ISCA ID
        #[serde(alias = "#ISCA ID")]
        pub isca_id: String,
        /// ISCA Region Name
        #[serde(alias = "ISCA Region Name")]
        pub isca_region_name: String,
        /// Genomic location.
        #[serde(alias = "Genomic Location")]
        pub genomic_location: String,
        /// Haploinsufficiency score.
        #[serde(alias = "Haploinsufficiency Score", deserialize_with = "parse_score")]
        pub haploinsufficiency_score: Option<u32>,
        /// Triplosensitivity score.
        #[serde(alias = "Triplosensitivity Score", deserialize_with = "parse_score")]
        pub triplosensitivity_score: Option<u32>,
        /// Haploinsufficiency Disease ID.
        #[serde(alias = "Haploinsufficiency Disease ID")]
        pub haploinsufficiency_disease_id: Option<String>,
        /// Haploinsufficiency Disease ID.
        #[serde(alias = "Triplosensitivity Disease ID")]
        pub triplosensitivity_disease_id: Option<String>,
    }

    /// Helper for parsing the scores which may have interesting values.
    fn parse_score<'de, D>(d: D) -> Result<Option<u32>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let tmp: String = serde::Deserialize::deserialize(d)?;
        if tmp.is_empty() || tmp == "Not yet evaluated" || tmp == "-1" {
            Ok(None)
        } else {
            Ok(Some(tmp.parse().map_err(serde::de::Error::custom)?))
        }
    }

    /// Load ClinGen region CSV file.
    ///
    /// # Result
    ///
    /// A map from region ID to region.
    pub fn load_clingen(path: &str) -> Result<HashMap<String, Region>, anyhow::Error> {
        tracing::info!("  loading ClinGen region curations from {}", path);
        let mut result = HashMap::new();

        // Construct reader and skip initial 5 lines.
        let mut reader = std::fs::File::open(path)
            .map_err(|e| anyhow::anyhow!("problem opening file: {}", e))
            .map(BufReader::new)?;

        {
            let mut buf = String::new();
            for _ in 0..5 {
                reader.read_line(&mut buf)?;
                buf.clear();
            }
        }

        let mut csv_reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(true)
            .flexible(true)
            .from_reader(reader);
        for record in csv_reader.deserialize() {
            let record: Region =
                record.map_err(|e| anyhow::anyhow!("problem parsing record: {}", e))?;
            if genomic_location_to_interval(&record.genomic_location).is_err() {
                tracing::warn!(
                    "skipping region with invalid genomic location: {}",
                    record.genomic_location
                );
                continue;
            }
            result.insert(record.isca_id.clone(), record);
        }

        Ok(result)
    }

    /// Convert genomic location string to `bio::bio_types::genome::Interval`.
    pub fn genomic_location_to_interval(
        genomic_location: &str,
    ) -> Result<bio::bio_types::genome::Interval, anyhow::Error> {
        let mut parts = genomic_location.split(':');
        let chrom = parts.next().ok_or_else(|| {
            anyhow::anyhow!(
                "could not parse chromosome from genomic location: {}",
                genomic_location
            )
        })?;
        let mut parts = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("could not parse region {}", genomic_location))?
            .split('-');
        let begin = parts
            .next()
            .unwrap()
            .parse::<u64>()
            .map_err(|e| anyhow::anyhow!("could not parse start position from: {}", e))?
            .saturating_sub(1);
        let end = parts
            .next()
            .unwrap()
            .parse::<u64>()
            .map_err(|e| anyhow::anyhow!("could not parse end position from: {}", e))?;
        Ok(bio::bio_types::genome::Interval::new(
            chrom.to_string(),
            begin..end,
        ))
    }
}

/// Command line arguments for `regions import` sub command.
#[derive(Parser, Debug, Clone)]
#[command(about = "import region annotation data", long_about = None)]
pub struct Args {
    /// Genome build to use in the build.
    #[arg(long, value_enum)]
    pub genome_release: common::cli::GenomeRelease,
    /// Path to ClinGen region annotation file.
    #[arg(long, required = true)]
    pub path_in_clingen: String,
    /// Path to output RocksDB directory.
    #[arg(long)]
    pub path_out_rocksdb: String,

    /// Name of the column family to import into.
    #[arg(long, default_value = "regions")]
    pub cf_name: String,
    /// Optional path to RocksDB WAL directory.
    #[arg(long)]
    pub path_wal_dir: Option<String>,
}

/// Perform import of the TSV file.
fn tsv_import(
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    args: &Args,
) -> Result<(), anyhow::Error> {
    let cf_data = db.cf_handle(&args.cf_name).unwrap();

    let regions = clingen::load_clingen(&args.path_in_clingen)?;

    for (_, region) in regions {
        let clingen::Region {
            isca_id,
            isca_region_name,
            genomic_location,
            haploinsufficiency_score,
            triplosensitivity_score,
            haploinsufficiency_disease_id,
            triplosensitivity_disease_id,
        } = region;
        let haploinsufficiency_score = clingen_gene::Score::try_from(haploinsufficiency_score)
            .map_err(|e| anyhow::anyhow!("problem parsing haplosensitivity score: {}", e))?;
        let triplosensitivity_score = clingen_gene::Score::try_from(triplosensitivity_score)
            .map_err(|e| anyhow::anyhow!("problem parsing triplosensitivity score: {}", e))?;
        let region = crate::pbs::regions::clingen::Region {
            isca_id,
            isca_region_name,
            genomic_location,
            haploinsufficiency_score: haploinsufficiency_score as i32,
            triplosensitivity_score: triplosensitivity_score as i32,
            haploinsufficiency_disease_id,
            triplosensitivity_disease_id,
        };
        let key = format!("clingen:{}", &region.isca_id);
        db.put_cf(&cf_data, key.as_bytes(), region.encode_to_vec())?;
    }

    Ok(())
}

/// Implementation of `cons import` sub command.
pub fn run(common: &common::cli::Args, args: &Args) -> Result<(), anyhow::Error> {
    tracing::info!("Starting 'regions import' command");
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
    db.put_cf(&cf_meta, "db-name", "regions")?;
    tracing::info!(
        "... done opening RocksDB for writing in {:?}",
        before_opening_rocksdb.elapsed()
    );

    tracing::info!("Importing TSV files ...");
    let before_import = std::time::Instant::now();
    tsv_import(&db, args)?;
    tracing::info!(
        "... done importing TSV files in {:?}",
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
    fn smoke_test_import() {
        let tmp_dir = TempDir::default();
        let common = common::cli::Args {
            verbose: Verbosity::new(1, 0),
        };
        let args = Args {
            genome_release: common::cli::GenomeRelease::Grch37,
            path_in_clingen: String::from(
                "tests/regions/clingen/ClinGen_region_curation_list_GRCh37.tsv",
            ),
            path_out_rocksdb: format!("{}", tmp_dir.join("out-rocksdb").display()),
            cf_name: String::from("regions"),
            path_wal_dir: None,
        };

        run(&common, &args).unwrap();
    }
}
