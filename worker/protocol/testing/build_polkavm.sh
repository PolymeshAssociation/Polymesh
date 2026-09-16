#!/usr/bin/env bash
set -euo pipefail
VERSION=${1:-"v0"}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
# Keep the cwd stable so `--remap-path-prefix` below is deterministic.
cd "$SCRIPT_DIR"

TARGET_JSON_PATH="$(polkatool get-target-json-path --bitness 64)"
#echo "$TARGET_JSON_PATH"

crate="polymesh-worker-protocol-testing"
lib_name="polymesh_worker_protocol_testing"
elf_path="$REPO_ROOT/target/riscv64emac-unknown-none-polkavm/release/$lib_name.elf"
# The built artifacts are committed under `worker/modules/testing/` and loaded from there by
# `worker/src/lib.rs` and the `worker_testing` integration test.
output_dir="$REPO_ROOT/worker/modules/testing/$VERSION"
output_path="$output_dir/$crate.polkavm"
mkdir -p "$output_dir"
rm -f "$output_path" "$elf_path"

echo "> Building: '$crate' (-> $output_path)"

RUSTFLAGS="--remap-path-prefix=$(pwd)= --remap-path-prefix=$HOME=~ -C codegen-units=1" \
cargo rustc --crate-type cdylib \
    -Z json-target-spec \
    -Z build-std=core,alloc \
    --target $TARGET_JSON_PATH \
		--no-default-features \
		--features polkavm,version_$VERSION \
    --release --lib -p $crate

polkatool link \
    --run-only-if-newer -s "$elf_path" \
    -o "$output_path"

cargo run -r -p polymesh-worker-tools -- compress "$output_path"
