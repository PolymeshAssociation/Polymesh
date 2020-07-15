#!/usr/bin/env bash

set -e
set -x
set -o pipefail

GIT_DIR=$1
CACHE_DIR=$2
ARTIFACT_DIR=$3
SEMVER_DIR=$4

pushd .
cd $GIT_DIR

cargo build --release || cargo build -j 1 --release

popd

mkdir -p $ARTIFACT_DIR

GIT_REF=""
if [[ -f ${GIT_DIR}/.git/short_ref ]]; then
    GIT_REF=$(cat ${GIT_DIR}/.git/short_ref)
elif [[ -f "${GIT_DIR}/.git/resource/head_sha" ]]; then
    GIT_REF=$(cat ${GIT_DIR}/.git/resource/head_sha)
else
    echo "no reference for the polymesh version found"
    exit 1
fi

SEMVER=$(cat $SEMVER_DIR/version)

cp ${GIT_DIR}/Dockerfile              ${ARTIFACT_DIR}/
cp ${SEMVER_DIR}/version              ${ARTIFACT_DIR}/tag_file
cp ${GIT_DIR}/target/release/polymesh ${ARTIFACT_DIR}/polymesh
cp ${GIT_DIR}/target/release/polymesh ${ARTIFACT_DIR}/polymesh-${SEMVER}
echo -n "latest-forked forked ${GIT_REF}" > ${ARTIFACT_DIR}/additional_tags

rsync -auv --size-only ${CARGO_HOME:-$HOME/.cargo}/ ${CACHE_DIR}/.cargo | grep -e "^total size" -B1 --color=never
rsync -auv --size-only ${GIT_DIR}/target/           ${CACHE_DIR}/target | grep -e "^total size" -B1 --color=never

