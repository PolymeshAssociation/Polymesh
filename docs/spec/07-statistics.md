# 07 — Transfer Restrictions (Statistics)

Sources: `pallets/statistics/src/lib.rs`, `primitives/src/statistics.rs`,
`primitives/src/transfer_compliance.rs`.
Related specs: [06-compliance](06-compliance.md) (claim-based rules; statistics are
count/percentage-based), [09-asset-transfers](09-asset-transfers.md), [03-claims](03-claims.md).

## 1. Purpose

Numeric transfer restrictions per asset: investor-count caps, ownership-percentage caps, and
claim-scoped variants (e.g. "max 50 non-accredited investors", "jurisdiction X holds ≤ 20%").
Built on **stat counters** maintained on every balance change, and **transfer conditions**
evaluated on every cross-identity fungible transfer. **NFTs are not statistics-checked.**

## 2. Data model

| Type | Shape | Ref |
|---|---|---|
| `StatType` | `{ operation_type: Count \| Balance, claim_issuer: Option<(ClaimType, IdentityId)> }` | primitives/src/statistics.rs:42-47 |
| `Stat1stKey` | `{ asset_id, stat_type }` | statistics.rs:71-76 |
| `Stat2ndKey` | `NoClaimStat \| Claim(StatClaim)` | statistics.rs:97-102 |
| `StatClaim` | `Accredited(bool) \| Affiliate(bool) \| Jurisdiction(Option<CountryCode>)` | statistics.rs:163-170 |
| `TransferCondition` | `MaxInvestorCount(u64) \| MaxInvestorOwnership(Permill) \| ClaimCount(StatClaim, issuer, min, Option<max>) \| ClaimOwnership(StatClaim, issuer, min, max)` | primitives/src/transfer_compliance.rs:30-48 |
| `AssetTransferCompliance` | `{ paused: bool, requirements: BoundedBTreeSet<TransferCondition> }` | transfer_compliance.rs:132-137 |

Storage (pallets/statistics/src/lib.rs): `ActiveAssetStats` (AssetId → bounded set of
`StatType`, :137), `AssetStats` ((asset, stat_type), key2 → u128, :147),
`AssetTransferCompliances` (:159), `TransferConditionExemptEntities`
((asset, op, claim_type), DID → bool, :169).

Limits (all runtimes): `MaxStatsPerAsset` = 10, `MaxTransferConditionsPerAsset` = 4
(`pallets/runtime/develop/src/runtime.rs:139-140`; +50 under benchmarks).

## 3. Configuration extrinsics (agent-gated via `ExternalAgents::ensure_perms`, :305-310)

| Extrinsic (call_index) | Behavior | Ref |
|---|---|---|
| `set_active_asset_stats(0)` | replace active stat set; cannot remove a type used by a transfer condition (`CannotRemoveStatTypeInUse` :328-349); removal wipes that stat's `AssetStats` (:352-362) | :218 → :316 |
| `batch_update_asset_stats(1)` | manual counter (re)initialization; stat must be active (`StatTypeMissing` :386); `None` value removes (:395-409) | :243 → :377 |
| `set_asset_transfer_compliance(2)` | replace conditions; each condition's stat type **must already be active** (`StatTypeMissing` :432-439); empty set removes entry (:444) | :269 → :415 |
| `set_entities_exempt(3)` | add/remove exempt DIDs per (asset, op, claim_type) key | :293 → :457 |

**Operational gotcha**: activating a stat does *not* backfill counters — the chain only tracks
changes from activation onward. Agents must `batch_update_asset_stats` to seed correct values,
else conditions evaluate against wrong counts. No protocol fees in this pallet.

## 4. Stat maintenance (on every balance change)

`update_asset_stats(asset, from_did?, to_did?, from_balance?, to_balance?, amount)` (:629-689) —
called from asset pallet on transfer (asset lib.rs:4224-4232), issue (from=None,
lib.rs:4145-4153), redeem (to=None, lib.rs:2264-2272). Controller transfers update stats too.

- Investor-count transitions (`investor_count_changes` :609-626): sender counted out iff
  post-balance == 0; receiver counted in iff post-balance == amount (was 0).
- `Count` stats: ±1 on the respective `Stat2ndKey` bucket (:535-581).
- `Balance` stats: ±amount per bucket (:488-528).
- Claim-scoped buckets resolved via `fetch_claim_as_key` (:584-602):
  `Identity::fetch_claim(did, claim_type, issuer, Some(Scope::Asset(asset_id)))` — **scope is
  always `Scope::Asset(asset)`** for stats, unlike compliance's exact-scope matching.
  **Claim changes do not retro-update stat buckets** — a claim issued/revoked after balances
  exist leaves counters stale until manually corrected via `batch_update_asset_stats`.

## 5. Enforcement (`verify_transfer_restrictions`, :984-1012)

Called from `Asset::validate_asset_transfer` (asset lib.rs:3414-3423); **also enforced in the
locked-settlement `simplified_fungible_transfer` path** (asset lib.rs:4405-4414) — unlike
compliance. Skipped for controller transfers (asset lib.rs:3401-3404).

- `paused ⇒ pass` (:997) — but note **no production extrinsic sets `paused`**
  (transfer_compliance.rs:134; only benchmark code writes it,
  pallets/statistics/src/benchmarking.rs:187). Effectively always active.
- **ALL conditions must pass** (AND, :1033-1047) — opposite of compliance's OR. Failure ⇒
  `InvalidTransferStatisticsFailure`.
- Evaluation uses **post-transfer projections** (sender−amount, receiver+amount, :1027-1031)
  against **pre-transfer stored counters**:

| Condition | Pass rule | Ref |
|---|---|---|
| `MaxInvestorCount(max)` | only checked when receiver is a *new* investor: stored count `< max` (i.e. count after entry ≤ max); sender-exit or investor-swap auto-pass | :692-734 |
| `MaxInvestorOwnership(max)` | `(receiver_balance + amount) / total_supply ≤ max` | :810-824 |
| `ClaimCount(claim, issuer, min, max?)` | sender exiting a matching bucket: fail if `count ≤ min`; receiver entering: fail if `count ≥ max` | :737-807 |
| `ClaimOwnership(claim, issuer, min, max)` | receiver-side: `(bucket + amount)/supply ≤ max`; sender-side: `(bucket − amount)/supply ≥ min`; both/neither match ⇒ pass | :827-890 |

- **Exemptions** (`is_exempt` :963-981): checked only after a condition fails; `Count`-type
  conditions exempt by **sender** DID, `Balance`-type by **receiver** DID. Exempt key =
  (asset, op, claim_type) — one exemption covers all conditions of that shape.
- Dry-run: `transfer_restrictions_report` (:1053-1095; runtime API
  `rpc/runtime-api/src/statistics.rs:25-35`, no JSON-RPC wrapper — use `state_call`).

## 6. Invariants & review checklist

- [ ] Any new balance-mutating path must call `update_asset_stats` with correct pre-change
      balances, or counters drift (and conditions misfire).
- [ ] `set_asset_transfer_compliance` ↔ `set_active_asset_stats` coupling: conditions require
      active stats; stats in use can't be deactivated. Keep both directions enforced.
- [ ] Stat counters are *not* claim-reactive: docs/UI must treat claim changes as requiring
      manual `batch_update_asset_stats`; on-chain code must not assume bucket accuracy.
- [ ] Exemption side (sender for Count / receiver for Balance) is intentional — e.g. an exempt
      treasury can send to new investors past the cap? No: MaxInvestorCount exemption is checked
      against **sender**, letting an exempt *sender* mint new investors past the cap. Confirm
      that any new condition type picks the correct side.
- [ ] AND semantics across conditions; a failing meter (`WeightLimitExceeded`) must fail closed.
- [ ] `paused` for statistics is currently unreachable in production — adding a setter changes
      security posture; do deliberately.

## 7. Test map

`pallets/runtime/tests/src/transfer_compliance_test.rs` (main suite: count/ownership/claim
conditions, exemptions), `asset_test.rs:139`, settlement tests for enforcement-in-instructions.
Integration: `integration/tests/statistics.rs`, `statistics_enforcement.rs`.
