#!/usr/bin/env bash

set -e
set -x
set -o pipefail

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

