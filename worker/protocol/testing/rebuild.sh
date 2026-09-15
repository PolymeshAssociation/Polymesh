#!/bin/bash
set -euo pipefail

# v0.1.0
./build_polkavm.sh && ./build_wasm.sh

# v1.0.0
./build_polkavm.sh v1 && ./build_wasm.sh v1

# v2.0.0
./build_polkavm.sh v2 && ./build_wasm.sh v2
