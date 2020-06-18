#!/bin/sh

echo "Building develop Runtime wasm..."
cd pallets/runtime/develop/wasm
export OUT_DIR="$(pwd)/target/wasm32-unknown-unknown/release/"
cargo build \
	--target=wasm32-unknown-unknown \
	--release \
	--no-default-features \
	--features no_std
wasm-gc \
	"${OUT_DIR}/polymesh_runtime_develop_wasm.wasm" \
	"$(pwd)/../../../../target/runtime_develop.compact.wasm"
cd ../../../..
