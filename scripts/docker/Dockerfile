FROM ubuntu:bionic

LABEL maintainer="Polymath team <fdevops@polymath.network>"
LABEL name="polymesh"
LABEL version="latest"

ENV DEBIAN_FRONTEND noninteractive

RUN apt update && \
    apt upgrade -y && \
    apt install autoconf automake autotools-dev \
        build-essential ca-certificates clang cmake curl file git gcc \
        libclang-dev libssl1.1 libssl-dev libtool pkg-config xutils-dev -y && \
    apt-get autoremove -y && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

RUN curl https://sh.rustup.rs -sSf | \
    sh -s -- --default-toolchain nightly -y

ENV PATH=/root/.cargo/bin:$PATH

RUN cargo --version && \
    rustc --version && \
    rustup --version && \
    rustup component add rustfmt --toolchain nightly && \
    rustup target add wasm32-unknown-unknown --toolchain nightly && \
    cargo +nightly install --git https://github.com/alexcrichton/wasm-gc --force
