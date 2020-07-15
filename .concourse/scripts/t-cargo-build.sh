#!/usr/bin/env bash

set -e
set -x
set -o pipefail

GIT_DIR=$1
CACHE_DIR=$2
SEMVER_DIR=$3

pushd .
cd $GIT_DIR

cargo build --release || cargo build -j 1 --release

popd

mkdir -p artifact
mkdir -p dockerdir

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

cp    ${GIT_DIR}/Dockerfile              dockerdir/
cp    ${SEMVER_DIR}/version              artifact/tag_file
cp    ${GIT_DIR}/target/release/polymesh artifact/polymesh
ln -s artifact/polymesh                  artifact/polymesh-${SEMVER}
echo -n "latest-forked forked" > dockerdir/additional_tags

rsync -auv --size-only ${CARGO_HOME:-$HOME/.cargo}/ ${CACHE_DIR}/.cargo | grep -e "^total size" -B1 --color=never
rsync -auv --size-only ${GIT_DIR}/target/           ${CACHE_DIR}/target | grep -e "^total size" -B1 --color=never

