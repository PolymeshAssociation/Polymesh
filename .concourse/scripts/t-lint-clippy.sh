#!/usr/bin/env bash

set -e
set -x
set -o pipefail

GIT_DIR=$1
CACHE_DIR=$2

mkdir -p $HOME/.cargo
mkdir -p ${GIT_DIR}/target

rsync -auv ${CACHE_DIR}/.cargo/ $HOME/.cargo
rsync -auv ${CACHE_DIR}/target/ ${GIT_DIR}/target

cd $GIT_DIR

cargo +nightly clippy -j 1


