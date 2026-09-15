#!/usr/bin/env bash
set -euo pipefail
VERSION=${1:-"v0"}
TESTING=${2:-""}
if [ -n "$TESTING" ]; then
    EXTRA_FLAG=",testing"
    NAME_TAG=".testing"
else
    EXTRA_FLAG=""
    NAME_TAG=""
fi

target="wasm32-unknown-unknown"
crate="polymesh-worker-protocol-dart-$VERSION"
lib_name="polymesh_worker_protocol_dart_$VERSION"
wasm_path="../target/$target/release/$lib_name.wasm"

output_path="./modules/dart/$VERSION/$crate$NAME_TAG.wasm"
rm -f "$output_path" "$wasm_path"

echo "> Building: '$crate' (-> $output_path)"

if [ "$VERSION" = "v0" ]; then
  CARGO_PACKAGE_ARGS=(--manifest-path protocol/dart-v0/Cargo.toml --target-dir ../target)
else
  CARGO_PACKAGE_ARGS=(-p "$crate")
fi

#RUSTFLAGS="-C target-feature=+simd128,+wide-arithmetic --remap-path-prefix=$(pwd)= --remap-path-prefix=$HOME=~ -C strip=symbols -C codegen-units=1" \
RUSTFLAGS="-C target-feature=+simd128 --remap-path-prefix=$(pwd)= --remap-path-prefix=$HOME=~ -C strip=symbols -C codegen-units=1" \
  cargo rustc --locked --crate-type cdylib \
	--target=$target \
	--no-default-features \
	--features wasm$EXTRA_FLAG \
  --release --lib "${CARGO_PACKAGE_ARGS[@]}"

cp "$wasm_path" "$output_path"

cargo run --locked -r -p polymesh-worker-tools -- compress "$output_path"
