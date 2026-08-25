# Integration tests — agent guide

Live-chain tests over RPC (not in-process FRAME mocks). Root overview: [`../AGENTS.md`](../AGENTS.md). CI reference: job `rust-integration-test` in `../.circleci/config.yml`.

## Prerequisites (node + eth-rpc)

**Do not start, restart, or kill the chain/eth-rpc unless the user asks or nothing is listening.** Prefer a node the user already started, or one background task for the whole session. Tail logs; do not block the session on the node process.

| Service | Default | Who needs it |
| --- | --- | --- |
| Polymesh node (WS) | `ws://127.0.0.1:9944` | all tests |
| eth-rpc (HTTP) | `http://127.0.0.1:8545` | `revive_*` only |

Suggested local startup (matches CI; run from **repo root**):

```sh
# Binary: prefer a fresh ci-runtime build (stable under load).
cargo build --locked --release --features ci-runtime
./target/release/polymesh --bob --dev --tmp --pool-limit 100000 \
  --unsafe-force-node-key-generation --no-prometheus --no-telemetry

# eth-rpc (docker). Start after the node is accepting WS.
docker run --rm --name parity-eth-rpc --network host \
  paritypr/eth-rpc:stable2606-73b734d9 \
  --node-rpc-url ws://127.0.0.1:9944 \
  --rpc-port 8545 --rpc-cors=all --allow-unprotected-txs
```

Env (export before compile **and** run):

```sh
export POLYMESH_NODE_URL=ws://127.0.0.1:9944   # required for download_metadata codegen
export ETH_RPC_URL=http://127.0.0.1:8545        # revive_* only
# optional: WAIT_FOR_FINALIZE=1
```

Sanity checks:

```sh
# node RPC
curl -s -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"system_health","params":[]}' \
  http://127.0.0.1:9933
# eth-rpc
curl -s -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"eth_blockNumber","params":[]}' \
  http://127.0.0.1:8545
```

After **any** chain wipe/restart (new `--tmp` dir, killed process, etc.):

```sh
cd integration && ./reset_db.sh   # resets accounts.db used by PolymeshTester
```

Do **not** run `reset_db.sh` between ordinary test runs on a healthy chain.

## Running tests

Always from `integration/` (separate workspace; own lockfile/toolchain):

```sh
cd integration
# full suite (what CI runs)
cargo nextest run --release --features current_release,timed --locked

# one binary / one test
cargo nextest run --release --features current_release -E 'binary(asset_controls)'
cargo nextest run --release --features current_release -E 'test(make_divisible)'

# compile only (catch warnings)
cargo nextest run --release --features current_release,timed --no-run
```

- Feature `current_release` (default): tests against current runtime + live metadata download.
- Feature `timed`: enables tests that sleep/poll for blocks or timestamps (`settlement_scheduling`, scheduled checkpoints, ballot windows, distribution reclaim, etc.). Gate those with `#[cfg(feature = "timed")]` (and usually `current_release`).
- Feature `previous_release`: upgrade-path suite; do not mix with `current_release` helpers blindly.
- nextest runs binaries **concurrently** — unique user names and tickers per test are mandatory.

## Writing a new test

1. **Find a sibling** under `tests/` for the same pallet/area and copy structure (module gate, imports, helpers).
2. **Gate the module**:
   ```rust
   #[cfg(feature = "current_release")]
   mod my_area_tests { /* ... */ }
   ```
   Timed-only files/tests: `#[cfg(all(feature = "current_release", feature = "timed"))]` or `#[cfg(feature = "timed")]` inside a `current_release` module.
3. **Scaffold**:
   ```rust
   #[tokio::test]
   #[test_log::test]
   async fn short_behavior_name() -> anyhow::Result<()> {
       let mut tester = PolymeshTester::new().await?;
       let mut users = tester.users(&["UniqueOwner", "UniqueInv"]).await?.into_iter();
       let mut owner = users.next().unwrap();
       // ...
       Ok(())
   }
   ```
4. **Reuse helpers** from `integration` (`AssetHelper`, extractors in `src/lib.rs`, revive/eth helpers). Prefer `AssetHelper::new` / `new_full` over hand-rolled `create_asset` unless you need indivisible assets or a specific mint destination.
5. **Assert intended product behavior**, not whatever the node currently does. If chain logic is wrong: keep the intended assertion and `#[ignore = "…"]` with a short reason.
6. **Compile clean** (no new warnings) under both `current_release` and `current_release,timed` before finishing.

### Metadata / types

- `current_release` enables `download_metadata`: types come from the **live** node at compile time. Node must be up **before** the first `cargo`/`nextest` invocation that builds `polymesh-api`.
- Use generated paths under `polymesh_api::types::…` (often `polymesh_primitives::…` or `pallet_*::…`). Do not invent field names — check a passing sibling test or `integration/target/doc/polymesh_api/` after a build.
- Pass `AssetHelper.asset_id` (`integration::AssetId`) straight into API calls when using live metadata. No conversion helpers.

### Assets, balances, portfolios

- `AssetHelper::new` / `new_full` mint **divisible** assets by default (`create_asset(..., true, EquityCommon, …)`). Amounts are `u128` in perbill-style units (1 unit of a divisible asset = `1_000_000` base units).
- Mint destination matters:
  - `AssetHolderKind::Account` — needed if you later `transfer_asset` (account-based).
  - `AssetHolderKind::DefaultPortfolio` — needed for settlement legs, capital distribution payment currency, portfolio balance queries.
- Dev protocol fees (rough): unique ticker registration ~500 POLYX, create asset ~2500 POLYX. Fund users via `tester.users` (pre-funded) or `balances().transfer_with_memo`.
- Existing POLYX transfer pattern: `balances().transfer_with_memo(dest.into(), amount, None)`.

### Identity / secondary keys / claims

- Secondary keys must be **DID-less** (`tester.new_signer_idx`); fund with POLYX before `join_identity_as_key`.
- `create_custody_portfolio` requires a prior `allow_identity_to_create_portfolios` from the portfolio owner.
- Child identities were removed in v8 — do not port those tests.
- First document id is `DocumentId(0)`. Compliance requirement ids start at `1`.
- Claim expiry is wall-clock ms from `timestamp().now()`.

### Settlement

- Current release **auto-affirms receivers** on incoming fungible legs (unless mandatory receiver affirmation is set). Only the sender (and mediators) usually need to `affirm_instruction`. Affirming an already-affirmed party → `UnexpectedAffirmationStatus`.
- `lock_instruction` / `unlock_instruction` require:
  - `SettlementType::SettleAfterLock`
  - caller is an instruction **mediator**
  - all required affirmations received
  - generous `Weight` limit (e.g. `Weight::from_parts(10_000_000_000, 10_000_000)`)
- After lock, execution is mediator-driven `execute_manual_instruction` while still locked (unlock returns to `Pending`). See `pallets/runtime/tests/src/settlement_pallet/`.
- Oversize legs fail at **affirm** (asset lock), not only at execute — for all-or-nothing failure tests prefer freezing an asset after affirm rather than an impossible lock amount.

### Capital distribution / corporate actions

- Payment asset for `distribute` must sit in the issuer **default portfolio**.
- Ballot attach needs a record date (`RecordDateSpec::Existing(cp_id)`).
- CA types: `pallet_corporate_actions::{CAKind, CADetails, RecordDateSpec, TargetIdentities}`; ballot types under `pallet_corporate_actions::ballot`.
- `remove_distribution` only works **before** `payment_at`; after expiry use `reclaim`, not remove.
- NFT: `NFTCollectionKeys(pub Vec<…>)`, `NFTs { asset_id, ids }`.

### Relayer / subsidy

- Relayer nonce storage is keyed by the **target** account (`RelayTxNonces`).
- `SubsidyFilter` (runtime) allows Asset, Balances, Identity, Settlement, etc. — **not** `System.remark`. Use a filtered call (e.g. tiny `balances.transfer_with_memo`) when asserting subsidy debits.
- Count-stat exemptions apply to the **sender** DID, not the receiver.

### Statistics / compliance

- Call `set_active_asset_stats` before `batch_update_asset_stats`.
- Investor-count tests: seed stats so the issuer’s own holding is counted when the limit is tight.

### Utility / economics

- Prefer `force_batch` when asserting per-item success/failure without aborting the batch.
- Sudo and dev-chain privileges are allowed on `--dev`.
- Chain committees on dev are seeded with `IdentityId(1)`, threshold often `(1, 2)`.

### Contracts / revive

- `revive_*` tests need eth-rpc. Use `EthNode` / revive helpers in `src/eth_helper.rs`, `src/revive_helper.rs`.
- Solidity artifacts: edit `contracts/`, run `contracts/build.sh` (solc **0.8.33**), commit `contracts/artifacts/*`.

## Extractors / shared helpers

Prefer existing event extractors in `src/lib.rs` (and helpers modules) over ad-hoc event scans: e.g. `get_instruction_id`, `get_checkpoint_id`, `get_ca_id`, `get_distribution_id`, `get_ballot_id`, `get_batch_results`. Add a new extractor next to them if a pallet event is reused.

## Failure triage

| Symptom | Likely cause |
| --- | --- |
| Compile: connection refused in proc-macro | Node not up before build; export `POLYMESH_NODE_URL` |
| `Invalid Transaction` / ancient birth block | Stale era under load, or chain restarted without client reconnect — retry; avoid restarting node mid-run |
| `Custom error: 4` on subsidised call | Call not in `SubsidyFilter` (`PalletNotSubsidised`) |
| `CallerIsNotAMediator` / lock fails | Wrong settlement type or non-mediator signer |
| `UnexpectedAffirmationStatus` | Double-affirm; receiver already auto-affirmed |
| Portfolio `Insufficient balance for a transaction` | Tokens minted to Account but spent from DefaultPortfolio (or reverse) |
| nextest name collisions / random fails | Reused user or ticker names across concurrent tests |
| eth-rpc empty code / lag | eth-rpc started before node, or not synced — wait / restart eth-rpc only |

Chain-logic defects (not test bugs): keep the failing intended assertion and `#[ignore = "…"]` with a short reason; call them out to the user.

## Checklist before finishing a new test

- [ ] Unique user names + tickers
- [ ] Correct mint destination (Account vs DefaultPortfolio)
- [ ] Feature gates (`current_release` / `timed`) match CI
- [ ] No unused `mut`/imports (clean `--no-run` build)
- [ ] Ran the new test binary against the live node
- [ ] Timed paths exercised with `--features current_release,timed` if applicable
- [ ] No chain/eth-rpc restart left the session broken; `reset_db.sh` only if chain was wiped
