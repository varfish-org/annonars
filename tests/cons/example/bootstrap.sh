#!/usr/bin/env bash

set -euo pipefail
set -x

rm -rf tests/cons/example/tgds.tsv.db
cargo run --all-features -- \
    cons import \
    --genome-release grch37 \
    --path-in-tsv tests/cons/example/tgds.tsv \
    --path-out-rocksdb tests/cons/example/tgds.tsv.db
rm -f tests/cons/example/tgds.tsv.db/*.log
