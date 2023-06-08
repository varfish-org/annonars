#!/usr/bin/env bash

set -euo pipefail
set -x

rm -rf tests/clinvar-minimal/clinvar-seqvars-grch37-tgds.tsv.db
cargo run --all-features -- \
    clinvar-minimal import -vvv \
    --genome-release grch37 \
    --path-in-tsv tests/clinvar-minimal/clinvar-seqvars-grch37-tgds.tsv \
    --path-out-rocksdb tests/clinvar-minimal/clinvar-seqvars-grch37-tgds.tsv.db
