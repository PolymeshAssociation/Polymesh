# 03 — Identity Claims

Sources: `pallets/identity/src/claims.rs`, `pallets/identity/src/lib.rs`,
`primitives/src/identity_claim.rs`, `primitives/src/cdd_id.rs`, `primitives/src/constants.rs`.
Related specs: [01-identity-keys](01-identity-keys.md), [06-compliance](06-compliance.md) (main
consumer), [07-statistics](07-statistics.md) (claim-scoped stats).

## 1. Purpose

Claims are attestations attached to a target DID by an issuer DID ("issuer says X about target").
They are the raw material for asset **compliance rules** (doc 06) and claim-scoped **transfer
statistics** (doc 07). Any identity can issue claims; *which* issuers matter is decided by each
asset's trusted-issuer configuration (doc 06), not by the identity pallet — with the exception of
CDD claims, which only DID registrars may issue.

## 2. Data model

### Claim variants (primitives/src/identity_claim.rs:79)

| Claim | Payload | ClaimType |
|---|---|---|
| `Accredited(Scope)` | scope | `Accredited` |
| `Affiliate(Scope)` | scope | `Affiliate` |
| `BuyLockup(Scope)` / `SellLockup(Scope)` | scope; lockup end = claim expiry | `BuyLockup`/`SellLockup` |
| `CustomerDueDiligence(CddId)` | `CddId` = opaque 32 bytes (primitives/src/cdd_id.rs:10); no scope | `CustomerDueDiligence` |
| `KnowYourCustomer(Scope)` | scope | `KnowYourCustomer` |
| `Jurisdiction(CountryCode, Scope)` | ISO country (primitives/src/jurisdiction.rs) + scope | `Jurisdiction` |
| `Exempted(Scope)` / `Blocked(Scope)` | scope | `Exempted`/`Blocked` |
| `Custom(CustomClaimTypeId, Option<Scope>)` | registered custom type id | `Custom(id)` |

`Scope` (identity_claim.rs:37): `Identity(IdentityId) | Asset(AssetId) | Custom(Vec<u8>)`.
Custom scopes are capped at 32 bytes (`ensure_custom_scopes_limited`,
pallets/identity/src/claims.rs:36).

`IdentityClaim` (identity_claim.rs:171) stores `{ claim_issuer, issuance_date,
last_update_date, expiry: Option<Moment>, claim }`.

### Storage (pallets/identity/src/lib.rs)

| Item | Key → Value | Ref |
|---|---|---|
| `Claims` | `Claim1stKey { target, claim_type }` → `Claim2ndKey { issuer, scope }` → `IdentityClaim` | lib.rs:389; key types pallets/identity/src/types.rs:80/86 |
| `CustomClaims` / `CustomClaimsInverse` | id ↔ name for custom claim types | lib.rs:402/408 |
| `CustomClaimIdSequence` | next `CustomClaimTypeId` | lib.rs:413 |

**Uniqueness**: one claim per `(target, claim_type, issuer, scope)`. Re-adding **upserts**:
`issuance_date` is preserved from the existing claim, `last_update_date`/`expiry` refresh
(claims.rs:116-140). There is at most one `Jurisdiction` claim per issuer+scope — adding a new
country replaces the previous one (same claim_type key).

## 3. Claim lifecycle

| Extrinsic (call_index) | Who may call | Behavior | Ref |
|---|---|---|---|
| `add_claim(6)` | any permissioned key of issuer DID; target DID must exist | upsert claim; CDD variant → registrar check; protocol fee `IdentityAddClaim` for non-CDD | lib.rs:643 → claims.rs:98/158 |
| `revoke_claim(7)` | issuer (permissioned key) | delete claim by `(target, claim_type, issuer, scope from claim)` | lib.rs:666 → claims.rs:170 |
| `revoke_claim_by_index(14)` | issuer (permissioned key) | same, scope passed explicitly (needed when scope unknown from claim value) | lib.rs:745 |
| `gc_add_cdd_claim(12)` / `gc_revoke_cdd_claim(13)` | `GCVotingMajorityOrigin` (committee) | add/remove a systematic CDD claim issued by `SystematicIssuers::Committee` | lib.rs:725/734 |
| `register_custom_claim_type(19)` | any permissioned identity | registers name→id (unique, length-limited) | lib.rs:838 → claims.rs:259 |

Notes:
- Claim issuance/revocation authorization is **only** "caller has extrinsic permission on
  `Identity::add_claim` for the issuer DID". Nothing restricts *which* claim types an identity may
  issue (except CDD). Consumers filter by trusted issuers.
- Expiry: claims are not deleted on expiry; readers filter — `fetch_claim` (claims.rs:46) returns
  only claims with `expiry > now`. `BuyLockup`/`SellLockup` invert this meaning (lockup active
  until expiry) — interpretation is up to the consumer (compliance conditions).
- Revocation of a *CDD* claim is `Operational` dispatch class (`revoke_claim_class`,
  lib.rs:1053).
- `add_claim` fails with `DidMustAlreadyExist` if the target doesn't exist
  (`ensure_signed_and_validate_claim_target`, claims.rs:185).

## 4. CDD claims after 8.0.0

Historically CDD (Customer Due Diligence) claims gated all transactions. Since 8.0.0:

- **No transaction path checks CDD**. Onboarding = DID existence (`is_did_active`,
  claims.rs:64). `cdd_register_did*` are deprecated (lib.rs:591-594, 851-854);
  `register_did`/`self_register_did` create no claim.
- CDD claims still exist as data: only DID-registrar identities may issue them
  (`base_add_cdd_claim` → `ensure_authorized_did_registrar`, claims.rs:158-167,198 — membership
  in `pallet_group` Instance2, "DidRegistrars", runtime index 8), and the GC can force-add/revoke
  them (lib.rs:725/734). Compliance rules may still *reference* them like any claim.
- **Systematic CDD claims**: group membership changes automatically maintain CDD claims for
  committee members / DID registrars via `ChangeMembers`/`InitializeMembers` hooks
  (lib.rs:1029-1049 → claims.rs:240/248), issued by `SystematicIssuers::CDDProvider` or
  `::Committee`.
- `CddId` is now effectively opaque; systematic/genesis claims use `CddId::default()`
  (all zeros, claims.rs:242).

### Systematic identities (primitives/src/constants.rs:109)

Chain-maintained identities with no known private key: `Committee` (= `GC_DID`,
constants.rs:177), `CDDProvider`, `Treasury`, `BlockRewardReserve`, `Settlement`,
`ClassicMigration`, `FiatTickersReservation`. Registered at genesis
(pallets/identity/src/lib.rs:518-521, keys.rs:651). GC-issued claims/authorizations use `GC_DID`
as issuer.

## 5. Consumers of claims

| Consumer | How | Ref |
|---|---|---|
| Compliance manager | `fetch_claims` per condition against per-asset trusted issuers; proposition evaluation over `Context { claims }` | pallets/compliance-manager/src/lib.rs:621-693; primitives/src/proposition/mod.rs:24 (doc 06) |
| Statistics | claim-scoped stat buckets & transfer conditions (`fetch_claim_as_key`) | pallets/statistics/src/lib.rs:584-606 (doc 07) |
| Corporate actions | CA target lists don't use claims, but distributions respect asset compliance (doc 12) | — |

## 6. Invariants & review checklist

- [ ] Non-registrar identities must never be able to create `CustomerDueDiligence` claims
      (`add_claim` special-case, lib.rs:651-654) — check any new claim-writing path.
- [ ] Claim reads for enforcement must filter expiry (`fetch_claim`, claims.rs:56) — direct
      `Claims::get` without expiry filtering is a bug for enforcement purposes.
- [ ] `Custom` claims must verify the type id exists (`base_add_claim`, claims.rs:105-110).
- [ ] Upsert semantics: verify consumers don't assume `issuance_date` = last write.
- [ ] Custom scope length ≤ 32 enforced on add (claims.rs:36-41).
- [ ] Claims are unbounded storage (`#[pallet::unbounded]`, lib.rs:388) — new claim payloads
      must stay length-limited.

## 7. Test map

- `pallets/runtime/tests/src/identity_test.rs`: claim add/revoke/expiry, custom claim types,
  GC CDD claims, `revoke_claim_by_index`.
- Compliance-side consumption: `compliance_manager_test.rs`, `transfer_compliance_test.rs`.
