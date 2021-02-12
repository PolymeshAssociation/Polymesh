#!/usr/bin/env bash

set -e
scriptTests=$(ls -1 ./tests | sed -e 's/\.js$//' | sort -n | tr '\n' ' ' | sed 's/^/ schema_test /')

for s in ${scriptTests[@]}; do
    npm run $s

done