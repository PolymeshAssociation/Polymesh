#!/usr/bin/env bash
set -euo pipefail
VERSION=${1:-"v1"}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
IMAGE="polymesh-worker-module-builder:local"
BUILD_DIR="$(mktemp -d)"

cleanup() {
    rm -rf "$BUILD_DIR"
}
trap cleanup EXIT

if ! docker info >/dev/null 2>&1; then
    echo "Docker is unavailable. Start Docker and run 'newgrp docker' if required." >&2
    exit 1
fi

docker build \
    --platform linux/amd64 \
    --tag "$IMAGE" \
    --file "$SCRIPT_DIR/Dockerfile" \
    "$SCRIPT_DIR"

mkdir -p "$BUILD_DIR/home"
docker run --rm \
    --platform linux/amd64 \
    --user "$(id -u):$(id -g)" \
    --env CARGO_HOME=/workspace/target/home/.cargo \
    --env CARGO_INCREMENTAL=0 \
    --env HOME=/workspace/target/home \
    --volume "$REPO_ROOT:/workspace" \
    --volume "$BUILD_DIR:/workspace/target" \
    --workdir /workspace/worker \
    "$IMAGE" \
    ./rebuild_modules.sh "$VERSION"
