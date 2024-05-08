//! RocksDB keys and their encoding.

/// A chromosomal position `CHROM-POS`.
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Pos {
    /// Chromosome name.
    pub chrom: String,
    /// 1-based start position.
    pub pos: i32,
}

impl Pos {
    /// Create new position.
    pub fn new(chrom: String, pos: i32) -> Self {
        Self { chrom, pos }
    }

    /// Create from the given chrom/pos pair.
    pub fn from(chrom: &str, pos: i32) -> Self {
        Self {
            chrom: chrom.to_string(),
            pos,
        }
    }

    /// Normalize chrom with `chrom_name_to_key`.
    pub fn with_key_as_chrom(&self) -> Self {
        Self {
            chrom: chrom_name_to_key(&self.chrom),
            pos: self.pos,
        }
    }
}

impl From<Pos> for Vec<u8> {
    fn from(val: Pos) -> Self {
        let mut result = Vec::new();

        result.extend_from_slice(chrom_name_to_key(&val.chrom).as_bytes());
        result.extend_from_slice(&val.pos.to_be_bytes());

        result
    }
}

impl From<&[u8]> for Pos {
    fn from(value: &[u8]) -> Self {
        let chrom = chrom_key_to_name(&value[0..2]);
        let pos = i32::from_be_bytes(value[2..6].try_into().unwrap());
        Self { chrom, pos }
    }
}

impl From<super::spdi::Pos> for Pos {
    fn from(other: super::spdi::Pos) -> Self {
        Self::new(other.sequence, other.position)
    }
}

/// A chromosomal change `CHROM-POS-REF-ALT`.
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Var {
    /// Chromosome name.
    pub chrom: String,
    /// 1-based start position.
    pub pos: i32,
    /// Reference allele string.
    pub reference: String,
    /// Alternative allele string.
    pub alternative: String,
}

impl Var {
    /// Create new VCF-style variant.
    pub fn new(chrom: String, pos: i32, reference: String, alternative: String) -> Self {
        Self {
            chrom,
            pos,
            reference,
            alternative,
        }
    }

    /// Create from the given VCF-style variant.
    pub fn from(chrom: &str, pos: i32, reference: &str, alternative: &str) -> Self {
        Self {
            chrom: chrom.to_string(),
            pos,
            reference: reference.to_string(),
            alternative: alternative.to_string(),
        }
    }

    /// Create for all alternate alleles from the given VCF record.
    pub fn from_vcf_allele(value: &noodles_vcf::variant::RecordBuf, allele_no: usize) -> Self {
        let chrom = value.reference_sequence_name().to_string();
        let pos: usize = value
            .variant_start()
            .expect("Telomeric breakends not supported")
            .get();
        let pos = i32::try_from(pos).unwrap();
        let reference = value.reference_bases().to_string();
        Var {
            chrom,
            pos,
            reference,
            alternative: value.alternate_bases().as_ref()[allele_no].to_string(),
        }
    }
}

impl From<Var> for Vec<u8> {
    fn from(val: Var) -> Self {
        let mut result = Vec::new();

        result.extend_from_slice(chrom_name_to_key(&val.chrom).as_bytes());
        result.extend_from_slice(&val.pos.to_be_bytes());
        result.extend_from_slice(val.reference.as_bytes());
        result.push(b'>');
        result.extend_from_slice(val.alternative.as_bytes());

        result
    }
}

impl From<super::spdi::Var> for Var {
    fn from(other: super::spdi::Var) -> Self {
        Self::new(
            other.sequence,
            other.position,
            other.deletion,
            other.insertion,
        )
    }
}

/// Convert chromosome to key in RocksDB.
pub fn chrom_name_to_key(name: &str) -> String {
    let chrom = if let Some(stripped) = name.strip_prefix("chr") {
        stripped
    } else {
        name
    };
    let chrom = if chrom == "M" {
        String::from("MT")
    } else if "XY".contains(chrom) {
        format!(" {chrom}")
    } else {
        String::from(chrom)
    };
    assert!(chrom.len() <= 2, "chrom = {:?}", chrom);
    assert!(!chrom.is_empty());
    if chrom.len() == 1 {
        format!("0{chrom}")
    } else {
        chrom
    }
}

/// Convert from RocksDB chromosome key part to chromosome name.
pub fn chrom_key_to_name(key: &[u8]) -> String {
    assert!(key.len() == 2);
    if key.starts_with(b"0") || key.starts_with(b" ") {
        std::str::from_utf8(&key[1..])
            .expect("could not decode UTF-8")
            .to_string()
    } else {
        std::str::from_utf8(key)
            .expect("could not decode UTF-8")
            .to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_pos() {
        let pos = Pos::from("chr1", 123);

        insta::assert_debug_snapshot!(pos);

        let buf: Vec<u8> = pos.into();

        insta::assert_debug_snapshot!(buf);
    }

    #[test]
    fn test_var() {
        let var = Var::from("chr1", 123, "A", "T");

        insta::assert_debug_snapshot!(var);

        let buf: Vec<u8> = var.into();

        insta::assert_debug_snapshot!(buf);
    }

    #[test]
    fn test_chrom_name_to_key() {
        assert_eq!(chrom_name_to_key("chr1"), "01");
        assert_eq!(chrom_name_to_key("chr21"), "21");
        assert_eq!(chrom_name_to_key("chrX"), " X");
        assert_eq!(chrom_name_to_key("chrY"), " Y");
        assert_eq!(chrom_name_to_key("chrM"), "MT");
        assert_eq!(chrom_name_to_key("chrMT"), "MT");

        assert_eq!(chrom_name_to_key("1"), "01");
        assert_eq!(chrom_name_to_key("21"), "21");
        assert_eq!(chrom_name_to_key("X"), " X");
        assert_eq!(chrom_name_to_key("Y"), " Y");
        assert_eq!(chrom_name_to_key("M"), "MT");
        assert_eq!(chrom_name_to_key("MT"), "MT");
    }

    #[test]
    fn test_chrom_key_to_name() {
        assert_eq!(chrom_key_to_name(b"01"), "1");
        assert_eq!(chrom_key_to_name(b"21"), "21");
        assert_eq!(chrom_key_to_name(b" X"), "X");
        assert_eq!(chrom_key_to_name(b" Y"), "Y");
        assert_eq!(chrom_key_to_name(b"MT"), "MT");
    }
}
