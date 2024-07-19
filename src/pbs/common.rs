//! Code generate for protobufs by `prost-build`.

/// Information about versions.
pub mod versions {
    include!(concat!(env!("OUT_DIR"), "/annonars.common.versions.rs"));
    include!(concat!(
        env!("OUT_DIR"),
        "/annonars.common.versions.serde.rs"
    ));
}
