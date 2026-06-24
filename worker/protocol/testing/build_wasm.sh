#!/usr/bin/env bash
set -euo pipefail
VERSION=${1:-"v0"}

target="wasm32-unknown-unknown"
crate="polymesh-worker-protocol-testing"
lib_name="polymesh_worker_protocol_testing"
wasm_path="../../../target/$target/release/$lib_name.wasm"

output_path="$VERSION/$crate.wasm"
rm -f "$output_path" "$wasm_path"

echo "> Building: '$crate' (-> $output_path)"

#RUSTFLAGS="-C target-feature=+simd128,+wide-arithmetic --remap-path-prefix=$(pwd)= --remap-path-prefix=$HOME=~ -C strip=symbols -C codegen-units=1" \
RUSTFLAGS="-C target-feature=+simd128 --remap-path-prefix=$(pwd)= --remap-path-prefix=$HOME=~ -C strip=symbols -C codegen-units=1" \
	cargo rustc --crate-type cdylib \
	--target=$target \
	--no-default-features \
	--features wasm,version_$VERSION \
  --release --lib -p $crate

cp $wasm_path $output_path

cargo run -r -p polymesh-worker-tools -- compress $output_path
