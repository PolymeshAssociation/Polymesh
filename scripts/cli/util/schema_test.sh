#!/usr/bin/env bash

set -e

( set -x; node ./util/schema_check.js args...; ) 2>&1 | tee output.log

errLen=$(cat output.log | grep 'Unknown\ types\ found\|ErrorOccurred' | wc -l)

if [[ $errLen -le 0 ]]
then
    rm output.log
    echo Passed
    exit 0
fi

rm output.log
echo Failed
exit 1
