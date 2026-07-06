---
name: testing
description: Apply when writing or running tests, or updating insta snapshots, in this repo.
---

# Testing

- Run: `cargo test --all-features`. Prefer red-green (see **karpathy-principle**).
- **insta** snapshots (YAML): after intended output changes, review with `cargo insta review` (or `cargo insta accept`). Never blindly accept. Snapshots live in per-module `snapshots/` dirs.
- **Selective Git LFS:** only `src/gnomad_sv/cli/snapshots/*.snap` is LFS-tracked; other `.snap` files are plain git. Have `git lfs` installed so those resolve.
- `rstest` for parameterized tests/fixtures; `test-log` + `tracing-test` to capture/assert on logs; `temp_testdir`/`tempfile` for scratch dirs; `pretty_assertions` for diffs.
- Integration tests under top-level `tests/` mirror the module list.
