# 02 — Permission Model & Enforcement Pipeline

Sources: `primitives/src/secondary_key.rs`, `primitives/src/subset.rs`,
`pallets/permissions/src/lib.rs`, `pallets/identity/src/keys.rs`,
`pallets/runtime/common/src/runtime.rs`.
Related specs: [01-identity-keys](01-identity-keys.md), [05-external-agents](05-external-agents.md)
(asset-scoped agent permissions), [08-portfolio](08-portfolio.md) (portfolio permission checks),
[14-fees-and-extensions](14-fees-and-extensions.md).

## 1. Purpose

Polymesh restricts what each account key may do *within its identity*. The **primary key has
unrestricted access** to the identity. **Secondary keys** carry a `Permissions` value restricting
(a) which extrinsics they may call, (b) which assets they may administer, and (c) which portfolios
they may operate on. This doc covers the data model, the per-call enforcement pipeline, and the
catalog of checks pallets must apply.

Layer summary (a call may pass through all four):

```
signed extrinsic
  │ 1. StoreCallMetadata TxExtension records (pallet, extrinsic) names
  ▼
pallet dispatch → Identity::ensure_origin_call_permissions(origin)
  │ 2. key → DID resolution; if secondary key: DID-not-frozen +
  │    extrinsic-permission subset check       (generic, same for all pallets)
  ▼
pallet-specific logic
  │ 3. secondary-key asset subset  → ExternalAgents::ensure_agent_asset_perms
  │    secondary-key portfolio subset → Portfolio::ensure_portfolio_custody_and_permission
  ▼
  │ 4. asset-scoped agent-group check (applies to primary keys too; doc 05)
  ▼ storage changes
```

## 2. Permission data model (`primitives/src/secondary_key.rs`)

```
Permissions {                                   // secondary_key.rs:217
    asset:     SubsetRestriction<AssetId>,      // = AssetPermissions, :41
    extrinsic: ExtrinsicPermissions,            // :107
    portfolio: SubsetRestriction<PortfolioId>,  // = PortfolioPermissions, :207
}
```

- `SubsetRestriction<A>` (primitives/src/subset.rs:28): `Whole` (everything) |
  `These(BTreeSet<A>)` (only these) | `Except(BTreeSet<A>)` (all but these).
- `Permissions::default()` = full access (`Whole` everywhere); `Permissions::empty()` = none
  (secondary_key.rs:226-234). **Caution when reviewing**: `default()` is *permissive*.
- `ExtrinsicPermissions` is two-level (secondary_key.rs:107): `Whole` |
  `These(BTreeMap<PalletName, PalletPermissions>)` | `Except(...)`, where
  `PalletPermissions { extrinsics: SubsetRestriction<ExtrinsicName> }` (secondary_key.rs:59)
  selects functions within the pallet.
- Matching: `ExtrinsicPermissions::sufficient_for(pallet, extrinsic)` (secondary_key.rs:162) —
  names are the literal Rust pallet module/function names from `GetCallMetadata` (e.g. pallet
  `"Asset"`, extrinsic `"issue"`).
- Per-key checks: `SecondaryKey::has_extrinsic_permission` (:464), `has_asset_permission` (:474),
  `has_portfolio_permission` (:483).

### Validation limits (applied wherever permissions are accepted as input)

`Identity::ensure_perms_length_limited` (pallets/identity/src/keys.rs:740):
- total complexity ≤ 1,000,000 (`MAX_PERMISSION_COMPLEXITY` keys.rs:53; complexity =
  name lengths (min 10/name) + 16·assets + 40·portfolios, secondary_key.rs:248-260);
- asset/portfolio set sizes and pallet/extrinsic counts ≤ `MAX_ASSETS`/`MAX_PORTFOLIOS` (2000)
  and `MAX_PALLETS`/`MAX_EXTRINSICS` (80) — primitives/src/identity.rs:43-54 (smaller under the
  `running-ci` feature, :27-39);
- **`Except` is forbidden for extrinsic permissions** at both levels
  (`ensure_no_except_perms` keys.rs:750, error `ExceptNotAllowedForExtrinsics`) because
  extrinsic renames/additions would silently widen an `Except` grant. `Except` **is** allowed
  for asset/portfolio subsets.

Callers of this validation: `set_secondary_key_permissions` (keys.rs:393),
`add_secondary_keys_with_authorization` (keys.rs:487), auth creation for
`JoinIdentity`/`RotatePrimaryKeyToSecondary` (auth.rs:37-41), `base_register_did`
(claims.rs:226). Weights scale with permission counts (`permissions_cost_perms`,
pallets/identity/src/lib.rs:144).

## 3. Layer 1 — recording call metadata (`pallets/permissions`)

`StoreCallMetadata` transaction extension (pallets/permissions/src/lib.rs:156):
- `prepare()` stores the dispatched call's pallet/function names into `CurrentPalletName`
  (lib.rs:110) and `CurrentDispatchableName` (lib.rs:115) using `GetCallMetadata` (lib.rs:222-233).
- `post_dispatch()` clears them (lib.rs:235-244).
- Wired into the runtime `TxExtension` tuple (pallets/runtime/common/src/runtime.rs:913),
  after `ChargeTransactionPayment`, so fee logic runs before metadata is stored.

**Nested calls**: wrappers that dispatch inner calls must swap metadata so permission checks see
the *inner* call: `with_call_metadata`/`swap_call_metadata` (lib.rs:251/263). Users:
- `Utility` batch/relay (pallets/utility/src/lib.rs:494,601)
- `MultiSig` proposal execution (pallets/multisig/src/lib.rs:1125)
- EVM precompiles dispatching runtime calls (pallets/precompiles/src/common.rs:102,227,262)

A wrapper that forgets this lets a secondary key smuggle a forbidden call inside an allowed
wrapper — check this on any new dispatch-wrapping code.

## 4. Layer 2 — the generic permission check

Entry points (used by nearly every extrinsic in asset/settlement/portfolio/CA/etc.):

| Entry point | Returns | Ref |
|---|---|---|
| `Identity::ensure_origin_call_permissions(origin)` | `PermissionedCallOriginData { sender, primary_did, secondary_key }` | keys.rs:719 |
| `Identity::ensure_perms(origin)` | just the DID | keys.rs:735 |
| `Identity::ensure_valid_origin(origin, must_be_primary_key)` | `(AccountId, IdentityId)`; optional primary-only mode | keys.rs:776 |
| `pallet_permissions::Pallet::ensure_call_permissions(who)` | `AccountCallPermissionsData` | permissions lib.rs:126 |

All delegate to the `CheckAccountCallPermissions` trait (permissions lib.rs:76); the runtime binds
`type Checker = Identity` (runtime.rs:640). Identity's implementation
(`ensure_valid_origin_permissions`, keys.rs:824-862) resolves the caller key:

| Caller `KeyRecord` | Result |
|---|---|
| none | `Err(MissingIdentity)` |
| `PrimaryKey(did)` | **pass unconditionally**; `secondary_key = None` |
| `SecondaryKey(did)`, DID frozen | `Err(UnauthorizedCallerFrozenDid)` (keys.rs:847) |
| `SecondaryKey(did)`, insufficient extrinsic perms for (`CurrentPalletName`, `CurrentDispatchableName`) | `Err(UnauthorizedCallerMissingPermissions)` (keys.rs:853) |
| `SecondaryKey(did)`, sufficient | pass; `secondary_key = Some(SecondaryKey)` |
| `MultiSigSignerKey(_)` | `Err(KeyNotAllowed)` — signers act *through* the multisig, doc 16 |
| `must_be_primary_key = true` and not primary | `Err(KeyNotAllowed)` (keys.rs:832-837) |

`check_account_call_permissions` (keys.rs:807) additionally rejects locked DIDs
(`is_did_locked` — currently a stub returning false, claims.rs:70).

**Convention**: `secondary_key == None` ⇒ caller is the primary key ⇒ skip layer-3 subset checks.
Every layer-3 helper follows this pattern (`if let Some(sk) = secondary_key { check... }`).

## 5. Layer 3 — asset & portfolio subset checks (per-pallet duty)

The generic check only covers *extrinsic* permissions. Pallets touching an asset or portfolio must
additionally check the secondary key's asset/portfolio subsets:

- **Asset scope**: `ExternalAgents::ensure_asset_perms` (pallets/external-agents/src/lib.rs:639)
  → `sk.has_asset_permission(asset_id)` (:649), error `SecondaryKeyNotAuthorizedForAsset`.
  Usually invoked via `ensure_agent_asset_perms` (:628) which also applies the agent-group check
  (layer 4, doc 05). Used by asset, compliance, statistics, corporate actions, sto, nft, etc.
- **Portfolio scope**: `Portfolio::ensure_portfolio_custody_and_permission`
  (pallets/portfolio/src/lib.rs:814) = custody check + `ensure_user_portfolio_permission`
  (:782) → `sk.has_portfolio_permission(portfolio_id)`, error `InsufficientPortfolioPermissions`.
  Used by settlement affirmation, portfolio moves, sto investment, etc. (doc 08).

**Review rule**: an extrinsic that operates on a caller-chosen asset/portfolio but only calls
`ensure_perms` (never a layer-3 helper) lets any secondary key with matching *extrinsic*
permissions act on *all* assets/portfolios of the identity. That is sometimes intentional
(e.g. pure-identity operations) but must be deliberate.

## 6. Primary-key-only actions

Enforced via `ensure_primary_key` (keys.rs:690) or `ensure_valid_origin(_, true)` — not
expressible through `Permissions`:

| Action | Ref |
|---|---|
| `Identity::set_secondary_key_permissions` | keys.rs:388 |
| `Identity::remove_secondary_keys` | keys.rs:414 |
| `Identity::add_secondary_keys_with_authorization` | keys.rs:450 |
| `Identity::freeze_secondary_keys` / `unfreeze_secondary_keys` | keys.rs:574 |

Also primary-key-relevant: primary key rotation targets (doc 01 §4) and multisig admin calls that
require the multisig's *creator/paying* identity primary key (doc 16). When adding a new
"identity administration" extrinsic, decide explicitly whether secondary keys may call it; default
to primary-only for anything that changes the key set or permissions (a secondary key must never
be able to escalate its own permissions).

## 7. Freezing

`freeze_secondary_keys` sets `IsDidFrozen` (keys.rs:570-583): all secondary keys are disabled at
once (layer-2 check keys.rs:847; also `get_identity` returns `None` for frozen secondary keys,
keys.rs:72). The primary key is unaffected and is the only key able to unfreeze. Freezing does
not cancel authorizations or unlink keys.

## 8. What is *not* checked anymore

- **CDD**: since 8.0.0 no transaction gate checks CDD claims; DID existence suffices
  (doc 01 §3, doc 03 §4).
- **DID locking**: `is_did_locked` is a TODO stub (claims.rs:70); the
  `UnauthorizedCallerDidInactive` error paths (keys.rs:697-700, 711-714, 816-819) are currently
  unreachable.

## 9. Invariants & review checklist

- [ ] Every signed extrinsic resolves its origin through one of the §4 entry points (or
      explicitly `ensure_did`/`ensure_primary_key` with justification). Raw `ensure_signed`
      without identity resolution is suspect outside low-level pallets
      (balances/indices/multisig signer calls/revive).
- [ ] Extrinsics operating on caller-chosen assets/portfolios apply layer-3 checks when
      `secondary_key.is_some()`.
- [ ] New dispatch wrappers use `with_call_metadata` around inner-call dispatch.
- [ ] Permission inputs pass `ensure_perms_length_limited`; extrinsic perms reject `Except`.
- [ ] No path grants a secondary key the ability to modify its own or others' permissions.
- [ ] Pallet/extrinsic renames break existing `These`-permissions silently (grants reference
      names, not indices) — renaming requires a migration or release note.
- [ ] `StoreCallMetadata` must remain positioned in `TxExtension` such that it runs for every
      dispatchable path (runtime.rs:901-917); revive/EVM entry (`SetOrigin`, doc 21) and
      off-chain-submitted calls need equivalent handling.

## 10. Test map

- `pallets/runtime/tests/src/identity_test.rs` (frozen keys, permission checks,
  `secondary_keys_with_auth`), `signed_extra.rs` (extension ordering),
  `utility_test.rs` (batch permission semantics), `multisig.rs` (nested call metadata),
  `portfolio.rs` / `external_agents_test.rs` (layer-3 checks).
- Permission subset unit tests: primitives/src/secondary_key.rs:524 (`has_permission_test`).
