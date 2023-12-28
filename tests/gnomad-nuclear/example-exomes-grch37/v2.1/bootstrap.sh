#!/usr/bin/env bash

set -euo pipefail
set -x

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

dir_up=$(echo $SCRIPT_DIR | rev | cut -d / -f 2 | rev)
data_kind=$(echo $dir_up | cut -d - -f 2)
genome_release=$(echo $dir_up | cut -d - -f 3)

if [[ $SCRIPT_DIR/gnomad-$data_kind.vcf \
        -nt $SCRIPT_DIR/gnomad-$data_kind.vcf.bgz ]]; then
    bgzip -c $SCRIPT_DIR/gnomad-$data_kind.vcf \
    > $SCRIPT_DIR/gnomad-$data_kind.vcf.bgz
    tabix -f $SCRIPT_DIR/gnomad-$data_kind.vcf.bgz
fi

gnomad_version=$(basename $SCRIPT_DIR | cut -d v -f 2)

if [[ $data_kind == "exomes" ]] && [[ $genome_release == "grch38" ]] && [[ $gnomad_version == "2.1" ]]; then
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
