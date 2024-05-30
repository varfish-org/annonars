//! Code generate for protobufs by `prost-build`.

/// Code generated for protobufs by `prost-build`.
pub mod class_by_freq {
    include!(concat!(
        env!("OUT_DIR"),
        "/annonars.clinvar_data.class_by_freq.rs"
    ));
    include!(concat!(
        env!("OUT_DIR"),
        "/annonars.clinvar_data.class_by_freq.serde.rs"
    ));
}

/// Code generated for protobufs by `prost-build`.
pub mod clinvar_public {
    include!(concat!(
        env!("OUT_DIR"),
        "/annonars.clinvar_data.clinvar_public.rs"
    ));
    include!(concat!(
        env!("OUT_DIR"),
        "/annonars.clinvar_data.clinvar_public.serde.rs"
    ));

    impl Chromosome {
        /// Return the chromosome name, e.g., "1", ..., "22", "X", "Y", "MT", "PAR", "Un".
        pub fn as_chr_name(&self) -> String {
            self.as_str_name().replace("CHROMOSOME_", "")
        }
    }
}

/// Code generated for protobufs by `prost-build`.
pub mod extracted_vars {
    include!(concat!(
        env!("OUT_DIR"),
        "/annonars.clinvar_data.extracted_vars.rs"
    ));
    include!(concat!(
        env!("OUT_DIR"),
        "/annonars.clinvar_data.extracted_vars.serde.rs"
    ));
}

/// Code generated for protobufs by `prost-build`.
pub mod gene_impact {
    include!(concat!(
        env!("OUT_DIR"),
        "/annonars.clinvar_data.gene_impact.rs"
    ));
    include!(concat!(
        env!("OUT_DIR"),
        "/annonars.clinvar_data.gene_impact.serde.rs"
    ));
}

/// Code generated for protobufs by `prost-build`.
pub mod phenotype_link {
    include!(concat!(
        env!("OUT_DIR"),
        "/annonars.clinvar_data.phenotype_link.rs"
    ));
    include!(concat!(
        env!("OUT_DIR"),
        "/annonars.clinvar_data.phenotype_link.serde.rs"
    ));
}
