//! Import of autosomal variant frequencies.

use crate::{common, freqs};

/// Write out the given record to the database.
fn write_record(
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    cf: &std::sync::Arc<rocksdb::BoundColumnFamily>,
    record_key: &common::keys::Var,
    record_genome: &mut Option<noodles_vcf::Record>,
    record_exome: &mut Option<noodles_vcf::Record>,
) -> Result<(), anyhow::Error> {
    let count_genomes = if let Some(record_genome) = record_genome {
        freqs::serialized::auto::Counts::from_vcf_allele(record_genome, 0)
    } else {
        freqs::serialized::auto::Counts::default()
    };
    let counts_exomes = if let Some(record_exome) = record_exome {
        freqs::serialized::auto::Counts::from_vcf_allele(record_exome, 0)
    } else {
        freqs::serialized::auto::Counts::default()
    };

    let auto_record = freqs::serialized::auto::Record {
        gnomad_genomes: count_genomes,
        gnomad_exomes: counts_exomes,
    };

    let mut buf = vec![0u8; freqs::serialized::auto::Record::buf_len()];
    auto_record.to_buf(&mut buf);
    let key: Vec<u8> = record_key.clone().into();

    db.put_cf(cf, &key, &buf)?;

    Ok(())
}

/// Import of autosomal variant frequencies.
pub fn import_region(
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    path_genome: Option<&String>,
    path_exome: Option<&String>,
    region: &noodles_core::region::Region,
) -> Result<(), anyhow::Error> {
    // Get handle to "autosomal" column family.
    let cf_auto = db.cf_handle("autosomal").unwrap();
    // Build `Vec` of readers and by-index map that tells whether it is genomes.
    let mut is_genome = Vec::new();
    let mut readers = Vec::new();
    if let Some(path_genome) = path_genome {
        is_genome.push(true);
        readers.push(noodles_vcf::indexed_reader::Builder::default().build_from_path(path_genome)?);
    }
    if let Some(path_exome) = path_exome {
        is_genome.push(false);
        readers.push(noodles_vcf::indexed_reader::Builder::default().build_from_path(path_exome)?);
    }
    // Read headers.
    let headers: Vec<_> = readers
        .iter_mut()
        .map(|reader| reader.read_header())
        .collect::<Result<_, _>>()?;

    // Seek to region obtaining `ueries`.
    let queries: Vec<_> = readers
        .iter_mut()
        .zip(&headers)
        .map(|(reader, header)| reader.query(header, region))
        .collect::<Result<_, _>>()?;
    // Construct the `MultiQuery`.
    let multi_query = super::reading::MultiQuery::new(queries)?;

    // Now iterate over the `MultiQuery` and write to the database.
    //
    // The key of the records stored in `record_{genome,exome}`.
    let mut record_key = None;
    // Record from gnomAD genomes (same position as record_exome, if either).
    let mut record_genome = None;
    // Record from gnomAD exomes (same position as record_genome, if either).
    let mut record_exome = None;
    for result in multi_query {
        let (idx, record) = result?;
        // Shortcut to whether this from gnomAD genomes.
        let is_genome = is_genome[idx];
        // Obtain the key of the next record.
        let curr_key = common::keys::Var::from_vcf_allele(&record, 0);

        // Act on the current record.
        match (
            record_genome.as_mut(),
            record_exome.as_mut(),
            is_genome,
            record_key.as_ref() == Some(&curr_key),
        ) {
            (None, None, _, true) => panic!("case cannot happen"),
            (None, None, true, false) => {
                // New key (=variant) at genome, no previous record.  Just update `record_genome`.
                record_genome = Some(record);
            }
            (None, None, false, false) => {
                // New key (=variant) at exome, no previous record.  Just update `record_exome`.
                record_exome = Some(record);
            }
            (None, Some(_), true, true) => {
                // Existing key (=variant) at genome, genome is unset.  Just update `record_genome`.
                record_genome = Some(record);
            }
            (None, Some(_), true, false) => {
                // New key (=variant) at genome, have previous exome record.  Write out records,
                // clear `record_exome`, and set `record_genome`.

                // Write out current records to database.
                if let Some(record_key) = record_key.as_ref() {
                    write_record(
                        &db,
                        &cf_auto,
                        record_key,
                        &mut record_genome,
                        &mut record_exome,
                    )?;
                }

                // Update current records.
                record_genome = Some(record);
                record_exome = None;
            }
            (None, Some(_), false, true) => {
                // Existing key (=variant) at exome, exome is already set.  We found a duplicate
                // record in exome which is an error.
                anyhow::bail!("Duplicate record in exomes at {:?}", &record);
            }
            (None, Some(_), false, false) => {
                // New key (=variant) at exome, have previous exome record.  Write out records,
                // and set `record_exome`.

                // Update current records.
                record_exome = Some(record);
            }
            (Some(_), None, true, true) => {
                // Existing key (=variant) at genome, genome is already set.  We found a duplicate.
                anyhow::bail!("Duplicate record in genomes at {:?}", &record);
            }
            (Some(_), None, true, false) => {
                // New key (=variant) at genome, have previous genome record.  Write out records,
                // and set `record_genome`.

                // Write out current records to database.
                if let Some(record_key) = record_key.as_ref() {
                    write_record(
                        &db,
                        &cf_auto,
                        record_key,
                        &mut record_genome,
                        &mut record_exome,
                    )?;
                }

                // Update current records.
                record_genome = Some(record);
            }
            (Some(_), None, false, true) => {
                // Existing key (=variant) at exome, exome is unset.  Just update `record_exome`.

                // Update current records.
                record_exome = Some(record);
            }
            (Some(_), None, false, false) => {
                // New key (=variant) at exome, have previous genome record.  Write out records,
                // clear `record_genome`, and set `record_exome`.

                // Write out current records to database.
                if let Some(record_key) = record_key.as_ref() {
                    write_record(
                        &db,
                        &cf_auto,
                        record_key,
                        &mut record_genome,
                        &mut record_exome,
                    )?;
                }

                // Update current records.
                record_genome = None;
                record_exome = Some(record);
            }
            (Some(_), Some(_), true, true) => {
                // Existing key (=variant) at genome, genome is already set.  We found a duplicate.
                anyhow::bail!("Duplicate record in genomes at {:?}", &record);
            }
            (Some(_), Some(_), true, false) => {
                // New key (=variant) at genome, have both exome and genome record.  Write out
                // both records, set `record_genome`, clear `record_exome`.

                // Write out current records to database.
                if let Some(record_key) = record_key.as_ref() {
                    write_record(
                        &db,
                        &cf_auto,
                        record_key,
                        &mut record_genome,
                        &mut record_exome,
                    )?;
                }

                // Update current records.
                record_genome = Some(record);
                record_exome = None;
            }
            (Some(_), Some(_), false, true) => {
                // Existing key (=variant) at exome, exome is already set.  We found a duplicate.
                anyhow::bail!("Duplicate record in exomes at {:?}", &record);
            }
            (Some(_), Some(_), false, false) => {
                // Write out current records to database.
                if let Some(record_key) = record_key.as_ref() {
                    write_record(
                        &db,
                        &cf_auto,
                        record_key,
                        &mut record_genome,
                        &mut record_exome,
                    )?;
                }

                // Update current records.
                record_genome = None;
                record_exome = Some(record);
            }
        }

        // Update record key when necessary.
        if record_key.as_ref() != Some(&curr_key) {
            record_key = Some(curr_key);
        }
    }

    Ok(())
}
