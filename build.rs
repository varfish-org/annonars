// The custom build script, needed as we use flatbuffers.

fn main() {
    println!("cargo:rerun-if-changed=src/cons/pbs.proto3");
    prost_build::Config::new()
        // Add serde serialization and deserialization to the generated
        // code.
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        // Rename the field attributes such that we can properly decode
        // ucsc-annotation TSV file with serde.
        .field_attribute(
            "annonars.cons.pbs.Record.chrom",
            "#[serde(rename = \"chromosome\")]",
        )
        .field_attribute(
            "annonars.cons.pbs.Record.begin",
            "#[serde(rename = \"start\")]",
        )
        .field_attribute(
            "annonars.cons.pbs.Record.end",
            "#[serde(rename = \"stop\")]",
        )
        .compile_protos(&["src/cons/pbs.proto3"], &["src/"])
        .unwrap();
}
