#!/usr/bin/env bash

set -e
set -x
set -o pipefail

CARGO_VERSION=$1

pwd
ls -a

cat $CARGO_VERSION


