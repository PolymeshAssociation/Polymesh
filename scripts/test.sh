#!/bin/sh

SKIP_WASM_BUILD=1 RUST_BACKTRACE=1 \
  cargo test \
  --package pallet-validators \
  --package pallet-group \
  --package pallet-sudo \
  --package pallet-pips \
  --package polymesh-primitives \
  --package node-rpc-runtime-api \
  --package polymesh-runtime-tests \
  --package asset-metadata \
  "$@"
