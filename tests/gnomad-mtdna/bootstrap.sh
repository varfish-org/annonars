#!/usr/bin/env bash

set -euo pipefail
set -x

if [[ tests/gnomad-mtdna/example/gnomad-mtdna.vcf.bgz \
        -nt tests/gnomad-mtdna/example/gnomad-mtdna.vcf ]]; then
    bgzip -c tests/gnomad-mtdna/example/gnomad-mtdna.vcf \
    > tests/gnomad-mtdna/example/gnomad-mtdna.vcf.bgz
    tabix -f tests/gnomad-mtdna/example/gnomad-mtdna.vcf.bgz
fi

rm -rf tests/gnomad-mtdna/example/gnomad-mtdna.vcf.bgz.db
cargo run --all-features -- \
    gnomad-mtdna import \
    --genome-release grch37 \
    --path-in-vcf tests/gnomad-mtdna/example/gnomad-mtdna.vcf.bgz \
    --path-out-rocksdb tests/gnomad-mtdna/example/gnomad-mtdna.vcf.bgz.db
rm -f tests/gnomad-mtdna/example/gnomad-mtdna.vcf.bgz.db/*.log
