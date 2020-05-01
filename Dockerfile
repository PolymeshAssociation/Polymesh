FROM rustlang/rust:nightly
WORKDIR /usr/src/app
# RUN apk add --no-cache --upgrade bash
# RUN apk update && apk add clang
RUN apt-get update && apt-get --yes install clang
COPY . .
# RUN ./scripts/init.sh
RUN rustup target add wasm32-unknown-unknown --toolchain nightly

# Install wasm-gc. It's useful for stripping slimming down wasm binaries.
RUN command -v wasm-gc || \
	cargo +nightly install --git https://github.com/alexcrichton/wasm-gc --force
RUN cargo build --release

CMD ["./target/release/polymesh --dev --pool-limit 100000 -d /tmp/pmesh-primary-node --ws-external --rpc-external"]