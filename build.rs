// The custo build script, used to (1) generate the Rust classes for the
// protobuf implementation and (2) use pbjson for proto3 JSON serialization.

use std::{env, path::PathBuf};

fn main() -> Result<(), anyhow::Error> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("protos");
    let proto_files = vec![
        "annonars/clinvar/minimal.proto",
        "annonars/clinvar/per_gene.proto",
        "annonars/clinvar/sv.proto",
        "annonars/cons/base.proto",
        "annonars/dbsnp/base.proto",
        "annonars/functional/refseq.proto",
        "annonars/genes/base.proto",
        "annonars/gnomad/exac_cnv.proto",
        "annonars/gnomad/gnomad2.proto",
        "annonars/gnomad/gnomad3.proto",
        "annonars/gnomad/gnomad_cnv4.proto",
        "annonars/gnomad/gnomad_sv2.proto",
        "annonars/gnomad/gnomad_sv4.proto",
        "annonars/gnomad/mtdna.proto",
        "annonars/gnomad/vep_common.proto",
        "annonars/gnomad/vep_gnomad2.proto",
        "annonars/gnomad/vep_gnomad3.proto",
        "annonars/helixmtdb/base.proto",
    ]
    .iter()
    .map(|f| root.join(f))
    .collect::<Vec<_>>();

    // Tell cargo to recompile if any of these proto files are changed
    for proto_file in &proto_files {
        println!("cargo:rerun-if-changed={}", proto_file.display());
    }

    let descriptor_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("proto_descriptor.bin");

    prost_build::Config::new()
        // Save descriptors to file
        .file_descriptor_set_path(&descriptor_path)
        // Override prost-types with pbjson-types
        .compile_well_known_types()
        .extern_path(".google.protobuf", "::pbjson_types")
        // Define the protobuf files to compile.
        .compile_protos(&proto_files, &[root])?;

    let descriptor_set = std::fs::read(descriptor_path).unwrap();
    pbjson_build::Builder::new()
        .register_descriptors(&descriptor_set)?
        .build(&[".annonars"])?;

    Ok(())
}
