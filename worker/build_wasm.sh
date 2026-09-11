#!/usr/bin/env bash
set -euo pipefail

target="wasm32-unknown-unknown"
crate="polymesh-worker-protocol-dart-v1"
lib_name="polymesh_worker_protocol_dart_v1"
wasm_path="../target/$target/release/$lib_name.wasm"

output_path="$crate.wasm"
rm -f "$output_path" "$wasm_path"

echo "> Building: '$crate' (-> $output_path)"

#RUSTFLAGS="-C target-feature=+simd128,+wide-arithmetic --remap-path-prefix=$(pwd)= --remap-path-prefix=$HOME=~ -C strip=symbols -C codegen-units=1" \
RUSTFLAGS="-C target-feature=+simd128 --remap-path-prefix=$(pwd)= --remap-path-prefix=$HOME=~ -C strip=symbols -C codegen-units=1" \
	cargo rustc --crate-type cdylib \
	--target=$target \
	--no-default-features \
	--features wasm \
  --release --lib -p $crate

cp $wasm_path $output_path

cargo run -r -p polymesh-worker-tools -- compress $output_path
