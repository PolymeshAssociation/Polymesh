#!/usr/bin/env bash

set -e
set -x
set -o pipefail

GIT_DIR=$1
CACHE_DIR=$2

mkdir -p ${CACHE_DIR}/.cargo
mkdir -p ${CACHE_DIR}/target
mkdir -p $HOME/.cargo
mkdir -p ${GIT_DIR}/target

pushd .
cd $GIT_DIR

cargo test \
  --package polymesh-runtime-common \
  --package pallet-staking \
  --package pallet-group \
  --package polymesh-primitives \
  --package pallet-pips-rpc \
  --package pallet-transaction-payment \
  --package pallet-cdd-offchain-worker \
  --features default_identity \
|| \
cargo test -j 1 \
  --package polymesh-runtime-common \
  --package pallet-staking \
  --package pallet-group \
  --package polymesh-primitives \
  --package pallet-pips-rpc \
  --package pallet-transaction-payment \
  --package pallet-cdd-offchain-worker \
  --features default_identity \

popd

rsync -auv $HOME/.cargo/ ${CACHE_DIR}/.cargo
rsync -auv ${GIT_DIR}/target/ ${CACHE_DIR}/target

