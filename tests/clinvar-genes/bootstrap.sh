#!/usr/bin/env bash

set -euo pipefail
set -x

export TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT

rm -rf tests/clinvar-genes/clinvar-genes.db
cargo run --all-features -- clinvar-genes import \
    --path-per-impact-jsonl tests/clinvar-genes/gene-variant-report.jsonl \
    --path-per-frequency-jsonl tests/clinvar-genes/gene-frequency-report.jsonl \
    --paths-variant-jsonl tests/clinvar-genes/clinvar-variants-grch37-seqvars.jsonl \
    --paths-variant-jsonl tests/clinvar-genes/clinvar-variants-grch38-seqvars.jsonl \
    --path-out-rocksdb tests/clinvar-genes/clinvar-genes.db
