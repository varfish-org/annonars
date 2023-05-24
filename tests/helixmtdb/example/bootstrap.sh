#!/usr/bin/env bash

set -euo pipefail
set -x

if [[ tests/gnomad-nuclear/example-genomes/gnomad-genomes.vcf \
        -nt tests/gnomad-nuclear/example-genomes/gnomad-genomes.vcf.bgz ]]; then
    bgzip -c tests/gnomad-nuclear/example-genomes/gnomad-genomes.vcf \
    > tests/gnomad-nuclear/example-genomes/gnomad-genomes.vcf.bgz
    tabix -f tests/gnomad-nuclear/example-genomes/gnomad-genomes.vcf.bgz
fi

rm -rf tests/gnomad-nuclear/example-genomes/gnomad-genomes.vcf.bgz.db
cargo run --all-features -- \
    gnomad-mtdna import \
    --import-fields-json '{
        "vep": true,
    }' \
    --genome-release grch37 \
    --gnomad-kind genomes \
    --path-in-vcf tests/gnomad-nuclear/example-genomes/gnomad-genomes.vcf.bgz \
    --path-out-rocksdb tests/gnomad-nuclear/example-genomes/gnomad-genomes.vcf.bgz.db
