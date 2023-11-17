#!/usr/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

set -x

# rm -f $SCRIPT_DIR/*.vcf*

filenames="
gnomad.v4.0.sv.chr1.vcf.gz
gnomad.v4.0.sv.chr2.vcf.gz
"

for f in $filenames; do
    curl \
        https://storage.googleapis.com/gcp-public-data--gnomad/release/4.0/genome_sv/$f \
    | zcat \
    | head -n 2000 \
    > $SCRIPT_DIR/${f%.gz}

    bgzip -c $SCRIPT_DIR/${f%.gz} \
    > $SCRIPT_DIR/$f

    tabix -f $SCRIPT_DIR/$f
done

rm -rf $SCRIPT_DIR/rocksdb

cargo run -- \
    gnomad-sv import \
    $(for f in $filenames; do \
        echo --path-in-vcf $SCRIPT_DIR/$f; \
    done) \
    --path-out-rocksdb $SCRIPT_DIR/rocksdb \
    --gnomad-kind genomes \
    --genome-release grch38 \
    --gnomad-version 4.0
