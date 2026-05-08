#!/bin/bash
set -euo pipefail

./build_polkavm.sh && ./build_polkavm_testing.sh && ./build_wasm.sh && ./build_wasm_testing.sh
