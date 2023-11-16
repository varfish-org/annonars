#!/usr/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

export TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT

cd $SCRIPT_DIR

wget ftp://ftp.broadinstitute.org/pub/ExAC_release/release0.3.1/cnv/exac-final.autosome-1pct-sq60-qc-prot-coding.cnv.bed \
    -O $TMPDIR/exac-final.autosome-1pct-sq60-qc-prot-coding.cnv.bed

grep --no-group-separator -A 10 ^track \
    $TMPDIR/exac-final.autosome-1pct-sq60-qc-prot-coding.cnv.bed \
> $SCRIPT_DIR/exac-final.autosome-1pct-sq60-qc-prot-coding.cnv.bed

rm -rf $SCRIPT_DIR/rocksdb

cargo run -- \
    gnomad-sv import \
    --path-in-vcf $SCRIPT_DIR/exac-final.autosome-1pct-sq60-qc-prot-coding.cnv.bed \
    --path-out-rocksdb $SCRIPT_DIR/rocksdb \
    --gnomad-kind exomes \
    --genome-release grch37 \
    --gnomad-version 1.0
