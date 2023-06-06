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
    if record_gnomad.is_none() && record_helix.is_none() {
        // Early exit, nothing to write out.
        return Ok(());
    }

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
    // dbg!(&record_key, &mito_record);

    let mut buf = vec![0u8; freqs::serialized::mt::Record::buf_len()];
    mito_record.to_buf(&mut buf);
    let key: Vec<u8> = record_key.clone().into();

    // tracing::info!("  key = {:?}, record = {:?}", &record_key, &mito_record);

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
    let cf_mito = db.cf_handle("mitochondrial").unwrap();
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
                Ok(result) => Ok(Some(result)),
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
    // Record from gnomAD genomes (same position as record_exome, if either).
    let mut record_gnomad = None;
    // Record from gnomAD exomes (same position as record_genome, if either).
    let mut record_helix = None;
    for result in multi_query {
        let (idx, record) = result?;
        // Obtain the key of the next record.
        let curr_key = common::keys::Var::from_vcf_allele(&record, 0);

        // Write out current records to database if we advance.
        if record_key.as_ref() != Some(&curr_key) {
            if let Some(record_key) = record_key.as_ref() {
                write_record(
                    db,
                    &cf_mito,
                    record_key,
                    &mut record_gnomad,
                    &mut record_helix,
                )?;
            }
            record_gnomad = None;
            record_helix = None;
        }

        // Shortcut to whether this from gnomAD genomes.
        if is_gnomad[idx] {
            record_gnomad = Some(record);
        } else {
            record_helix = Some(record);
        }

        // Updating the current key cannot cause harm.
        record_key = Some(curr_key);
    }

    // Write final records to database.
    if let Some(record_key) = record_key.as_ref() {
        write_record(
            db,
            &cf_mito,
            record_key,
            &mut record_gnomad,
            &mut record_helix,
        )?;
    }

    Ok(())
}
