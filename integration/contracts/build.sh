#!/usr/bin/env bash
#
# Compiles the Solidity test contracts to EVM bytecode + ABI.
#
# The generated artifacts are checked into git so that running the integration
# tests does not require a Solidity toolchain. Re-run this script (and commit
# the result) whenever a `.sol` file changes; CI verifies that the checked-in
# artifacts match the sources.
#
# Requirements:
#   solc   - https://github.com/argotorg/solidity/releases (0.8.33 in CI)
#   resolc - https://github.com/paritytech/revive/releases (0.6.0 in CI, optional)
#
# Usage:
#   ./build.sh          # compile into ./artifacts
#   ./build.sh --check  # compile into a temp dir and diff against ./artifacts

set -euo pipefail

cd "$(dirname "$0")"

# Contracts that get deployed (interfaces are only used for the Rust bindings).
CONTRACTS=(Counter TestERC20 SimpleSwap)

command -v solc >/dev/null || {
	echo "error: solc not found in PATH" >&2
	exit 1
}

BUILD_DIR="$(mktemp -d)"
OUT_DIR="artifacts"
CHECK=0
if [[ "${1:-}" == "--check" ]]; then
	CHECK=1
	OUT_DIR="$(mktemp -d)"
fi
trap 'rm -rf "$BUILD_DIR"; [[ "$CHECK" == "1" ]] && rm -rf "$OUT_DIR"' EXIT

echo "using $(solc --version | tail -n1)"

mkdir -p "$OUT_DIR"

for name in "${CONTRACTS[@]}"; do
	solc --optimize --optimize-runs 200 --overwrite \
		--bin --abi \
		-o "$BUILD_DIR" \
		"${name}.sol" >/dev/null

	# `solc` writes one file per contract found in the compilation unit; we only
	# keep the artifacts for the contract that matches the file name.
	cp "$BUILD_DIR/${name}.bin" "$OUT_DIR/${name}.bin"
	cp "$BUILD_DIR/${name}.abi" "$OUT_DIR/${name}.abi"
	echo "  $(printf '%-12s' "$name") $(wc -c <"$OUT_DIR/${name}.bin") bytes of hex"
done

# PolkaVM blobs are optional: the runtime has `AllowEVMBytecode = true`, so the
# EVM bytecode above is deployable as-is. Build them when `resolc` is available
# so both back ends stay covered.
if command -v resolc >/dev/null; then
	echo "using $(resolc --version | head -n1)"
	for name in "${CONTRACTS[@]}"; do
		resolc --optimization 3 --overwrite --bin -o "$BUILD_DIR/pvm" "${name}.sol" >/dev/null
		cp "$BUILD_DIR/pvm/${name}.sol:${name}.pvm" "$OUT_DIR/${name}.polkavm"
		echo "  $(printf '%-12s' "$name") $(wc -c <"$OUT_DIR/${name}.polkavm") bytes of hex (PolkaVM)"
	done
else
	echo "resolc not found, skipping PolkaVM artifacts"
fi

if [[ "$CHECK" == "1" ]]; then
	if ! diff -ru artifacts "$OUT_DIR"; then
		echo >&2
		echo "error: checked-in artifacts are out of date, run integration/contracts/build.sh" >&2
		exit 1
	fi
	echo "artifacts are up to date"
fi
