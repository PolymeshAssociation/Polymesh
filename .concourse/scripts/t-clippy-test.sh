#!/usr/bin/env bash

set -e
set -x
set -o pipefail

GIT_DIR=$1

cd $GIT_DIR

# Fetch submodules.  Workaround for https://github.com/telia-oss/github-pr-resource/pull/200
if [ ! -z "$SUBMODULE_ACCESS_TOKEN" ]; then
    git submodule init
    set +x
    git config submodule.external/cryptography.url "https://${SUBMODULE_ACCESS_TOKEN}@github.com/PolymathNetwork/cryptography.git"
    git config submodule.pallets/settlement.url    "https://${SUBMODULE_ACCESS_TOKEN}@github.com/PolymathNetwork/polymesh-settlement-module.git"
    set -x
    git submodule update pallets/settlement
    git submodule update external/cryptography
fi

cargo +nightly clippy -- -A clippy::all -W clippy::complexity -W clippy::perf