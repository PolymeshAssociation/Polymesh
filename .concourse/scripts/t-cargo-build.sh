#!/usr/bin/env bash

set -e
set -x
set -o pipefail

cargo build --release || cargo build -j 1 --release

