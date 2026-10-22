#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Rebuilds the testing protocol modules into `worker/modules/testing/<version>/`.
if [ "$#" -gt 0 ]; then
    VERSIONS=("$@")
else
    VERSIONS=(v0 v1 v2)
fi

for VERSION in "${VERSIONS[@]}"; do
    "$SCRIPT_DIR/build_polkavm.sh" "$VERSION"
    "$SCRIPT_DIR/build_wasm.sh" "$VERSION"
done
