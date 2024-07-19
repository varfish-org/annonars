//! Code generate for protobufs by `prost-build`.

/// Code generate for protobufs by `prost-build`.
pub mod interface {
    include!(concat!(env!("OUT_DIR"), "/annonars.server.interface.rs"));
    include!(concat!(
        env!("OUT_DIR"),
        "/annonars.server.interface.serde.rs"
    ));
}
