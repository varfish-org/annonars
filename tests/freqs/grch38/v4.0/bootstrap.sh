#!/usr/bin/env bash

set -euo pipefail
set -x

if [[ "${EXTRACT-false}" == "true" ]]; then
    EXOMES=https://storage.googleapis.com/gcp-public-data--gnomad/release/4.0/vcf/exomes
    tabix --print-header \
        $EXOMES/gnomad.exomes.v4.0.sites.chr1.vcf.bgz \
        chr1:55039893-55039951 \
    > tests/freqs/grch38/v4.0/gnomad-exomes.1.vcf
    bgzip -c tests/freqs/grch38/v4.0/gnomad-exomes.1.vcf \
    > tests/freqs/grch38/v4.0/gnomad-exomes.1.vcf.bgz
    tabix -f tests/freqs/grch38/v4.0/gnomad-exomes.1.vcf.bgz

    tabix --print-header \
        $EXOMES/gnomad.exomes.v4.0.sites.chrX.vcf.bgz \
        chrX:253600-255602 \
    > tests/freqs/grch38/v4.0/gnomad-exomes.X.vcf
    bgzip -c tests/freqs/grch38/v4.0/gnomad-exomes.X.vcf \
    > tests/freqs/grch38/v4.0/gnomad-exomes.X.vcf.bgz
    tabix -f tests/freqs/grch38/v4.0/gnomad-exomes.X.vcf.bgz

    tabix --print-header \
        $EXOMES/gnomad.exomes.v4.0.sites.chrY.vcf.bgz \
        chrY:2786700-2796709 \
    > tests/freqs/grch38/v4.0/gnomad-exomes.Y.vcf
    bgzip -c tests/freqs/grch38/v4.0/gnomad-exomes.Y.vcf \
    > tests/freqs/grch38/v4.0/gnomad-exomes.Y.vcf.bgz
    tabix -f tests/freqs/grch38/v4.0/gnomad-exomes.Y.vcf.bgz

    GENOMES=https://storage.googleapis.com/gcp-public-data--gnomad/release/4.0/vcf/genomes
    tabix --print-header \
        $GENOMES/gnomad.genomes.v4.0.sites.chr1.vcf.bgz \
        chr1:55039893-55039951 \
    > tests/freqs/grch38/v4.0/gnomad-genomes.1.vcf
    bgzip -c tests/freqs/grch38/v4.0/gnomad-genomes.1.vcf \
    > tests/freqs/grch38/v4.0/gnomad-genomes.1.vcf.bgz
    tabix -f tests/freqs/grch38/v4.0/gnomad-genomes.1.vcf.bgz

    tabix --print-header \
        $GENOMES/gnomad.genomes.v4.0.sites.chrX.vcf.bgz \
        chrX:10000-20000 \
    > tests/freqs/grch38/v4.0/gnomad-genomes.X.vcf
    bgzip -c tests/freqs/grch38/v4.0/gnomad-genomes.X.vcf \
    > tests/freqs/grch38/v4.0/gnomad-genomes.X.vcf.bgz
    tabix -f tests/freqs/grch38/v4.0/gnomad-genomes.X.vcf.bgz

    tabix --print-header \
        $GENOMES/gnomad.genomes.v4.0.sites.chrY.vcf.bgz \
        chrY:2781500-2790000 \
    > tests/freqs/grch38/v4.0/gnomad-genomes.Y.vcf
    bgzip -c tests/freqs/grch38/v4.0/gnomad-genomes.Y.vcf \
    > tests/freqs/grch38/v4.0/gnomad-genomes.Y.vcf.bgz
    tabix -f tests/freqs/grch38/v4.0/gnomad-genomes.Y.vcf.bgz
fi

rm -rf tests/freqs/grch38/v4.0/example/freqs.db
cargo run --all-features -- \
    freqs import \
    -vvv \
    --genome-release grch38 \
    --path-out-rocksdb tests/freqs/grch38/v4.0/example/freqs.db \
    \
    --gnomad-exomes-version "4.0" \
    --path-gnomad-exomes-auto tests/freqs/grch38/v4.0/gnomad-exomes.1.vcf.bgz \
    --path-gnomad-exomes-xy tests/freqs/grch38/v4.0/gnomad-exomes.X.vcf.bgz \
    --path-gnomad-exomes-xy tests/freqs/grch38/v4.0/gnomad-exomes.Y.vcf.bgz \
    \
    --gnomad-genomes-version "4.0" \
    --path-gnomad-genomes-auto tests/freqs/grch38/v4.0/gnomad-genomes.1.vcf.bgz \
    --path-gnomad-genomes-xy tests/freqs/grch38/v4.0/gnomad-genomes.X.vcf.bgz \
    --path-gnomad-genomes-xy tests/freqs/grch38/v4.0/gnomad-genomes.Y.vcf.bgz \
    \
    --gnomad-mtdna-version "3.1.1" \
    --path-gnomad-mtdna tests/freqs/grch37/v2.1/reading/gnomad.chrM.vcf.bgz \
    \
    --helixmtdb-version "20200327" \
    --path-helixmtdb tests/freqs/grch37/v2.1/reading/helix.chrM.vcf.bgz
