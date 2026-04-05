#!/usr/bin/env bash

target="wasm32-wasip1-threads"
crate="polymesh-worker-protocol-dart-v0"
bin_name="$crate"
wasm_path="../target/$target/release/$bin_name.wasm"

output_path="${crate}_wasi.wasm"
rm $output_path $wasm_path

echo "> Building: '$crate' (-> $output_path)"

RUSTFLAGS="-C target-feature=+simd128,+wide-arithmetic" cargo build \
	--target=$target \
	--release --bin $crate -p $crate

cp $wasm_path $output_path

