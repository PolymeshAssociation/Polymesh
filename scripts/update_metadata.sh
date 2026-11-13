#!/usr/bin/env bash
# Regenerates the metadata snapshots in .metadata/ for the dev, testnet and mainnet runtimes.
#
# CI's check_{dev,testnet,mainnet}_api jobs compare these snapshots with a running node, so run
# this after any change to extrinsics, events, errors, storage or constants, and commit the result.
#
# Builds the node (plain release, as CI does) and metadata-tools first. Cargo skips the build when
# nothing has changed. Set SKIP_BUILD=1 to use the existing binaries as they are.
#
# Usage, from anywhere in the repo:  ./scripts/update_metadata.sh
# Optional: RPC_PORT (default 9955) and P2P_PORT (default 30355), so a dev node already running on
# the default ports doesn't get in the way.
set -euo pipefail
cd "$(dirname "$0")/.."

POLYMESH=./target/release/polymesh
MD_TOOL=./metadata-tools/target/release/metadata-tools
MD_DIR=./.metadata/
RPC_PORT="${RPC_PORT:-9955}"
P2P_PORT="${P2P_PORT:-30355}"
RPC_URL="ws://127.0.0.1:$RPC_PORT"

if [ -z "${SKIP_BUILD:-}" ]; then
  echo "Build node"
  # No extra features: `ci-runtime` changes develop runtime constants, and CI checks against a
  # plain release build.
  cargo build --locked --release
  echo "Build metadata-tools"
  (cd metadata-tools && cargo build --release)
fi

CHAIN_PID=""
stop_node() {
  if [ -n "$CHAIN_PID" ]; then
    kill "$CHAIN_PID" 2>/dev/null || true
    wait "$CHAIN_PID" 2>/dev/null || true
    CHAIN_PID=""
  fi
}
trap stop_node EXIT

get_chain_metadata() {
  local chain=$1
  echo "Start node for chain: $chain"
  "$POLYMESH" \
    --tmp \
    --chain "$chain" \
    --no-prometheus --no-telemetry --no-mdns \
    --rpc-port "$RPC_PORT" \
    --port "$P2P_PORT" \
    1>&2 &
  CHAIN_PID=$!

  echo "Wait for node to start"
  local tries=0
  until "$MD_TOOL" runtime-version "$RPC_URL" >/dev/null 2>&1; do
    if ! kill -0 "$CHAIN_PID" 2>/dev/null; then
      echo "Node for $chain exited before its RPC came up" >&2
      exit 1
    fi
    tries=$((tries + 1))
    if [ "$tries" -ge 120 ]; then
      echo "Node for $chain didn't answer on $RPC_URL within 120s" >&2
      exit 1
    fi
    sleep 1
  done

  echo "Download metadata"
  "$MD_TOOL" download "$RPC_URL" --output-folder "$MD_DIR"

  echo "Stop node"
  stop_node
}

get_chain_metadata dev
get_chain_metadata testnet-local
get_chain_metadata mainnet-local

git status --short -- "$MD_DIR"
