//! Code generate for protobufs by `prost-build`.

/// Code generated for protobufs by `prost-build`.
pub mod class_by_freq {
    include!(concat!(env!("OUT_DIR"), "/clinvar_data.class_by_freq.rs"));
    include!(concat!(
        env!("OUT_DIR"),
        "/clinvar_data.class_by_freq.serde.rs"
    ));
}

/// Code generated for protobufs by `prost-build`.
pub mod clinvar_public {
    include!(concat!(env!("OUT_DIR"), "/clinvar_data.clinvar_public.rs"));
    include!(concat!(
        env!("OUT_DIR"),
        "/clinvar_data.clinvar_public.serde.rs"
    ));
}

/// Code generated for protobufs by `prost-build`.
pub mod extracted_vars {
    include!(concat!(env!("OUT_DIR"), "/clinvar_data.extracted_vars.rs"));
    include!(concat!(
        env!("OUT_DIR"),
        "/clinvar_data.extracted_vars.serde.rs"
    ));
}

/// Code generated for protobufs by `prost-build`.
pub mod gene_impact {
    include!(concat!(env!("OUT_DIR"), "/clinvar_data.gene_impact.rs"));
    include!(concat!(
        env!("OUT_DIR"),
        "/clinvar_data.gene_impact.serde.rs"
    ));
}

/// Code generated for protobufs by `prost-build`.
pub mod phenotype_link {
    include!(concat!(env!("OUT_DIR"), "/clinvar_data.phenotype_link.rs"));
    include!(concat!(
        env!("OUT_DIR"),
        "/clinvar_data.phenotype_link.serde.rs"
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
