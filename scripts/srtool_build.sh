#!/bin/bash

CHAIN=$1
CHAIN=${CHAIN:-'mainnet'}

PACKAGE=polymesh-runtime-$CHAIN
RUNTIME_DIR=pallets/runtime/$CHAIN

SRTOOL="paritytech/srtool:1.62.0"

echo "srtool: ${SRTOOL}"
echo "chain runtime: ${CHAIN}"

docker pull $SRTOOL

# Add if need to use a proxy.
# -e HTTP_PROXY=$HTTP_PROXY -e HTTPS_PROXY=$HTTPS_PROXY \

docker run --rm -it \
  -e PACKAGE=$PACKAGE \
  -e RUNTIME_DIR=$RUNTIME_DIR \
  -v $PWD:/build -v /tmp/cargo:/cargo-home \
  --user root $SRTOOL \
  build
