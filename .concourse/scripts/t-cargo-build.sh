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

CHANGE_LIST=$(cat .git/resource/changed_files | grep -v '^.concourse\|^Dockerfile\|scripts/cli')

if [ ! -z "$CHANGE_LIST" ] || [ ! -f "target/release/polymesh" ]; then
    cargo build --release || cargo build -j 1 --release
fi

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
LDLIBS=$(ldd ${GIT_DIR}/target/release/polymesh | grep -o '/\S*')

echo -n "distroless-latest distroless-${GIT_REF}" > ${ARTIFACT_DIR}/additional_tags.distroless
echo -n "debian-latest debian-${GIT_REF}"         > ${ARTIFACT_DIR}/additional_tags.debian
mkdir -p ${ARTIFACT_DIR}/usr/local/bin
mkdir -p ${ARTIFACT_DIR}/var/lib/polymesh
touch ${ARTIFACT_DIR}/var/lib/polymesh/.keep
cp ${GIT_DIR}/Dockerfile.distroless   ${ARTIFACT_DIR}/
cp ${GIT_DIR}/Dockerfile.debian       ${ARTIFACT_DIR}/
cp ${SEMVER_DIR}/version              ${ARTIFACT_DIR}/tag_file
cp ${GIT_DIR}/target/release/polymesh ${ARTIFACT_DIR}/usr/local/bin/polymesh
cp ${GIT_DIR}/target/release/polymesh ${ARTIFACT_DIR}/polymesh-${SEMVER}
for LIB in $LDLIBS; do
    mkdir -p ${ARTIFACT_DIR}/$(dirname $LIB | cut -c 2-)
    cp $LIB  ${ARTIFACT_DIR}/$(dirname $LIB | cut -c 2-)/
done
cat << EOF > ${ARTIFACT_DIR}/.dockerignore
Dockerfile.distroless
Dockerfile.debian
polymesh-${SEMVER}
tag_file
additional_tags
EOF

rsync -auv --size-only ${CARGO_HOME:-$HOME/.cargo}/ ${CACHE_DIR}/.cargo | grep -e "^total size" -B1 --color=never
rsync -auv --size-only ${GIT_DIR}/target/           ${CACHE_DIR}/target | grep -e "^total size" -B1 --color=never

