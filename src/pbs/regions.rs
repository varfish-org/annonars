//! Code generate for protobufs by `prost-build`.

/// Code generate for protobufs by `prost-build`.
pub mod clingen {
    use crate::regions::cli::import::clingen::genomic_location_to_interval;

    include!(concat!(env!("OUT_DIR"), "/annonars.regions.clingen.rs"));
    include!(concat!(
        env!("OUT_DIR"),
        "/annonars.regions.clingen.serde.rs"
    ));

    impl TryInto<bio::bio_types::genome::Interval> for Region {
        type Error = anyhow::Error;

        fn try_into(self) -> Result<bio::bio_types::genome::Interval, Self::Error> {
            genomic_location_to_interval(&self.genomic_location)
        }
    }

    impl TryInto<bio::bio_types::genome::Interval> for &Region {
        type Error = anyhow::Error;

        fn try_into(self) -> Result<bio::bio_types::genome::Interval, Self::Error> {
            genomic_location_to_interval(&self.genomic_location)
        }
    }
}
