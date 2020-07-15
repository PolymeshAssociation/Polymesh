#!/usr/bin/env bash

set -e
set -x
set -o pipefail

GIT_DIR=$1
CACHE_DIR=$2

mkdir -p ${GIT_DIR}/scripts/cli/node_modules
mkdir -p ${CACHE_DIR}/scripts/cli/node_modules

pushd .

cd $GIT_DIR/scripts/cli

npm install

popd

rsync -auv --size-only ${GIT_DIR}/scripts/cli/node_modules/ ${CACHE_DIR}/scripts/cli/node_modules | grep -e "^total size" -B1 --color=never

