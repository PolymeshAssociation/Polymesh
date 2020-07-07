#!/usr/bin/env bash

set -e
set -x
set -o pipefail

mkdir -p t-rustc-version
rustc --version > t-rustc-version/version

