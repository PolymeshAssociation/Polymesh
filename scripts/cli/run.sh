#!/usr/bin/env bash
trap cleanup INT

cleanup() {
	./node_modules/.bin/pm2 kill
	rm -rf /tmp/pmesh-*-node*
}

set -xe

cd "$(dirname "$0")"

polymesh_binary=../../target/release/polymesh

pool_limit=${POOL_LIMIT:=100000}

# Cleanup
cleanup

./node_modules/.bin/pm2 start environment.config.js --only pmesh-primary-node

sleep 2

npx pm2 start environment.config.js --only "pmesh-peer-node-1,pmesh-peer-node-2"
#,pmesh-peer-node-3,pmesh-peer-node-4"
