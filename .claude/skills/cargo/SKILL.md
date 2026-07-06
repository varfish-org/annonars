---
name: cargo
description: Apply for build/test/lint, dependency or feature changes, and editing Cargo.toml or .cargo config in this repo.
---

# Cargo & manifest conventions

## Layout
- **Single crate** (no `[workspace]`), lib + bin (`autobins = false`). CI still passes `--workspace` — keep using it in commands.

## Commands
- Build `cargo build` · test `cargo test --all-features` · format `cargo fmt` (check `cargo fmt -- --check`) · lint `cargo clippy --all-features --workspace -- -D warnings` · coverage `cargo llvm-cov --all-features --workspace`.
- Run the binary: `cargo run -- <args>`.
- Requires `protoc` + system `librocksdb-dev libsnappy-dev libsqlite3-dev`.

## Features
- `default = ["jemalloc", "server"]`. `server` gates actix/utoipa; `jemalloc` the allocator.

## Manifest (TOML) conventions
- Keep `[dependencies]` alphabetically ordered (match existing).
- `.cargo/config.toml` sets `ROCKSDB_LIB_DIR`/`SNAPPY_LIB_DIR=/usr/lib/` and a thumbv7em linker — don't remove/break these.
- release-please owns the `version` field (and the `version:` in `openapi.schema.yaml`) — don't bump by hand.
