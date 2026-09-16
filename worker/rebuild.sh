#!/usr/bin/env bash
set -euo pipefail
VERSIONS=("${@:-v1}")

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
IMAGE="polymesh-worker-module-builder:local"
BUILD_DIR="$(mktemp -d)"
RUST_TOOLCHAIN="$(awk -F'"' '/^channel[[:space:]]*=/ { print $2; exit }' "$REPO_ROOT/rust-toolchain.toml")"

cleanup() {
    rm -rf "$BUILD_DIR"
}
trap cleanup EXIT

if ! docker info >/dev/null 2>&1; then
    echo "Docker is unavailable. Start Docker and run 'newgrp docker' if required." >&2
    exit 1
fi

for VERSION in "${VERSIONS[@]}"; do
    case "$VERSION" in
        v0)
            VERSION_RUST_TOOLCHAIN=nightly-2025-12-01
            VERSION_POLKATOOL_VERSION=0.33.0
            ;;
        v1)
            VERSION_RUST_TOOLCHAIN="$RUST_TOOLCHAIN"
            VERSION_POLKATOOL_VERSION=0.33.0
            ;;
        *)
            echo "Unknown DART module version: $VERSION" >&2
            exit 1
            ;;
    esac

    image="$IMAGE-$VERSION"
    docker build \
        --platform linux/amd64 \
        --build-arg "RUST_TOOLCHAIN=$VERSION_RUST_TOOLCHAIN" \
        --build-arg "POLKATOOL_VERSION=$VERSION_POLKATOOL_VERSION" \
        --tag "$image" \
        --file "$SCRIPT_DIR/Dockerfile" \
        "$SCRIPT_DIR"

    mkdir -p "$BUILD_DIR/$VERSION/home"
    docker run --rm \
        --platform linux/amd64 \
        --user "$(id -u):$(id -g)" \
        --env CARGO_HOME=/workspace/target/home/.cargo \
        --env CARGO_INCREMENTAL=0 \
        --env HOME=/workspace/target/home \
        --volume "$REPO_ROOT:/workspace" \
        --volume "$BUILD_DIR/$VERSION:/workspace/target" \
        --workdir /workspace/worker \
        "$image" \
        ./rebuild_modules.sh "$VERSION"
done
