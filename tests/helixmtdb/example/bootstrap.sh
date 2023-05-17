#!/usr/bin/env bash

set -euo pipefail
set -x

if [[ tests/helixmtdb/example/helixmtdb.vcf.bgz \
        -nt tests/helixmtdb/example/helixmtdb.vcf ]]; then
    bgzip -c tests/helixmtdb/example/helixmtdb.vcf \
    > tests/helixmtdb/example/helixmtdb.vcf.bgz
    tabix -f tests/helixmtdb/example/helixmtdb.vcf.bgz
fi

rm -rf tests/helixmtdb/example/helixmtdb.vcf.bgz.db
cargo run --all-features -- \
    helixmtdb import \
    --genome-release grch37 \
    --path-in-vcf tests/helixmtdb/example/helixmtdb.vcf.bgz \
    --path-out-rocksdb tests/helixmtdb/example/helixmtdb.vcf.bgz.db
rm -f tests/helixmtdb/example/helixmtdb.vcf.bgz.db/*.log
