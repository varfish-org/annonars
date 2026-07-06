//! Import of SpliceAI predictions (VCF INFO/SpliceAI) into a RocksDB track database.

use std::collections::HashMap;

use clap::Parser;
use noodles::vcf::variant::record::AlternateBases as _;
use noodles::vcf::variant::record_buf::info::field::{value::Array, Value};
use noodles::vcf::variant::RecordBuf;
use prost::Message;

use crate::common;
use crate::common::contig::ContigDict;
use crate::common::keys::Var;
use crate::pbs::seqvars::base::{SpliceAiPrediction, SpliceAiRecord};

/// Command line arguments for `seqvars spliceai` (build a SpliceAI track database).
#[derive(Parser, Debug, Clone)]
#[command(about = "Construct SpliceAI score RocksDB database", long_about = None)]
pub struct Args {
    /// Path to the input SpliceAI VCF file (plain or bgzip-compressed).
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
    #[arg(long, default_value = "spliceai")]
    pub cf_name: String,
    /// Optional path to a RocksDB WAL directory.
    #[arg(long)]
    pub path_wal_dir: Option<String>,
}

/// Extract an INFO field value as a string (joining array entries with `,`).
fn info_string(val: &Value) -> Option<String> {
    match val {
        Value::String(s) => Some(s.to_string()),
        Value::Array(Array::String(arr)) => {
            Some(arr.iter().flatten().cloned().collect::<Vec<_>>().join(","))
        }
        _ => None,
    }
}

/// Main entry point for `seqvars spliceai`.
pub fn run(_common: &common::cli::Args, args: &Args) -> Result<(), anyhow::Error> {
    tracing::info!("Building SpliceAI track database");
    let dict = ContigDict::from_fai(&args.path_reference_fai)?;

    let (db, cf_names) = crate::seqvars::open_track_db_for_write(
        &args.path_out_rocksdb,
        &args.cf_name,
        args.path_wal_dir.as_deref(),
    )?;
    crate::seqvars::write_track_meta(&db, "spliceai", "1.0", &args.assembly, &dict)?;
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

        let Some(spliceai) = record
            .info()
            .get("SpliceAI")
            .flatten()
            .and_then(info_string)
        else {
            continue;
        };
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

        let mut by_allele: HashMap<String, Vec<SpliceAiPrediction>> = HashMap::new();
        for block in spliceai.split(',') {
            let f: Vec<&str> = block.split('|').collect();
            if f.len() < 10 {
                anyhow::bail!(
                    "malformed SpliceAI block (expected >=10 fields): {:?}",
                    block
                );
            }
            let allele = f[0].to_string();
            by_allele
                .entry(allele.clone())
                .or_default()
                .push(SpliceAiPrediction {
                    allele,
                    symbol: f[1].to_string(),
                    ds_ag: f[2].parse()?,
                    ds_al: f[3].parse()?,
                    ds_dg: f[4].parse()?,
                    ds_dl: f[5].parse()?,
                    dp_ag: f[6].parse()?,
                    dp_al: f[7].parse()?,
                    dp_dg: f[8].parse()?,
                    dp_dl: f[9].parse()?,
                });
        }

        // Keep only alleles that are actually present in the record's ALTs.
        let alts: Vec<String> = (0..record.alternate_bases().len())
            .map(|i| record.alternate_bases().as_ref()[i].to_string())
            .collect();
        for (allele, predictions) in by_allele {
            if !alts.contains(&allele) {
                continue;
            }
            let var = Var::new(chrom.clone(), pos, reference.clone(), allele);
            let key = var.encode_with_id(chrom_id);
            let value = SpliceAiRecord { predictions }.encode_to_vec();
            db.put_cf(&cf_data, key, value)?;
            count += 1;
        }
    }
    tracing::info!("  wrote {} SpliceAI records", count);

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
    fn smoke_import_spliceai_and_read_back() {
        let tmp = TempDir::default();
        let fai = tmp.join("ref.fa.fai");
        let vcf = tmp.join("spliceai.vcf");
        let out = tmp.join("out-rocksdb");
        std::fs::write(&fai, "1\t249250621\n").unwrap();
        std::fs::write(
            &vcf,
            "##fileformat=VCFv4.2\n\
             ##contig=<ID=1>\n\
             ##INFO=<ID=SpliceAI,Number=.,Type=String,Description=\"SpliceAI\">\n\
             #CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\n\
             1\t100\t.\tA\tT\t.\t.\tSpliceAI=T|BRCA1|0.01|0.02|0.03|0.04|10|20|30|40\n",
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
            cf_name: "spliceai".to_string(),
            path_wal_dir: None,
        };
        run(&common, &args).unwrap();

        let db = rocksdb::DB::open_cf_for_read_only(
            &rocksdb::Options::default(),
            &out,
            ["meta", "spliceai"],
            false,
        )
        .unwrap();
        let dict = crate::seqvars::read_contig_dict(&db).unwrap();
        let cf_data = db.cf_handle("spliceai").unwrap();

        let key = Var::from("1", 100, "A", "T").encode_with_id(dict.id_of("1").unwrap());
        let raw = db.get_cf(&cf_data, key).unwrap().expect("record present");
        let rec = SpliceAiRecord::decode(&raw[..]).unwrap();
        assert_eq!(rec.predictions.len(), 1);
        assert_eq!(rec.predictions[0].symbol, "BRCA1");
        assert!((rec.predictions[0].ds_ag - 0.01).abs() < 1e-6);
        assert_eq!(rec.predictions[0].dp_dl, 40);
    }
}
