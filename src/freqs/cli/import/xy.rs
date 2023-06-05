//! Import of gonomosomal frequencies from gnomAD.

use hgvs::static_data::Assembly;

use crate::{common, freqs};

/// Helper for reading through gnomAD mtDNA and HelixMtDb data;
pub struct Reader {
    /// CSV reader for the gnomAD mitochondrial records.
    gnomad_genomes_reader: Option<freqs::cli::reading::MultiVcfReader>,
    /// Next variant from gnomAD.
    gnomad_genomes_next: Option<noodles_vcf::Record>,
    /// CSV reader for the HelixMtDb records.
    gnomad_exomes_reader: Option<freqs::cli::reading::MultiVcfReader>,
    /// Next variant from gnomAD.
    gnomad_exomes_next: Option<noodles_vcf::Record>,
}

impl Reader {
    /// Construct new reader with optional paths to gnomAD genomes and exomes data.
    ///
    /// Optionally, you can provide an assembly to validate the VCF contigs against.
    pub fn new(
        path_gnomad_genomes: Option<&[&str]>,
        path_gnomad_exomes: Option<&[&str]>,
        assembly: Option<Assembly>,
    ) -> Result<Self, anyhow::Error> {
        let mut gnomad_genomes_reader = path_gnomad_genomes
            .map(|path_gnomad_genomes| {
                tracing::info!("Opening gnomAD exomes file {:?}", &path_gnomad_genomes);
                freqs::cli::reading::MultiVcfReader::new(path_gnomad_genomes, assembly)
            })
            .transpose()?;

        let gnomad_genomes_next =
            if let Some(gnomad_genomes_reader) = gnomad_genomes_reader.as_mut() {
                gnomad_genomes_reader.pop()?.0
            } else {
                None
            };

        let mut gnomad_exomes_reader = path_gnomad_exomes
            .map(|path_gnomad_exomes| {
                tracing::info!("Opening gnomAD genomes file {:?}", &path_gnomad_exomes);
                freqs::cli::reading::MultiVcfReader::new(path_gnomad_exomes, assembly)
            })
            .transpose()?;

        let gnomad_exomes_next = if let Some(gnomad_exomes_reader) = gnomad_exomes_reader.as_mut() {
            gnomad_exomes_reader.pop()?.0
        } else {
            None
        };

        Ok(Self {
            gnomad_genomes_reader,
            gnomad_exomes_reader,
            gnomad_genomes_next,
            gnomad_exomes_next,
        })
    }

    /// Run the reading of the chrMT frequencies.
    ///
    /// Returns whether there is a next value.
    pub fn run<F>(&mut self, mut func: F) -> Result<bool, anyhow::Error>
    where
        F: FnMut(
            common::keys::Var,
            freqs::serialized::xy::Counts,
            freqs::serialized::xy::Counts,
        ) -> Result<(), anyhow::Error>,
    {
        match (&self.gnomad_genomes_next, &self.gnomad_exomes_next) {
            (None, Some(gnomad_exomes)) => {
                func(
                    common::keys::Var::from_vcf_allele(gnomad_exomes, 0),
                    freqs::serialized::xy::Counts::default(),
                    freqs::serialized::xy::Counts::from_vcf_allele(gnomad_exomes, 0),
                )?;
                self.gnomad_exomes_next = self.gnomad_exomes_reader.as_mut().unwrap().pop()?.0;
            }
            (Some(gnomad_genomes), None) => {
                func(
                    common::keys::Var::from_vcf_allele(gnomad_genomes, 0),
                    freqs::serialized::xy::Counts::from_vcf_allele(gnomad_genomes, 0),
                    freqs::serialized::xy::Counts::default(),
                )?;
                self.gnomad_genomes_next = self.gnomad_genomes_reader.as_mut().unwrap().pop()?.0;
            }
            (Some(gnomad_genomes), Some(gnomad_exomes)) => {
                let var_gnomad_genomes = common::keys::Var::from_vcf_allele(gnomad_genomes, 0);
                let var_gnomad_exomes = common::keys::Var::from_vcf_allele(gnomad_exomes, 0);
                match var_gnomad_genomes.cmp(&var_gnomad_exomes) {
                    std::cmp::Ordering::Less => {
                        func(
                            var_gnomad_genomes,
                            freqs::serialized::xy::Counts::from_vcf_allele(gnomad_genomes, 0),
                            freqs::serialized::xy::Counts::default(),
                        )?;
                        self.gnomad_genomes_next =
                            self.gnomad_genomes_reader.as_mut().unwrap().pop()?.0;
                    }
                    std::cmp::Ordering::Equal => {
                        func(
                            var_gnomad_genomes,
                            freqs::serialized::xy::Counts::from_vcf_allele(gnomad_genomes, 0),
                            freqs::serialized::xy::Counts::from_vcf_allele(gnomad_exomes, 0),
                        )?;
                        self.gnomad_genomes_next =
                            self.gnomad_genomes_reader.as_mut().unwrap().pop()?.0;
                        self.gnomad_exomes_next =
                            self.gnomad_exomes_reader.as_mut().unwrap().pop()?.0;
                    }
                    std::cmp::Ordering::Greater => {
                        func(
                            var_gnomad_exomes,
                            freqs::serialized::xy::Counts::default(),
                            freqs::serialized::xy::Counts::from_vcf_allele(gnomad_exomes, 0),
                        )?;
                        self.gnomad_exomes_next =
                            self.gnomad_exomes_reader.as_mut().unwrap().pop()?.0;
                    }
                }
            }
            (None, None) => (),
        }

        Ok(self.gnomad_genomes_next.is_some() || self.gnomad_exomes_next.is_some())
    }
}
