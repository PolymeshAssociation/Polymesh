#!/usr/bin/env bash

set -e
set -x
set -o pipefail

GIT_DIR=$1
CACHE_DIR=$2

# Restore the rust build cache
mkdir -p ${CARGO_HOME:-$HOME/.cargo}
mkdir -p ${GIT_DIR}/target
rsync -auv --size-only ${CACHE_DIR}/.cargo/ ${CARGO_HOME:-$HOME/.cargo}  | grep -e "^total size" -B1 --color=never
rsync -auv --size-only ${CACHE_DIR}/target/ ${GIT_DIR}/target            | grep -e "^total size" -B1 --color=never

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
|| \
cargo test -j 1 \
  --package polymesh-runtime-common \
  --package pallet-staking \
  --package pallet-group \
  --package polymesh-primitives \
  --package node-rpc-runtime-api \
  --package pallet-transaction-payment \
  --package pallet-cdd-offchain-worker \
  --features default_identity \

