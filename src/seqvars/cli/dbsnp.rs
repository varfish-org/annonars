//! Import of dbSNP RS identifiers (VCF) into a RocksDB track database.

use clap::Parser;
use itertools::Itertools as _;
use noodles::vcf::variant::record::AlternateBases as _;
use noodles::vcf::variant::RecordBuf;
use prost::Message;

use crate::common;
use crate::common::contig::ContigDict;
use crate::common::keys::Var;
use crate::pbs::seqvars::base::DbsnpRecord;

/// Command line arguments for `seqvars dbsnp` (build a dbSNP track database).
#[derive(Parser, Debug, Clone)]
#[command(about = "Construct dbSNP RocksDB database", long_about = None)]
pub struct Args {
    /// Path to the input dbSNP VCF file (plain or bgzip-compressed).
    #[arg(long)]
    pub path_in_vcf: String,
    /// Path to the reference FASTA index (`.fai`) defining the contig dictionary.
    #[arg(long)]
    pub path_reference_fai: String,
    /// Assembly / genome-release label to record in the meta CF.
    #[arg(long)]
    pub assembly: String,
    /// Path to the output RocksDB directory.
    #[arg(long)]
    pub path_out_rocksdb: String,
    /// Name of the data column family.
    #[arg(long, default_value = "dbsnp")]
    pub cf_name: String,
    /// Optional path to a RocksDB WAL directory.
    #[arg(long)]
    pub path_wal_dir: Option<String>,
}

/// Main entry point for `seqvars dbsnp`.
pub fn run(_common: &common::cli::Args, args: &Args) -> Result<(), anyhow::Error> {
    tracing::info!("Building dbSNP track database");
    let dict = ContigDict::from_fai(&args.path_reference_fai)?;

    let (db, cf_names) = crate::seqvars::open_track_db_for_write(
        &args.path_out_rocksdb,
        &args.cf_name,
        args.path_wal_dir.as_deref(),
    )?;
    crate::seqvars::write_track_meta(&db, "dbsnp", "1.0", &args.assembly, &dict)?;
    let cf_data = db
        .cf_handle(&args.cf_name)
        .ok_or_else(|| anyhow::anyhow!("data column family {} missing", &args.cf_name))?;

    let mut reader = noodles::vcf::io::reader::Builder::default()
        .build_from_path(&args.path_in_vcf)
        .map_err(|e| anyhow::anyhow!("failed to open {}: {}", &args.path_in_vcf, e))?;
    let header = reader.read_header()?;

    let mut count: u64 = 0;
    for result in reader.record_bufs(&header) {
        let record: RecordBuf = result?;

        let rs_id = record.ids().as_ref().iter().join(";");
        if rs_id.is_empty() {
            continue;
        }
        let chrom = record.reference_sequence_name().to_string();
        let Some(chrom_id) = dict.id_of(&chrom) else {
            anyhow::bail!(
                "contig {:?} from input is not in the reference dictionary",
                chrom
            );
        };
        let Some(pos) = record.variant_start() else {
            continue;
        };
        let pos = i32::try_from(pos.get())?;
        let reference = record.reference_bases().to_string();

        for allele_no in 0..record.alternate_bases().len() {
            let alt = record.alternate_bases().as_ref()[allele_no].to_string();
            let var = Var::new(chrom.clone(), pos, reference.clone(), alt.clone());
            let key = var.encode_with_id(chrom_id);
            let value = DbsnpRecord {
                allele: alt,
                rs_id: rs_id.clone(),
            }
            .encode_to_vec();
            db.put_cf(&cf_data, key, value)?;
            count += 1;
        }
    }
    tracing::info!("  wrote {} dbSNP records", count);

    let cf_refs = cf_names.iter().map(String::as_str).collect::<Vec<_>>();
    rocksdb_utils_lookup::force_compaction_cf(&db, &cf_refs, Some("  "), true)?;

    tracing::info!("All done. Have a nice day!");
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use clap_verbosity_flag::Verbosity;
    use temp_testdir::TempDir;

    #[test]
    fn smoke_import_dbsnp_and_read_back() {
        let tmp = TempDir::default();
        let fai = tmp.join("ref.fa.fai");
        let vcf = tmp.join("dbsnp.vcf");
        let out = tmp.join("out-rocksdb");
        std::fs::write(&fai, "1\t249250621\n").unwrap();
        std::fs::write(
            &vcf,
            "##fileformat=VCFv4.2\n\
             ##contig=<ID=1>\n\
             #CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\n\
             1\t100\trs123\tA\tT\t.\t.\t.\n\
             1\t200\t.\tC\tG\t.\t.\t.\n",
        )
        .unwrap();

        let common = common::cli::Args {
            verbose: Verbosity::new(0, 0),
        };
        let args = Args {
            path_in_vcf: format!("{}", vcf.display()),
            path_reference_fai: format!("{}", fai.display()),
            assembly: "GRCh37".to_string(),
            path_out_rocksdb: format!("{}", out.display()),
            cf_name: "dbsnp".to_string(),
            path_wal_dir: None,
        };
        run(&common, &args).unwrap();

        let db = rocksdb::DB::open_cf_for_read_only(
            &rocksdb::Options::default(),
            &out,
            ["meta", "dbsnp"],
            false,
        )
        .unwrap();
        let dict = crate::seqvars::read_contig_dict(&db).unwrap();
        let cf_data = db.cf_handle("dbsnp").unwrap();

        // The record with an RS id is present ...
        let key = Var::from("1", 100, "A", "T").encode_with_id(dict.id_of("1").unwrap());
        let raw = db.get_cf(&cf_data, key).unwrap().expect("record present");
        let rec = DbsnpRecord::decode(&raw[..]).unwrap();
        assert_eq!(rec.rs_id, "rs123");
        assert_eq!(rec.allele, "T");

        // ... the record without an RS id is skipped.
        let key_missing = Var::from("1", 200, "C", "G").encode_with_id(dict.id_of("1").unwrap());
        assert!(db.get_cf(&cf_data, key_missing).unwrap().is_none());
    }
}
