#!/usr/bin/env bash
# Aggregated pre-commit checks of polymesh Rust code
set -e

# This particular script needs to work in and out of git context
#script_dir=${GIT_WORK_TREE:=$(dirname $0)/..}
root_dir=$(git rev-parse --git-dir)/..

$root_dir/scripts/rustfmt.sh
$root_dir/scripts/cargo-check.sh
$root_dir/scripts/cargo-test.sh
