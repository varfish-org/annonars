#!/usr/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

set -x

filenames="
gnomad_v2.1_sv.sites.vcf.gz
gnomad_v2.1_sv.nonneuro.sites.vcf.gz
gnomad_v2.1_sv.controls_only.sites.vcf.gz
"

rm -f $SCRIPT_DIR/*.vcf*

for f in $filenames; do
    curl https://storage.googleapis.com/gcp-public-data--gnomad/papers/2019-sv/$f \
    | zcat \
    | head -n 350 \
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
    --genome-release grch37 \
    --gnomad-version 2.1
