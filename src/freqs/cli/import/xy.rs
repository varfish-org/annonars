//! Import of gonosomal variant frequencies.

use crate::{common, freqs};

/// Write out the given record to the database.
fn write_record(
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    cf: &std::sync::Arc<rocksdb::BoundColumnFamily>,
    record_key: &common::keys::Var,
    record_genome: &mut Option<noodles::vcf::variant::RecordBuf>,
    record_exome: &mut Option<noodles::vcf::variant::RecordBuf>,
) -> Result<(), anyhow::Error> {
    if record_genome.is_none() && record_exome.is_none() {
        // Early exit, nothing to write out.
        return Ok(());
    }

    let count_genomes = if let Some(record_genome) = record_genome {
        freqs::serialized::xy::Counts::from_vcf_allele(record_genome, 0)
    } else {
        freqs::serialized::xy::Counts::default()
    };
    let counts_exomes = if let Some(record_exome) = record_exome {
        freqs::serialized::xy::Counts::from_vcf_allele(record_exome, 0)
    } else {
        freqs::serialized::xy::Counts::default()
    };

    let gono_record = freqs::serialized::xy::Record {
        gnomad_genomes: count_genomes,
        gnomad_exomes: counts_exomes,
    };

    let mut buf = vec![0u8; freqs::serialized::xy::Record::buf_len()];
    gono_record.to_buf(&mut buf);
    let key: Vec<u8> = record_key.clone().into();

    // tracing::info!("  key = {:?}, record = {:?}", &record_key, &gono_record);

    db.put_cf(cf, key, &buf)?;

    Ok(())
}

/// Import of gonosomal variant frequencies.
pub fn import_region(
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    path_genome: Option<&String>,
    path_exome: Option<&String>,
    region: &noodles::core::region::Region,
) -> Result<(), anyhow::Error> {
    // Get handle to "gonosomal" column family.
    let cf_gono = db.cf_handle("gonosomal").unwrap();
    // Build `Vec` of readers and by-index map that tells whether it is genomes.
    let mut is_genome = Vec::new();
    let mut readers = Vec::new();
    if let Some(path_genome) = path_genome {
        is_genome.push(true);
        readers.push(
            noodles::vcf::io::indexed_reader::Builder::default().build_from_path(path_genome)?,
        );
    }
    if let Some(path_exome) = path_exome {
        is_genome.push(false);
        readers.push(
            noodles::vcf::io::indexed_reader::Builder::default().build_from_path(path_exome)?,
        );
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
    let multi_query = super::reading::MultiQuery::new(queries, &headers)?;

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
        // Obtain the key of the next record.
        let curr_key = common::keys::Var::from_vcf_allele(&record, 0);

        // Write out current records to database if we advance.
        if record_key.as_ref() != Some(&curr_key) {
            if let Some(record_key) = record_key.as_ref() {
                write_record(
                    db,
                    &cf_gono,
                    record_key,
                    &mut record_genome,
                    &mut record_exome,
                )?;
            }
            record_genome = None;
            record_exome = None;
        }

        // Shortcut to whether this from gnomAD genomes.
        if is_genome[idx] {
            record_genome = Some(record);
        } else {
            record_exome = Some(record);
        }

        // Updating the current key cannot cause harm.
        record_key = Some(curr_key);
    }

    // Write final records to database.
    if let Some(record_key) = record_key.as_ref() {
        write_record(
            db,
            &cf_gono,
            record_key,
            &mut record_genome,
            &mut record_exome,
        )?;
    }

    Ok(())
}
