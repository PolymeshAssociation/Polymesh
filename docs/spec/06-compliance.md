# 06 — Asset Compliance

Sources: `pallets/compliance-manager/src/lib.rs`, `primitives/src/compliance_manager.rs`,
`primitives/src/condition.rs`, `primitives/src/proposition/{mod.rs,base.rs}`.
Related specs: [03-claims](03-claims.md) (the evaluated data), [05-external-agents](05-external-agents.md)
(who configures), [09-asset-transfers](09-asset-transfers.md) (when evaluated).

## 1. Purpose

Per-asset transfer rules over **claims**: a transfer passes if *any* configured requirement is
satisfied, where a requirement = conditions on the **sender** identity AND conditions on the
**receiver** identity. Configured by asset agents; evaluated on every cross-identity transfer of
the asset (fungible and NFT).

## 2. Data model

| Type | Shape | Ref |
|---|---|---|
| `AssetCompliance` | `{ paused: bool, requirements: Vec<ComplianceRequirement> }` | primitives/src/compliance_manager.rs:116-121 |
| `ComplianceRequirement` | `{ sender_conditions, receiver_conditions, id: u32 }` | compliance_manager.rs:28-35 |
| `Condition` | `{ condition_type, issuers: Vec<TrustedIssuer> }` | primitives/src/condition.rs:131-136 |
| `ConditionType` | `IsPresent(Claim) \| IsAbsent(Claim) \| IsAnyOf(Vec<Claim>) \| IsNoneOf(Vec<Claim>) \| IsIdentity(TargetIdentity)` | condition.rs:41-52 |
| `TargetIdentity` | `ExternalAgent \| Specific(IdentityId)` | condition.rs:30-35 |
| `TrustedIssuer` | `{ issuer, trusted_for: Any \| Specific(Vec<ClaimType>) }` | condition.rs:79-85, 69-74 |

Storage: `AssetCompliances` (AssetId → `AssetCompliance`, lib.rs:217),
`TrustedClaimIssuer` (AssetId → default `Vec<TrustedIssuer>`, lib.rs:223).

## 3. Configuration extrinsics (all agent-gated via `ExternalAgents::ensure_perms`)

| Extrinsic (call_index) | Behavior | Ref |
|---|---|---|
| `add_compliance_requirement(0)` | append with id = latest+1 (:551); dedup conditions; complexity check; **protocol fee `ComplianceManagerAddComplianceRequirement`** (:567) | :283 → :540 |
| `remove_compliance_requirement(1)` | remove by id; `InvalidComplianceRequirementId` (:321) | :309 |
| `replace_asset_compliance(2)` | replace all; sorted/dedup by id, `DuplicateComplianceRequirements` (:363-370); complexity | :350 |
| `reset_asset_compliance(3)` | remove entry entirely (**also clears `paused`**) (:404) | :402 |
| `pause_asset_compliance(4)` / `resume_asset_compliance(5)` | toggle `paused` (:746-754) | :419/:435 |
| `add_default_trusted_claim_issuer(6)` | append; issuer DID must exist (:583); dup ⇒ `IncorrectOperationOnTrustedIssuer` (:597); complexity re-check (:603) | :452 → :578 |
| `remove_default_trusted_claim_issuer(7)` | remove; absent ⇒ same error (:481) | :472 |
| `change_compliance_requirement(8)` | replace one by id (:518) | :504 |

**Complexity cap** (the only size limit): Σ over all conditions of
`claims_count × max(issuers, default_issuer_count)` ≤ `MaxConditionComplexity` = **50** in all
runtimes (`base_verify_compliance_complexity` lib.rs:780-798; e.g.
`pallets/runtime/develop/src/runtime.rs:132`); exceeding ⇒ `ComplianceRequirementTooComplex`.

## 4. Evaluation (`ComplianceFnConfig` impl, lib.rs:869-940)

`is_compliant(asset, sender_did, receiver_did)` (lib.rs:870-890):

1. **Paused ⇒ pass. Zero requirements ⇒ pass** (lib.rs:878-881). A new asset with no compliance
   configured is freely transferable (subject to statistics, doc 07).
2. Else `is_any_requirement_compliant` (lib.rs:841-866): **OR over requirements**; each
   requirement passes iff **ALL** `sender_conditions` hold for the sender **AND ALL**
   `receiver_conditions` hold for the receiver (lib.rs:850-860, AND helper :717-730).

Condition evaluation (`is_condition_satisfied` lib.rs:733-743 → `proposition::run`
primitives/src/proposition/mod.rs:107-122):

| ConditionType | Semantics |
|---|---|
| `IsPresent(c)` | a trusted issuer has issued a matching, unexpired claim `c` |
| `IsAbsent(c)` | negation of IsPresent |
| `IsAnyOf(cs)` / `IsNoneOf(cs)` | membership / non-membership over the fetched claim set |
| `IsIdentity(Specific(did))` | evaluated identity == did (primitives/src/proposition/base.rs:17-22) |
| `IsIdentity(ExternalAgent)` | evaluated identity is **any agent of the asset** (`GroupOfAgent` lookup, lib.rs:741) |

- **Trusted issuers**: per-condition `issuers` if non-empty, else the asset's default
  `TrustedClaimIssuer` list (`issuers_for` lib.rs:641-651). An issuer counts only if
  `trusted_for` covers the claim type (condition.rs:100-105).
- **Claim fetching** (`fetch_claims` lib.rs:621-636): looks up
  `Identity::fetch_claim(target, claim_type, issuer, scope)` with **exactly the scope embedded
  in the condition's claim** — scope matching is exact, no widening; expiry filtered (doc 03 §3).
- CDD special case: a condition claim `CustomerDueDiligence(default CddId)` matches any CDD
  claim (primitives/src/proposition/base.rs:44-49).
- One-sided variant `is_holder_compliant` (lib.rs:892-929) used for holder-freeze reporting
  (asset lib.rs:4011-4016): passes if any requirement's relevant side holds.

Evaluation is weight-metered (`WeightMeter`, `WeightLimitExceeded` lib.rs:261) — failures of the
meter fail the transfer, not the block.

### Call sites

- Fungible: `Asset::validate_asset_transfer` → `is_compliant` (pallets/asset/src/lib.rs:3426);
  skipped for controller transfers (lib.rs:3401-3404) and same-identity moves (never reaches —
  doc 09 §2); **not re-checked** in `simplified_fungible_transfer` for locked settlements
  (asset lib.rs:4381; doc 10 §6).
- NFT: `validate_nft_transfer` → `is_compliant` (pallets/nft/src/lib.rs:688-695).
- **Not checked** on issue/redeem (asset lib.rs:4113/2226).
- Dry-run RPC: `compliance_report` (lib.rs:948-1000; runtime API
  `rpc/runtime-api/src/compliance.rs:24-48`, JSON-RPC `compliance_complianceReport`
  `rpc/src/compliance.rs:31-41`). No requirements ⇒ `any_requirement_satisfied = true` (:956-962).

## 5. Invariants & review checklist

- [ ] Empty/paused compliance **allows** transfers — deliberate default-open design; adding a
      first requirement flips the asset to default-closed (only matching transfers pass).
      Confirm intent when changing this asymmetry.
- [ ] OR-of-requirements / AND-of-conditions structure must be preserved; short-circuits at
      lib.rs:861 and :726.
- [ ] Claim lookups must remain expiry-filtered and exact-scope (`fetch_claim`); any caching
      must not outlive claim revocation.
- [ ] `IsIdentity(ExternalAgent)` depends on `GroupOfAgent` — agent removal instantly changes
      compliance outcomes.
- [ ] Complexity checks must run on every mutation path (add/replace/change + trusted-issuer
      adds) — they bound transfer-time weight.
- [ ] Requirement ids must stay unique (auto-increment :551; replace dedups :363).

## 6. Test map

`pallets/runtime/tests/src/compliance_manager_test.rs` (evaluation matrix, reports),
`transfer_compliance_test.rs` (interaction with statistics), asset/settlement transfer tests
exercising `validate_asset_transfer`. Proposition unit tests: primitives/src/proposition/base.rs:166-304.
Integration: `integration/tests/compliance.rs`, `compliance_enforcement.rs`.
