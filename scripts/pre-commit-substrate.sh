#!/usr/bin/env sh
# pre-commit checks for polymesh_substrate Rust code

set -e

worktree=${GIT_WORK_TREE:=$PWD}

pushd $GIT_WORK_TREE/polymesh_substrate
	# Do linting, has to account for wasm gimmicks
	./check.sh
	echo cargo-check OK

	# Check formatting
	cargo fmt --all -- --check
	echo cargo-fmt OK
popd
