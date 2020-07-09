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

mkdir -p artifact

POLYMESH_VERSION=""
if [[ -f "${GIT_DIR}/.git/resource/head_sha" ]]; then
    POLYMESH_VERSION=$(cat ${GIT_DIR}/.git/resource/head_sha)
elif [[ -f ${GIT_DIR}/.git/describe_ref ]]; then
    POLYMESH_VERSION=$(cat ${GIT_DIR}/.git/describe_ref)
else
    echo "no reference for the polymesh version found"
    ls -l ${GIT_DIR}
    exit 1
fi

cp ${GIT_DIR}/target//release/polymesh artifact/polymesh-${POLYMESH_VERSION}

rsync -auv ${CARGO_HOME:-$HOME/.cargo}/ ${CACHE_DIR}/.cargo
rsync -auv ${GIT_DIR}/target/ ${CACHE_DIR}/target

