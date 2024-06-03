//! Mitochondrial counts.

use byteorder::{ByteOrder, LittleEndian};
use noodles::vcf::variant::record::AlternateBases;

use crate::common;

/// Record type for storing AN, AC_hom, AC_het counts for chrMT.
#[derive(Default, Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Counts {
    /// Total number of alleles.
    pub an: u32,
    /// Number of homoplasmic alleles.
    pub ac_hom: u32,
    /// Number of heteroplasmic alleles.
    pub ac_het: u32,
}

impl Counts {
    /// Create from the given VCF record.
    pub fn from_vcf_allele(value: &noodles::vcf::variant::RecordBuf, _allele_no: usize) -> Self {
        assert_eq!(
            value.alternate_bases().len(),
            1,
            "only one alternate allele is supported",
        );
        let ac_hom = common::noodles::get_i32(value, "AC_hom").unwrap() as u32;
        let ac_het = common::noodles::get_i32(value, "AC_het").unwrap() as u32;
        let an = common::noodles::get_i32(value, "AN").unwrap() as u32;

        Counts { ac_hom, ac_het, an }
    }

    /// Read from buffer.
    pub fn from_buf(buf: &[u8]) -> Self {
        Self {
            an: LittleEndian::read_u32(&buf[0..4]),
            ac_hom: LittleEndian::read_u32(&buf[4..8]),
            ac_het: LittleEndian::read_u32(&buf[8..12]),
        }
    }

    /// Write to buffer.
    pub fn to_buf(&self, buf: &mut [u8]) {
        LittleEndian::write_u32(&mut buf[0..4], self.an);
        LittleEndian::write_u32(&mut buf[4..8], self.ac_hom);
        LittleEndian::write_u32(&mut buf[8..12], self.ac_het);
    }
}

/// Record type for the "mitochondrial" column family.
#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Record {
    /// Counts from gnomAD mtDNA.
    pub gnomad_mtdna: Counts,
    /// Counts from HelixMtDb.
    pub helixmtdb: Counts,
}

impl Record {
    /// Read from buffer.
    pub fn from_buf(buf: &[u8]) -> Self {
        Self {
            gnomad_mtdna: Counts::from_buf(&buf[0..12]),
            helixmtdb: Counts::from_buf(&buf[12..24]),
        }
    }

    /// Write to buffer.
    pub fn to_buf(&self, buf: &mut [u8]) {
        self.gnomad_mtdna.to_buf(&mut buf[0..12]);
        self.helixmtdb.to_buf(&mut buf[12..24]);
    }

    /// Length of the buffer.
    pub fn buf_len() -> usize {
        24
    }
}
