# AGENTS.md

## Overview

Polymesh is a Substrate-based L1 blockchain (Rust/FRAME). The root package builds the `polymesh` node binary (`src/bin/main.rs`); runtime WASM is built per chain by `substrate-wasm-builder` during `cargo build`.

## Build & toolchain

- Toolchain is pinned nightly via `rust-toolchain.toml` (with `rust-src`, `wasm32v1-none`). Let rustup pick it; don't force stable.
- First full `cargo build --release` is slow (builds all three runtime WASMs).
- Set `SKIP_WASM_BUILD=1` for clippy/unit-test iteration (what CI does); CI also builds/tests with `RUSTFLAGS=-D warnings`.
- All `sp-*`/`sc-*`/`frame-*` deps come from the Polymesh fork of polkadot-sdk (branch pinned in root `[workspace.dependencies]`). Use workspace deps, not crates.io equivalents, to avoid mismatched duplicates. Several other crypto crates are patched in `[patch.crates-io]` to Polymesh forks.

## Verification (order used by CI)

```sh
./scripts/rustfmt.sh                     # == cargo fmt -- --check
SKIP_WASM_BUILD=1 cargo clippy -- -A clippy::all -W clippy::complexity -W clippy::perf   # non-standard flags
./scripts/test.sh                        # canonical unit-test subset (sets SKIP_WASM_BUILD/RUST_BACKTRACE)
cargo test -p <crate>                    # single package, e.g. -p pallet-asset
```

Two extra CI checks that break silently-unrelated-looking PRs:
- `./scripts/check_spec_and_cargo_version.sh` — all three runtimes must share one identical `spec_version`, encoded `8_001_000` ⇔ workspace version `8.1.0`. Bump both together.
- `./scripts/check_storage_versions.sh` — each pallet's `storage_migration_ver!` must equal its `StorageVersion::new(...)`. Update both whenever pallet storage changes.

## Layout

- Root workspace: node (`src/`), `pallets/`, `primitives/`, `rpc/`, `worker/`, `native-crypto/`, `precompiles/` (EVM precompiles with Solidity interfaces).
- **Three runtimes**: `pallets/runtime/{develop,testnet,mainnet}` — runtime changes usually need wiring in all three. Shared config in `pallets/runtime/common`; shared tests in `pallets/runtime/tests` (`polymesh-runtime-tests`, ext_builder-based).
- Pallet weights live centrally in `pallets/weights/src/*.rs`, not inside pallets.
- `integration/` and `metadata-tools/` are **separate** cargo workspaces (own lockfiles/toolchains), excluded from the root.
- Dev chains: `--dev` / `--chain dev` (develop runtime), plus `--chain testnet-dev`, `--chain mainnet-dev`. Raw chain specs tracked in `src/chain_specs/`.

## Integration tests (`integration/`)

They drive a **live chain over RPC**, not an in-process mock. Detailed authoring rules live in [`integration/AGENTS.md`](integration/AGENTS.md) — read that before adding or debugging tests.

**Chain + eth-rpc are long-lived dependencies.** Prefer the user starts them (or start once as a background task); do **not** restart or kill them mid-session unless asked. Match CI (`rust-integration-test` in `.circleci/config.yml`):

```sh
# Build once (from repo root). Prefer ci-runtime for local runs.
cargo build --locked --release --features ci-runtime

# Terminal / background 1 — Polymesh node (WS :9944)
./target/release/polymesh --bob --dev --tmp --pool-limit 100000 \
  --unsafe-force-node-key-generation --no-prometheus --no-telemetry

# Terminal / background 2 — eth-rpc (HTTP :8545); required for revive_* tests
docker run --rm --name parity-eth-rpc --network host \
  paritypr/eth-rpc:stable2606-73b734d9 \
  --node-rpc-url ws://127.0.0.1:9944 \
  --rpc-port 8545 --rpc-cors=all --allow-unprotected-txs

# Terminal 3 — tests (node must already be up before first compile if using download_metadata)
export POLYMESH_NODE_URL=ws://127.0.0.1:9944
export ETH_RPC_URL=http://127.0.0.1:8545
cd integration && ./reset_db.sh   # only after a fresh/restarted chain
cargo nextest run --release --features current_release,timed --locked
```

- Default feature `current_release` pins the matching `polymesh-api` version (`previous_release` exists for upgrade testing).
- `timed` gates tests that wait on blocks/timestamps; CI always enables it.
- `download_metadata` (enabled under `current_release`) codegen needs the node **up before** `cargo` starts.
- After any chain wipe/restart: `cd integration && ./reset_db.sh` before re-running tests.

## Generated artifacts committed to git (CI lint verifies freshness)

Regenerate and commit after editing sources:
- `integration/contracts/artifacts/*` ← `integration/contracts/build.sh` after editing `.sol` files (needs `solc` 0.8.33; `resolc` optional for PolkaVM blobs).
- `precompiles/src/interfaces/FungibleAssetStub.bin` ← `scripts/build_precompile_stub.sh` (requires exactly solc 0.8.33).
- `worker/*.polkavm|.wasm` protocol blobs ← rebuild scripts under `worker/`.
- `.metadata/<chain>/*.meta` snapshots ← compared against running dev/testnet/mainnet nodes by `metadata-tools check`; intentional extrinsic/storage metadata changes require regenerating snapshots or CI fails.

## Misc

- Benchmarks need a release binary built with `--features runtime-benchmarks` (see README); resulting weight updates go into `pallets/weights/src/`.
- Branches: `develop` is the working branch; `staging` leads releases; `mainnet`/`testnet` track deployed code. Docker publishes/releases only trigger off these branches.
