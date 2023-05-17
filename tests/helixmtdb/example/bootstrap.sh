#!/usr/bin/env bash

set -euo pipefail
set -x

rm -rf tests/gnomad_mtdna/example/gnomad_mtdna.vcf.bgz.db
cargo run --all-features -- \
    gnomad-mtdna import \
    --genome-release grch37 \
    --path-in-vcf tests/gnomad_mtdna/example/gnomad_mtdna.vcf.bgz \
    --path-out-rocksdb tests/gnomad_mtdna/example/gnomad_mtdna.vcf.bgz.db
rm -f tests/gnomad_mtdna/example/gnomad_mtdna.vcf.bgz.db/*.log
