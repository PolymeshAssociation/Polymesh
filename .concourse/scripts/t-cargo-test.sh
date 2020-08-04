#!/usr/bin/env bash

set -e
set -x
set -o pipefail

GIT_DIR=$1

cd $GIT_DIR

cargo test \
  --package polymesh-runtime-common \
  --package pallet-staking \
  --package pallet-group \
  --package polymesh-primitives \
  --package node-rpc-runtime-api \
  --package pallet-transaction-payment \
  --package pallet-cdd-offchain-worker \
  --features default_identity \

