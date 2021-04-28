#! /bin/bash

pallet=$1
extrinsic=$2

if [[ -z "${pallet}" ]]; then
    pallet="*"
fi

if [[ -z "${extrinsic}" ]]; then
    extrinsic="*"
fi

cargo build --release --features=runtime-benchmarks,running-ci && \
./target/release/polymesh benchmark -p=${pallet} -e=${extrinsic} -r=1 -s=1
