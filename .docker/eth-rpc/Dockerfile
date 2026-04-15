FROM debian:trixie-slim
ARG ETH_RPC_BIN=eth-rpc-amd64

RUN DEBIAN_FRONTEND=noninteractive apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        libsqlite3-0 && \
    apt-get autoremove -y && \
    apt-get clean && \
    find /var/lib/apt/lists/ -type f -not -name lock -delete

COPY ./${ETH_RPC_BIN} /usr/local/bin/eth-rpc

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

HEALTHCHECK \
    --interval=10s \
    --start-period=30s \
    --timeout=5s \
    --retries=6 \
    CMD curl -sf -X POST -H 'Content-Type: application/json' \
        -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
        http://localhost:8545/
