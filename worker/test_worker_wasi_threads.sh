#!/bin/bash
target="wasm32-wasip1-threads"

RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+simd128,+wide-arithmetic" cargo build \
	--target=$target --release --no-default-features --features std && \
RUST_LOG="debug" wasmtime --config wasmtime_wasi_threads.toml ../target/$target/release/polymesh-worker.wasm
