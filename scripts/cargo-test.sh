#!/usr/bin/env bash
set -e

script_dir=$(dirname $0)
pushd $script_dir/../runtime 1>/dev/null
	if [ -z "${VERBOSE-}" ] ; then
		cargo test || (echo "cargo-test FAIL" && false)
		echo cargo-test OK
	else # rustfmt output not suppresed
		cargo test -- --nocapture || (echo "cargo-test FAIL" && false)
		echo cargo-test OK
	fi
popd 1>/dev/null
