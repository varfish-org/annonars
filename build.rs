// The custom build script, needed as we use flatbuffers.

fn main() {
    println!("cargo:rerun-if-changed=src/cons/pbs.proto3");
    prost_build::compile_protos(&["src/cons/pbs.proto3"], &["src/"]).unwrap();
}
