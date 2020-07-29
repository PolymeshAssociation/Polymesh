#!/usr/bin/env bash

set -e
set -x
set -o pipefail

GIT_DIR=$1
CACHE_DIR=$2
ARTIFACT_DIR=$3
SEMVER_DIR=$4

SEMVER=$(cat $SEMVER_DIR/version)
GIT_REF=""
if [[ -f ${GIT_DIR}/.git/short_ref ]]; then
    GIT_REF=$(cat ${GIT_DIR}/.git/short_ref)
elif [[ -f "${GIT_DIR}/.git/resource/head_sha" ]]; then
    GIT_REF=$(cat ${GIT_DIR}/.git/resource/head_sha)
else
    echo "no reference for the polymesh version found"
    exit 1
fi

# Prime the build cache directory
mkdir -p "${CACHE_DIR}"
if [ ! -f ".git/resource/changed_files" ] || grep -v '^.concourse\|^Dockerfile\|^scripts/cli' ".git/resource/changed_files" || [ ! -f "target/release/polymesh" ]; then
    touch ${CACHE_DIR}/.new_binary
else
    rm -f ${CACHE_DIR}/.new_binary
fi

pushd .
cd $GIT_DIR
# Compile if any of the following conditions is met:
#  - This is a branch merge, not a PR
#  - This is a PR where the source minus exceptions changed
#  - The polymesh binary is missing
if [ ! -f ".git/resource/changed_files" ] || grep -v '^.concourse\|^Dockerfile\|^scripts/cli' ".git/resource/changed_files" || [ ! -f "target/release/polymesh" ]; then
    rm -f target/release/polymesh
    cargo +$TOOLCHAIN build --release || cargo build -j 1 --release
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
cp    ${GIT_DIR}/.concourse/dockerfiles/Dockerfile.distroless ${ARTIFACT_DIR}/
cp    ${GIT_DIR}/.concourse/dockerfiles/Dockerfile.debian     ${ARTIFACT_DIR}/
cp    ${SEMVER_DIR}/version                                   ${ARTIFACT_DIR}/tag_file
cp    ${GIT_DIR}/target/release/polymesh                      ${ARTIFACT_DIR}/usr/local/bin/polymesh
cp    ${GIT_DIR}/target/release/polymesh                      ${ARTIFACT_DIR}/polymesh-${SEMVER}
cp -a /lib/x86_64-linux-gnu/*                                 ${ARTIFACT_DIR}/lib/x86_64-linux-gnu
for LIB in $LDLIBS; do
    mkdir -p ${ARTIFACT_DIR}/$(dirname $LIB | cut -c 2-)
    cp $LIB  ${ARTIFACT_DIR}/$(dirname $LIB | cut -c 2-)/
done
cat << EOF > ${ARTIFACT_DIR}/.dockerignore
Dockerfile.distroless
Dockerfile.debian
polymesh-${SEMVER}
tag_file
additional_tags.distroless
additional_tags.debian
EOF

# Sync the cross-task build cache
rsync -auv --size-only ${CARGO_HOME:-$HOME/.cargo}/ ${CACHE_DIR}/.cargo | grep -e "^total size" -B1 --color=never
rsync -auv --size-only ${GIT_DIR}/target/           ${CACHE_DIR}/target | grep -e "^total size" -B1 --color=never

