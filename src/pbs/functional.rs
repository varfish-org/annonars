//! Code generate for protobufs by `prost-build`.

/// Code generate for protobufs by `prost-build`.
pub mod refseq {
    include!(concat!(env!("OUT_DIR"), "/annonars.functional.refseq.rs"));
    include!(concat!(
        env!("OUT_DIR"),
        "/annonars.functional.refseq.serde.rs"
    ));
}
