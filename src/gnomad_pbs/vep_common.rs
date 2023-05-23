//! Protocolbuffers for common gnomAD VEP data structures.

include!(concat!(
    env!("OUT_DIR"),
    "/annonars.gnomad.v1.vep_common.rs"
));

impl From<(String, f32)> for Prediction {
    fn from((prediction, score): (String, f32)) -> Self {
        Self { prediction, score }
    }
}
