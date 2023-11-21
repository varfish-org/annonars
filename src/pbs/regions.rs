//! Code generate for protobufs by `prost-build`.

/// Code generate for protobufs by `prost-build`.
pub mod clingen {
    include!(concat!(env!("OUT_DIR"), "/annonars.regions.clingen.rs"));
    include!(concat!(
        env!("OUT_DIR"),
        "/annonars.regions.clingen.serde.rs"
    ));
}
