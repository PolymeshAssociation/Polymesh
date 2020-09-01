#!/usr/bin/env bash

set -e
set -x
set -o pipefail

GIT_DIR=$1
ARTIFACT_DIR=$2
VERSION_DIR=$3

VERSION=$(cat $VERSION_DIR/version)
GIT_REF=""
if [[ -f ${GIT_DIR}/.git/short_ref ]]; then
    ## commit
    GIT_REF=$(cat ${GIT_DIR}/.git/short_ref)
elif [[ -f "${GIT_DIR}/.git/resource/head_sha" ]]; then
    ## Pull request
    GIT_REF=$(cat ${GIT_DIR}/.git/resource/head_sha)
else
    echo "no reference for the polymesh version found"
    exit 1
fi

pushd .
cd $GIT_DIR

# Fetch submodules.  Workaround for https://github.com/telia-oss/github-pr-resource/pull/200
if [ ! -z "$SUBMODULE_ACCESS_TOKEN" ]; then
    git submodule init
    set +x
    git config submodule.external/cryptography.url "https://${SUBMODULE_ACCESS_TOKEN}@github.com/PolymathNetwork/cryptography.git"
    git config submodule.pallets/settlement.url    "https://${SUBMODULE_ACCESS_TOKEN}@github.com/PolymathNetwork/polymesh-settlement-module.git"
    set -x
    git submodule update pallets/settlement
    git submodule update external/cryptography
fi

# Compile if any of the following conditions is met:
#  - This is a branch merge, not a PR
#  - This is a PR where the source minus exceptions changed
#  - The polymesh binary is missing
if [ ! -f ".git/resource/changed_files" ] || grep -v '^.concourse\|^Dockerfile\|^scripts/cli' ".git/resource/changed_files" || [ ! -f "target/release/polymesh" ]; then
    rm -f target/release/polymesh
    sed -i -e "s/^version = .*$/version = \"$VERSION\"/" Cargo.toml
    cargo build --release
fi
popd

LDLIBS=$(ldd ${GIT_DIR}/target/release/polymesh | grep -o '/\S*')

# Prepare files for Polymesh containers
mkdir -p $ARTIFACT_DIR
mkdir -p ${ARTIFACT_DIR}/usr/local/bin
mkdir -p ${ARTIFACT_DIR}/var/lib/polymesh
mkdir -p ${ARTIFACT_DIR}/lib/x86_64-linux-gnu
touch ${ARTIFACT_DIR}/var/lib/polymesh/.keep
echo -n "${GIT_REF}-distroless" > ${ARTIFACT_DIR}/additional_tags.distroless
echo -n "${GIT_REF}-debian"     > ${ARTIFACT_DIR}/additional_tags.debian
cp    ${GIT_DIR}/.concourse/dockerfiles/Dockerfile.distroless                                               ${ARTIFACT_DIR}/
cp    ${GIT_DIR}/.concourse/dockerfiles/Dockerfile.debian                                                   ${ARTIFACT_DIR}/
cp    ${VERSION_DIR}/version                                                                                ${ARTIFACT_DIR}/tag_file
cp    ${GIT_DIR}/target/release/polymesh                                                                    ${ARTIFACT_DIR}/usr/local/bin/polymesh
cp    ${GIT_DIR}/target/release/polymesh                                                                    ${ARTIFACT_DIR}/polymesh-${VERSION}
cp    ${GIT_DIR}/target/release/wbuild/polymesh-runtime-develop/polymesh_runtime_develop.compact.wasm       ${ARTIFACT_DIR}/polymesh_runtime_develop-${VERSION}.wasm
cp    ${GIT_DIR}/target/release/wbuild/polymesh-runtime-testnet/polymesh_runtime_testnet.compact.wasm       ${ARTIFACT_DIR}/polymesh_runtime_testnet-${VERSION}.wasm
cp -a /lib/x86_64-linux-gnu/*                                                                               ${ARTIFACT_DIR}/lib/x86_64-linux-gnu
for LIB in $LDLIBS; do
    mkdir -p ${ARTIFACT_DIR}/$(dirname $LIB | cut -c 2-)
    cp $LIB  ${ARTIFACT_DIR}/$(dirname $LIB | cut -c 2-)/
done
cat << EOF > ${ARTIFACT_DIR}/.dockerignore
Dockerfile.distroless
Dockerfile.debian
polymesh-${VERSION}
tag_file
additional_tags.distroless
additional_tags.debian
EOF
