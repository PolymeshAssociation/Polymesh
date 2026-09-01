# 22 — Treasury, Committees & Groups

Sources: `pallets/treasury/src/lib.rs`, `pallets/committee/src/lib.rs`,
`pallets/group/src/lib.rs`, runtime wiring in `pallets/runtime/*/src/runtime.rs`.
Related specs: [19-pips](19-pips.md) (GC decisions), [03-claims](03-claims.md) (systematic CDD
claims for members), [20-staking-validators](20-staking-validators.md) (slashes → treasury).

## 1. Committee pallet (instanced voting bodies)

Instances (runtime indices 9-14): **PolymeshCommittee** (`Instance1` — the Governance
Committee/GC), **TechnicalCommittee** (`Instance3`), **UpgradeCommittee** (`Instance4`); each
paired with a `pallet_group` instance holding its membership.

### Model (pallets/committee/src/lib.rs)

- Members are **IdentityIds** (`Members` storage :183, mirrored from the group pallet via
  `MembershipChanged`). Proposals are runtime calls stored by hash (`ProposalOf` :167, `Voting`
  :173, `PolymeshVotes { index, ayes, nays, expiry }` :142-151).
- Threshold: `VoteThreshold` (n, d) — pass when `votes × d ≥ n × seats` (:509-512); GC default
  is 2/3 (chain spec). Both approval and rejection use the same threshold with a
  plurality requirement (`main ≥ other`, :541-548).
- `ReleaseCoordinator` (:191): a member with PIP-rescheduling power (doc 19 §3).
- Proposal expiry: `ExpiresAfter` (:196); expired proposals are pruned on touch (:611-625).
- **Execution origin**: passing proposals dispatch with `RawOrigin::Endorsed` (:627-631,
  origin enum :130-134) — this is what `VMO<Instance>` ("voting majority origin") matches.
  Other pallets gate on it, e.g. `GCVotingMajorityOrigin = VMO<GovernanceCommittee>`
  (`pallets/runtime/develop/src/runtime.rs:250`).

### Extrinsics

| Extrinsic (idx) | Who | Ref |
|---|---|---|
| `set_vote_threshold(0)` | `VoteThresholdOrigin` (= the committee's own VMO) | :338 |
| `set_release_coordinator(1)` | `CommitteeOrigin` (VMO); target must be a member | :356 |
| `set_expires_after(2)` | VMO | :370 |
| `vote_or_propose(3)` | committee member | propose (auto-aye) or vote by call hash; first vote must approve (`FirstVoteReject`) | :402-415 |
| `vote(4)` | committee member | aye/nay (switch allowed, duplicate rejected); executes/rejects when threshold met (:466-471, `execute_if_passed` :536-560) | :429 |

Single-member committees execute proposals immediately (:646-648, `seats() < 2`). **By design**:
liveness beats the single-member takeover risk — governance must keep working even if membership
collapses (e.g. mass abdication around a chain upgrade), and committee seats are only reachable
through root/GC-controlled membership in the first place. Do not report the fast path itself;
review proposals that would brick enactment instead. Members who leave mid-vote have their votes
retracted by the group hooks (`remove_vote_from` :519-534).

## 2. Group pallet (membership registries)

Instances: `Instance1` GC membership, `Instance2` **DidRegistrars** (formerly CDD providers),
`Instance3`/`Instance4` technical/upgrade membership.

- Storage: `ActiveMembers` (sorted Vec, :178), `InactiveMembers` (with optional expiry — a
  disabled member's past actions stay valid until `expiry`, :185), `ActiveMembersLimit` (:192,
  ≤ `COMMITTEE_MEMBERS_MAX`).
- Extrinsics (:235-405): `set_active_members_limit(0)` [`LimitOrigin`],
  `disable_member(1)`/`remove_member(3)` [`RemoveOrigin`] — disable keeps prior claims valid,
  remove invalidates them (doc comments :250-259); `add_member(2)` [`AddOrigin`];
  `swap_member(4)` [`SwapOrigin`]; `reset_members(5)` [`ResetOrigin`];
  `abdicate_membership(6)` [the member itself].
- Membership changes propagate via `MembershipInitialized`/`MembershipChanged`:
  committees resync `Members` (and reset votes / release coordinator if needed); the
  DidRegistrars instance notifies **Identity**, which maintains systematic CDD claims
  (doc 03 §4).

### Origin configuration (develop runtime, `pallets/runtime/develop/src/runtime.rs`)

| Instance | Add/Remove/Swap | Reset | Limit |
|---|---|---|---|
| GC membership (Instance1) | **root** (:270-274) | root | root |
| Technical/Upgrade membership | own committee VMO (:295-297) | `VMO<GovernanceCommittee>` (:299) | root |
| DidRegistrars (Instance2) | **root** (:324-329) | root | root |

So: GC composition and DID-registrar membership change only via PIPs (root); sub-committees
manage their own membership but the GC can reset them.

## 3. Treasury pallet

- Account: `PalletId "pm/trsry"` (primitives/src/constants.rs:89; `account_id()`
  pallets/treasury/src/lib.rs:196), associated with the Treasury systematic DID.
- Funding: staking slashes (`Slash = Treasury`, doc 20 §4) and voluntary `reimbursement(1)`
  (any permissioned identity, :130 → :170-189).
- Spending: `disbursement(0)` — **root only** (i.e. via PIP/GC), pays each beneficiary
  identity's **primary key** (:118 → :140-167; unknown identity ⇒ `InvalidIdentity`;
  aggregate balance check).

## 4. Invariants & review checklist

- [ ] `Endorsed` origin must only be constructible by threshold-satisfied proposal execution —
      it is the root-equivalent for many pallets via `VMO`.
- [ ] Threshold math `votes × d ≥ n × seats` with plurality (`ayes ≥ nays` / vice versa) —
      changing it changes every VMO-gated pallet.
- [ ] Group→committee membership sync must retract votes of removed members and clear the
      release coordinator when they leave.
- [ ] `disable_member` vs `remove_member` semantics for DidRegistrars: disable preserves
      historical CDD claim validity; remove revokes systematic claims (identity `ChangeMembers`
      hook).
- [ ] Single-member auto-execute (`seats() < 2`) is intentional liveness — keep it unless any
      replacement still guarantees proposals can enact when a committee shrinks to one seat.
- [ ] Treasury disbursement targets primary keys — identities without a primary key
      (post-unlink) must fail cleanly (`InvalidIdentity`).

## 5. Test map

`pallets/runtime/tests/src/committee_test.rs` (thresholds, expiry, release coordinator,
membership sync), `group_test.rs` (origins, disable/remove semantics), `treasury_test.rs`
(disbursement/reimbursement).
