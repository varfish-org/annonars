//! Reading of autosomal records.

use hgvs::static_data::Assembly;
use noodles_vcf::Record as VcfRecord;

use crate::{common, freqs};

/// Helper for reading through gnomAD mtDNA and HelixMtDb data;
pub struct Reader {
    /// CSV reader for the gnomAD autosomal records.
    gnomad_reader: Option<freqs::cli::import::reading::MultiVcfReader>,
    /// Next variant from gnomAD.
    gnomad_next: Option<VcfRecord>,
    /// CSV reader for the HelixMtDb records.
    helix_reader: Option<freqs::cli::import::reading::MultiVcfReader>,
    /// Next variant from gnomAD.
    helix_next: Option<VcfRecord>,
}

impl Reader {
    /// Construct new reader with optional paths to HelixMtDb and gnomAD mtDNA.
    ///
    /// Optionally, you can provide an assembly to validate the VCF contigs against.
    pub fn new(
        path_gnomad: Option<&str>,
        path_helix: Option<&str>,
        assembly: Option<Assembly>,
    ) -> Result<Self, anyhow::Error> {
        let mut gnomad_reader = path_gnomad
            .as_ref()
            .map(|path_gnomad| {
                tracing::info!("Opening gnomAD autosomal file {}", &path_gnomad);
                freqs::cli::import::reading::MultiVcfReader::new(&[path_gnomad], assembly)
            })
            .transpose()?;
        let mut helix_reader = path_helix
            .as_ref()
            .map(|path_helix| {
                tracing::info!("Opening gnomAD autosomal file {}", &path_helix);
                freqs::cli::import::reading::MultiVcfReader::new(&[path_helix], assembly)
            })
            .transpose()?;

        let gnomad_next = if let Some(gnomad_reader) = gnomad_reader.as_mut() {
            gnomad_reader.pop()?.0
        } else {
            None
        };
        let helix_next = if let Some(helix_reader) = helix_reader.as_mut() {
            helix_reader.pop()?.0
        } else {
            None
        };

        Ok(Self {
            gnomad_reader,
            helix_reader,
            gnomad_next,
            helix_next,
        })
    }

    /// Run the reading of the chrMT frequencies.
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
        match (&self.gnomad_next, &self.helix_next) {
            (None, Some(helix)) => {
                assert_eq!(
                    helix.alternate_bases().len(),
                    1,
                    "only one alternate allele is supported"
                );
                func(
                    common::keys::Var::from_vcf_allele(helix, 0),
                    freqs::serialized::mt::Counts::default(),
                    freqs::serialized::mt::Counts::from_vcf_allele(helix, 0),
                )?;
                self.helix_next = self.helix_reader.as_mut().unwrap().pop()?.0;
            }
            (Some(gnomad), None) => {
                assert_eq!(
                    gnomad.alternate_bases().len(),
                    1,
                    "only one alternate allele is supported"
                );
                func(
                    common::keys::Var::from_vcf_allele(gnomad, 0),
                    freqs::serialized::mt::Counts::from_vcf_allele(gnomad, 0),
                    freqs::serialized::mt::Counts::default(),
                )?;
                self.gnomad_next = self.gnomad_reader.as_mut().unwrap().pop()?.0;
            }
            (Some(gnomad), Some(helix)) => {
                assert_eq!(
                    helix.alternate_bases().len(),
                    1,
                    "only one alternate allele is supported"
                );
                assert_eq!(
                    gnomad.alternate_bases().len(),
                    1,
                    "only one alternate allele is supported"
                );
                let var_gnomad = common::keys::Var::from_vcf_allele(gnomad, 0);
                let var_helix = common::keys::Var::from_vcf_allele(helix, 0);
                match var_gnomad.cmp(&var_helix) {
                    std::cmp::Ordering::Less => {
                        func(
                            var_gnomad,
                            freqs::serialized::mt::Counts::from_vcf_allele(gnomad, 0),
                            freqs::serialized::mt::Counts::default(),
                        )?;
                        self.gnomad_next = self.gnomad_reader.as_mut().unwrap().pop()?.0;
                    }
                    std::cmp::Ordering::Equal => {
                        func(
                            var_gnomad,
                            freqs::serialized::mt::Counts::from_vcf_allele(gnomad, 0),
                            freqs::serialized::mt::Counts::from_vcf_allele(helix, 0),
                        )?;
                        self.helix_next = self.helix_reader.as_mut().unwrap().pop()?.0;
                        self.gnomad_next = self.gnomad_reader.as_mut().unwrap().pop()?.0;
                    }
                    std::cmp::Ordering::Greater => {
                        func(
                            var_helix,
                            freqs::serialized::mt::Counts::default(),
                            freqs::serialized::mt::Counts::from_vcf_allele(helix, 0),
                        )?;
                        self.helix_next = self.helix_reader.as_mut().unwrap().pop()?.0;
                    }
                }
            }
            (None, None) => (),
        }

        Ok(self.gnomad_next.is_some() || self.helix_next.is_some())
    }
}
