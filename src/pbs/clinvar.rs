//! Code generate for protobufs by `prost-build`.

/// Code generate for protobufs by `prost-build`.
pub mod per_gene {
    include!(concat!(env!("OUT_DIR"), "/annonars.clinvar.per_gene.rs"));
    include!(concat!(
        env!("OUT_DIR"),
        "/annonars.clinvar.per_gene.serde.rs"
    ));
}

/// Code generate for protobufs by `prost-build`.
pub mod minimal {
    include!(concat!(env!("OUT_DIR"), "/annonars.clinvar.minimal.rs"));
    include!(concat!(
        env!("OUT_DIR"),
        "/annonars.clinvar.minimal.serde.rs"
    ));
}

/// Code generate for protobufs by `prost-build`.
pub mod sv {
    include!(concat!(env!("OUT_DIR"), "/annonars.clinvar.sv.rs"));
    include!(concat!(env!("OUT_DIR"), "/annonars.clinvar.sv.serde.rs"));

    // TODO: TryFrom would be more appropriate
    impl From<crate::pbs::clinvar_data::extracted_vars::ExtractedVcvRecord>
        for bio::bio_types::genome::Interval
    {
        fn from(val: crate::pbs::clinvar_data::extracted_vars::ExtractedVcvRecord) -> Self {
            let crate::pbs::clinvar_data::clinvar_public::location::SequenceLocation {
                chr,
                start,
                stop,
                ..
            } = val
                .sequence_location
                .as_ref()
                .expect("missing sequence_location");
            let start = start.expect("missing start");
            let stop = stop.expect("missing stop");
            match crate::pbs::clinvar_data::clinvar_public::Chromosome::try_from(*chr) {
                Ok(chr) => bio::bio_types::genome::Interval::new(
                    chr.as_chr_name(),
                    (start as u64 - 1)..(stop as u64),
                ),
                Err(e) => panic!("problem converting chromosome {} to Chromosome: {}", chr, e),
            }
        }
    }

    // TODO: TryFrom would be more appropriate
    impl From<&crate::pbs::clinvar_data::extracted_vars::ExtractedVcvRecord>
        for bio::bio_types::genome::Interval
    {
        fn from(val: &crate::pbs::clinvar_data::extracted_vars::ExtractedVcvRecord) -> Self {
            let crate::pbs::clinvar_data::clinvar_public::location::SequenceLocation {
                chr,
                start,
                stop,
                ..
            } = val
                .sequence_location
                .as_ref()
                .expect("missing sequence_location");
            let start = start.expect("missing start");
            let stop = stop.expect("missing stop");
            match crate::pbs::clinvar_data::clinvar_public::Chromosome::try_from(*chr) {
                Ok(chr) => bio::bio_types::genome::Interval::new(
                    chr.as_chr_name(),
                    (start as u64 - 1)..(stop as u64),
                ),
                Err(e) => panic!("problem converting chromosome {} to Chromosome: {}", chr, e),
            }
        }
    }
}
