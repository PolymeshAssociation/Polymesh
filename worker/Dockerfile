ARG RUST_TOOLCHAIN
FROM --platform=linux/amd64 polymeshassociation/rust:debian-${RUST_TOOLCHAIN}

ARG RUST_TOOLCHAIN
ARG POLKATOOL_VERSION

RUN rustup component add rust-src && \
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
