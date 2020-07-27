#!/usr/bin/env bash

set -e
set -x
set -o pipefail

GIT_DIR=$1
CACHE_DIR=$2

# Build new CLI tests if the only if the cli code changed
if [ ! -f "${GIT_DIR}/.git/resource/changed_files" ] || grep '^scripts/cli' "${GIT_DIR}/.git/resource/changed_files"; then
    touch ${CACHE_DIR}/.new_cli
else
    rm -f ${CACHE_DIR}/.new_cli
    exit 0
fi

mkdir -p ${GIT_DIR}/scripts/cli/node_modules
mkdir -p ${CACHE_DIR}/scripts/cli/node_modules

pushd .

cd $GIT_DIR/scripts/cli

npm install

popd

# Sync the task cache for use by downstream tasks
rsync -auv --size-only ${GIT_DIR}/scripts/cli/node_modules/ ${CACHE_DIR}/scripts/cli/node_modules | grep -e "^total size" -B1 --color=never

