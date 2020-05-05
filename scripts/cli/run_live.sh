#!/usr/bin/env bash
trap cleanup INT

cleanup() {
	./node_modules/.bin/pm2 kill
	rm -rf /tmp/pmesh-*-node*
	mkdir -p /tmp/pmesh-primary-node/chains/alberbaran-testnet/keystore
	cp /Users/adamdossa/Downloads/operator-key-generator-master/keys/operator_1/6* /tmp/pmesh-primary-node/chains/alberbaran-testnet/keystore
	mkdir -p /tmp/pmesh-peer-node-1/chains/alberbaran-testnet/keystore
	cp /Users/adamdossa/Downloads/operator-key-generator-master/keys/operator_2/6* /tmp/pmesh-peer-node-1/chains/alberbaran-testnet/keystore
	mkdir -p /tmp/pmesh-peer-node-2/chains/alberbaran-testnet/keystore
	cp /Users/adamdossa/Downloads/operator-key-generator-master/keys/operator_3/6* /tmp/pmesh-peer-node-2/chains/alberbaran-testnet/keystore
	mkdir -p /tmp/pmesh-peer-node-3/chains/alberbaran-testnet/keystore
	cp /Users/adamdossa/Downloads/operator-key-generator-master/keys/operator_4/6* /tmp/pmesh-peer-node-3/chains/alberbaran-testnet/keystore
	mkdir -p /tmp/pmesh-peer-node-4/chains/alberbaran-testnet/keystore
	cp /Users/adamdossa/Downloads/operator-key-generator-master/keys/operator_5/6* /tmp/pmesh-peer-node-4/chains/alberbaran-testnet/keystore
}

set -xe

cd "$(dirname "$0")"

polymesh_binary=../../target/release/polymesh

pool_limit=${POOL_LIMIT:=100000}

# Cleanup
cleanup

./node_modules/.bin/pm2 start live.config.js --only pmesh-primary-node

sleep 2

./node_modules/.bin/pm2 start live.config.js --only "pmesh-peer-node-1,pmesh-peer-node-2,pmesh-peer-node-3,pmesh-peer-node-4"