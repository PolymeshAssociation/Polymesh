#!/usr/bin/env bash

set -e
set -x
set -o pipefail

GIT_DIR=$1
CACHE_DIR=$2
ARTIFACT_DIR=$3

mkdir -p ${GIT_DIR}/scripts/cli/node_modules
mkdir -p ${CACHE_DIR}/scripts/cli/node_modules

rsync -auv ${CACHE_DIR}/scripts/cli/node_modules/ ${GIT_DIR}/scripts/cli/node_modules


$ARTIFACT_DIR/polymesh-$(cat $ARTIFACT_DIR/VERSION) --dev --pool-limit 100000 -d /tmp/pmesh-primary-node > /dev/null &

$POLYMESH_PID=$!

cd $GIT_DIR/scripts/cli

npm test

kill $POLYMESH_PID
wait $POLYMESH_PID || true


