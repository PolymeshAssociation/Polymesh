ARG RUST_TOOLCHAIN
FROM --platform=linux/amd64 polymeshassociation/rust:debian-${RUST_TOOLCHAIN}

ARG RUST_TOOLCHAIN
ARG POLKATOOL_VERSION

RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        binutils-aarch64-linux-gnu \
        gcc-aarch64-linux-gnu \
        libc6-dev-arm64-cross && \
    rm -rf /var/lib/apt/lists/*

RUN rustup target add --toolchain "${RUST_TOOLCHAIN}" aarch64-unknown-linux-gnu && \
    rustup component add --toolchain "${RUST_TOOLCHAIN}" rust-src && \
    cargo install polkatool \
        --version "=${POLKATOOL_VERSION}" \
        --locked && \
    polkatool --version

ENV LANG=C.UTF-8 \
    LC_ALL=C.UTF-8 \
    RUSTUP_TOOLCHAIN=${RUST_TOOLCHAIN} \
    SOURCE_DATE_EPOCH=0 \
    TZ=UTC

WORKDIR /workspace/worker
