#!/usr/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

set -x

curl https://ftp.ncbi.nlm.nih.gov/genomes/all/annotation_releases/9606/105.20201022/GCF_000001405.25_GRCh37.p13/GCF_000001405.25_GRCh37.p13_genomic.gff.gz \
| zgrep '^#\|RefSeqFE' \
> $SCRIPT_DIR/GCF_000001405.25_GRCh37.p13_genomic.functional.gff

rm -rf $SCRIPT_DIR/GCF_000001405.25_GRCh37.p13_genomic.db
cargo run --all-features -- \
    functional import -vvv \
    --genome-release grch37 \
    --path-in-gff $SCRIPT_DIR/GCF_000001405.25_GRCh37.p13_genomic.functional.gff \
    --path-out-rocksdb $SCRIPT_DIR/GCF_000001405.25_GRCh37.p13_genomic.db

curl https://ftp.ncbi.nlm.nih.gov/genomes/all/annotation_releases/9606/110/GCF_000001405.40_GRCh38.p14/GCF_000001405.40_GRCh38.p14_genomic.gff.gz \
| zgrep '^#\|RefSeqFE' \
> $SCRIPT_DIR/GCF_000001405.40_GRCh38.p14_genomic.functional.gff

rm -rf $SCRIPT_DIR/GCF_000001405.40_GRCh38.p14_genomic.db
cargo run --all-features -- \
    functional import -vvv \
    --genome-release grch38 \
    --path-in-gff $SCRIPT_DIR/GCF_000001405.40_GRCh38.p14_genomic.functional.gff \
    --path-out-rocksdb $SCRIPT_DIR/GCF_000001405.40_GRCh38.p14_genomic.db
