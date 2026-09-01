# 11 — Checkpoints (Balance Snapshots)

Sources: `pallets/asset/src/checkpoint/mod.rs`, `primitives/src/checkpoint.rs`.
Related specs: [04-asset-lifecycle](04-asset-lifecycle.md), [12-corporate-actions](12-corporate-actions.md)
(main consumer — ballots and capital distributions read balances "as of" a checkpoint).

## 1. Purpose

A checkpoint captures every holder's balance and the total supply of an asset at a moment, using
**copy-on-write**: nothing is stored at creation; a holder's pre-change balance is recorded the
first time it changes after the checkpoint. Corporate actions use checkpoints so votes/dividends
are computed from balances at the record date regardless of later transfers.

## 2. Storage (pallets/asset/src/checkpoint/mod.rs)

| Item | Key → Value | Ref |
|---|---|---|
| `CheckpointIdSequence` | AssetId → `CheckpointId` (first = 1) | :154 |
| `Timestamps` | (AssetId, CheckpointId) → Moment | :178 |
| `TotalSupply` | (AssetId, CheckpointId) → Balance at checkpoint | :123 |
| `Balance` | ((AssetId, CheckpointId), DID) → recorded balance | :137 |
| `BalanceUpdates` | (AssetId, DID) → `Vec<CheckpointId>` where a record exists | :161 |
| `SchedulesMaxComplexity` | global cap on aggregate pending scheduled points | :192 |
| `ScheduleIdSequence` / `ScheduledCheckpoints` / `SchedulePoints` / `ScheduleRefCount` | schedule machinery (§4) | :198/:216/:243/:235 |
| `CachedNextCheckpoints` | AssetId → next-due cache across schedules | :208 |

## 3. Core mechanics

- **Manual creation**: `create_checkpoint` (call 0, :285) — asset agent
  (`ExternalAgents::ensure_perms`, :288); records `TotalSupply` + `Timestamp` only
  (`create_at`, :632-646).
- **Copy-on-write updates**: `advance_update_balances` (:428-435) is invoked by the asset pallet
  **before every `BalanceOf` mutation** with the pre-change (did, balance) pairs — issue
  (asset lib.rs:4127-4131), redeem (lib.rs:2252-2255), transfer (lib.rs:4193-4199).
  `update_balances` (:442-454) writes the pre-change balance under the *latest* checkpoint only
  if that DID has no record there yet, and appends to `BalanceUpdates`.
- **Reads**: `balance_at(asset, did, cp)` (:404-424) — binary-search the DID's `BalanceUpdates`
  for the first recorded checkpoint ≥ cp (`find_ceiling` :683); if none, the balance hasn't
  changed since, so callers fall back to the **current** balance (`Asset::get_balance_at`,
  asset lib.rs:3357-3360).

## 4. Schedules

- `ScheduleCheckpoints` = ordered set of future moments (primitives/src/checkpoint.rs:30-33);
  `from_period` caps a repeating period at 10 points (checkpoint.rs:46-64).
- `create_schedule` (call 2, :334) — agent; non-empty, all moments future, per-schedule **and**
  aggregate pending count ≤ `SchedulesMaxComplexity` (:534-555); protocol fee
  `CheckpointCreateSchedule` (:563). `set_schedules_max_complexity` (call 1, :302) is root/PIP.
- `remove_schedule` (call 3, :360) — agent; fails with `ScheduleNotRemovable` if
  `ScheduleRefCount > 0` (:588-591). Corporate actions take refs on schedules they depend on
  (`inc_schedule_ref` :649, CA side pallets/corporate-actions/src/lib.rs:1132/1143).
- **Lazy materialization**: scheduled checkpoints are created inside `advance_schedules`
  (:457-523) — i.e. only when the *next balance-mutating operation* of that asset occurs after
  the due moment. There is **no `on_initialize` hook**. Due moments become checkpoints
  (`create_at` :502-506) with the *scheduled* timestamp; exhausted schedules are deleted
  (:493-496); `CachedNextCheckpoints` maintained (:512-519).

Consequence: a checkpoint's `Timestamps` value can predate its actual creation block. Consumers
(CAs) treat the scheduled moment as authoritative. A dormant asset (no transfers) materializes
overdue checkpoints only on its next activity — reads via `balance_at` before materialization
return `None`/current-balance, which is consistent because no balance changed in between.

## 5. Invariants & review checklist

- [ ] Asset pallet must call `advance_update_balances` with **pre-change** balances before
      *every* `BalanceOf` write — any new mint/burn/transfer path included; missing calls
      silently corrupt historical balances.
- [ ] `Balance` records are immutable once written (first-write-wins per (cp, did));
      no code should overwrite them.
- [ ] `ScheduleRefCount` discipline: CA code must inc on attach and dec on detach
      (corporate-actions lib.rs:1109-1117), else schedules become unremovable or vanish
      under a dependent CA.
- [ ] `SchedulesMaxComplexity` bounds per-transfer work in `advance_schedules`; schedule
      creation must keep enforcing the aggregate cap (:542-548).
- [ ] `CheckpointIdSequence` monotonicity — ids order checkpoints for `find_ceiling`.

## 6. Test map

`pallets/runtime/tests/src/asset_test.rs` — `checkpoints_fuzz_test` (:354), schedule tests
(:737-953); `corporate_actions_test.rs` — scheduled-checkpoint consumption
(`vote_scheduled_checkpoint` :1733, `dist_claim_scheduled_checkpoint` :2328).
