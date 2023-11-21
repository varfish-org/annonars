//! Code generate for protobufs by `prost-build`.

include!(concat!(env!("OUT_DIR"), "/annonars.gnomad.vep_common.rs"));
include!(concat!(
    env!("OUT_DIR"),
    "/annonars.gnomad.vep_common.serde.rs"
));

impl From<(String, f32)> for Prediction {
    fn from((prediction, score): (String, f32)) -> Self {
        Self { prediction, score }
    }
}
