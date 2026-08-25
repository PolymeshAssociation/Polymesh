# 19 — PIPs (On-Chain Governance)

Sources: `pallets/pips/src/{lib.rs,types.rs}`.
Related specs: [22-treasury-committees](22-treasury-committees.md) (committee origins, release
coordinator), [14-fees-and-extensions](14-fees-and-extensions.md) (`PipsPropose` fee).

## 1. Purpose

Polymesh Improvement Proposals: on-chain proposals (arbitrary root-dispatched calls) that the
community signals on with POLYX-bonded votes, and the **Governance Committee (GC)** decides on.
Community sentiment is advisory; the GC (via committee voting-majority origin) approves/rejects.
Approved PIPs execute as **root** through the scheduler.

## 2. Data model (pallets/pips/src/types.rs)

- `Proposer<AccountId>`: `Community(AccountId) | Committee(Technical | Upgrade)` (:125, :115).
- `ProposalState`: `Pending | Rejected | Scheduled | Failed | Executed | Expired` (:187-202).
  Active = Pending or Scheduled (lib.rs `is_active`).
- `Vote(bool, Balance)` — aye/nay with bonded "conviction" deposit (:177);
  `VotingResult { ayes_count, ayes_stake, nays_count, nays_stake }` (:163).
- `SnapshottedPip { id, weight: (bool, Balance) }` — net stake sign/magnitude (:238);
  ordering `compare_spip` ranks by (sign, magnitude).
- `SnapshotResult`: `Approve | Reject | Skip` (:269).
- `DepositInfo { owner, amount }` (:207).

### Storage (pallets/pips/src/lib.rs)

Config values (all root-set, i.e. themselves PIP-changeable): `PruneHistoricalPips` (:400),
`MinimumProposalDeposit` (:404), `DefaultEnactmentPeriod` (:408), `PendingPipExpiry` (:413),
`MaxPipSkipCount` (:418), `ActivePipLimit` (:422).
State: `Proposals` (:457), `ProposalStates` (:512), `ProposalMetadata` (:439), `ProposalResult`
(:462), `ProposalVotes` (:467), `Deposits` (:444), `LiveQueue` (live priority queue, :483),
`SnapshotQueue`/`SnapshotMeta` (:492/:496), `PipSkipCount` (:502), `CommitteePips` (:508),
`PipToSchedule` (:472), `PendingRefunds`/`VotesToBePruned` (lazy cleanup queues, :517/:521),
`ActivePipCount` (:434).
Genesis (src/chain_spec/common.rs:185-197): min deposit 2,000 POLYX, max skip 2, prune=false.

## 3. Extrinsics

| Extrinsic (idx) | Who | Behavior | Ref |
|---|---|---|---|
| `set_prune_historical_pips(0)`, `set_min_proposal_deposit(1)`, `set_default_enactment_period(2)`, `set_pending_pip_expiry(3)`, `set_max_pip_skip_count(4)`, `set_active_pip_limit(5)` | **root** | governance parameters | :571-694 |
| `propose(6)` | community (signed, permissioned) **or** Technical/Upgrade committee origin | §4 | :711-826 |
| `vote(7)` | any permissioned signer | bonded, **non-additive** vote (replaces previous; deposit re-locked to the new amount, :888-895); proposer voting again must keep ≥ min deposit (:873-880); only community PIPs in Pending | :851-914 |
| `approve_committee_proposal(8)` | GC voting majority | schedule a **committee** PIP (community ones go via snapshot) | :932-951 |
| `reject_proposal(9)` | GC voting majority | reject any active PIP; unschedule/unsnapshot; refunds queued | :971-984 |
| `prune_proposal(10)` | GC voting majority | GC storage cleanup of non-active PIPs | :1001-1012 |
| `reschedule_execution(11)` | **release coordinator** of the GC | move a Scheduled PIP's execution block (min next block) | :1027-1059 |
| `clear_snapshot(12)` | any **GC member** | drop current snapshot | :1073-1093 |
| `snapshot(13)` | any **GC member** | clone `LiveQueue` into `SnapshotQueue` + meta | :1109-1142 |
| `enact_snapshot_results(14)` | GC voting majority | apply Approve/Reject/Skip to the snapshot, **lowest priority first zipped in reverse** (§5) | :1170-1253 |
| `execute_scheduled_pip(15)` / `expire_scheduled_pip(16)` | **root** (scheduler-dispatched) | execute / expire | :1268-1309 |

## 4. Community proposal lifecycle

1. **Propose** (:713-826): community proposer bonds `deposit ≥ MinimumProposalDeposit` (locked
   under `PIPS_LOCK_ID`, :1347-1360); `ActivePipCount < ActivePipLimit` enforced (committee PIPs
   exempt and deposit-free, :752-757); protocol fee `PipsPropose` charged for both. The proposal
   auto-casts an aye vote at the deposit (:806) and enters `LiveQueue` (:809). If
   `PendingPipExpiry` is set, an expiry task is scheduled (:788-791, root-origin
   `expire_scheduled_pip`).
2. **Voting** (:853-914): each voter bonds a deposit; vote weight = deposit; changing a vote
   adjusts the lock up/down. `LiveQueue` re-sorts on every vote (net stake ordering,
   `aggregate_result` :1453-1466).
3. **Snapshot** (:1113-1142): a GC member freezes the queue.
4. **Enact** (:1172-1253): GC majority submits per-PIP results matched against the snapshot
   queue **from the end** (highest priority first in `results`); mismatch ⇒ `SnapshotIdMismatch`.
   `Skip` bumps `PipSkipCount` and fails once it would exceed `MaxPipSkipCount`
   (`CannotSkipPip`, :1206-1210); skipped PIPs stay pending. Reject ⇒ refund queue; Approve ⇒
   scheduled.
5. **Scheduling** (:1502-1533): execution at `now + max(DefaultEnactmentPeriod, 1)` via the
   named scheduler task; release coordinator can reschedule (:1029-1059).
6. **Execution** (:1668-1687): dispatched as **root**; result ⇒ `Executed` or `Failed`;
   `maybe_prune` applies `PruneHistoricalPips`.
7. **Expiry**: Pending PIPs past `PendingPipExpiry` are expired by the scheduled task
   (:1296-1309) → `Expired` state.

**Refunds**: deposits are *not* slashed in any path; every terminal transition queues the PIP in
`PendingRefunds`, drained lazily in `on_idle` (`remove_pending_storage` :1716-1762) which
releases locks (bounded per block by `MaxRefundsAndVotesPruned`); vote records pruned similarly.

Committee (Technical/Upgrade) PIPs: no deposit, no community voting, no snapshot — GC approves
directly via `approve_committee_proposal`.

## 5. Invariants & review checklist

- [ ] Deposits: sum of `Deposits` per account ≤ their `PIPS_LOCK_ID` lock; every terminal state
      must enqueue refunds (no slash paths exist — introducing one is a design change).
- [ ] `LiveQueue` must stay sorted and in sync with `ProposalResult` (insert :1432, adjust
      :1482); snapshot enactment relies on exact id matching in reverse order.
- [ ] Skip accounting: `PipSkipCount` monotonic, capped by `MaxPipSkipCount`.
- [ ] `ActivePipCount` increment/decrement symmetry (`decrement_count_if_active`) — a drift
      bricks community proposing via `TooManyActivePips`.
- [ ] Scheduled execution/expiry tasks must be cancelled when PIPs are rejected/rescheduled
      (`maybe_unschedule_pip` :1577, `unschedule_pip` :1610) or the scheduler root-dispatches a
      stale task.
- [ ] PIP execution is root dispatch of arbitrary calls — the GC approval origin
      (`VotingMajorityOrigin`) is the entire security boundary.

## 6. Test map

`pallets/runtime/tests/src/pips_test.rs` (proposal lifecycle, snapshots, enactment, skips,
expiry, refunds). Committee interplay: `committee_test.rs`.
