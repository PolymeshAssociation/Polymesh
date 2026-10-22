#!/usr/bin/env bash
set -euo pipefail
VERSION=${1:-"v0"}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
# Keep the cwd stable so `--remap-path-prefix` below is deterministic.
cd "$SCRIPT_DIR"

target="wasm32-unknown-unknown"
crate="polymesh-worker-protocol-testing"
lib_name="polymesh_worker_protocol_testing"
wasm_path="$REPO_ROOT/target/$target/release/$lib_name.wasm"

# The built artifacts are committed under `worker/modules/testing/` and loaded from there by
# `worker/src/lib.rs` and the `worker_testing` integration test.
output_dir="$REPO_ROOT/worker/modules/testing/$VERSION"
output_path="$output_dir/$crate.wasm"
mkdir -p "$output_dir"
rm -f "$output_path" "$wasm_path"

echo "> Building: '$crate' (-> $output_path)"

#RUSTFLAGS="-C target-feature=+simd128,+wide-arithmetic --remap-path-prefix=$(pwd)= --remap-path-prefix=$HOME=~ -C strip=symbols -C codegen-units=1" \
RUSTFLAGS="-C target-feature=+simd128 --remap-path-prefix=$(pwd)= --remap-path-prefix=$HOME=~ -C strip=symbols -C codegen-units=1" \
	cargo rustc --crate-type cdylib \
	--target=$target \
	--no-default-features \
	--features wasm,version_$VERSION \
  --release --lib -p $crate

cp "$wasm_path" "$output_path"

cargo run -r -p polymesh-worker-tools -- compress "$output_path"
