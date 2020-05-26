# A container that launches the Polymesh node in unsafe development mode with an open WS port 9944.
#
# Build from the repo root with `docker build -t polymesh -f deploy/Dockerfile .`

FROM rust:1.43.1-slim-buster

RUN apt-get update && apt-get upgrade -y && \
    apt-get install -y \
        gcc \
        g++ \
        pkg-config \
        cmake \
        libssl-dev \
        git \
        clang \
        libclang-dev

RUN rustup install nightly-2020-04-17 && \
    rustup target add wasm32-unknown-unknown --toolchain nightly-2020-04-17 && \
    cargo +nightly-2020-04-17 install --git https://github.com/alexcrichton/wasm-gc --force

# Hack to use an older version of nightly
RUN mv /usr/local/rustup/toolchains/nightly-2020-04-17-x86_64-unknown-linux-gnu /usr/local/rustup/toolchains/nightly-x86_64-unknown-linux-gnu

RUN rustup default nightly

COPY . /home/polymesh

WORKDIR /home/polymesh

RUN cargo build --release

EXPOSE 9944

CMD [ "./target/release/polymesh", "--dev", "--unsafe-ws-external" ]
