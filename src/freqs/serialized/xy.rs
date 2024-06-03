//! gonosomal counts.

use byteorder::{ByteOrder, LittleEndian};
use noodles::vcf::variant::record::AlternateBases;

use crate::common;

/// Record type for storing AN, AC_hom, AC_het, AC_hemi counts for chrX/chrY.
#[derive(Default, Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Counts {
    /// Total number of alleles.
    pub an: u32,
    /// Number of hom. alt. alleles.
    pub ac_hom: u32,
    /// Number of het. alt. alleles.
    pub ac_het: u32,
    /// Number of hemi. alt. alleles.
    pub ac_hemi: u32,
}

impl Counts {
    /// Create from the given VCF record.
    pub fn from_vcf_allele(value: &noodles::vcf::variant::RecordBuf, _allele_no: usize) -> Self {
        assert_eq!(
            value.alternate_bases().len(),
            1,
            "only one alternate allele is supported",
        );
        tracing::trace!("@ {:?}", &value);

        let an = common::noodles::get_i32(value, "AN").unwrap() as u32;

        let ac_hom_xx = common::noodles::get_i32(value, "nhomalt_female")
            .or_else(|_| common::noodles::get_i32(value, "nhomalt_XX"))
            .unwrap_or_default() as u32;
        let ac_xx = common::noodles::get_i32(value, "AC_female")
            .or_else(|_| common::noodles::get_i32(value, "AC_XX"))
            .unwrap_or_default() as u32;

        let ac_hom_xy = common::noodles::get_i32(value, "nhomalt_male")
            .or_else(|_| common::noodles::get_i32(value, "nhomalt_XY"))
            .expect("neither found: nhomalt_male, nhomalt_XY") as u32;
        let ac_xy = common::noodles::get_i32(value, "AC_male")
            .or_else(|_| common::noodles::get_i32(value, "AC_XY"))
            .expect("neither found: AC_male, AC_XY") as u32;

        let nonpar = common::noodles::get_flag(value, "nonpar").unwrap_or(false)
            || common::noodles::get_flag(value, "non_par").unwrap_or(false);

        if nonpar {
            // not in PAR
            Counts {
                ac_hom: ac_hom_xx,
                ac_het: ac_xx.saturating_sub(2 * ac_hom_xx),
                ac_hemi: ac_xy,
                an,
            }
        } else {
            // is in PAR
            Counts {
                ac_hom: ac_hom_xx + ac_hom_xy,
                ac_het: ac_xx.saturating_sub(2 * ac_hom_xx + 2 * ac_hom_xy),
                ac_hemi: 0,
                an,
            }
        }
    }

    /// Read from buffer.
    pub fn from_buf(buf: &[u8]) -> Self {
        Self {
            an: LittleEndian::read_u32(&buf[0..4]),
            ac_hom: LittleEndian::read_u32(&buf[4..8]),
            ac_het: LittleEndian::read_u32(&buf[8..12]),
            ac_hemi: LittleEndian::read_u32(&buf[12..16]),
        }
    }

    /// Write to buffer.
    pub fn to_buf(&self, buf: &mut [u8]) {
        LittleEndian::write_u32(&mut buf[0..4], self.an);
        LittleEndian::write_u32(&mut buf[4..8], self.ac_hom);
        LittleEndian::write_u32(&mut buf[8..12], self.ac_het);
        LittleEndian::write_u32(&mut buf[12..16], self.ac_hemi);
    }
}

/// Record type for the "mitochondrial" column family.
#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Record {
    /// Counts from gnomAD exomes.
    pub gnomad_exomes: Counts,
    /// Counts from gnomAD genomes.
    pub gnomad_genomes: Counts,
}

impl Record {
    /// Read from buffer.
    pub fn from_buf(buf: &[u8]) -> Self {
        Self {
            gnomad_exomes: Counts::from_buf(&buf[0..16]),
            gnomad_genomes: Counts::from_buf(&buf[16..32]),
        }
    }

    /// Write to buffer.
    pub fn to_buf(&self, buf: &mut [u8]) {
        self.gnomad_exomes.to_buf(&mut buf[0..16]);
        self.gnomad_genomes.to_buf(&mut buf[16..32]);
    }

    /// Length of the buffer.
    pub fn buf_len() -> usize {
        32
    }
}
