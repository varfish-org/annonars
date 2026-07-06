# annonars — agent guide

Genome annotation on Rust + RocksDB: import curated datasets (ClinVar, gnomAD, dbSNP, CADD/dbNSFP TSVs, conservation, HelixMtDb, gene info, ClinGen regions) into RocksDB keyed by SPDI, and query them via CLI + REST. Org `varfish-org`, license Apache-2.0.

## Layout
- **Single crate** (not a workspace), lib + bin. CI still invokes `--workspace`.
- Per-dataset module shape: `mod.rs` + `cli/{import,query}.rs` (+ optional `pbs.rs`, `snapshots/`). Shared code in `src/common/` (`keys.rs`, `spdi.rs`, `noodles.rs`); generated protobuf in `src/pbs/`; REST in `src/server/`; central errors in `src/error.rs`.
- CLI subcommands per dataset plus `tsv` and `server`.

## Build / test / lint (real commands)
- Prereqs: `protoc`; system `librocksdb-dev libsnappy-dev libsqlite3-dev`. `.cargo/config.toml` points RocksDB/snappy at `/usr/lib/` — don't break it.
- Build `cargo build` · test `cargo test --all-features` · format `cargo fmt -- --check` · lint `cargo clippy --all-features --workspace -- -D warnings` · coverage `cargo llvm-cov --all-features --workspace`.
- Run: `cargo run -- <args>`.

## House conventions
- `anyhow` for app/CLI paths, `thiserror` for library error types (`src/error.rs`). `tracing` (not `println!`). `rayon` for data parallelism. No `unwrap`/`expect`/`panic!` in library code.
- Edition 2021. `#![warn(missing_docs)]` is on — document every public item. **CI fails on clippy warnings** (`-D warnings`).
- release-please owns `CHANGELOG.md` and version fields (incl. the `version:` in `openapi.schema.yaml`) — never hand-edit.

## Skills (`.claude/skills/`) — load the matching one before the task
- **karpathy-principle** — always, when writing/editing code.
- **rust** — Rust idioms & error handling.
- **cargo** — build/test/deps/features/manifests.
- **protobuf** — editing `.proto` or generated types.
- **openapi** — changing the REST API surface.
- **contributing** — branching, commits, `gh`, PRs.
- **issue-workflow** — creating/tracking issues, multi-step plans.
- **testing** — tests & insta snapshots.
- **rocksdb** — storage, keys, import/query paths.
