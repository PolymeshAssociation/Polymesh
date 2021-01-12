#! /bin/bash

pallet=$1
extrinsic=$1

if [[ -z "${pallet}" ]]; then
    pallet="*"
fi

if [[ -z "${extrinsic}" ]]; then
    extrinsic="*"
fi

cargo build --release --features=runtime-benchmarks && \
./target/release/polymesh benchmark -p=${pallet} -e=${extrinsic}
