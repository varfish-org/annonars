#!/usr/bin/env bash

set -euo pipefail
set -x

export TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT

base=https://github.com/varfish-org/clinvar-data-jsonl/releases/download/clinvar-weekly-20240528
wget -O $TMPDIR/clinvar-data-extract-vars-20240528+0.15.5.tar.gz \
    $base/clinvar-data-extract-vars-20240528+0.15.5.tar.gz
tar -C $TMPDIR -xf $TMPDIR/clinvar-data-extract-vars-20240528+0.15.5.tar.gz
set +o pipefail
zgrep -w HGNC:20324 $TMPDIR/clinvar-data-extract-vars-20240528+0.15.5/clinvar-variants-grch37-seqvars.jsonl.gz \
> tests/clinvar-minimal/clinvar-seqvars-grch37-tgds.jsonl
set -o pipefail

rm -rf tests/clinvar-minimal/clinvar-seqvars-grch37-tgds.db
cargo run --all-features -- \
    clinvar-minimal import -vvv \
    --genome-release grch37 \
    --path-in-jsonl tests/clinvar-minimal/clinvar-seqvars-grch37-tgds.jsonl \
    --path-out-rocksdb tests/clinvar-minimal/clinvar-seqvars-grch37-tgds.db
