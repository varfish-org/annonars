#!/usr/bin/env bash

set -euo pipefail
set -x

if [[ tests/gnomad-nuclear/example-exomes/gnomad-exomes.vcf \
        -nt tests/gnomad-nuclear/example-exomes/gnomad-exomes.vcf.bgz ]]; then
    bgzip -c tests/gnomad-nuclear/example-exomes/gnomad-exomes.vcf \
    > tests/gnomad-nuclear/example-exomes/gnomad-exomes.vcf.bgz
    tabix -f tests/gnomad-nuclear/example-exomes/gnomad-exomes.vcf.bgz
fi

rm -rf tests/gnomad-nuclear/example-exomes/gnomad-exomes.vcf.bgz.db
cargo run --all-features -- \
    gnomad-mtdna import \
    --import-fields-json '{
        "vep": true,
    }' \
    --genome-release grch37 \
    --gnomad-kind exomes \
    --path-in-vcf tests/gnomad-nuclear/example-exomes/gnomad-exomes.vcf.bgz \
    --path-out-rocksdb tests/gnomad-nuclear/example-exomes/gnomad-exomes.vcf.bgz.db
rm -f tests/gnomad-nuclear/example-exomes/gnomad-exomes.vcf.bgz.db/*.log
