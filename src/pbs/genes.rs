//! Code generate for protobufs by `prost-build`.

/// Code generate for protobufs by `prost-build`.
pub mod base {
    use crate::regions::cli::import::clingen::genomic_location_to_interval;
    use biocommons_bioutils::assemblies::Assembly;

    include!(concat!(env!("OUT_DIR"), "/annonars.genes.base.rs"));
    include!(concat!(env!("OUT_DIR"), "/annonars.genes.base.serde.rs"));

    impl ClingenDosageRecord {
        /// Obtain interval for the given assembly.
        pub fn get_interval(
            &self,
            assembly: Assembly,
        ) -> Result<bio::bio_types::genome::Interval, anyhow::Error> {
            match assembly {
                Assembly::Grch37 | Assembly::Grch37p10 => {
                    genomic_location_to_interval(&self.genomic_location_37)
                }
                Assembly::Grch38 => genomic_location_to_interval(&self.genomic_location_38),
            }
        }
    }
}
