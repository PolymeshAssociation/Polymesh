#!/usr/bin/env bash

set -e
set -x
set -o pipefail

GIT_DIR=$1

cd $GIT_DIR

# Fetch submodules.  Workaround for https://github.com/telia-oss/github-pr-resource/pull/200
if [ -f .git/resource/head_sha ]; then
    git submodule init
    set +x
    git config submodule.external/cryptography.url "https://${SUBMODULE_ACCESS_TOKEN}@github.com/PolymathNetwork/cryptography.git"
    git config submodule.pallets/settlement.url    "https://${SUBMODULE_ACCESS_TOKEN}@github.com/PolymathNetwork/polymesh-settlement-module.git"
    set -x
    git submodule update pallets/settlement
    git submodule update external/cryptography
fi

cargo test \
  --package polymesh-runtime-common \
  --package pallet-staking \
  --package pallet-group \
  --package polymesh-primitives \
  --package node-rpc-runtime-api \
  --package pallet-transaction-payment \
  --package pallet-cdd-offchain-worker \
  --features default_identity \

