#! /bin/bash

pallet=$1
extrinsic=$2

if [[ -z "${pallet}" ]]; then
    pallet="*"
fi

if [[ -z "${extrinsic}" ]]; then
    extrinsic="*"
fi

BUILD_DUMMY_WASM_BINARY=1 cargo build --release --features=runtime-benchmarks && \
./target/release/polymesh benchmark -p=${pallet} -e=${extrinsic}
