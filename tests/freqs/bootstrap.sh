#!/usr/bin/env bash

set -euo pipefail
set -x

rm -rf tests/freqs/example/freqs.db
cargo run --all-features -- \
    freqs import \
    --genome-release grch37 \
    --path-out-rocksdb tests/freqs/example/freqs.db \
    --path-gnomad-exomes-auto tests/gnomad-nuclear/example-exomes-grch37/gnomad-exomes.vcf.bgz \
    --path-gnomad-genomes-auto tests/gnomad-nuclear/example-genomes-grch37/gnomad-genomes.vcf.bgz \
    --path-gnomad-mtdna tests/gnomad-mtdna/example/gnomad-mtdna.vcf.bgz \
    --path-helix-mtdb tests/helixmtdb/example/helixmtdb.vcf.bgz
