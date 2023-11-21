#!/usr/bin/env bash

rm -rf tests/regions/clingen/rocksdb

cargo run -- regions import \
    --genome-release grch37 \
    --path-in-clingen tests/regions/clingen/ClinGen_region_curation_list_GRCh37.tsv \
    --path-out-rocksdb tests/regions/clingen/rocksdb
