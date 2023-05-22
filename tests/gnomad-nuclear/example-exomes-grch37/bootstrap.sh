#!/usr/bin/env bash

set -euo pipefail
set -x

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

script_dir_name=$(basename $SCRIPT_DIR)
data_kind=$(echo $script_dir_name | cut -d - -f 2)
genome_release=$(echo $script_dir_name | cut -d - -f 3)

if [[ $SCRIPT_DIR/gnomad-$data_kind.vcf \
        -nt $SCRIPT_DIR/gnomad-$data_kind.vcf.bgz ]]; then
    bgzip -c $SCRIPT_DIR/gnomad-$data_kind.vcf \
    > $SCRIPT_DIR/gnomad-$data_kind.vcf.bgz
    tabix -f $SCRIPT_DIR/gnomad-$data_kind.vcf.bgz
fi

if [[ $data_kind == "genomes" ]] && [[ $genome_release == "grch38" ]]; then
    rf_info=false
else
    rf_info=true
fi

rm -rf $SCRIPT_DIR/gnomad-$data_kind.vcf.bgz.db
cargo run --all-features -- \
    gnomad-nuclear import \
    --import-fields-json "{
        \"vep\": true,
        \"var_info\": true,
        \"global_cohort_pops\": true,
        \"all_cohorts\": true,
        \"rf_info\": ${rf_info},
        \"quality\": true,
        \"age_hists\": true,
        \"depth_details\": true
    }" \
    --genome-release $genome_release \
    --gnomad-kind $data_kind \
    --path-in-vcf $SCRIPT_DIR/gnomad-$data_kind.vcf.bgz \
    --path-out-rocksdb $SCRIPT_DIR/gnomad-$data_kind.vcf.bgz.db \
|| rm -rf $SCRIPT_DIR/gnomad-$data_kind.vcf.bgz.db
rm -f $SCRIPT_DIR/gnomad-$data_kind.vcf.bgz.db/*.log
