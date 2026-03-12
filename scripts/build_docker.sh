#!/usr/bin/env bash

#################################
#   Polymesh Docker Image Build
#################################

set -euo pipefail

## Variables
#
# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;36m'
YELLOW='\033[1;33m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# Track timing
START_TIME=$(date +%s)

# Lock file path
SCRIPT_NAME=$(basename "$0")
LOCK_FILE="/tmp/${SCRIPT_NAME%.*}.lock"

# Script directory and repo root (detected via git)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"

# Defaults
VARIANT="debian"
ARCH="amd64"
BRANCH=$(git -C "$REPO_ROOT" rev-parse --abbrev-ref HEAD)

## Functions
#
# Print colorized messages
print_msg() {
    local color=$1
    local symbol=$2
    local message=$3
    printf "\n%b%s%b %b\n\n" "$color" "$symbol" "$NC" "$message" >&2
}

# Convenience functions for different message types
print_info()    { print_msg "$BLUE"   "→" "$1"; }
print_success() { print_msg "$GREEN"  "✓" "$1"; }
print_error()   { print_msg "$RED"    "✗" "$1"; }
print_warning() { print_msg "$YELLOW" "⚠" "$1"; }

# Print a banner with dynamic sizing
print_banner() {
    local title="$1" padding=4
    local inner=$((${#title} + padding * 2))
    local line spaces
    printf -v line '%*s' "$inner" ''; line="${line// /─}"
    printf -v spaces '%*s' "$padding" ''
    echo -e "${BOLD}${MAGENTA}╭${line}╮${NC}"
    echo -e "${BOLD}${MAGENTA}│${NC}${spaces}${BOLD}${CYAN}${title}${NC}${spaces}${BOLD}${MAGENTA}│${NC}"
    echo -e "${BOLD}${MAGENTA}╰${line}╯${NC}"
}

# Format time in seconds to a readable format
format_time() {
    local seconds=$1
    local hours=$((seconds / 3600))
    local minutes=$(((seconds % 3600) / 60))
    local secs=$((seconds % 60))

    if [ "$hours" -gt 0 ]; then
        echo "${hours}h:${minutes}m:${secs}s"
    elif [ "$minutes" -gt 0 ]; then
        echo "${minutes}m:${secs}s"
    else
        echo "${secs}s"
    fi
}

# Calculate and display total execution time
calculate_and_display_total_time() {
    local end_time
    end_time=$(date +%s)
    local total_time=$((end_time - START_TIME))
    local total_formatted
    total_formatted=$(format_time "$total_time")
    print_info "Done. Total execution time: $total_formatted."
}

# Display usage information
usage() {
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --variant   debian|distroless  (default: debian)"
    echo "  --arch      amd64|arm64        (default: amd64)"
    echo "  --branch    branch name        (default: current git branch)"
    echo "  --help"
    exit "${1:-0}"
}

# Derive the builder image tag from rust-toolchain.toml so it stays in sync
# automatically when the toolchain is updated.
RUST_NIGHTLY="$(grep 'channel' "$REPO_ROOT/rust-toolchain.toml" | sed -e 's/channel = "//' -e 's/"//' | tr -d ' ')"
RUST_BUILDER_IMAGE_AMD64="polymeshassociation/rust:debian-${RUST_NIGHTLY}"
RUST_BUILDER_IMAGE_ARM64="polymeshassociation/rust-arm64:debian-${RUST_NIGHTLY}"

# Check for required dependencies
check_dependencies() {
    if ! command -v docker &>/dev/null; then
        print_error "docker not found. Install it from https://docs.docker.com/get-docker"
        exit 1
    fi
    if ! docker info &>/dev/null; then
        print_error "Docker daemon is not running."
        exit 1
    fi

    # Host arch 
    local host_arch
    host_arch=$(docker info --format '{{.Architecture}}' 2>/dev/null || uname -m)
    case "$host_arch" in
        aarch64|arm64) host_arch="arm64" ;;
        x86_64|amd64)  host_arch="amd64" ;;
    esac

    if [[ "$ARCH" != "$host_arch" ]]; then
        if ! docker buildx inspect 2>/dev/null | grep -q "linux/${ARCH}"; then
            print_error "Cross-platform emulation is required to build ${ARCH} on ${host_arch}."
            print_info "Enable it with one of:"
            echo "  docker run --privileged --userns=host --rm tonistiigi/binfmt --install all"
            echo "  sudo apt-get install -y qemu-user-static binfmt-support  (Debian/Ubuntu)"
            exit 1
        fi
    fi
}

# Build the Linux binary inside the official Rust Docker image.
# Uses a temporary named Docker volume for target/ to avoid host permission
# issues (e.g. userns-remap). The volume is removed before and after each build.
build_rust_binary() {
    local platform="linux/${ARCH}"
    local target_volume="polymesh-build-target-${ARCH}"

    if [[ "$ARCH" == "arm64" ]]; then
        local builder_image="$RUST_BUILDER_IMAGE_ARM64"
        BINARY="$REPO_ROOT/polymesh-arm64"
    else
        local builder_image="$RUST_BUILDER_IMAGE_AMD64"
        BINARY="$REPO_ROOT/polymesh"
    fi

    docker volume rm "$target_volume" 2>/dev/null || true
    trap 'docker volume rm "polymesh-build-target-${ARCH}" 2>/dev/null || true; rm -f "${LOCK_FILE}"' EXIT

    print_info "Pulling builder image: ${builder_image} (${platform})..."
    docker pull --platform "$platform" "$builder_image"

    print_info "Building Linux binary inside Docker (cargo build --locked --release)..."
    docker run --rm \
        --platform "$platform" \
        -v "$REPO_ROOT:/build:ro" \
        -v "${target_volume}:/build/target" \
        -w /build \
        -e RUSTFLAGS="-D warnings" \
        "$builder_image" \
        cargo build --locked --release

    # Using already pulled builder image to extract the binary
    print_info "Extracting binary..."
    docker run --rm \
        --platform "$platform" \
        -v "${target_volume}:/target:ro" \
        "$builder_image" \
        cat /target/release/polymesh >| "$BINARY"
    chmod +x "$BINARY"

    print_success "Binary ready at ${BINARY}."
}

# Build the Docker image
build_docker_image() {
    print_info "Building Docker image..."
    docker buildx build \
        --platform "linux/${ARCH}" \
        --load \
        -f "$REPO_ROOT/$DOCKERFILE" \
        --tag "$TAG_LATEST" \
        --tag "$TAG_VERSION" \
        "$REPO_ROOT"
    print_success "Image built:"
    echo "      ${TAG_LATEST}"
    echo "      ${TAG_VERSION}"
}

## Main script execution
#

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --variant) VARIANT="$2"; shift 2 ;;
        --arch)    ARCH="$2";    shift 2 ;;
        --branch)  BRANCH="$2";  shift 2 ;;
        --help)    usage ;;
        *) print_error "Unknown option: $1"; usage 1 ;;
    esac
done

# Validate inputs
case "$VARIANT" in
    debian|distroless) ;;
    *) print_error "--variant must be 'debian' or 'distroless'"; exit 1 ;;
esac
case "$ARCH" in
    amd64|arm64) ;;
    *) print_error "--arch must be 'amd64' or 'arm64'"; exit 1 ;;
esac

# Check if lock file exists, if not create it and set trap on exit
if { set -C; 2>/dev/null true >"${LOCK_FILE}"; }; then
    trap 'rm -f ${LOCK_FILE}' EXIT
else
    print_error "Lock file ${LOCK_FILE} exists — another instance may be running."
    exit 1
fi

# Resolve version (mirrors scripts/version.sh logic)
COMMIT=$(git -C "$REPO_ROOT" rev-parse --short=10 HEAD)
if [[ "$BRANCH" == "develop" ]]; then
    VERSION="$COMMIT"
else
    VERSION=$(grep '^version' "$REPO_ROOT/Cargo.toml" | head -1 | cut -d'=' -f2 | sed 's/[^a-zA-Z0-9\.]//g')
fi

# Select Dockerfile and image name
if [[ "$ARCH" == "arm64" ]]; then
    DOCKERFILE=".docker/arm64/Dockerfile.${VARIANT}"
    IMAGE_NAME="polymeshassociation/polymesh-arm64"
else
    DOCKERFILE=".docker/Dockerfile.${VARIANT}"
    IMAGE_NAME="polymeshassociation/polymesh-amd64"
fi

TAG_LATEST="${IMAGE_NAME}:latest-${BRANCH}-${VARIANT}"
TAG_VERSION="${IMAGE_NAME}:${VERSION}-${BRANCH}-${VARIANT}"

# Clear the terminal
clear

# Display banner
print_banner "Polymesh Docker Image Build"
echo ""

# Print build summary
echo -e "  ${BOLD}Variant:${NC}    ${CYAN}${VARIANT}${NC}"
echo -e "  ${BOLD}Arch:${NC}       ${CYAN}${ARCH}${NC}"
echo -e "  ${BOLD}Branch:${NC}     ${CYAN}${BRANCH}${NC}"
echo -e "  ${BOLD}Version:${NC}    ${CYAN}${VERSION}${NC}"
echo -e "  ${BOLD}Builder:${NC}    ${CYAN}$([[ "$ARCH" == "arm64" ]] && echo "$RUST_BUILDER_IMAGE_ARM64" || echo "$RUST_BUILDER_IMAGE_AMD64")${NC}"
echo -e "  ${BOLD}Dockerfile:${NC} ${CYAN}${DOCKERFILE}${NC}"
echo -e "  ${BOLD}Tags:${NC}       ${CYAN}${TAG_LATEST}${NC}"
echo -e "              ${CYAN}${TAG_VERSION}${NC}"
echo ""

# Check dependencies
check_dependencies
echo ""

# Run build steps
build_rust_binary
echo ""
build_docker_image
echo ""

# Total execution time
calculate_and_display_total_time
