#!/usr/bin/env bash

target="wasm32-unknown-unknown"
crate="polymesh-worker-protocol-dart-v0"
lib_name="polymesh_worker_protocol_dart_v0"
wasm_path="../target/$target/release/$lib_name.wasm"

output_path="${crate}.testing.wasm"
rm $output_path $wasm_path

echo "> Building: '$crate' (-> $output_path)"

#RUSTFLAGS="-C target-feature=+simd128,+wide-arithmetic" \
RUSTFLAGS="--remap-path-prefix=$(pwd)= --remap-path-prefix=$HOME=~ -C strip=symbols" \
	cargo build \
	--target=$target \
	--no-default-features \
	--features wasm,testing \
  --release --lib -p $crate

cp $wasm_path $output_path

cargo run -r -p polymesh-worker-tools -- compress $output_path
