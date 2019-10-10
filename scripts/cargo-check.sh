#!/usr/bin/env bash
set -e

script_dir=$(dirname $0)
pushd $script_dir/../ 1>/dev/null
	# Do linting, the custom script has to account for wasm gimmicks
	cargo check || (echo "cargo-check FAIL" && false)
	echo cargo-check OK
popd 1>/dev/null
