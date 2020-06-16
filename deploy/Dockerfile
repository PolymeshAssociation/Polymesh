# A container that launches the Polymesh node in unsafe development mode with an open WS port 9944.
#
# Build from the repo root with `docker build -t polymesh -f deploy/Dockerfile .`

FROM cimg/rust:1.43.0-node

RUN sudo apt-get update && \
    sudo apt-get upgrade -y && \
    sudo apt-get install -y \
        gcc \
        g++ \
        pkg-config \
        cmake \
        libssl-dev \
        git \
        clang \
        libclang-dev

RUN git clone https://github.com/Chainsafe/Polymesh.git &&\
  cd Polymesh &&\
  git checkout develop
WORKDIR Polymesh

RUN rustup install nightly-2020-04-17 && \
    rustup target add wasm32-unknown-unknown --toolchain nightly-2020-04-17 && \
    cargo +nightly-2020-04-17 install --git https://github.com/alexcrichton/wasm-gc --force

# Hack to use an older version of nightly
RUN mv ~/.rustup/toolchains/nightly-2020-04-17-x86_64-unknown-linux-gnu ~/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu
RUN rustup default nightly

RUN cargo build -j 1 --release 
RUN cd ./scripts/cli && npm i
EXPOSE 9944

CMD ./deploy/start.sh
