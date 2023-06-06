#!/usr/bin/env bash

set -euo pipefail
set -x

if [[ "${EXTRACT-false}" == "true" ]]; then
    EXOMES=/data/sshfs/data/cephfs-1/work/projects/cubi_varfish_data/2023-05-11_varfish-db-downloader-bollonaster/varfish-db-downloader/annos/grch37/gnomad_exomes/download
    tabix --print-header \
        $EXOMES/gnomad.exomes.r2.1.1.sites.1.vcf.bgz \
        1:55505599-55516888 \
    > tests/freqs/gnomad-exomes.1.vcf
    bgzip -c tests/freqs/gnomad-exomes.1.vcf \
    > tests/freqs/gnomad-exomes.1.vcf.bgz
    tabix -f tests/freqs/gnomad-exomes.1.vcf.bgz

    tabix --print-header \
        $EXOMES/gnomad.exomes.r2.1.1.sites.X.vcf.bgz \
        X:69902557-69902557 \
    > tests/freqs/gnomad-exomes.X.vcf
    bgzip -c tests/freqs/gnomad-exomes.X.vcf \
    > tests/freqs/gnomad-exomes.X.vcf.bgz
    tabix -f tests/freqs/gnomad-exomes.X.vcf.bgz

    tabix --print-header \
        $EXOMES/gnomad.exomes.r2.1.1.sites.Y.vcf.bgz \
        Y:4967199-4967199 \
    > tests/freqs/gnomad-exomes.Y.vcf
    bgzip -c tests/freqs/gnomad-exomes.Y.vcf \
    > tests/freqs/gnomad-exomes.Y.vcf.bgz
    tabix -f tests/freqs/gnomad-exomes.Y.vcf.bgz

    GENOMES=/data/sshfs/data/cephfs-1/work/projects/cubi_varfish_data/2023-05-11_varfish-db-downloader-bollonaster/varfish-db-downloader/annos/grch37/gnomad_genomes/download
    tabix --print-header \
        $GENOMES/gnomad.genomes.r2.1.1.sites.1.vcf.bgz \
        1:55505599-55505599 \
    > tests/freqs/gnomad-genomes.1.vcf
    bgzip -c tests/freqs/gnomad-genomes.1.vcf \
    > tests/freqs/gnomad-genomes.1.vcf.bgz
    tabix -f tests/freqs/gnomad-genomes.1.vcf.bgz

    tabix --print-header \
        $GENOMES/gnomad.genomes.r2.1.1.sites.X.vcf.bgz \
        X:69902634-69902634 \
    > tests/freqs/gnomad-genomes.X.vcf
    bgzip -c tests/freqs/gnomad-genomes.X.vcf \
    > tests/freqs/gnomad-genomes.X.vcf.bgz
    tabix -f tests/freqs/gnomad-genomes.X.vcf.bgz
fi

rm -rf tests/freqs/example/freqs.db
cargo run --all-features -- \
    freqs import \
    -vvv \
    --genome-release grch37 \
    --path-out-rocksdb tests/freqs/example/freqs.db \
    \
    --gnomad-exomes-version "2.1.1" \
    --path-gnomad-exomes-auto tests/freqs/gnomad-exomes.1.vcf.bgz \
    --path-gnomad-exomes-xy tests/freqs/gnomad-exomes.X.vcf.bgz \
    --path-gnomad-exomes-xy tests/freqs/gnomad-exomes.Y.vcf.bgz \
    \
    --gnomad-genomes-version "2.1.1" \
    --path-gnomad-genomes-auto tests/freqs/gnomad-genomes.1.vcf.bgz \
    --path-gnomad-genomes-xy tests/freqs/gnomad-genomes.X.vcf.bgz \
    \
    --gnomad-mtdna-version "3.1.1" \
    --path-gnomad-mtdna tests/freqs/reading/gnomad.chrM.vcf.bgz \
    \
    --helixmtdb-version "20200327" \
    --path-helixmtdb tests/freqs/reading/helix.chrM.vcf.bgz
