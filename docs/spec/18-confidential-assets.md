# 18 — Confidential Assets (DART)

Sources: `pallets/confidential-assets/src/{lib.rs,settlement.rs,curve_tree.rs}`,
`worker/` (proof-execution engine), external crate `polymesh-dart`
(github.com/PolymeshAssociation/polymesh-dart, pinned in root `Cargo.toml:39-41`).
Availability: **develop & testnet only** (pallet index 70); not in mainnet.
Related specs: [14-fees-and-extensions](14-fees-and-extensions.md), [README](README.md) runtime matrix.

## 1. Purpose & privacy model

Confidential assets provide **sender, receiver, asset-id and amount privacy** (lib.rs:4-13) via
the DART protocol (ZK proofs implemented in the external `polymesh-dart` crate):

- No plaintext balances on chain: account/asset state exists as commitments — leaves in
  **curve trees**. State transitions consume the old state with a **nullifier** and append a new
  commitment (lib.rs:2477-2496, 2564-2618).
- Registration is public (accounts/keys map to DIDs: `AccountDid`, `EncryptionKeyDid`
  lib.rs:773-779), but transfer extrinsics require only `ensure_signed` — the submitting origin
  proves authority *inside the ZK proof*, not via the identity pipeline (e.g. `create_settlement`
  lib.rs:1418, affirmations :1441/:1464).
- Membership proofs reference a historical tree **root** (`root_block`); the pallet accepts only
  roots younger than configured maximums — bounding prover staleness and enabling pruning.
- Proof verification runs **outside the runtime** in the polymesh-worker engine (§6).

## 2. Data model & storage (pallets/confidential-assets/src/lib.rs)

- Assets: `Details` (supply/owner/data :734), `Keys` — per-asset **auditors**
  (decrypt-only observers) and **mediators** (must affirm), bounded by `MaxAssetAuditors`/
  `MaxAssetMediators` = 2 (:739, :68-74); every asset needs ≥1 auditor or mediator
  (`NoAuditorsOrMediators`, :2521-2525). `NextAssetId` :730, names/symbols/decimals :744-755.
- Accounts: `AccountDid`/`EncryptionKeyDid`/`DidAccounts` (:773-791),
  `AccountAssetRegistrations` (per-asset init guard :814-822), `FeeAccountDid` (:795).
- **Three curve trees** (storage groups):
  - Asset tree (mutable leaves; leaf index = asset id): `AssetLeaves`/`AssetInnerNodes`/
    `AssetCurveTreeCurrentRoot`/`AssetCurveTreeRoots`(historical)/last-update/last-pruned
    (:828-865).
  - Account tree (append-only + nullifiers): `AccountLeaves`, `NextAccountLeafIndex`,
    `LastCommittedAccountLeafIndex` (batched inserts), inner nodes, roots,
    `AccountStateCommitmentNullifiers` (:871-928).
  - Fee-account tree (append-only + nullifiers): mirror set (:934-998).
- Settlements: `SettlementState` (:1002), `SettlementLegs` (encrypted legs :1020-1025),
  `SettlementPendingAffirmations` / `SettlementPendingFinalizations` (:1035-1042),
  `LegAffirmationStatus` (:1047-1056), memo (:1007).
- Worker session: `CurrentWorkerSessionId` (:1060).

## 3. Extrinsics (selected; full list lib.rs:1153-1883)

| Call (idx) | Who | Behavior | Ref |
|---|---|---|---|
| `register_accounts(0)` / `register_encryption_keys(1)` | permissioned DID | link account/encryption keys to DID (proof-verified) | :1153/:1210 |
| `create_asset(2)` | permissioned DID (issuer) | asset with auditor/mediator key sets | :1250 |
| `register_account_assets(3)` | account owner | init per-asset account state (batched proof) | :1282 |
| `mint_asset(4)` | account owner **and** asset owner | supply-capped (`MaxTotalSupply = polymesh_dart::MAX_BALANCE`) | :1353-1372 |
| `create_settlement(5)`, `sender/receiver/mediator_affirmation(6-8)`, `sender_update_counter(9)`, revert affirmations (10/11), `receiver_claim(12)`, `batched_settlement(13)` | **any signed account** — authority proven in ZK | §4 | :1414-1601 |
| `register_fee_accounts(14)` / `topup_fee_accounts(15)` | permissioned DID | move public POLYX into the pallet fee pool; private fee balance becomes a fee-tree commitment | :1620/:1689 |
| `submit_batched_proofs(16)` | any signed | atomic batch of ops | :1771 |
| `relayer_submit_batched_proofs(17)` | any signed **relayer** | private fee payment (§5) | :1800 |
| `execute_instant_settlement(18)`, instant affirmations (19/20) | any signed | single-tx create+affirm+execute | :1822-1883 |

No freeze/burn extrinsics; no manual root updates (hooks maintain roots, §6).

## 4. Settlement flow (settlement.rs)

- Parties per leg: Sender, Receiver, 0..N Mediators (`LegAffirmParty` :36-40). Legs stored
  encrypted. Creation verifies a `SettlementProof` against an **asset-tree root** (lib.rs:
  1970-1987); pending affirmations = (2 + mediators) per leg (:1998-2000).
- Sender/receiver affirmations carry an account-state update: nullifier spend + new commitment,
  verified against an **account-tree root** (:240-331; lib.rs:2477-2496). Mediator affirmation
  carries `accept: bool` — reject flips the settlement to Rejected (lib.rs:2246-2259).
- State machine (`SettlementStatus` :83-88): `Pending → Executed → Finalized` or
  `Pending → Rejected → Finalized`. Execution is a status flip when pending affirmations hit 0
  (:484-551); actual balance effects happen through each party's own proofs — sender finalizes
  with `sender_update_counter` (:361-385), receiver credits funds with `receiver_claim`
  (:458-482). When pending finalizations hit 0, storage is pruned (`finalize` :554-597).
- Reverts: sender/receiver can revert affirmations while Pending/Rejected (:390-455); reverting
  a Pending settlement rejects it (:416-419). Transition guards in `set_party_status`
  (:135-198).

## 5. Private fee payment (fee accounts + relayer)

Normal confidential extrinsics pay public POLYX fees, which links the submitting key. For full
privacy:
1. Users pre-fund **fee accounts**: public POLYX is deposited into the pallet pool account
   (`PalletId "pm/dartf"`, lib.rs:92-95); the user's balance becomes a private commitment in the
   fee-account tree (:1620-1673, :1689-1758).
2. A third-party **relayer** submits the user's batched proofs via
   `relayer_submit_batched_proofs` (:1800 → :2338-2386): the `FeePaymentWithBatchedProofs`
   includes a `FeeAccountPaymentProof` bound to the batch content hash (:2356), spending the
   user's private fee balance (nullifier double-spend check :2411). The pool reimburses the
   relayer publicly (:2429); the fee must cover the weight-derived tx fee (:2399-2402,
   commission permitted). **The batch rolls back atomically on failure but the relayer is still
   paid** (:2360-2383) — relayers are compensated for wasted work; users must trust their proofs.

This decouples the fee-paying origin from sender/receiver/mediator identities.

## 6. Curve trees, roots & the worker engine

- Trees are recomputed **once per block**: extrinsics only append leaves; `on_finalize` commits
  batched leaves and re-roots (`finalize_block` lib.rs:2936-2944, `commit_leaves_to_tree`
  :2895-2913); `on_initialize` starts the per-block worker session and prunes old roots
  (:2926-2934).
- Roots are timestamped per block (`TimestampedTreeRoot`, curve_tree.rs:159-254) and kept
  historically; a proof's `root_block` is accepted only if younger than
  `MaxAssetCurveTreeRootAge` (24h) / `MaxAccountCurveTreeRootAge` (2d) /
  `MaxFeeAccountCurveTreeRootAge` (2d) (curve_tree.rs:50-148; testnet values
  `pallets/runtime/testnet/src/runtime.rs:112-116`; short values under `ci-runtime`).
- `MinCurveTreeRootUpdateInterval` (10 min): quiescent trees get their root re-stamped so provers
  always find a fresh root (lib.rs:2735-2786). Pruning keeps ~100 recent blocks, ≤10 pruned per
  block (lib.rs:97-105, 2788-2893).
- **Worker**: proof verification executes in `polymesh-worker` (native or chain-upgradeable
  PolkaVM/WASM modules — committed blobs `worker/polymesh-worker-protocol-dart-v1.*`;
  `worker/README.md`). The pallet calls it synchronously via a runtime interface
  (`submit_and_wait` lib.rs:2970-2974); request enum `VerifyDartAssetRequest`
  (worker/protocol/dart-v1/src/verify.rs:30-124) covers registration, minting, settlement,
  affirmation, revert, claim, fee-payment and key-distribution proofs. A worker panic ⇒ proof
  rejected (worker/native/src/lib.rs:48-74).

## 7. Invariants & review checklist

- [ ] **Nullifier uniqueness** (`NullifierAlreadyUsed`) is the double-spend defense — every
      state-consuming proof path must check-and-insert atomically.
- [ ] Root age windows bound proof staleness; extending max ages or pruning windows changes the
      security/liveness tradeoff — keep `RECENT_BLOCKS_TO_KEEP` ≥ age windows in blocks.
- [ ] Leaf commits are batched per block; any new leaf-writing path must go through the
      `NextLeafIndex`/`LastCommitted*` machinery or root recomputation misses leaves.
- [ ] Settlement pending counters (affirmations/finalizations) must be exact — premature zero ⇒
      premature execute/finalize.
- [ ] Fee-pool solvency: pool balance must always cover Σ outstanding private fee balances
      (deposits on register/topup; withdrawals only via verified payment proofs).
- [ ] Proof context binding: batched proofs bind to content hash (`fee_payment_ctx`), settlement
      proofs bind to root_block — never verify a proof without its binding context.
- [ ] Worker sessions must bracket every block (`NoCurrentWorkerSession` guard); upgrading the
      worker protocol blob is consensus-critical.

## 8. Test map

Pallet `testing.rs` (off-chain prover mirroring all trees, proof generation via the worker
testing module). Integration: `integration/tests/confidential_transfers.rs` (+`_negative.rs`),
helper `integration/src/confidential_assets_helper.rs` (client-side proof building).
Worker backends: `worker/tester/` with sample proofs in `worker/tester/data/`.
