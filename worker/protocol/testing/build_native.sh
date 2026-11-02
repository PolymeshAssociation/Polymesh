#!/usr/bin/env bash
set -euo pipefail
VERSION=${1:-"v0"}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
# Keep the cwd stable so `--remap-path-prefix` below is deterministic.
cd "$SCRIPT_DIR"

target="x86_64-unknown-linux-gnu"
crate="polymesh-worker-protocol-testing"
lib_name="libpolymesh_worker_protocol_testing"
so_path="$REPO_ROOT/target/$target/release/$lib_name.so"

output_dir="$REPO_ROOT/worker/modules/testing/$VERSION"
output_path="$output_dir/$crate.native.so"
mkdir -p "$output_dir"
rm -f "$output_path" "$so_path"

echo "> Building: '$crate' (-> $output_path)"

RUSTFLAGS="-C panic=abort --remap-path-prefix=$(pwd)= --remap-path-prefix=$HOME=~ -C strip=symbols -C codegen-units=1" \
	cargo rustc --crate-type cdylib \
	-Z build-std=core,alloc \
	--target=$target \
	--no-default-features \
	--features native_module,testing,version_$VERSION \
	--release --lib -p $crate

cp "$so_path" "$output_path"

cargo run -r -p polymesh-worker-tools -- compress "$output_path"
