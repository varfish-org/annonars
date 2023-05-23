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
    gnomad_version=3.1
else
    gnomad_version=2.1
fi

if [[ $data_kind == "exomes" ]] && [[ $genome_release == "grch38" ]]; then
    liftover=true
else
    liftover=false
fi

rm -rf $SCRIPT_DIR/gnomad-$data_kind.vcf.bgz.db
cargo run --all-features -- \
    gnomad-nuclear import \
    --import-fields-json "{
        \"vep\": true,
        \"var_info\": true,
        \"global_cohort_pops\": true,
        \"all_cohorts\": true,
        \"rf_info\": true,
        \"effect_info\": true,
        \"liftover\": $liftover,
        \"quality\": true,
        \"age_hists\": true,
        \"depth_details\": true
    }" \
    --genome-release $genome_release \
    --gnomad-kind $data_kind \
    --gnomad-version $gnomad_version \
    --path-in-vcf $SCRIPT_DIR/gnomad-$data_kind.vcf.bgz \
    --path-out-rocksdb $SCRIPT_DIR/gnomad-$data_kind.vcf.bgz.db \
|| { rm -rf $SCRIPT_DIR/gnomad-$data_kind.vcf.bgz.db; exit 1; }
rm -f $SCRIPT_DIR/gnomad-$data_kind.vcf.bgz.db/*.log
