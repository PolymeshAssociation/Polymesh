FROM ubuntu:bionic

LABEL maintainer="Polymath team <fdevops@polymath.network>"
LABEL name="polymesh"
LABEL version="latest"

ENV DEBIAN_FRONTEND noninteractive

RUN apt update && \
    apt upgrade && \
    apt install --no-install-recommends autoconf automake autotools-dev \
        build-essential ca-certificates clang cmake curl file git gcc \
        libclang-dev libssl1.1 libssl-dev libtool pkg-config xutils-dev -y && \
    apt-get autoremove -y && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

RUN curl https://sh.rustup.rs -sSf | \
    sh -s -- --default-toolchain nightly -y

# Stuff below only for testing. Delete before release.

ADD ../../polymesh_substrate $HOME/polymesh_substrate

RUN cd $HOME/polymesh_substrate && \
    source ~/.cargo/env && \
    ./init.sh && \
    ./build.sh && \
    cargo build --release && \
    ./target/release/polymesh-substrate --version


