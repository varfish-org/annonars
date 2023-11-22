#!/usr/bin/env bash

set -euo pipefail
set -x

if [[ tests/gnomad-nuclear/example-genomes/gnomad-genomes.vcf \
        -nt tests/gnomad-nuclear/example-genomes/gnomad-genomes.vcf.bgz ]]; then
    bgzip -c tests/gnomad-nuclear/example-genomes/gnomad-genomes.vcf \
    > tests/gnomad-nuclear/example-genomes/gnomad-genomes.vcf.bgz
    tabix -f tests/gnomad-nuclear/example-genomes/gnomad-genomes.vcf.bgz
fi

rm -rf tests/helixmtdb/example/helixmtdb.vcf.bgz.db
cargo run --all-features -- \
    helixmtdb import \
    --genome-release grch37 \
    --path-in-vcf tests/helixmtdb/example/helixmtdb.vcf.bgz \
    --path-out-rocksdb tests/helixmtdb/example/helixmtdb.vcf.bgz.db
