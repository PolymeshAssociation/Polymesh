#!/usr/bin/env bash

crate="polymesh-worker-protocol-dart-v0"
bin_name="$crate"
wasm_path="../target/wasm32-unknown-unknown/release/$bin_name.wasm"

output_path="$crate.wasm"
rm $output_path $wasm_path

echo "> Building: '$crate' (-> $output_path)"

RUSTFLAGS="-C target-feature=+simd128" cargo build \
	--target=wasm32-unknown-unknown \
	--no-default-features \
	--features wasm \
	--release --bin $crate -p $crate

cp $wasm_path $output_path

