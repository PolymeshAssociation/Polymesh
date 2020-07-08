#!/usr/bin/env bash

set -e
set -x
set -o pipefail

cargo +nightly clippy -j 1


