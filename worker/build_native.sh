#!/usr/bin/env bash
set -euo pipefail
VERSION=${1:-"v1"}
TESTING=${2:-""}
if [ -n "$TESTING" ]; then
    EXTRA_FLAG=",testing"
    NAME_TAG=".testing"
else
    EXTRA_FLAG=""
    NAME_TAG=""
fi

if [ "$VERSION" = "v0" ]; then
  echo "Native modules are not supported for DART v0" >&2
  exit 1
fi

target="x86_64-unknown-linux-gnu"
crate="polymesh-worker-protocol-dart-$VERSION"
lib_name="libpolymesh_worker_protocol_dart_$VERSION"
so_path="../target/$target/release/$lib_name.so"

output_path="./modules/dart/$VERSION/$crate$NAME_TAG.native.so"
rm -f "$output_path" "$so_path"

worker_tools() {
  if [ -n "${POLYMESH_WORKER_TOOLS:-}" ]; then
    "$POLYMESH_WORKER_TOOLS" "$@"
  else
    cargo run --locked -r -p polymesh-worker-tools -- "$@"
  fi
}

echo "> Building: '$crate' (-> $output_path)"

RUSTFLAGS="-C panic=abort --remap-path-prefix=$(pwd)= --remap-path-prefix=$HOME=~ -C strip=symbols -C codegen-units=1" \
  cargo rustc --locked --crate-type cdylib \
	-Z build-std=core,alloc \
	--target=$target \
	--no-default-features \
	--features native_module$EXTRA_FLAG \
	--release --lib -p "$crate"

cp "$so_path" "$output_path"

worker_tools compress "$output_path"
