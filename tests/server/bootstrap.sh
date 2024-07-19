#!/usr/bin/bash

# Download annonars file reduced to "for dev" subset.

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

REDUCED_DEV_PATHS="
cadd-grch37-1.6+0.39.0
cadd-grch38-1.6+0.39.0
dbnsfp-grch37-4.5a+0.39.0
dbnsfp-grch37-4.5c+0.39.0
dbnsfp-grch38-4.5a+0.39.0
dbnsfp-grch38-4.5c+0.39.0
dbsnp-grch37-b151+0.39.0
dbsnp-grch38-b151+0.39.0
gnomad-exomes-grch37-2.1.1+0.39.0
gnomad-exomes-grch38-4.1+0.39.0
gnomad-genomes-grch37-2.1.1+0.39.0
gnomad-genomes-grch38-4.1+0.39.0
"
FULL_PATHS="
alphamissense-grch37-1+0.33.0
alphamissense-grch38-1+0.33.0
functional-grch37-105.20201022+0.33.0
functional-grch38-110+0.33.0
genes-3.1+4.0+4.5+20230606+10.1+20240105+0.33.0
gnomad-mtdna-grch37-3.1+0.33.0
gnomad-mtdna-grch38-3.1+0.33.0
helixmtdb-grch37-20200327+0.33.0
helixmtdb-grch38-20200327+0.33.0
"

mkdir -p $SCRIPT_DIR/annonars

for path in $REDUCED_DEV_PATHS; do
    s5cmd \
        --endpoint-url https://ceph-s3-ext.cubi.bihealth.org \
        --no-sign-request \
        sync "s3://varfish-public/reduced-dev/annonars/$path/*" \
        $SCRIPT_DIR/annonars/$path/
done

for path in $FULL_PATHS; do
    s5cmd \
        --endpoint-url https://ceph-s3-ext.cubi.bihealth.org \
        --no-sign-request \
        sync "s3://varfish-public/full/annonars/$path/*" \
        $SCRIPT_DIR/annonars/$path/
done
