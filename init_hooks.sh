#!/usr/bin/env sh
set -x
# Make sure we're in this script's directory
pushd $(dirname $0)
ln -sf $PWD/scripts/pre-commit-substrate.sh .git/hooks/pre-commit
popd
