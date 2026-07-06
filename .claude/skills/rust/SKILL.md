---
name: rust
description: Apply when writing or editing Rust code in this crate — idioms, error handling, logging, docs, and lint rules.
---

# Rust conventions

- **Edition 2021.**
- **Errors:** `anyhow::Result` for application/CLI paths (add `.context(...)`); `thiserror` for library error types (central `src/error.rs` `Error` enum, `#[from]`/`#[source]`).
- **No panics in library code:** no `unwrap`/`expect`/`panic!`/`todo!` on reachable paths — return `Result`. Tests may `unwrap`.
- **Docs:** `#![warn(missing_docs)]` is on — every public item needs a doc comment.
- **Logging:** `tracing` spans/events, not `println!`. Verbosity via `clap-verbosity-flag`.
- **Parallelism:** `rayon` iterators for data-parallel imports (`RAYON_NUM_THREADS` honored).
- **Lints are gated:** CI runs `cargo clippy --all-features --workspace -- -D warnings` — warnings fail the build. Run `cargo fmt` before finishing.
- Prefer crates already in the tree (itertools, indexmap, rustc-hash, strum, enum-map) over hand-rolling.
