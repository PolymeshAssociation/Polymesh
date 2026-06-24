#!/usr/bin/env bash
set -euo pipefail
VERSION=${1:-"v0"}

TARGET_JSON_PATH="$(polkatool get-target-json-path --bitness 64)"
#echo "$TARGET_JSON_PATH"

crate="polymesh-worker-protocol-testing"
lib_name="polymesh_worker_protocol_testing"
elf_path="../../../target/riscv64emac-unknown-none-polkavm/release/$lib_name.elf"
output_path="$VERSION/$crate.polkavm"
rm -f "$output_path" "$elf_path"

echo "> Building: '$crate' (-> $output_path)"

RUSTFLAGS="--remap-path-prefix=$(pwd)= --remap-path-prefix=$HOME=~ -C codegen-units=1" \
cargo build  \
    -Z build-std=core,alloc \
    --target $TARGET_JSON_PATH \
		--no-default-features \
		--features polkavm,version_$VERSION \
    --release --lib -p $crate

polkatool link \
    --run-only-if-newer -s $elf_path \
    -o $output_path

cargo run -r -p polymesh-worker-tools -- compress $output_path
