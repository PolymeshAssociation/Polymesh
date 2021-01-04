#! /bin/bash

pallet=$1

if [[ -z "${pallet}" ]]; then
    pallet="*"
fi

cargo build --release --features=runtime-benchmarks && \
./target/release/polymesh benchmark -p=${pallet} -e=*
