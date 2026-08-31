# 16 — MultiSig

Sources: `pallets/multisig/src/lib.rs`, `primitives/src/multisig.rs`,
`pallets/runtime/common/src/fee_details.rs`.
Related specs: [01-identity-keys](01-identity-keys.md) (KeyRecords, authorizations),
[14-fees-and-extensions](14-fees-and-extensions.md) (fee redirection).

## 1. Purpose

An m-of-n multisig **account** whose signers approve proposals (arbitrary runtime calls)
executed with the multisig account as origin. The multisig account is itself an identity key
(secondary key of the creator's identity by default); its signers are dedicated keys that belong
to no identity. Signers never need POLYX — fees are redirected (§6).

## 2. Data model & storage (pallets/multisig/src/lib.rs)

| Item | Purpose | Ref |
|---|---|---|
| `MultiSigSigners` (ms, signer) → bool / `NumberOfSigners` | accepted signers | :721/:726 |
| `MultiSigSignsRequired` | threshold; doubles as existence marker | :730 |
| `Proposals` / `ProposalVoteCounts` / `ProposalStates` / `Votes` | proposal call, approvals/rejections, state, per-signer vote | :741/:776/:783/:748 |
| `NextProposalId` / `MultiSigNonce` | sequences | :735/:717 |
| `PayingDid` | identity whose primary key pays proposal fees | :762 |
| `AdminDid` | admin identity (via-admin extrinsics) | :769 |
| `AuthToProposalId` (ms, auth_id) → proposal_id | join-identity proposal mapping | :800 |
| `LastInvalidProposal` | proposals ≤ id are invalidated | :811 |
| `ExecutionReentry` | reentrancy guard | :796 |
| `TransactionVersion` | all pending proposals wiped on tx-version bump (`on_runtime_upgrade` :189-214) | :807 |

`ProposalState = Active { until: Option<Moment> } | ExecutionSuccessful | ExecutionFailed |
Rejected` (primitives/src/multisig.rs:29-42).

## 3. Creation & identity linkage

`create_multisig(signers, sigs_required, permissions)` (idx 0, :224-244 → :996-1029):
- Caller: permissioned key; **custom permissions require the primary key**
  (`ensure_valid_origin(origin, permissions.is_some())`, :232-233).
- Multisig address = `hash(b"MULTI_SIG", nonce, caller)` (:1283-1289) — deterministic, unlinked.
- Each signer gets an `AddMultiSigSigner` authorization (:929-943); threshold bounds checked
  (`ensure_sigs_in_bounds`: threshold ≥ 1, signers ≥ threshold, :900-904).
- `PayingDid` = creator DID (:1015); the multisig account **immediately joins the creator's
  identity as a secondary key** via `unsafe_join_identity` with the given permissions
  (default `Permissions::empty()`) (:1026, :241).
- Signer acceptance (`accept_multisig_signer`, idx 4, :317 → :1224-1272): consumes the auth;
  signer key must be completely unlinked (not identity- or multisig-linked, :1243-1247); no
  multisig-as-signer nesting (:1233); records `KeyRecord::MultiSigSignerKey(ms)` (:1259-1262);
  **invalidates all outstanding proposals** (:1251).
- The multisig can later become another identity's key (or primary key) only through the normal
  identity auth flows executed via proposals (`approve_join_identity`/`join_identity`, §5).

## 4. Signer & threshold management

| Extrinsic (idx) | Origin | Ref |
|---|---|---|
| `add_multisig_signers(5)` / `remove_multisig_signers(6)` | **the multisig account itself** (i.e. via an executed proposal) | :332/:343 |
| `add_multisig_signers_via_admin(7)` / `remove_multisig_signers_via_admin(8)` | **primary key of `AdminDid`** (:864-872) | :366/:385 |
| `change_sigs_required(9)` / `change_sigs_required_via_admin(10)` | multisig itself / admin | :406/:422 |
| `add_admin(11)` / `remove_admin(17)` / `remove_payer(13)` | multisig itself | :439/:552/:474 |
| `remove_admin_via_admin(12)` / `remove_payer_via_payer(14)` | admin primary key / payer primary key (:874-882) | :457/:489 |

Rules: removing signers can't violate `signers ≥ threshold` (:973-977); threshold changes are
bounds-checked (:1304) and **invalidate pending proposals** (:1306); max 50 signers
(`MaxMultiSigSigners = 50`, `pallets/runtime/develop/src/runtime.rs:113`; checks :853-862).

## 5. Proposal lifecycle

- `create_proposal(ms, call, expiry)` (idx 1, :246 → :1032-1057): **signers only** (:1038);
  expiry must be future; **proposer auto-approves** (:1056) — a 1-of-n multisig executes
  immediately.
- `approve(ms, id, max_weight)` (idx 2, :280 → :1060-1098): signer-only; proposal Active,
  unexpired, not invalidated (:906-927, `LastInvalidProposal` :1327-1335); `AlreadyVoted` guard
  (:1069-1072); executes at threshold (:1081).
- Execution (:1101-1168): call taken from storage; `max_weight` must cover the call
  (`MaxWeightTooLow` :1117-1122); dispatched as `Signed(multisig)` wrapped in
  **`with_call_metadata`** (:1125, permission checks see the inner call — doc 02 §3) with a
  reentrancy guard (`NestingNotAllowed` :1127-1135); state → `ExecutionSuccessful/Failed`;
  `ProposalExecuted { result }`.
- `reject(ms, id, max_weight)` (idx 3, :301 → :1171-1221): proposer may retract while sole
  approver (:1183-1190); rejection quorum `rejections > NumberOfSigners − threshold` ⇒
  `Rejected` and proposal removed (:1203-1216).
- **Join-identity flow**: `approve_join_identity(ms, auth_id)` (idx 15, :512-538) — first call
  creates an internal proposal `join_identity { auth_id }` and records `AuthToProposalId`;
  subsequent calls approve it. `join_identity(16)` (:541-549) is callable only by the multisig
  itself (proposal execution) and delegates to `Identity::join_identity`.
- Invalidation events: signer set changes (:1251, :986) and threshold changes (:1306) bump
  `LastInvalidProposal`; runtime tx-version bumps wipe everything (:189-214).

## 6. Fee payment (signers pay nothing)

`fee_details.rs` redirection (doc 14 §3): `create_proposal`/`approve`/`reject` → `PayingDid`'s
primary key, else the multisig account itself (:92-111, :160-183); `accept_multisig_signer` →
auth issuer's primary key (:133-140); `approve_join_identity` → JoinIdentity auth issuer
(:141-159). Duplicate votes are rejected at the transaction-pool level (`AlreadyVoted`
pre-checks, :143-147/:170-174) so griefing by re-voting doesn't drain the payer. Failed
dispatches decrement auth retry counts (doc 01 §5).

## 7. Invariants & review checklist

- [ ] `threshold ≥ 1 ∧ accepted_signers ≥ threshold` at every mutation point (create/remove/
      change; :900-904).
- [ ] Signer keys must be exclusively multisig-linked (`MultiSigSignerKey`); they can never be
      identity keys simultaneously (identity pipeline rejects them, doc 02 §4).
- [ ] Any change to the signer set or threshold must invalidate outstanding proposals — an old
      proposal must not execute under a new quorum regime.
- [ ] Proposal execution must keep `with_call_metadata` + reentrancy guard; removing either
      enables permission bypass or recursive execution.
- [ ] Multisig-origin admin calls (`add_multisig_signers` etc.) must only be reachable via
      executed proposals (origin = ms account).
- [ ] Fee-redirection pre-checks in fee_details must stay consistent with dispatch-time vote
      logic (a mismatch enables free spam or blocks legit votes).

## 8. Test map

`pallets/runtime/tests/src/multisig.rs` (30 tests: creation/threshold bounds :87, join :108,
signer add/remove :261/:361, primary-key rotation :452-527, admin flows :542-984, approval
closure :712, rejections :791), `transaction_payment_test.rs:775` (AlreadyVoted at pool),
`fee_details.rs` (payer matrix).
