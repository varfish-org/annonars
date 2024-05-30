#!/usr/bin/env bash

set -euo pipefail
set -x

export TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT

rm -rf tests/clinvar-sv/clinvar-sv-grch37.db
cargo run --all-features -- \
    clinvar-sv import -vvv \
    --genome-release grch37 \
    --path-in-jsonl tests/clinvar-sv/clinvar-variants-grch37-seqvars.jsonl \
    --path-in-jsonl tests/clinvar-sv/clinvar-variants-grch37-strucvars.jsonl \
    --path-out-rocksdb tests/clinvar-sv/clinvar-sv-grch37.db
