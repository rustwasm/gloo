#!/usr/bin/env bash

set -eux
cd $(dirname $0)

for c in crates/*; do
    cd "$c"
    cargo readme --template ../../.README.tpl > README.md
    cd -
done
