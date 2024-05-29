//! Code generate for protobufs by `prost-build`.

/// Code generate for protobufs by `prost-build`.
pub mod minimal {
    include!(concat!(env!("OUT_DIR"), "/annonars.clinvar.minimal.rs"));
    include!(concat!(
        env!("OUT_DIR"),
        "/annonars.clinvar.minimal.serde.rs"
    ));
}

/// Code generate for protobufs by `prost-build`.
pub mod per_gene {
    include!(concat!(env!("OUT_DIR"), "/annonars.clinvar.per_gene.rs"));
    include!(concat!(
        env!("OUT_DIR"),
        "/annonars.clinvar.per_gene.serde.rs"
    ));
}

/// Code generate for protobufs by `prost-build`.
pub mod sv {
    include!(concat!(env!("OUT_DIR"), "/annonars.clinvar.sv.rs"));
    include!(concat!(env!("OUT_DIR"), "/annonars.clinvar.sv.serde.rs"));

    impl From<Record> for bio::bio_types::genome::Interval {
        fn from(val: Record) -> Self {
            bio::bio_types::genome::Interval::new(
                val.chromosome,
                (val.start as u64 - 1)..(val.stop as u64),
            )
        }
    }

    impl From<&Record> for bio::bio_types::genome::Interval {
        fn from(val: &Record) -> Self {
            bio::bio_types::genome::Interval::new(
                val.chromosome.clone(),
                (val.start as u64 - 1)..(val.stop as u64),
            )
        }
    }
}
