#!/usr/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

# Helper script to download protobuf files from elsewhere.

cd $SCRIPT_DIR

mkdir -p clinvar_data
for name in class_by_freq clinvar_public extracted_vars gene_impact phenotype_link; do
    wget \
        -O clinvar_data/$name.proto \
        https://raw.githubusercontent.com/varfish-org/clinvar-this/main/protos/clinvar_data/pbs/$name.proto
done
sed \
    -i \
    -e 's/clinvar_data\.pbs\./clinvar_data./g' \
    -e 's|clinvar_data/pbs/|clinvar_data/|g' \
    clinvar_data/*.proto
