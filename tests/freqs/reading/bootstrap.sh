#!/usr/bin/bash

set -euo pipefail
set -x

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

cd $SCRIPT_DIR
for x in *.vcf; do
    bgzip -c $x \
    > $x.bgz

    tabix -f $x.bgz
done
