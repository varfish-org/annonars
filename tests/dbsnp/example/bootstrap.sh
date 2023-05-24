#!/usr/bin/env bash

set -euo pipefail
set -x

if [[ tests/dbsnp/example/dbsnp.brca1.vcf \
        -nt tests/dbsnp/example/dbsnp.brca1.vcf.bgz ]]; then
    bgzip -c tests/dbsnp/example/dbsnp.brca1.vcf \
    > tests/dbsnp/example/dbsnp.brca1.vcf.bgz
    tabix -f tests/dbsnp/example/dbsnp.brca1.vcf.bgz
fi

rm -rf tests/dbsnp/example/dbsnp.brca1.vcf.gz.db
cargo run --all-features -- \
    dbsnp import \
    --genome-release grch37 \
    --path-in-vcf tests/dbsnp/example/dbsnp.brca1.vcf.bgz \
    --path-out-rocksdb tests/dbsnp/example/dbsnp.brca1.vcf.bgz.db
