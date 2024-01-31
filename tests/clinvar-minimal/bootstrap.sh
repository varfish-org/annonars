#!/usr/bin/env bash

set -euo pipefail
set -x

export TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT

base=https://github.com/varfish-org/clinvar-data-jsonl/releases/download/clinvar-weekly-20231015
wget -O $TMPDIR/clinvar-data-extract-vars-20231015+0.12.0.tar.gz \
    $base/clinvar-data-extract-vars-20231015+0.12.0.tar.gz
tar -C $TMPDIR -xf $TMPDIR/clinvar-data-extract-vars-20231015+0.12.0.tar.gz
set +o pipefail
zcat $TMPDIR/clinvar-data-extract-vars-20231015+0.12.0/clinvar-variants-grch37-seqvars.jsonl.gz \
| egrep "pathogenic|benign|uncertain" \
| grep -w HGNC:20324 \
> $TMPDIR/clinvar-variants-grch37-seqvars.jsonl
set -o pipefail

rm -rf tests/clinvar-minimal/clinvar-seqvars-grch37-tgds.tsv.db
cargo run --all-features -- \
    clinvar-minimal import -vvv \
    --genome-release grch37 \
    --path-in-jsonl $TMPDIR/clinvar-variants-grch37-seqvars.jsonl \
    --path-out-rocksdb tests/clinvar-minimal/clinvar-seqvars-grch37-tgds.tsv.db
