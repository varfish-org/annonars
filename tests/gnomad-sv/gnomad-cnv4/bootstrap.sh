#!/usr/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

set -x

export TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT

rm -f $SCRIPT_DIR/*.vcf*

for token in all non_neuro non_neuro_controls; do
    curl \
        https://storage.googleapis.com/gcp-public-data--gnomad/release/4.0/exome_cnv/gnomad.v4.0.cnv.$token.vcf.gz \
    | zcat \
    | head -n 200 \
    > $SCRIPT_DIR/gnomad.v4.0.cnv.$token.vcf

    bgzip -c $SCRIPT_DIR/gnomad.v4.0.cnv.$token.vcf \
    > $SCRIPT_DIR/gnomad.v4.0.cnv.$token.vcf.gz

    tabix -f $SCRIPT_DIR/gnomad.v4.0.cnv.$token.vcf.gz

    rm -rf $SCRIPT_DIR/rocksdb
done

cargo run -- \
    gnomad-sv import \
    --path-in-vcf $SCRIPT_DIR/gnomad.v4.0.cnv.all.vcf.gz \
    --path-in-vcf $SCRIPT_DIR/gnomad.v4.0.cnv.non_neuro.vcf.gz \
    --path-in-vcf $SCRIPT_DIR/gnomad.v4.0.cnv.non_neuro_controls.vcf.gz \
    --path-out-rocksdb $SCRIPT_DIR/rocksdb \
    --gnomad-kind exomes \
    --genome-release grch38 \
    --gnomad-version 4.0
