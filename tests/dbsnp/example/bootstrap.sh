#!/usr/bin/env bash

set -euo pipefail
set -x

rm -rf tests/dbsnp/example/dbsnp.brca1.vcf.gz.db
cargo run --all-features -- \
    dbsnp import \
    --genome-release grch37 \
    --path-in-tsv tests/dbsnp/example/dbsnp.brca1.vcf.gz \
    --path-out-rocksdb tests/dbsnp/example/dbsnp.brca1.vcf.gz.db
rm -f tests/dbsnp/example/dbsnp.brca1.vcf.gz.db/*.log
