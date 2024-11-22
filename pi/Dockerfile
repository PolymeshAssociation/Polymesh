# Original source: https://gist.github.com/pepyakin/2ff227c2d837a2eacd8d3879d5e0c94f
# Includes some customizations for polymesh

FROM rust:latest

RUN dpkg --add-architecture arm64 && \
    apt-get update && apt-get upgrade -y && \
    apt-get install -y \
				build-essential \
        gcc-aarch64-linux-gnu \
        g++-aarch64-linux-gnu \
        binutils-aarch64-linux-gnu \
        pkg-config \
        clang \
        git \
        make \
				protobuf-compiler \
				libudev-dev \
        libssl-dev:arm64

# Install the same nightly as in `../rust-toolchain.toml`
RUN rustup install nightly-2024-11-14 && \
    rustup default nightly-2024-11-14
# Install targets w32-u-u and arvm7
RUN rustup target add wasm32-unknown-unknown && \
    rustup target add aarch64-unknown-linux-gnu
# Add components
RUN rustup component add rustfmt

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER aarch64-linux-gnu-gcc
ENV CARGO_HOME /home/rust/src/target/cargo_home
ENV PKG_CONFIG_ALLOW_CROSS 1
ENV PKG_CONFIG_PATH /usr/lib/aarch64-linux-gnu/pkgconfig/

# Disallow the `pkg-config` crate to look for the config for zlib, because build.rs of `libz-sys`
# gets confused and pulls the system-wide library (i.e. of the host) instead of the target when
# cross-compiling. This essentially leads to static linking of zlib.
#
# Alternatively, we can supply LIBZ_SYS_STATIC=1. Weirdly enough, installing libgtk-3-dev:arm64
# also solves the problem somehow.
#
# Here is the related issue: https://github.com/rust-lang/libz-sys/issues/49
ENV ZLIB_NO_PKG_CONFIG 1

RUN useradd rust --user-group --create-home --shell /bin/bash
WORKDIR /home/rust/src
