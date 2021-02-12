#!/usr/bin/env bash

set -e
schemaTest="schema_test "
otherTests=$(ls -1 ./tests | sed -e 's/\.js$//' | sed '/^9_offchain_worker_test/d')
echo $otherTests
scriptsArray="$schemaTest$otherTests"

for s in ${scriptsArray[@]}; do
    npm run $s

done