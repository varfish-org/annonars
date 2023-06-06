//! Import of mitochondrial variant frequencies.

use crate::{common, freqs};

/// Write out the given record to the database.
fn write_record(
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    cf: &std::sync::Arc<rocksdb::BoundColumnFamily>,
    record_key: &common::keys::Var,
    record_gnomad: &mut Option<noodles_vcf::Record>,
    record_helix: &mut Option<noodles_vcf::Record>,
) -> Result<(), anyhow::Error> {
    let count_gnomad = if let Some(record_gnomad) = record_gnomad {
        freqs::serialized::mt::Counts::from_vcf_allele(record_gnomad, 0)
    } else {
        freqs::serialized::mt::Counts::default()
    };
    let count_helix = if let Some(record_helix) = record_helix {
        freqs::serialized::mt::Counts::from_vcf_allele(record_helix, 0)
    } else {
        freqs::serialized::mt::Counts::default()
    };

    let mito_record = freqs::serialized::mt::Record {
        gnomad_mtdna: count_gnomad,
        helixmtdb: count_helix,
    };

    let mut buf = vec![0u8; freqs::serialized::mt::Record::buf_len()];
    mito_record.to_buf(&mut buf);
    let key: Vec<u8> = record_key.clone().into();

    db.put_cf(cf, key, &buf)?;

    Ok(())
}

/// Import of mitochondrial variant frequencies.
pub fn import_region(
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    path_gnomad: Option<&String>,
    path_helix: Option<&String>,
    region: &noodles_core::region::Region,
) -> Result<(), anyhow::Error> {
    // Get handle to "mitochondrial" column family.
    let cf_mt = db.cf_handle("mitochondrial").unwrap();
    // Build `Vec` of readers and by-index map that tells whether it is genomes.
    let mut is_gnomad = Vec::new();
    let mut readers = Vec::new();
    let mut paths = Vec::new();
    if let Some(path_gnomad) = path_gnomad {
        is_gnomad.push(true);
        paths.push(path_gnomad);
        readers.push(noodles_vcf::indexed_reader::Builder::default().build_from_path(path_gnomad)?);
    }
    if let Some(path_helix) = path_helix {
        is_gnomad.push(false);
        paths.push(path_helix);
        readers.push(noodles_vcf::indexed_reader::Builder::default().build_from_path(path_helix)?);
    }
    // Read headers.
    let headers: Vec<_> = readers
        .iter_mut()
        .map(|reader| reader.read_header())
        .collect::<Result<_, _>>()?;

    // Seek to region obtaining `queries`.
    let queries: Vec<_> = readers
        .iter_mut()
        .zip(&headers)
        .filter_map(|(reader, header)| {
            match reader.query(header, region) {
                Ok(result) => {
                    Ok(Some(result))
                },
                Err(e) => {
                    let needle = "region reference sequence does not exist in reference sequences";
                    if e.to_string().contains(needle) {
                        Ok(None)
                    } else {
                        Err(e)
                    }
                }
            }
            .transpose()
        })
        .collect::<Result<_, _>>()?;
    // Construct the `MultiQuery`.
    let multi_query = super::reading::MultiQuery::new(queries)?;

    // Now iterate over the `MultiQuery` and write to the database.
    //
    // The key of the records stored in `record_{genome,exome}`.
    let mut record_key = None;
    // Record from gnomAD genomes (same position as record_helix, if either).
    let mut record_gnomad = None;
    // Record from gnomAD exomes (same position as record_gnomad, if either).
    let mut record_helix = None;
    for result in multi_query {
        let (idx, record) = result?;
        // Shortcut to whether this from gnomAD genomes.
        let is_gnomad = is_gnomad[idx];
        // Obtain the key of the next record.
        let curr_key = common::keys::Var::from_vcf_allele(&record, 0);

        // Act on the current record.
        match (
            record_gnomad.as_mut(),
            record_helix.as_mut(),
            is_gnomad,
            record_key.as_ref() == Some(&curr_key),
        ) {
            (None, None, _, true) => panic!("case cannot happen"),
            (None, None, true, false) => {
                // New key (=variant) at genome, no previous record.  Just update `record_gnomad`.
                record_gnomad = Some(record);
            }
            (None, None, false, false) => {
                // New key (=variant) at exome, no previous record.  Just update `record_helix`.
                record_helix = Some(record);
            }
            (None, Some(_), true, true) => {
                // Existing key (=variant) at genome, genome is unset.  Just update `record_gnomad`.
                record_gnomad = Some(record);
            }
            (None, Some(_), true, false) => {
                // New key (=variant) at genome, have previous exome record.  Write out records,
                // clear `record_helix`, and set `record_gnomad`.

                // Write out current records to database.
                if let Some(record_key) = record_key.as_ref() {
                    write_record(
                        db,
                        &cf_mt,
                        record_key,
                        &mut record_gnomad,
                        &mut record_helix,
                    )?;
                }

                // Update current records.
                record_gnomad = Some(record);
                record_helix = None;
            }
            (None, Some(_), false, true) => {
                // Existing key (=variant) at exome, exome is already set.  We found a duplicate
                // record in exome which is an error.
                anyhow::bail!("Duplicate record in exomes at {:?}", &record);
            }
            (None, Some(_), false, false) => {
                // New key (=variant) at exome, have previous exome record.  Write out records,
                // and set `record_helix`.

                // Update current records.
                record_helix = Some(record);
            }
            (Some(_), None, true, true) => {
                // Existing key (=variant) at genome, genome is already set.  We found a duplicate.
                anyhow::bail!("Duplicate record in genomes at {:?}", &record);
            }
            (Some(_), None, true, false) => {
                // New key (=variant) at genome, have previous genome record.  Write out records,
                // and set `record_gnomad`.

                // Write out current records to database.
                if let Some(record_key) = record_key.as_ref() {
                    write_record(
                        db,
                        &cf_mt,
                        record_key,
                        &mut record_gnomad,
                        &mut record_helix,
                    )?;
                }

                // Update current records.
                record_gnomad = Some(record);
            }
            (Some(_), None, false, true) => {
                // Existing key (=variant) at exome, exome is unset.  Just update `record_helix`.

                // Update current records.
                record_helix = Some(record);
            }
            (Some(_), None, false, false) => {
                // New key (=variant) at exome, have previous genome record.  Write out records,
                // clear `record_gnomad`, and set `record_helix`.

                // Write out current records to database.
                if let Some(record_key) = record_key.as_ref() {
                    write_record(
                        db,
                        &cf_mt,
                        record_key,
                        &mut record_gnomad,
                        &mut record_helix,
                    )?;
                }

                // Update current records.
                record_gnomad = None;
                record_helix = Some(record);
            }
            (Some(_), Some(_), true, true) => {
                // Existing key (=variant) at genome, genome is already set.  We found a duplicate.
                anyhow::bail!("Duplicate record in genomes at {:?}", &record);
            }
            (Some(_), Some(_), true, false) => {
                // New key (=variant) at genome, have both exome and genome record.  Write out
                // both records, set `record_gnomad`, clear `record_helix`.

                // Write out current records to database.
                if let Some(record_key) = record_key.as_ref() {
                    write_record(
                        db,
                        &cf_mt,
                        record_key,
                        &mut record_gnomad,
                        &mut record_helix,
                    )?;
                }

                // Update current records.
                record_gnomad = Some(record);
                record_helix = None;
            }
            (Some(_), Some(_), false, true) => {
                // Existing key (=variant) at exome, exome is already set.  We found a duplicate.
                anyhow::bail!("Duplicate record in exomes at {:?}", &record);
            }
            (Some(_), Some(_), false, false) => {
                // Write out current records to database.
                if let Some(record_key) = record_key.as_ref() {
                    write_record(
                        db,
                        &cf_mt,
                        record_key,
                        &mut record_gnomad,
                        &mut record_helix,
                    )?;
                }

                // Update current records.
                record_gnomad = None;
                record_helix = Some(record);
            }
        }

        // Update record key when necessary.
        if record_key.as_ref() != Some(&curr_key) {
            record_key = Some(curr_key);
        }
    }

    // Write final records to database.
    if let Some(record_key) = record_key.as_ref() {
        write_record(
            db,
            &cf_mt,
            record_key,
            &mut record_gnomad,
            &mut record_helix,
        )?;
    }

    Ok(())
}
