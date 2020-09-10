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

cargo test \
  --package node-rpc-runtime-api \
  --package pallet-cdd-offchain-worker \
  --package pallet-group \
  --package pallet-staking \
  --package pallet-transaction-payment \
  --package polymesh-primitives \
  --package polymesh-runtime-tests \
  --features default_identity
CACHE_SIZE=$(du -s target | awk '{ print $1 }')
if [[ $CACHE_SIZE -gt 10000000 ]]; then
    cargo sweep -s
    cargo sweep -f -r
    cargo clean
fi

