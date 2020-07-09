#!/usr/bin/env bash

set -e
set -x
set -o pipefail

GIT_DIR=$1
CACHE_DIR=$2

pushd .
cd $GIT_DIR

cargo build --release || cargo build -j 1 --release

popd

rsync -auv $HOME/.cargo/ ${CACHE_DIR}/.cargo
rsync -auv ${GIT_DIR}/target/ ${CACHE_DIR}/target

