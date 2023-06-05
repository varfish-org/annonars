//! Import of autosomal frequencies from gnomAD.

use hgvs::static_data::Assembly;

use crate::{common, freqs};

/// Helper for reading through gnomAD genomes and exomes data.
pub struct Reader {
    /// CSV reader for the gnomAD genomes.
    genomes_reader: Option<freqs::cli::reading::MultiVcfReader>,
    /// Next variant from gnomAD genomes.
    genomes_next: Option<noodles_vcf::Record>,
    /// CSV reader for the gnomAD exomes.
    exomes_reader: Option<freqs::cli::reading::MultiVcfReader>,
    /// Next variant from gnomAD exomes
    exomes_next: Option<noodles_vcf::Record>,
}

impl Reader {
    /// Construct new reader with optional paths to gnomAD genomes and exomes files.
    ///
    /// Optionally, you can provide an assembly to validate the VCF contigs against.
    pub fn new(
        path_genomes: Option<&str>,
        path_exomes: Option<&str>,
        assembly: Option<Assembly>,
    ) -> Result<Self, anyhow::Error> {
        let mut genomes_reader = path_genomes
            .as_ref()
            .map(|path_genomes| {
                tracing::info!("Opening gnomAD genome autosomal file {}", &path_genomes);
                freqs::cli::reading::MultiVcfReader::new(&[path_genomes], assembly)
            })
            .transpose()?;
        let mut exomes_reader = path_exomes
            .as_ref()
            .map(|path_exomes| {
                tracing::info!("Opening gnomAD exomes autosomal file {}", &path_exomes);
                freqs::cli::reading::MultiVcfReader::new(&[path_exomes], assembly)
            })
            .transpose()?;

        let genomes_next = if let Some(genomes_reader) = genomes_reader.as_mut() {
            genomes_reader.pop()?.0
        } else {
            None
        };
        let exomes_next = if let Some(exomes_reader) = exomes_reader.as_mut() {
            exomes_reader.pop()?.0
        } else {
            None
        };

        Ok(Self {
            genomes_reader,
            exomes_reader,
            genomes_next,
            exomes_next,
        })
    }

    /// Run the reading of the autosomal frequencies.
    ///
    /// Returns whether there is a next value.
    pub fn run<F>(&mut self, mut func: F) -> Result<bool, anyhow::Error>
    where
        F: FnMut(
            common::keys::Var,
            freqs::serialized::mt::Counts,
            freqs::serialized::mt::Counts,
        ) -> Result<(), anyhow::Error>,
    {
        match (&self.genomes_next, &self.exomes_next) {
            (None, Some(exomes_record)) => {
                assert_eq!(
                    exomes_record.alternate_bases().len(),
                    1,
                    "only one alternate allele is supported"
                );
                func(
                    common::keys::Var::from_vcf_allele(exomes_record, 0),
                    freqs::serialized::mt::Counts::default(),
                    freqs::serialized::mt::Counts::from_vcf_allele(exomes_record, 0),
                )?;
                self.exomes_next = self.exomes_reader.as_mut().unwrap().pop()?.0;
            }
            (Some(genomes_record), None) => {
                assert_eq!(
                    genomes_record.alternate_bases().len(),
                    1,
                    "only one alternate allele is supported"
                );
                func(
                    common::keys::Var::from_vcf_allele(genomes_record, 0),
                    freqs::serialized::mt::Counts::from_vcf_allele(genomes_record, 0),
                    freqs::serialized::mt::Counts::default(),
                )?;
                self.genomes_next = self.genomes_reader.as_mut().unwrap().pop()?.0;
            }
            (Some(genomes_record), Some(exomes_record)) => {
                assert_eq!(
                    exomes_record.alternate_bases().len(),
                    1,
                    "only one alternate allele is supported"
                );
                assert_eq!(
                    genomes_record.alternate_bases().len(),
                    1,
                    "only one alternate allele is supported"
                );
                let var_genomes = common::keys::Var::from_vcf_allele(genomes_record, 0);
                let var_exomes = common::keys::Var::from_vcf_allele(exomes_record, 0);
                match var_genomes.cmp(&var_exomes) {
                    std::cmp::Ordering::Less => {
                        func(
                            var_genomes,
                            freqs::serialized::mt::Counts::from_vcf_allele(genomes_record, 0),
                            freqs::serialized::mt::Counts::default(),
                        )?;
                        self.genomes_next = self.genomes_reader.as_mut().unwrap().pop()?.0;
                    }
                    std::cmp::Ordering::Equal => {
                        func(
                            var_genomes,
                            freqs::serialized::mt::Counts::from_vcf_allele(genomes_record, 0),
                            freqs::serialized::mt::Counts::from_vcf_allele(exomes_record, 0),
                        )?;
                        self.exomes_next = self.exomes_reader.as_mut().unwrap().pop()?.0;
                        self.genomes_next = self.genomes_reader.as_mut().unwrap().pop()?.0;
                    }
                    std::cmp::Ordering::Greater => {
                        func(
                            var_exomes,
                            freqs::serialized::mt::Counts::default(),
                            freqs::serialized::mt::Counts::from_vcf_allele(exomes_record, 0),
                        )?;
                        self.exomes_next = self.exomes_reader.as_mut().unwrap().pop()?.0;
                    }
                }
            }
            (None, None) => (),
        }

        Ok(self.genomes_next.is_some() || self.exomes_next.is_some())
    }
}
