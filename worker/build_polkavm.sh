#!/usr/bin/env bash

TARGET_JSON_PATH="$(polkatool get-target-json-path --bitness 64)"
echo "$TARGET_JSON_PATH"

crate="polymesh-worker-protocol-dart-v0"
lib_name="polymesh_worker_protocol_dart_v0"
elf_path="../target/riscv64emac-unknown-none-polkavm/release/$lib_name.elf"
output_path="$crate.polkavm"
rm $output_path $elf_path

echo "> Building: '$crate' (-> $output_path)"

RUSTFLAGS="--remap-path-prefix=$(pwd)= --remap-path-prefix=$HOME=~" \
cargo build  \
    -Z build-std=core,alloc \
    --target $TARGET_JSON_PATH \
		--no-default-features \
		--features polkavm \
    --release --lib -p $crate

polkatool link \
    --run-only-if-newer -s $elf_path \
    -o $output_path
