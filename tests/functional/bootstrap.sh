#!/usr/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

set -x

curl https://ftp.ncbi.nlm.nih.gov/genomes/all/annotation_releases/9606/105.20201022/GCF_000001405.25_GRCh37.p13/GCF_000001405.25_GRCh37.p13_genomic.gff.gz \
| zgrep '^#\|RefSeqFE' \
> $SCRIPT_DIR/GCF_000001405.25_GRCh37.p13_genomic.functional.gff

curl https://ftp.ncbi.nlm.nih.gov/genomes/all/annotation_releases/9606/110/GCF_000001405.40_GRCh38.p14/GCF_000001405.40_GRCh38.p14_genomic.gff.gz \
| zgrep '^#\|RefSeqFE' \
> $SCRIPT_DIR/GCF_000001405.40_GRCh38.p14_genomic.functional.gff
