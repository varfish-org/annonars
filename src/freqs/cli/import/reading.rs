//! Reading of variants.

use std::collections::{BTreeMap, HashMap};

use biocommons_bioutils::assemblies::{Assembly, Sequence, ASSEMBLY_INFOS};

use crate::common::cli::CANONICAL;

/// Error type related to `ContigMap`.
#[derive(thiserror::Error, Debug)]
pub enum ContigMapError {
    /// The sequence name/alias/accession is not known.
    #[error("the sequence name/alias/accession `{0}` is not known")]
    UnknownSequence(String),
}

/// Provide mapping from contig names to numeric contig IDs.
pub struct ContigMap {
    /// The corresponding assembly.
    pub assembly: Assembly,
    /// Map from contig name to contig ID.
    pub name_map: HashMap<String, usize>,
}

impl ContigMap {
    /// Create a new mapping with the given assembly.
    ///
    /// NB: Grch37 does not include chrMT, Grch7p10 does.
    pub fn new(assembly: Assembly) -> Self {
        let mut name_map = HashMap::new();
        let info = &ASSEMBLY_INFOS[assembly];
        for (idx, seq) in info.sequences.iter().enumerate() {
            name_map.insert(seq.name.clone(), idx);
            name_map.insert(seq.refseq_ac.clone(), idx);
            name_map.insert(seq.genbank_ac.clone(), idx);
            for alias in seq.aliases.iter() {
                name_map.insert(alias.clone(), idx);
            }
        }

        Self { assembly, name_map }
    }

    /// Map chromosome to index.
    pub fn chrom_to_idx(
        &self,
        chrom: &noodles_vcf::record::Chromosome,
    ) -> Result<usize, ContigMapError> {
        match chrom {
            noodles_vcf::record::Chromosome::Name(s)
            | noodles_vcf::record::Chromosome::Symbol(s) => self.chrom_name_to_idx(s),
        }
    }

    /// Map chromosome name to index.
    pub fn chrom_name_to_idx(&self, chrom: &str) -> Result<usize, ContigMapError> {
        self.name_map
            .get(chrom)
            .copied()
            .ok_or_else(|| ContigMapError::UnknownSequence(chrom.to_string()))
    }

    /// Map chromosome name to `sequence``.
    pub fn chrom_name_to_seq(&self, chrom: &str) -> Result<&Sequence, ContigMapError> {
        Ok(&ASSEMBLY_INFOS[self.assembly].sequences[self.chrom_name_to_idx(chrom)?])
    }
}

/// Key for sorting the records.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Key {
    /// Chromosome.
    chrom: String,
    /// Noodles position.
    pos: noodles_vcf::record::Position,
    /// Reference allele.
    reference: String,
    /// First (and only) alternate allelele.
    alternative: String,
    /// Index of the reader.
    idx: usize,
}

/// Build a key from a VCF record.
fn build_key(record: &noodles_vcf::Record, i: usize) -> Key {
    Key {
        chrom: record.chromosome().to_string(),
        pos: record.position(),
        reference: record.reference_bases().to_string(),
        alternative: record
            .alternate_bases()
            .first()
            .expect("must have alternate allele")
            .to_string(),
        idx: i,
    }
}

/// Read through multiple `noodles_vcf::vcf::reader::Query`s at once.
pub struct MultiQuery<'r, 'h, R>
where
    R: std::io::Read + std::io::Seek,
{
    /// One query for each input file.
    queries: Vec<noodles_vcf::reader::Query<'r, 'h, R>>,
    /// The current smallest-by-coordinate records.
    records: BTreeMap<Key, noodles_vcf::Record>,
}

impl<'r, 'h, R> MultiQuery<'r, 'h, R>
where
    R: std::io::Read + std::io::Seek,
{
    /// Construct a new `MultiQuery`.
    pub fn new(
        mut record_iters: Vec<noodles_vcf::reader::Query<'r, 'h, R>>,
    ) -> std::io::Result<Self> {
        let mut records = BTreeMap::new();

        for (i, iter) in record_iters.iter_mut().enumerate() {
            if let Some(result) = iter.next() {
                let record = result?;
                let key = build_key(&record, i);
                records.insert(key, record);
            }
        }

        Ok(Self {
            queries: record_iters,
            records,
        })
    }
}

impl<'r, 'h, R> Iterator for MultiQuery<'r, 'h, R>
where
    R: std::io::Read + std::io::Seek,
{
    type Item = std::io::Result<(usize, noodles_vcf::Record)>;

    /// Return next item if any.
    fn next(&mut self) -> Option<Self::Item> {
        let (Key { idx, .. }, record) = self.records.pop_first()?;

        if let Some(result) = self.queries[idx].next() {
            match result {
                Ok(record) => {
                    let key = build_key(&record, idx);
                    self.records.insert(key, record);
                }
                Err(e) => return Some(Err(e)),
            }
        }

        Some(Ok((idx, record)))
    }
}

/// Guess the assembly from the given header.
///
/// If the header only contains chrM, for example, the result may be ambiguous. Use `ambiguous_ok`
/// to allow or disallow this.  You can specify an initial value for the assembly to overcome
/// issues.  If the result is incompatible with the `initial_assembly` then an error will
/// be returned.
pub fn guess_assembly(
    vcf_header: &noodles_vcf::Header,
    ambiguous_ok: bool,
    initial_assembly: Option<Assembly>,
) -> Result<Assembly, anyhow::Error> {
    let mut result = initial_assembly;

    let assembly_infos = [
        (Assembly::Grch37p10, &ASSEMBLY_INFOS[Assembly::Grch37p10]),
        (Assembly::Grch38, &ASSEMBLY_INFOS[Assembly::Grch38]),
    ];

    // Check each assembly.
    for (assembly, info) in assembly_infos.iter() {
        // Collect contig name / length pairs for the assembly.
        let contig_map = ContigMap::new(*assembly);
        let mut lengths = HashMap::new();
        for seq in &info.sequences {
            if CANONICAL.contains(&seq.name.as_str()) {
                lengths.insert(
                    contig_map.name_map.get(seq.name.as_str()).unwrap(),
                    seq.length,
                );
            }
        }

        // Count compatible and incompatible contigs.
        let mut incompatible = 0;
        let mut compatible = 0;
        for (name, data) in vcf_header.contigs() {
            if let Some(length) = data.length() {
                let idx = contig_map.name_map.get(name.as_ref());
                if let Some(idx) = idx {
                    let name = &info.sequences[*idx].name;
                    if CANONICAL.contains(&name.as_ref()) {
                        if *lengths.get(idx).unwrap() == length {
                            compatible += 1;
                        } else {
                            incompatible += 1;
                        }
                    }
                }
            } else {
                tracing::warn!(
                    "Cannot guess assembly because no length for contig {}",
                    &name
                );
                compatible = 0;
                break;
            }
        }

        if compatible > 0 && incompatible == 0 {
            // Found a compatible assembly.  Check if we already have one and bail out if
            // ambiguity is not allowed.  Anyway, we only keep the first found compatible
            // assembly.
            if let Some(result) = result {
                if result != *assembly && !ambiguous_ok {
                    return Err(anyhow::anyhow!(
                        "Found ambiguity;  initial={:?}, previous={:?}, current={:?}",
                        initial_assembly,
                        result,
                        assembly,
                    ));
                }
                // else: do not re-assign
            } else {
                result = Some(*assembly);
            }
        } else {
            // Found incompatible assembly, bail out if is the initial assembly.
            if let Some(initial_assembly) = initial_assembly {
                if initial_assembly == *assembly {
                    return Err(anyhow::anyhow!(
                        "Incompatible with initial assembly {:?}",
                        result.unwrap()
                    ));
                }
            }
        }
    }

    if let Some(result) = result {
        Ok(result)
    } else {
        Err(anyhow::anyhow!("No matching assembly found"))
    }
}

#[cfg(test)]
mod test {
    use biocommons_bioutils::assemblies::Assembly;

    use super::*;

    #[test]
    fn guess_assembly_helix_chrmt_ambiguous_ok_initial_none() -> Result<(), anyhow::Error> {
        let path = "tests/freqs/grch37/v2.1/reading/helix.chrM.vcf";
        let mut reader = noodles_vcf::reader::Builder::default().build_from_path(path)?;
        let header = reader.read_header()?;

        let actual = guess_assembly(&header, true, None)?;
        assert_eq!(actual, Assembly::Grch37p10);

        Ok(())
    }

    #[test]
    fn guess_assembly_helix_chrmt_ambiguous_ok_initial_override() -> Result<(), anyhow::Error> {
        let path = "tests/freqs/grch37/v2.1/reading/helix.chrM.vcf";
        let mut reader = noodles_vcf::reader::Builder::default().build_from_path(path)?;
        let header = reader.read_header()?;

        let actual = guess_assembly(&header, true, Some(Assembly::Grch37p10))?;
        assert_eq!(actual, Assembly::Grch37p10);

        Ok(())
    }

    #[test]
    fn guess_assembly_helix_chrmt_ambiguous_ok_initial_override_fails() -> Result<(), anyhow::Error>
    {
        let path = "tests/freqs/grch37/v2.1/reading/helix.chrM.vcf";
        let mut reader = noodles_vcf::reader::Builder::default().build_from_path(path)?;
        let header = reader.read_header()?;

        assert!(guess_assembly(&header, false, Some(Assembly::Grch37)).is_err());

        Ok(())
    }

    #[test]
    fn guess_assembly_helix_chrmt_ambiguous_fail() -> Result<(), anyhow::Error> {
        let path = "tests/freqs/grch37/v2.1/reading/helix.chrM.vcf";
        let mut reader = noodles_vcf::reader::Builder::default().build_from_path(path)?;
        let header = reader.read_header()?;

        assert!(guess_assembly(&header, false, None).is_err());

        Ok(())
    }

    #[test]
    fn contig_map_smoke() {
        ContigMap::new(Assembly::Grch37p10);
        ContigMap::new(Assembly::Grch38);
    }

    #[test]
    fn test_multiquery() -> Result<(), anyhow::Error> {
        let mut readers = vec![
            noodles_vcf::indexed_reader::Builder::default()
                .build_from_path("tests/freqs/grch37/v2.1/reading/gnomad.chrM.vcf.bgz")?,
            noodles_vcf::indexed_reader::Builder::default()
                .build_from_path("tests/freqs/grch37/v2.1/reading/helix.chrM.vcf.bgz")?,
        ];

        let headers: Vec<_> = readers
            .iter_mut()
            .map(|reader| reader.read_header())
            .collect::<Result<_, _>>()?;

        let start = noodles_core::position::Position::try_from(1)?;
        let stop = noodles_core::position::Position::try_from(16569)?;
        let region = noodles_core::region::Region::new("chrM", start..=stop);

        let queries: Vec<_> = readers
            .iter_mut()
            .zip(&headers)
            .map(|(reader, header)| reader.query(header, &region))
            .collect::<Result<_, _>>()?;

        let multi_query = MultiQuery::new(queries)?;

        let mut records = Vec::new();
        for result in multi_query {
            records.push(result?);
        }

        insta::assert_debug_snapshot!(records);

        Ok(())
    }
}
