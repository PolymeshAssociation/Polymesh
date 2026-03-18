ARG BUILDER_IMAGE=polymeshassociation/rust:debian-nightly-2025-12-01
FROM ${BUILDER_IMAGE} AS builder

WORKDIR /polymesh
COPY . /polymesh

RUN cargo fetch
RUN cargo build --workspace --locked --release -p pallet-revive-eth-rpc --bin eth-rpc

FROM debian:stable-slim

RUN DEBIAN_FRONTEND=noninteractive apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        libsqlite3-0 && \
    apt-get autoremove -y && \
    apt-get clean && \
    find /var/lib/apt/lists/ -type f -not -name lock -delete

COPY --from=builder /polymesh/target/release/eth-rpc /usr/local/bin

USER root
RUN useradd -m -u 4001 -U -s /bin/sh -d /home/polymesh polymesh && \
    /usr/local/bin/eth-rpc --help

USER polymesh

# 8545 is the default port for the JSON-RPC server
# 9616 is the default port for the Prometheus metrics
EXPOSE 8545 9616
ENTRYPOINT ["/usr/local/bin/eth-rpc"]

# We call the help by default
CMD ["--help"]
