#! /bin/bash

pallet=$1
extrinsic=$2

if [[ -z "${pallet}" ]]; then
    pallet="*"
fi

if [[ -z "${extrinsic}" ]]; then
    extrinsic="*"
fi

SKIP_WASM_BUILD=1 cargo build --release --features=runtime-benchmarks && \
./target/release/polymesh benchmark -p=${pallet} -e=${extrinsic} -r=1 -s=1
