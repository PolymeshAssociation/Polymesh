# 10 — Settlement Engine

Sources: `pallets/settlement/src/lib.rs`, `primitives/src/settlement.rs`,
`primitives/src/crypto.rs` (receipts).
Related specs: [09-asset-transfers](09-asset-transfers.md) (leg execution &
`transfer_funds`), [08-portfolio](08-portfolio.md) (custody/locks), [04-asset-lifecycle](04-asset-lifecycle.md)
(mandatory mediators, venue-filter admin).

## 1. Purpose

Atomic multi-leg, multi-party asset exchanges (DvP etc.). Parties **affirm** an instruction
(locking their outgoing assets); when all affirmations (holders + off-chain receipts +
mediators) are in, the instruction executes all legs atomically. Includes venue scoping,
off-chain receipts, per-instruction mediators, and a two-phase-commit **lock** mode for
off-chain/cross-chain coordination.

## 2. Data model (primitives/src/settlement.rs)

- `InstructionStatus`: `Unknown` (pruned/invalid) | `Pending` | `Failed` | `Success(block)` |
  `Rejected(block)` | `LockedForExecution` (:50-64).
- `SettlementType`: `SettleOnAffirmation` | `SettleOnBlock(b)` | `SettleManual(b)` (executable
  on/after b) | `SettleAfterLock` (:128-138).
- `Leg`: `Fungible { sender, receiver: AssetHolder, asset_id, amount }` |
  `NonFungible { sender, receiver, nfts }` |
  `OffChain { sender_identity, receiver_identity, ticker, amount }` (:182-214).
- `LegStatus`: `PendingTokenLock` | `ExecutionPending` (locked) |
  `ExecutionToBeSkipped(signer, receipt_uid)` (off-chain, receipt claimed) (:84-92).
- `AffirmationStatus`: `Unknown | Pending | Affirmed` (:97-105);
  `MediatorAffirmationStatus`: `Unknown | Pending | Affirmed { expiry }` (:755-766).
- `Instruction { venue_id: Option<VenueId>, settlement_type, trade_date, value_date, ... }`
  (:164-177).
- Receipts: `Receipt { instruction_id, leg_id, sender/receiver identity, ticker, amount }`
  (:248-261); `ReceiptDetails { uid, instruction_id, leg_id, signer, signature, expires_at,
  metadata }` (:292-307).

Storage highlights (pallets/settlement/src/lib.rs): `VenueInfo` (:613, `Venue { creator,
venue_type }`), `VenueSigners` (:631), `NumberOfVenueSigners` (:759), `VenueFiltering` /
`VenueAllowList` (:707/:712), `InstructionDetails` (:644), `InstructionLegs` (:741),
`InstructionLegStatus` (:654), `InstructionAffirmsPending` (:666), `AffirmsReceived` (:671),
`UserAffirmations` (:689), `OffChainAffirmations` (:747), `ReceiptsUsed` (:702),
`InstructionMediatorsAffirmations` (:764), `InstructionStatuses` (:731),
`MandatoryReceiverAffirmation` (:683), `LockedTimestamp`/`UnlockedTimestamp`/
`InstructionRelockCount` (:776/:781/:786), `VenueCounter`/`InstructionCounter` (:718/:722).

Runtime constants (mainnet `pallets/runtime/mainnet/src/runtime.rs:97-106`; develop differs on
lock timings, `develop/src/runtime.rs:98-107`): `MaxNumberOfFungibleAssets` 10,
`MaxNumberOfNFTsPerLeg` 10, `MaxNumberOfNFTs` 100, `MaxNumberOfOffChainAssets` 10,
`MaxNumberOfVenueSigners` 50, `MaxInstructionMediators` 4, `MaximumLockPeriod` 24h (develop
24min), `RelockCooldown` 4h (develop 10min), `MaxRelockCount` 3.
**No protocol fees in settlement** — costs are weight-based only.

## 3. Venues

- `create_venue(0)` (:816) — **any permissioned DID**; details length-limited; initial signers ≤
  `MaxNumberOfVenueSigners`. `update_venue_details(1)` / `update_venue_type(2)` /
  `update_venue_signers(7)` — **venue creator only** (`ensure_venue_creator` :1748-1752;
  signers add/remove :2675-2728).
- **Instruction↔venue rule**: instructions may have `venue_id: None`; if `Some`, **only the venue
  creator can create the instruction** (:1792-1794). Venue signers exist solely to sign off-chain
  receipts. Off-chain legs require a venue (`OffChainAssetsMustHaveAVenue` :2928).
- **Venue filtering** (per asset): asset agents toggle `set_venue_filtering(4)` and manage the
  allow-list via `allow_venues(5)` / `disallow_venues(6)` (agent-gated,
  `ExternalAgents::ensure_perms` :933/:957/:981). Enforced at **instruction creation** per leg
  (`ensure_venue_filtering` :2857-2870 — filtering on ⇒ instruction must have an allowed venue)
  and **re-checked at execution and lock** (`ensure_allowed_venue` :2841 from
  `validate_execute_instruction_pre_conditions` :2094). Disallowing a venue after creation
  blocks execution.

## 4. Instruction lifecycle

### Creation

`add_instruction(9)`, `add_and_affirm_instruction(10)`, `*_with_mediators(19/20)`,
`*_with_count(15/16/17)` variants → `base_add_instruction` (:1754-1898):

- `SettleOnBlock` must be a future block (:1765); **`SettleAfterLock` requires ≥1 instruction
  mediator** (:1771-1779); value_date ≥ trade_date (:1784).
- Per-leg validation (`ensure_valid_leg` :2886-2936): fungible/NFT sender-DID ≠ receiver-DID
  (`SameSenderReceiver` :2901/:2915), amount > 0, venue filtering, NFT per-leg caps, off-chain
  distinct identities. Instruction-wide caps (:2990-3004).
- Pending-affirmation count = sender holders + non-pre-approved receivers + off-chain legs +
  mediators (primitives :538-542). Receiver auto-affirm policy: doc 09 §5. **Asset-level
  `MandatoryMediators` are merged into the mediator set** (:1948-1949).
- `SettleOnBlock` schedules execution via the substrate scheduler under a named task
  (`schedule_instruction` :2344-2370, root origin, priority constant).

### Affirmation

`affirm_instruction(11)` → `base_affirm_instruction` (:2467): caller must control the holder
(custody+perms via `Asset::ensure_holder_permissions`, asset lib.rs:3889-3906) and the
affirmation must be Pending (:2487-2503). Affirming **locks the sender-side assets**
(`lock_asset` :1697-1713 → portfolio/account locked balances, NFT locks). When pending count
hits 0 and type is `SettleOnAffirmation`, execution is scheduled for the next block
(`maybe_schedule_instruction` :2323-2337).

**There is no affirmation withdrawal** — the withdraw extrinsics were removed (call indices
12/18/22 are gaps; `AffirmationWithdrawn`/`MediatorAffirmationWithdrawn` events :125/:175 are
declared but never emitted). To back out, a party **rejects** the instruction.

### Mediators

Per-instruction mediators (bounded `MaxInstructionMediators`) + per-asset mandatory mediators.
`affirm_instruction_as_mediator(21)` (:1398 → :3256-3310) with optional **expiry** — expired
mediator affirmations block execution (`MediatorAffirmationExpired`, checked :2126-2148).
`reject_instruction_as_mediator(23)` (:1416). Mediators count toward pending affirmations.

### Rejection

`reject_instruction(13)` → `base_reject_instruction` (:2730-2817). Pending/Failed: any party
holder, venue creator, mediator, or off-chain-leg party (`ensure_valid_caller` :3317-3346).
LockedForExecution: **mediator only**, unless the lock period has expired — then any valid party
(:2777-2794). Releases locks, cancels scheduled task, prunes, sets `Rejected(block)`.

### Execution

- Scheduled: `execute_scheduled_instruction(14)` — **root only** (:1196), dispatched by the
  scheduler; failure emits `FailedToExecuteInstruction` and marks `Failed` (:2873-2882).
- Manual: `execute_manual_instruction(8)` (:1024 → :3011-3097) with leg counts + weight limit
  (RPC `get_execute_instruction_info` supplies them). Branches: Pending requires
  `SettleManual(b)` reached (:3099-3113); **Failed = retry by any valid caller**; Locked =
  mediator-only fast path (§6).
- Core: `execute_instruction` (:2017-2070) — pre-conditions (status, all affirmations incl.
  mediator expiry, venue allow-list :2077-2148) → transactional `release_locks` (:2312) +
  `transfer_assets` (:2215-2260 → `Asset::base_transfer` / `Nft::base_nft_transfer`; off-chain
  legs are no-ops :2255) → prune → `Success(block)`. A failing leg emits `LegFailedExecution`
  and the whole instruction rolls back to `Failed`.

### State machine

```
Unknown ──add──▶ Pending ──lock──▶ LockedForExecution
                  │  ▲                    │ │
                  │  └──────unlock────────┘ │
   execute ok ────┼────────────────────────▶│ Success(block)   (terminal)
   execute err ──▶ Failed ──retry ok──▶ Success
                  Pending/Failed/Locked ──reject──▶ Rejected(block) (terminal)
```
Transitions: Pending :1808; Locked :3470; unlock→Pending :3488; Success :2053/:3541;
Failed :2010-2012; Rejected :2809-2812. Terminal statuses persist; everything else is pruned
(`prune_instruction` :2273-2310).

## 5. Off-chain legs & receipts

`Leg::OffChain` represents value moving outside the chain (e.g. fiat). Each off-chain leg must be
affirmed with a **receipt** signed by a **venue signer**:

- `affirm_with_receipts(3)` (:903 → :2374-2465): instruction must have a venue (:2390);
  per-receipt validation (:3153-3217): instruction id match, unique (signer, uid)
  (`DuplicateReceiptUid`), one receipt per leg, signer ∈ `VenueSigners`
  (`UnauthorizedSigner`), not replayed (`ReceiptsUsed` ⇒ `ReceiptAlreadyClaimed`), leg is
  OffChain and Pending.
- Signature = sr25519/ed25519 over `ChainScopedMessage { genesis_hash, uid,
  "Polymesh Settlement Receipt", expires_at, Receipt {...} }` (:3191-3209;
  primitives/src/crypto.rs:89,93) — chain-scoped, expiring, uid-replay-protected.
- Effects: leg → `ExecutionToBeSkipped(signer, uid)`, `ReceiptsUsed[signer][uid] = true`,
  off-chain affirmation Affirmed, pending count −1 (:2415-2443). At execution the leg is
  skipped (asset already moved off-chain). `mark_receipt_as_used` (:3138) is also called by STO
  (pallets/sto/src/lib.rs:993).

## 6. Locking — two-phase commit (`SettleAfterLock`)

Purpose: guarantee an instruction *will* execute (e.g. after an off-chain/cross-chain
counterpart settles), by freezing validation outcomes at lock time.

- `lock_instruction(24)` (:1453 → `base_lock_instruction` :3384-3475): **mediator only**
  (:3392); type must be `SettleAfterLock` (:3396). Validation at lock = full pre-conditions
  (all affirmations, mediator expiry, venue allow-list, :3451) **plus a complete execution
  dry-run in a storage transaction that always rolls back** (:3453-3468) — compliance,
  statistics, balances are all exercised. On success: status `LockedForExecution`,
  `LockedTimestamp = now` (:3470-3471), event `InstructionLocked`.
- Execution of a locked instruction: `execute_manual_instruction` locked branch (:3065-3087),
  **mediator only** → `simplified_asset_transfer` (:3501-3547): requires
  `now − LockedTimestamp ≤ MaximumLockPeriod` (:3550-3561, `ExceededMaximumLockingPeriod`);
  releases locks; **fungible: compliance skipped, statistics re-verified**
  (asset `simplified_fungible_transfer` lib.rs:4381-4429, stats at :4405-4414); **NFT:
  compliance skipped**, ownership/frozen-holder checked (nft :953-981) → `Success`.
  The bounded lock period is what makes skipping sound: rule changes made after locking take
  effect only once the lock expires (execution then requires unlock/relock, re-running full
  validation).
- `unlock_instruction(27)` (:1554 → :3479-3494): mediator only; → `Pending`,
  records `UnlockedTimestamp`.
- Relock protections: relock over a live lock only after `MaximumLockPeriod + RelockCooldown`
  (:3402-3413); after explicit unlock, `RelockCooldown` applies (:3414-3421); total relocks ≤
  `MaxRelockCount` (:3426-3435).
- Reject-while-locked: mediator only within the lock window; anyone valid after expiry
  (:2777-2794).

## 7. Invariants & review checklist

- [ ] Affirmation ⇒ lock: every path that marks a holder affirmation Affirmed must lock the
      sender-side assets, and every terminal path (execute/reject) must release exactly those
      locks (`release_locks` :2312-2319).
- [ ] `InstructionAffirmsPending` must equal outstanding (holders + off-chain legs + mediators);
      double-decrements would enable premature execution (first-affirm-only decrement for
      mediators :3287-3289).
- [ ] Locked instructions: no full re-validation at execution *by design* — any new check added
      to `validate_asset_transfer` must be considered for `simplified_*` too, or documented as
      lock-skipped; the lock dry-run and `MaximumLockPeriod` are the safety envelope.
- [ ] Venue allow-list must be enforced at creation **and** execution/lock (assets rely on
      revocation working for pending instructions).
- [ ] Receipt security: signer ∈ `VenueSigners` at claim time; (signer, uid) never reusable;
      chain-scoped signatures only.
- [ ] Same-DID legs must stay rejected (`SameSenderReceiver`).
- [ ] Pruning must never delete `InstructionStatuses` terminal states (audit trail).
- [ ] `SettleAfterLock` must keep requiring ≥1 mediator (:1771-1779) — mediators are the only
      actors who can lock/execute/unlock.

## 8. Test map

`pallets/runtime/tests/src/settlement_pallet/` — add_instruction, execute_instruction,
lock_instruction (565 lines), unlock_instruction, manual_execution, reject_instruction,
allow_disallow_venues, transfer_funds; plus legacy `settlement_test.rs` (venue caps, filtering,
NFT leg limits, receipts). Integration: settlement flows in `integration/tests/`.
