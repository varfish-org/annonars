//! Code generate for protobufs by `prost-build`.

pub mod base {
    //! Records for sequence variant annotation tracks and the unified record.
    include!(concat!(env!("OUT_DIR"), "/annonars.seqvars.base.rs"));
    include!(concat!(env!("OUT_DIR"), "/annonars.seqvars.base.serde.rs"));
}
