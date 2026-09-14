#!/usr/bin/env bash
set -euo pipefail
VERSION=${1:-"v1"}

./build_polkavm.sh "$VERSION"
./build_polkavm.sh "$VERSION" "testing"
./build_wasm.sh "$VERSION"
./build_wasm.sh "$VERSION" "testing"