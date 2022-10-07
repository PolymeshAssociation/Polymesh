#!/usr/bin/env bash

set -e

echo "*** Initializing WASM build environment"

# Fetch name of preferred toolchain...
TOOLCHAIN=stable
# ...and install it + wasm32 target.
rustup toolchain install $TOOLCHAIN
rustup target add wasm32-unknown-unknown --toolchain $TOOLCHAIN
