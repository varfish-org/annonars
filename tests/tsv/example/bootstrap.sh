#!/usr/bin/env bash

set -euo pipefail
set -x

rm -rf tests/tsv/example/data.tsv.gz.db
cargo run --all-features -- \
    tsv import \
    --add-default-null-values \
    --genome-release grch37 \
    --path-in-tsv tests/tsv/example/data.tsv.gz \
    --path-out-rocksdb tests/tsv/example/data.tsv.gz.db \
    --db-name example-tsv \
    --db-version 1.0 \
    --col-chrom CHROM \
    --col-start POS \
    --col-ref REF \
    --col-alt ALT
