#!/usr/bin/env sh
# pre-commit checks for polymesh_substrate Rust code

set -e

worktree=${GIT_WORK_TREE:=$PWD}

# rustfmt all top-level, non-artifact `src` dirs, all of *.rs inside
find polymesh_substrate -type d -name "src" -not -path "*/target/*" | xargs -i find {} -type f -name "*.rs" | xargs rustfmt +nightly --check

pushd $GIT_WORK_TREE/polymesh_substrate
	# Do linting, has to account for wasm gimmicks
	./check.sh
	echo cargo-check OK
popd
