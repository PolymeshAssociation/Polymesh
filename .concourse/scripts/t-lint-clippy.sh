#!/usr/bin/env bash

set -e
set -x
set -o pipefail

GIT_DIR=$1
CACHE_DIR=$2

mkdir -p ${CACHE_DIR}/.cargo
mkdir -p ${CACHE_DIR}/target
mkdir -p $HOME/.cargo
mkdir -p ${GIT_DIR}/target

pushd .
cd $GIT_DIR

cargo +nightly clippy -j 1

popd

rsync -auv $HOME/.cargo/ ${CACHE_DIR}/.cargo
rsync -auv ${GIT_DIR}/target/ ${CACHE_DIR}/target

