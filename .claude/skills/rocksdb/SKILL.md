---
name: rocksdb
description: Apply when working with RocksDB storage — importing datasets, key encoding, column families, or query paths.
---

# RocksDB

- `rocksdb` with `multi-threaded-cf`; each dataset is its own DB, values are prost-encoded protobuf (see **protobuf**).
- **Keys:** encoded in `src/common/keys.rs`; variants keyed by **SPDI** coordinates (`src/common/spdi.rs`, 1-based inclusive). Reuse these helpers — don't hand-encode keys.
- Column families and per-dataset metadata are written at import time; `rocksdb-utils-lookup` provides shared read helpers.
- RocksDB/snappy are bundled with `librocksdb-sys` and built from source. Don't link a system RocksDB: it may not read databases written by the bundled one (`format_version` 7).
- Adding a dataset: follow the module shape `mod.rs` + `cli/{import,query}.rs` (+ optional `pbs.rs`, `snapshots/`).
- Renumbering/retyping protobuf fields breaks already-built databases — treat as a data migration.
