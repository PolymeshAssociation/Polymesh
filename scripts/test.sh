#!/bin/sh

SKIP_WASM_BUILD=1 RUST_BACKTRACE=1 \
  cargo test \
  --package pallet-staking \
  --package pallet-group \
  --package pallet-sudo \
  --package pallet-pips \
  --package polymesh-primitives \
  --package node-rpc-runtime-api \
  --package pallet-transaction-payment \
  --package polymesh-runtime-tests \
  --package pallet-balances:0.1.0 \
  --package asset-metadata \
  --features default_identity "$@"
