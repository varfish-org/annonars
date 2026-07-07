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

/// Number of big-endian bytes of the interned contig id in a compact key
/// (see [`Var::encode_with_id`]); 3 bytes ⇒ a 24-bit contig id space.
pub const CONTIG_ID_LEN: usize = 3;
/// Number of big-endian bytes of the position (`i32`) in a compact key.
const POS_LEN: usize = 4;
/// Separator byte between the reference and alternative allele in a compact key.
const ALLELE_SEP: u8 = 0x00;
/// Byte offset of the position field within a compact key.
const POS_OFFSET: usize = CONTIG_ID_LEN;
/// Byte offset of the allele block within a compact key.
const ALLELES_OFFSET: usize = CONTIG_ID_LEN + POS_LEN;

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
    pub fn from_vcf_allele(value: &noodles::vcf::variant::RecordBuf, allele_no: usize) -> Self {
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

    /// Serialize into a compact, assembly-agnostic binary key using an interned
    /// 24-bit contig ID (rather than the fixed 2-byte chromosome key).
    ///
    /// Layout: 3 bytes contig ID (big-endian) + 4 bytes `pos` (big-endian) + REF
    /// bytes + `0x00` separator + ALT bytes. Decode with [`Self::decode_with_ctx`].
    pub fn encode_with_id(&self, chrom_id: u32) -> Vec<u8> {
        assert!(
            (chrom_id as usize) < (1 << (CONTIG_ID_LEN * 8)),
            "Contig ID exceeds 24-bit limit"
        );

        let mut result =
            Vec::with_capacity(ALLELES_OFFSET + self.reference.len() + 1 + self.alternative.len());

        // Big-endian ID, dropping the leading byte(s) (value fits in 24 bits).
        let id_bytes = chrom_id.to_be_bytes();
        result.extend_from_slice(&id_bytes[id_bytes.len() - CONTIG_ID_LEN..]);
        result.extend_from_slice(&self.pos.to_be_bytes());
        result.extend_from_slice(self.reference.as_bytes());
        result.push(ALLELE_SEP);
        result.extend_from_slice(self.alternative.as_bytes());

        result
    }

    /// Deserialize a compact key produced by [`Self::encode_with_id`], resolving
    /// the contig ID to a name via the given id→name table (the contig dictionary).
    pub fn decode_with_ctx(value: &[u8], id_to_chrom: &[String]) -> Self {
        assert!(
            value.len() > ALLELES_OFFSET,
            "Corrupted database key: underlying byte array too short"
        );

        let mut id_bytes = [0u8; 4];
        let id_start = id_bytes.len() - CONTIG_ID_LEN;
        id_bytes[id_start..].copy_from_slice(&value[0..CONTIG_ID_LEN]);
        let chrom_id = u32::from_be_bytes(id_bytes);

        let chrom = id_to_chrom
            .get(chrom_id as usize)
            .cloned()
            .expect("Corrupted database: contig ID missing from metadata context map");

        let pos = i32::from_be_bytes(value[POS_OFFSET..POS_OFFSET + POS_LEN].try_into().unwrap());

        let alleles_buf = &value[ALLELES_OFFSET..];
        let null_idx = alleles_buf
            .iter()
            .position(|&b| b == 0x00)
            .expect("Corrupted database key: missing allele null-terminator");

        let reference = std::str::from_utf8(&alleles_buf[0..null_idx])
            .expect("Invalid UTF-8 sequence in reference allele")
            .to_string();
        let alternative = std::str::from_utf8(&alleles_buf[null_idx + 1..])
            .expect("Invalid UTF-8 sequence in alternative allele")
            .to_string();

        Self {
            chrom,
            pos,
            reference,
            alternative,
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

    #[test]
    fn test_var_encode_decode_with_id_roundtrip() {
        let id_to_chrom = vec!["1".to_string(), "X".to_string(), "MT".to_string()];

        let var = Var::from("X", 12345, "AC", "T");
        let key = var.encode_with_id(1);

        // 3 (id) + 4 (pos) + 2 (ref) + 1 (sep) + 1 (alt) = 11 bytes.
        assert_eq!(key.len(), 11);
        assert_eq!(&key[0..3], &[0x00, 0x00, 0x01]);

        let decoded = Var::decode_with_ctx(&key, &id_to_chrom);
        assert_eq!(decoded, var);
    }

    #[test]
    fn test_var_encode_with_id_is_position_sortable() {
        // Same contig: byte order must follow position order.
        let a = Var::from("1", 100, "A", "T").encode_with_id(0);
        let b = Var::from("1", 101, "A", "T").encode_with_id(0);
        assert!(a < b);
    }
}
