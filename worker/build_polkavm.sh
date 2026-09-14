#!/usr/bin/env bash
set -euo pipefail
VERSION=${1:-"v0"}
TESTING=${2:-""}
if [ -n "$TESTING" ]; then
    EXTRA_FLAG=",testing"
    NAME_TAG=".testing"
else
    EXTRA_FLAG=""
    NAME_TAG=""
fi

TARGET_JSON_PATH="$(polkatool get-target-json-path --bitness 64)"
#echo "$TARGET_JSON_PATH"

crate="polymesh-worker-protocol-dart-$VERSION"
lib_name="polymesh_worker_protocol_dart_$VERSION"
elf_path="../target/riscv64emac-unknown-none-polkavm/release/$lib_name.elf"
output_path="./modules/dart/$VERSION/$crate$NAME_TAG.polkavm"
rm -f "$output_path" "$elf_path"

echo "> Building: '$crate' (-> $output_path)"

RUSTFLAGS="--remap-path-prefix=$(pwd)= --remap-path-prefix=$HOME=~ -C codegen-units=1" \
cargo rustc --locked --crate-type cdylib \
    -Z build-std=core,alloc \
    --target $TARGET_JSON_PATH \
		--no-default-features \
		--features polkavm$EXTRA_FLAG \
    --release --lib -p $crate

polkatool link \
    --run-only-if-newer -s $elf_path \
    -o $output_path

cargo run --locked -r -p polymesh-worker-tools -- compress "$output_path"
