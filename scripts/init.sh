#!/usr/bin/env bash

set -e

echo "*** Initializing WASM build environment"

# Fetch name of preferred toolchain...
TOOLCHAIN=$(< $(dirname $0)/../rust-toolchain)
# ...and install it + wasm32 target.
rustup toolchain install $TOOLCHAIN
rustup target add wasm32-unknown-unknown --toolchain $TOOLCHAIN

# Install wasm-gc. It's useful for stripping slimming down wasm binaries.
command -v wasm-gc || \
	cargo install --git https://github.com/alexcrichton/wasm-gc --force
