#!/usr/bin/env bash

set -e
set -x
set -o pipefail

GIT_DIR=$1
NPM_CACHE_DIR=$2
CARGO_CACHE_DIR=$3

mkdir -p ${GIT_DIR}/scripts/cli/node_modules
mkdir -p ${NPM_CACHE_DIR}/scripts/cli/node_modules

if [ ! -f "$NPM_CACHE_DIR/.new_cli" ] && [ ! -f "$CARGO_CACHE_DIR/.new_binary" ]; then
    exit 0
fi

rsync -auv --size-only ${NPM_CACHE_DIR}/scripts/cli/node_modules/ ${GIT_DIR}/scripts/cli/node_modules | grep -e "^total size" -B1 --color=never

$CARGO_CACHE_DIR/target/release/polymesh --dev --pool-limit 100000 -d /tmp/pmesh-primary-node > /dev/null &

POLYMESH_PID=$!

cd $GIT_DIR/scripts/cli

WAIT_COUNT=0
while ! nc -z localhost 9944; do
    if [ $WAIT_COUNT -gt 60 ]; then
        echo "Timed out waiting for polymesh websocket port to become available"
        exit 1
    fi
    sleep 1
    WAIT_COUNT=$((WAIT_COUNT+1))
done

npm test

kill $POLYMESH_PID
wait $POLYMESH_PID || true


