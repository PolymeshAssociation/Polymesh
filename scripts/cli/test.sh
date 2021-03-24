#!/usr/bin/env bash

set -e
scriptTests=$(ls -1 ./dist/tests | sed -e 's/\.js$//' |  sed -e 's/\.d.ts$//' | sort -n | uniq | tr '\n' ' ' | sed 's/^/ schema_test /')

for s in ${scriptTests[@]}; do
    yarn $s

done