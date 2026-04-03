#!/bin/bash
target="wasm32-wasip1"

RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+simd128,+wide-arithmetic" cargo build \
	--target=$target --release --no-default-features --features std && \
RUST_LOG="debug" wasmtime --config wasmtime_wasi.toml ../target/$target/release/polymesh-worker.wasm
