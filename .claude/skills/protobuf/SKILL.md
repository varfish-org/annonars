---
name: protobuf
description: Apply when editing .proto files or the generated protobuf types (src/pbs/*) in this repo.
---

# Protobuf

- Schemas: 26 `.proto` files under `protos/annonars/**` (packages like `annonars.genes.base`, `annonars.gnomad.gnomad4`, `clinvar_data`), proto3.
- Codegen at build time via the top-level `build.rs`: `prost-build` + `pbjson-build` (serde/JSON for `.annonars` and `.clinvar_data`). Well-known types map to `pbjson_types`; a descriptor set goes to `OUT_DIR`.
- Generated Rust is `include!`d through `src/pbs/*.rs` (e.g. `src/pbs/gnomad/gnomad4.rs`). Don't edit generated code — edit the `.proto` and rebuild (`cargo build`, needs `protoc`).

## Safe-edit rules (wire + on-disk compatibility)
- **Never renumber or reuse an existing field number.** Add new fields with new numbers.
- Add fields; don't mutate/repurpose existing ones. To retire a field, `reserved` its number/name.
- Changing a field's type or label is breaking — add a new field instead.
- Protobuf messages are the **RocksDB value format** (prost-encoded). Renumbering/retyping breaks already-built databases — treat as a data migration.
