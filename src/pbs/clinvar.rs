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
}
