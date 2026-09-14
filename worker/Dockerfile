FROM --platform=linux/amd64 rust:1.93.0-bookworm@sha256:7274e0edb5b47eda8053b350ebf3d489f7e0f65d2d7e77b16076299c7c047c28

ARG RUST_TOOLCHAIN=nightly-2025-12-01
ARG POLKATOOL_VERSION=0.33.0

RUN rustup toolchain install "${RUST_TOOLCHAIN}" \
        --profile minimal \
        --component rust-src \
        --target wasm32-unknown-unknown && \
    cargo install polkatool \
        --version "=${POLKATOOL_VERSION}" \
        --locked && \
    rustup default "${RUST_TOOLCHAIN}" && \
    rustc --version && \
    polkatool --version

ENV LANG=C.UTF-8 \
    LC_ALL=C.UTF-8 \
    RUSTUP_TOOLCHAIN=${RUST_TOOLCHAIN} \
    SOURCE_DATE_EPOCH=0 \
    TZ=UTC

WORKDIR /workspace/worker