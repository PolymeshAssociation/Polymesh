# 12 — Corporate Actions, Ballots & Capital Distributions

Sources: `pallets/corporate-actions/src/lib.rs` (LIB), `.../ballot/mod.rs` (BAL),
`.../distribution/mod.rs` (DIST).
Related specs: [11-checkpoints](11-checkpoints.md) (record dates), [05-external-agents](05-external-agents.md)
(`PolymeshV1CAA` group), [08-portfolio](08-portfolio.md) (distribution locks),
[09-asset-transfers](09-asset-transfers.md) (benefit payouts are compliance-checked transfers).

"CAA" below = an external agent whose group grants the CA pallets — `AgentGroup::PolymeshV1CAA`
grants exactly `CorporateAction` + `CorporateBallot` + `CapitalDistribution`
(pallets/external-agents/src/lib.rs:683-687); `Full`/suitable custom groups also qualify.

## 1. Corporate actions base (LIB)

### Data model

- `CAId { asset_id, local_id }` (LIB:296-304); per-asset sequence (`CAIdSequence` LIB:417).
- `CAKind`: `PredictableBenefit | UnpredictableBenefit | IssuerNotice | Reorganization | Other`
  (LIB:182-205); `is_benefit()` = the two benefit kinds (LIB:207-212).
- `TargetIdentities { identities, treatment: Include | Exclude }` (LIB:156-163);
  **default is `Exclude` with an empty list ⇒ everyone targeted** (LIB:138-143);
  `targets(did)` via binary search (LIB:173-178).
- Withholding tax `Tax = Permill` (LIB:125): CA-level default + per-DID overrides;
  `tax_of(did)` (LIB:277-286).
- `CorporateAction { kind, decl_date, record_date: Option<RecordDate>, targets,
  default_withholding_tax, withholding_tax }` (LIB:259-275) — **targets/taxes are snapshotted
  from the asset defaults at creation** (LIB:1014-1021); later default changes don't affect
  existing CAs.
- `RecordDateSpec`: `Scheduled(Moment) | ExistingSchedule(ScheduleId) | Existing(CheckpointId)`
  (LIB:245-255) → resolved to `CACheckpoint::{Scheduled(id, idx), Existing(id)}` (LIB:219-231).

### Extrinsics

| Extrinsic (idx) | Who | Behavior | Ref |
|---|---|---|---|
| `set_max_details_length(0)` | **root** (PIP) | global cap | LIB:479-486 |
| `set_default_targets(1)` / `set_default_withholding_tax(2)` / `set_did_withholding_tax(3)` | CAA | asset-level defaults (bounded `MaxTargetIds`/`MaxDidWhts`) | LIB:501/:533/:562 |
| `initiate_corporate_action(4)` | CAA | create CA; `decl_date ≤ now` (LIB:997), `decl_date ≤ record_date` (LIB:1003-1009); record-date handling §2 | LIB:621 → :957-1038 |
| `link_ca_doc(5)` | CAA | **replace** doc links (docs must exist) | LIB:669-687 |
| `remove_ca(6)` | CAA | removes CA + attached ballot (only before start) / distribution (only before payment_at); decrements schedule ref | LIB:707-737 |
| `change_record_date(7)` | CAA | re-resolve record date; constrained by attached ballot/distribution timing | LIB:754-794 |
| `initiate_corporate_action_and_distribute(8)` / `..._and_ballot(9)` | CAA | atomic combos | LIB:796/:854 |

### Record dates & checkpoints (§ LIB:1119-1159)

- `Scheduled(date)` ⇒ creates a checkpoint schedule with **initial ref count 1** (LIB:1127-1135).
- `ExistingSchedule(id)` ⇒ pins the schedule (`inc_schedule_ref`) and records the index of its
  next checkpoint (LIB:1137-1144).
- `Existing(cp)` ⇒ uses a materialized checkpoint (LIB:1146-1151).
- Reads: `record_date_cp` (LIB:1082-1100) maps `Scheduled(id, idx)` through `SchedulePoints`;
  `balance_at_cp` (LIB:1070-1080) — **falls back to the live balance if the scheduled checkpoint
  hasn't materialized yet** (lazy checkpoints, doc 11 §4; sound because no balance change ⇒
  live == checkpoint value).

## 2. Ballots (BAL) — corporate voting

- Attach: `attach_ballot(0)` — CAA; CA kind must be **`IssuerNotice`** (`CANotNotice`
  BAL:715-718); range `start ≤ end`, `now ≤ end`; **record date required and ≤ start**
  (BAL:720, LIB:1058-1068); one ballot per CA; protocol fee `CorporateBallotAttachBallot`
  (BAL:743). Motions ≤ 8 with ≤ 128 choices each (weight guard, BAL:103-104).
- Config changes `change_end(2)` / `change_meta(3)` / `change_rcv(4)` / `remove_ballot(5)` —
  CAA, all **strictly before start** (BAL:810-817).
- `vote(1)` (BAL:437-532) — any permissioned signer whose DID is **targeted by the CA**
  (BAL:449-450):
  - within `[start, end]` inclusive (BAL:445-446);
  - one `BallotVote { power, fallback }` per choice, flat across motions; count must match
    (BAL:453-459);
  - **voting power = balance at the record-date checkpoint** (BAL:498-501 → LIB:1070-1080);
    per-motion Σ power ≤ voting power — full power is reusable across motions (BAL:503-511);
  - RCV: fallback must point to a *different* choice in the *same* motion (BAL:483-485);
    fallbacks forbidden when RCV off (BAL:489-496);
  - **re-voting replaces** the previous vote and adjusts the running tally (BAL:513-527).
- Results are a flat per-choice `Vec<Balance>` tally (BAL:326-335); RCV fallback resolution is
  an off-chain concern.

## 3. Capital distributions (DIST) — dividends

- `distribute(0)` (DIST:233 → :642-718) — CAA **with custody+permission of the source
  portfolio** (DIST:670-679): CA must be a benefit kind (`CANotBenefit`); **record date required
  and ≤ payment_at** (DIST:686-690); `amount`/`per_share` nonzero; expiry after payment;
  one distribution per CA; protocol fee `CapitalDistributionDistribute` (DIST:695); **locks
  `amount` in the source portfolio** (DIST:698-699). `Distribution { from, currency, per_share,
  amount, remaining, reclaimed, payment_at, expires_at }` (DIST:101-124).
  Note: nothing forbids `currency == the CA's asset` (compliance still applies at payout).
- Payout (`transfer_benefit`, DIST:536-605) — via `claim(1)` (holder claims own) or
  `push_benefit(2)` (CAA pushes to a holder):
  1. not already paid (`HolderPaid`), within `[payment_at, expires_at)`, holder targeted by CA;
  2. **benefit = balance_at_record_date × per_share / 1_000_000** (truncating, DIST:612-619);
  3. `remaining -= benefit` (checked);
  4. tax = `ca.tax_of(holder)`; `gain = benefit − tax·benefit`; indivisible currencies round
     gain down to whole units (DIST:568-573);
  5. the full `benefit` is **unlocked** but only `gain` is transferred — the withheld tax stays
     (unlocked) in the distributor's portfolio for off-chain remittance (DIST:566-576);
  6. transfer = **`Asset::base_transfer` to the holder's default portfolio — full compliance
     and statistics checks apply** (DIST:578-590); a non-compliant holder cannot be paid.
- `reclaim(3)` — CAA + **custodian of the source portfolio** (DIST:462-477): only after expiry;
  unlocks `remaining`, marks reclaimed. (`NotDistributionCreator` error is declared but unused —
  the real gate is custody, DIST:396-397 vs :473-477.)
- `remove_distribution(4)` — CAA; only **before `payment_at`**; unlocks everything
  (DIST:507-525).

## 4. Invariants & review checklist

- [ ] CA snapshot semantics: targets/taxes copied at initiation must stay immutable per-CA.
- [ ] Schedule ref-counting: every record-date attach/detach path must inc/dec
      (`handle_record_date`/`dec_strong_ref_count`, LIB:1108-1159) or checkpoints get removed
      under live CAs / schedules become unremovable.
- [ ] Ballot voting power and distribution benefits must read `balance_at_cp` (checkpoint), never
      the live balance directly — the live-balance *fallback* is only valid pre-materialization.
- [ ] Distribution accounting: `remaining + Σ paid benefits = amount` until reclaim;
      the lock covers `remaining` at all times (lock at create, unlock per-claim/reclaim/remove).
- [ ] Payouts must remain compliance-checked transfers (`base_transfer`) — switching to an
      unverified move would bypass the currency asset's rules.
- [ ] Ballot mutation lockout after start (BAL:810-817); distribution mutation lockout after
      payment_at (DIST:527-534).
- [ ] Re-vote tally math: subtract-then-add (BAL:513-527) must stay atomic per vote.

## 5. Test map

`pallets/runtime/tests/src/corporate_actions_test.rs` (2330 lines: CAA gating :238, CA init
matrix :488-722, record-date changes :878, schedule refs :1019; ballots :1074-1733 incl. RCV and
scheduled-checkpoint voting; distributions :1744-2328 incl. rounding and no-remaining cases).
Integration: `integration/tests/corporate_actions.rs`, `corporate_ballot.rs`,
`capital_distribution.rs`, `ca_extended.rs`.
