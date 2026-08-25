# 20 — Staking & Permissioned Validators

Sources: `pallets/validators/src/{lib.rs,permissioned.rs,inflation.rs,types.rs}` (local pallet),
forked `pallet-staking` (FORK = `substrate/frame/staking` in the pinned
`PolymeshAssociation/polkadot-sdk` checkout), runtime wiring in
`pallets/runtime/common/src/runtime.rs` (RTC).
Related specs: [22-treasury-committees](22-treasury-committees.md) (governance origins),
[01-identity-keys](01-identity-keys.md) (validator identities).

## 1. Purpose

Polymesh runs NPoS staking (forked `pallet-staking`) with a Polymesh twist: **validators must be
pre-approved by governance** ("permissioned identities"), each approved identity has a bounded
number of validator slots, commissions are capped chain-wide, slashing is governance-switchable,
era payouts are automatic, and inflation is capped by a fixed yearly reward once total issuance
reaches a threshold.

## 2. Validators pallet (registry + policy)

Storage (pallets/validators/src/lib.rs): `PermissionedIdentity` (DID →
`PermissionedIdentityPrefs { intended_count, running_count }`, :127-130; types.rs:9-19),
`SlashingAllowedFor` (`SlashingSwitch: Validator | ValidatorAndNominator | None`, default
**None**, :132-135; types.rs:44-52), `ValidatorCommissionCap` (Perbill, :137-140),
`CurrentPayoutEra` / `PendingPayouts` (auto-payout queue, :142-156).

| Extrinsic (idx) | Origin | Behavior | Ref |
|---|---|---|---|
| `add_permissioned_validator(0)` | `AdminOrigin` (= Root; GC reaches it via PIPs) | approve DID; default `intended_count = 1`, capped by `MaxValidatorPerIdentity × validator_count` | lib.rs:274 → permissioned.rs:218-256 |
| `remove_permissioned_validator(1)` | AdminOrigin | de-approve (does **not** auto-chill — see §3) | lib.rs:291 → permissioned.rs:258 |
| `change_slashing_allowed_for(3)` | root | set the slashing switch | lib.rs:301 → permissioned.rs:278 |
| `update_permissioned_validator_intended_count(4)` | AdminOrigin | adjust slots | lib.rs:311 → permissioned.rs:288 |
| `chill_from_governance(5)` | AdminOrigin | chill all given stashes **and remove the identity's permission** (permissioned.rs:356) | lib.rs:326 → permissioned.rs:333-363 |
| `set_commission_cap(6)` | AdminOrigin | set cap and **clamp every existing validator's commission** to it (permissioned.rs:320-323) | lib.rs:341 → permissioned.rs:307 |

Automatic payouts: `end_era` snapshots session validators into `PendingPayouts`
(permissioned.rs:168-181); `on_initialize` drains them weight-metered by `MaxPayoutWeight`
(lib.rs:262-267; permissioned.rs:366-478) calling `do_payout_stakers_by_page` — stakers don't
need to claim manually.

## 3. The `PermissionedStaking` hook (fork ↔ validators pallet)

The fork adds `Config::Permissioned: PermissionedStaking<T>` (FORK/src/pallet/mod.rs:342-347;
trait FORK/src/permissioned_staking.rs:11-78); runtime binds it to `Validators` (RTC:349).
Enforcement points inside forked staking:

| Hook | Fork call site | Validators impl |
|---|---|---|
| `on_validate` — commission ≤ cap (also for **existing** validators re-calling `validate`); new validators need a DID that is permissioned with a free slot (`running_count < intended_count`) | FORK mod.rs:1359-1363 | permissioned.rs:103-129 |
| `on_chill` / `on_nominate` / `on_kill` — release the slot + key refcount | impls.rs:408-413/mod.rs:1445-1448/impls.rs:810-813 | permissioned.rs:132-145, 196-211 |
| `is_validator_compliant` — election snapshot filters: only compliant validators become targets/self-voters (DID exists, permissioned, bond ≥ `MinValidatorBond`) | impls.rs:969-990, 1060-1066 | permissioned.rs:148-154 |
| `who_to_slash` / slashing gates — offences zeroed when switch is `None`; nominators slashed only under `ValidatorAndNominator` | impls.rs:1283-1292; slashing.rs:299-305, 615-628 | permissioned.rs:163-165 |
| `reapable` — Polymesh allows reaping at `amount <= ED` | mod.rs:1848-1855, impls.rs:209-212 | permissioned.rs:99-101 |
| `add_pending_payouts` — era-end payout trigger | impls.rs:603-606 | permissioned.rs:168-181 |

Key consequence: **de-permissioning does not force a chill** — the validator merely stops being
electable at the next election (snapshot filter). `chill_from_governance` is the forcible path.

## 4. Inflation & rewards

- `EraPayout = pallet_validators::PolymeshConvertCurve<RewardCurve>` (RTC:338;
  permissioned.rs:21-46). `compute_total_payout` (inflation.rs:32-64): standard NPoS curve
  (min 2.5%, **max 14%**, ideal stake 70%, falloff 5% — identical in all runtimes, e.g.
  `pallets/runtime/mainnet/src/runtime.rs:186-195`) **until total issuance ≥
  `MaxVariableInflationTotalIssuance` (1B POLYX)**; then a fixed `FixedYearlyReward`
  (140M POLYX/yr, prorated per era) applies with **zero remainder** (inflation.rs:51-60).
- `RewardRemainder = ()` — remainder dropped, not minted (RTC:329); rewards minted from void;
  **slashes go to Treasury** (`Slash = Treasury`, RTC:331).

## 5. Runtime parameters

| Const | mainnet | testnet | develop |
|---|---|---|---|
| SessionsPerEra / BondingDuration / SlashDeferDuration | 6 / 28 / 14 (`mainnet/src/runtime.rs:158-160`) | 6 / 28 / 14 | 3 / 7 / 4 (`develop/src/runtime.rs:162-164`) |
| MaxValidatorPerIdentity | 33% (:166) | 33% | 33% |
| MaxVariableInflationTotalIssuance / FixedYearlyReward | 1B / 140M POLYX (:164-165) | same | same |
| MaxPayoutWeight | 20% of block (:167) | 10% | 5% |

Election: `ElectionProviderMultiPhase` with **signed phase disabled** (`SignedPhase = 0`,
`pallets/runtime/common/src/lib.rs:155-157`), unsigned phase = ¼ epoch, `MaxWinners = 1000`,
SequentialPhragmen + `OffchainRandomBalancing`, on-chain fallback (RTC:762-827).
`cancel_deferred_slash` requires AdminOrigin (FORK mod.rs:1715-1726); `SlashDeferDuration`
gives governance 14 eras (mainnet) to cancel.

## 6. Invariants & review checklist

- [ ] `running_count ≤ intended_count` per permissioned identity; every validate/chill/nominate/
      kill path must inc/dec through the hook (slot leaks block honest validators).
- [ ] Commission cap must be enforced on *both* new and re-submitted `validate()` calls
      (fork mod.rs:1359-1363 — regression-tested by
      `staking_extra_tests.rs:83` `existing_validator_cannot_bypass_commission_cap`).
- [ ] Election compliance filters (targets *and* self-votes) are the only thing excluding
      de-permissioned validators — keep both call sites (impls.rs:972, :1063).
- [ ] Slashing switch default None: enabling slashing is a governance decision; offence handling
      must keep zeroing fractions when disabled.
- [ ] Validator stashes hold identity key refcounts while validating
      (`AccountKeyRefCount`, staking_extra_tests.rs:12-81) — keys can't leave their DID
      mid-validation.
- [ ] Inflation cap boundary: at issuance ≥ 1B the fixed branch must return zero remainder
      (treasury gets nothing from era payouts by design).

## 7. Test map

`pallets/validators/src/tests.rs` (ported upstream suite, 9.4k lines) + `mock.rs`;
Polymesh-specific: `pallets/runtime/tests/src/staking_extra_tests.rs` (permission lifecycle,
refcounts, commission-cap bypass). Fork-side hook tests in FORK/src/tests.rs.
