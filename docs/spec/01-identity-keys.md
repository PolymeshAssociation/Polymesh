# 01 — Identity & Key Management

Sources: `pallets/identity/src/{lib.rs,keys.rs,auth.rs,types.rs}`,
`primitives/src/secondary_key.rs`, `primitives/src/authorization.rs`, `primitives/src/crypto.rs`.
Related specs: [02-permissions](02-permissions.md) (how keys are permission-checked),
[03-claims](03-claims.md), [14-fees-and-extensions](14-fees-and-extensions.md) (who pays for
auth-accepting calls), [16-multisig](16-multisig.md).

## 1. Purpose

Every actor on Polymesh is an **identity** (`IdentityId`, a DID): a container that groups account
keys, claims, portfolios, and asset roles. Account keys sign transactions; identities carry the
on-chain rights. The identity pallet owns the key↔DID mapping, DID creation, the generic
two-phase **authorization** machinery, and claims (doc 03).

## 2. Data model

### Types

| Type | Definition | Notes |
|---|---|---|
| `IdentityId` | `primitives/src/identity_id.rs` | 32-byte DID |
| `DidRecord` | primitives/src/identity.rs:75 | `{ primary_key: Option<AccountId> }` — `None` after the primary key was unlinked without replacement |
| `KeyRecord` | primitives/src/secondary_key.rs:287 | `PrimaryKey(IdentityId) \| SecondaryKey(IdentityId) \| MultiSigSignerKey(AccountId)` |
| `SecondaryKey` | primitives/src/secondary_key.rs:432 | `{ key: AccountId, permissions: Permissions }` (perms detailed in doc 02) |
| `Signatory` | primitives/src/secondary_key.rs:343 | `Identity(IdentityId) \| Account(AccountId)` — target of an authorization |
| `Authorization` | primitives/src/authorization.rs:108 | `{ authorization_data, authorized_by: IdentityId, expiry: Option<Moment>, auth_id: u64, count: u32 }` |
| `AuthorizationData` | primitives/src/authorization.rs:30 | variant per two-phase operation, see §5 |

### Storage (pallets/identity/src/lib.rs)

| Item | Key → Value | Ref |
|---|---|---|
| `DidRecords` | DID → `DidRecord` | lib.rs:379 |
| `KeyRecords` | AccountId → `KeyRecord` | lib.rs:417 |
| `DidKeys` | (DID, AccountId) → bool — reverse index of all keys of a DID | lib.rs:440 |
| `IsDidFrozen` | DID → bool — secondary keys frozen | lib.rs:384 |
| `KeyExtrinsicPermissions` | AccountId → `ExtrinsicPermissions` | lib.rs:423 |
| `KeyAssetPermissions` | AccountId → `AssetPermissions` | lib.rs:429 |
| `KeyPortfolioPermissions` | AccountId → `PortfolioPermissions` | lib.rs:435 |
| `AccountKeyRefCount` | AccountId → u64 strong refs (blocks unlinking) | lib.rs:486 |
| `MultiPurposeNonce` | u64 nonce for DID generation | lib.rs:445 |
| `OffChainAuthorizationNonce` | DID → u64 nonce for off-chain key-add signatures | lib.rs:449 |
| `Authorizations` | (Signatory, auth_id) → `Authorization` | lib.rs:455 |
| `AuthorizationsGiven` | (issuer DID, auth_id) → Signatory | lib.rs:467 |
| `NumberOfGivenAuths` | DID → u32 (capped by `MaxGivenAuths`) | lib.rs:491 |
| `OutdatedAuthorizations` | Signatory → u64 threshold; auths with id ≤ threshold are invalid | lib.rs:495 |
| `CurrentAuthId` | u64 global auth id counter | lib.rs:500 |
| claims storage | see doc 03 | lib.rs:389-413 |

Per-key permissions are stored in three separate maps (not inside a `SecondaryKey` struct);
`get_key_permissions` (keys.rs:107) reassembles them, defaulting each missing map to its
`Default` (= full access — but maps are always written when a secondary key is linked, via
`set_key_permissions` keys.rs:204).

### Key model invariants

1. **One key, one identity**: a key can be linked to at most one DID (or one multisig).
   Enforced by `add_key_record` (keys.rs:227, no-op if already linked) and
   `can_add_key_record`/`ensure_key_did_unlinked` (keys.rs:194-202); error `AlreadyLinked`.
2. **One primary key per identity**: `DidRecords[did].primary_key` is a single value; rotation
   replaces it atomically (`common_rotate_primary_key`, keys.rs:296).
3. **Primary keys cannot be frozen** — freezing affects only secondary keys (`get_identity`
   keys.rs:69-76 returns `None` for a frozen *secondary* key but always resolves primary keys).
4. **Strong references block unlinking**: `AccountKeyRefCount` > 0 ⇒
   `ensure_key_unlinkable_from_did` fails with `AccountKeyIsBeingUsed` (keys.rs:185). Refs are
   added by pallets holding balances against the key (asset holdings, NFTs — see
   `add_account_key_ref_count` keys.rs:175 callers).
5. **MultiSig signer keys are not identity keys**: `KeyRecord::MultiSigSignerKey` maps signer →
   multisig account; the multisig account itself is the identity key (see doc 16).

## 3. DID creation

| Path | Origin requirement | Ref |
|---|---|---|
| `register_did(target_account)` | caller DID ∈ `DidRegistrars` group (pallet_group Instance2) | lib.rs:882, claims.rs:214 (`base_register_did`) |
| `self_register_did()` | any unlinked signed account (permissionless self-onboarding) | lib.rs:898 |
| `cdd_register_did`, `cdd_register_did_with_cdd` | DID registrar; **deprecated since 8.0.0** (CDD not enforced); only path that supports initial secondary keys | lib.rs:597, lib.rs:857 |
| genesis config | — | lib.rs:512-578 |
| systematic identities | chain-internal (`register_systematic_id` keys.rs:651) | `SystematicIssuers`, primitives/src/constants.rs:109 |

All funnel into `register_did_without_cdd` (keys.rs:602):
1. target must be unlinked (keys.rs:608);
2. secondary keys must not contain the primary key nor duplicates (keys.rs:610-617);
3. DID = `blake2_256(USER, babe_randomness(nonce), nonce)` via `make_did` (keys.rs:586) —
   `MultiPurposeNonce` increments even on failure for unpredictability (keys.rs:588);
4. protocol fee `IdentityRegisterDid` charged (keys.rs:623);
5. primary key linked, `InitialPOLYX` deposited (0 on mainnet/develop, 100k POLYX testnet —
   `pallets/runtime/testnet/src/runtime.rs:153`);
6. secondary keys are **not** linked directly — a `JoinIdentity` authorization is created per
   key (keys.rs:634-639), which each key must accept.

Since 8.0.0, **DID existence == active** (`is_did_active` claims.rs:64); no CDD claim is required
to transact. `is_did_locked` is a stub always returning `false` (claims.rs:70, TODO).

## 4. Key management operations

### Extrinsic & authorization matrix

| Extrinsic (call_index) | Who may call | Behavior | Ref |
|---|---|---|---|
| `accept_primary_key(2)` | new key (auth target) | consume `RotatePrimaryKey` auth; old primary key **unlinked** | lib.rs:620 → keys.rs:280 |
| `rotate_primary_key_to_secondary(15)` | new key (auth target) | consume `RotatePrimaryKeyToSecondary(perms)`; old primary key becomes secondary with `perms` | lib.rs:770 → keys.rs:364 |
| `join_identity_as_key(4)` | key (auth target) | consume `JoinIdentity(perms)`; link key as secondary | lib.rs:627 → keys.rs:512 |
| `leave_identity_as_key(5)` | the secondary key itself | unlink self; blocked if `AccountKeyRefCount` > 0 | lib.rs:634 → keys.rs:550 |
| `add_secondary_keys_with_authorization(16)` | **primary key only** | batch-link keys that signed an off-chain `ChainScopedMessage` | lib.rs:792 → keys.rs:445 |
| `set_secondary_key_permissions(17)` | **primary key only** | overwrite one secondary key's permissions | lib.rs:806 → keys.rs:383 |
| `remove_secondary_keys(18)` | **primary key only** | unlink keys (each must have refcount 0); outdates their pending auths | lib.rs:822 → keys.rs:410 |
| `freeze_secondary_keys(8)` / `unfreeze_secondary_keys(9)` | **primary key only** | set/clear `IsDidFrozen` | lib.rs:683/690 → keys.rs:570 |
| `add_authorization(10)` | any permissioned key of issuer DID | create an authorization | lib.rs:698 → auth.rs:30 |
| `remove_authorization(11)` | issuer (revoke) or target (reject) | delete an authorization | lib.rs:712 → auth.rs:94 |

"Primary key only" is enforced via `ensure_primary_key` (keys.rs:690): the caller's `KeyRecord`
must be `PrimaryKey(_)`, else `KeyNotAllowed`. Note this does **not** go through the
extrinsic-permission pipeline — a secondary key with `Whole` extrinsic permissions still cannot
call these.

### Primary key rotation (`common_rotate_primary_key`, keys.rs:296)

Rules, in order:
1. Identity must currently have a primary key (else `InvalidAccountKey`).
2. New key must be **unlinked**, or already a **secondary key of the same DID** (promote-in-place);
   a key of another DID / a multisig signer key ⇒ `AlreadyLinked` (keys.rs:304-323).
3. If the old primary key is being dropped (plain `accept_primary_key`), it must have
   `AccountKeyRefCount == 0` (keys.rs:325-327).
4. Storage updates: new key becomes `PrimaryKey(did)`, `DidRecords[did]` repointed; when promoting
   a secondary key its old record is overwritten (its per-key permission maps are *not* explicitly
   cleared here — the maps are only removed by `remove_key_record` keys.rs:262; promoted keys keep
   stale permission entries that are ignored while primary). Events: `SecondaryKeysRemoved` (if
   promoted), `PrimaryKeyUpdated`, and `SecondaryKeysAdded` (if old key demoted with perms).

Both rotation extrinsics are auth-accepting: the *issuing identity's primary key pays the fees*
(fee redirection in `pallets/runtime/common/src/fee_details.rs:199-220`, see doc 14).

### Batch key addition with off-chain signatures (keys.rs:445)

`add_secondary_keys_with_authorization` verifies for each key an sr25519/ed25519 signature over a
`ChainScopedMessage { genesis_hash, nonce, label: "Polymesh Identity Add Secondary Key",
expires_at, did }` (primitives/src/crypto.rs:80,93; wrapped in `<Bytes>...</Bytes>` for Polkadot-JS
compat, crypto.rs:59-77). The per-DID `OffChainAuthorizationNonce` increments once per batch
(keys.rs:459), so old signatures cannot be replayed; `expires_at` must be in the future
(crypto.rs:110-113 via `ChainScopedMessage::new`, error `AuthorizationExpired`). Protocol fee
charged per key (keys.rs:453). Duplicate keys in a batch ⇒ `DuplicateKey`; already-linked keys ⇒
`AlreadyLinked`.

## 5. Authorization machinery (two-phase operations)

Flow: issuer calls `add_authorization` (or a pallet calls `add_auth` internally) → target later
accepts via a *type-specific* extrinsic that runs `accept_auth_with` (auth.rs:183) → auth is
validated (exists, not outdated, not expired), the per-type closure applies the change, the auth
is consumed. Rejection/revocation via `remove_authorization` (auth.rs:94): issuer may revoke;
target may reject.

| `AuthorizationData` variant | Consuming extrinsic | Ref |
|---|---|---|
| `RotatePrimaryKey` | `Identity::accept_primary_key` | keys.rs:286-289 |
| `RotatePrimaryKeyToSecondary(Permissions)` | `Identity::rotate_primary_key_to_secondary` | keys.rs:370-377 |
| `JoinIdentity(Permissions)` | `Identity::join_identity_as_key`; also `MultiSig::approve_join_identity` (multisig-as-key flow, doc 16) | keys.rs:512-534 |
| `TransferTicker(Ticker)` | `Asset::accept_ticker_transfer` | pallets/asset/src/lib.rs:2059 |
| `TransferAssetOwnership(AssetId)` | `Asset::accept_asset_ownership_transfer` | pallets/asset/src/lib.rs:2081 |
| `BecomeAgent(AssetId, AgentGroup)` | `ExternalAgents::accept_become_agent` | pallets/external-agents/src/lib.rs:391 |
| `AddMultiSigSigner(AccountId)` | `MultiSig::accept_multisig_signer` | pallets/multisig/src/lib.rs:1225 |
| `PortfolioCustody(PortfolioId)` | `Portfolio::accept_portfolio_custody` | pallets/portfolio/src/lib.rs:852 |
| `AttestPrimaryKeyRotation`, `OldAddRelayerPayingKey` | deprecated, no consumer | primitives/src/authorization.rs:32,52 |

Mechanics and limits:
- Auth ids are globally unique (`CurrentAuthId` increment, auth.rs:66).
- Per-issuer cap: `NumberOfGivenAuths < MaxGivenAuths` (=1024 all runtimes,
  `pallets/runtime/develop/src/runtime.rs:148`), else `ExceededNumberOfGivenAuths` (auth.rs:58-62).
- `JoinIdentity`/`RotatePrimaryKeyToSecondary` payloads are complexity-checked at creation
  (auth.rs:37-41).
- Expiry is checked at acceptance (`expiry > now`, auth.rs:192-195); expired auths are *not*
  garbage-collected automatically.
- **Outdating**: when a secondary key is removed, all auths targeting it with
  `auth_id <= CurrentAuthId` are invalidated via `OutdatedAuthorizations` (keys.rs:429-441,
  checked in `ensure_authorization` auth.rs:222-226). Despite the comment at keys.rs:428, there is
  no `on_initialize` cleanup — outdated auth storage entries persist and are only rejected on use.
- **Retry counting**: `count` starts at `MaxAuthRetries` (=10, runtime.rs:149; auth.rs:74).
  For the fee-redirected accept calls, a *failed* dispatch decrements the count
  (`polymesh-transaction-payment` post_dispatch, pallets/transaction-payment/src/lib.rs:533-536 →
  fee_details.rs:297-315 → auth.rs:231). `get_non_expired_auth` (auth.rs:162) treats `count == 0`
  as unusable, so the payer lookup fails and the tx becomes invalid at the pool — this stops a
  malicious target from draining the issuer via repeatedly failing accepts. Note: the
  `AuthorizationRetryLimitReached` event (lib.rs:353) is declared but never emitted.
- Fee redirection: for `join_identity_as_key`, `accept_primary_key`,
  `rotate_primary_key_to_secondary`, `accept_multisig_signer` and
  `remove_authorization{auth_issuer_pays: true}`, the **auth issuer's primary key** is charged
  instead of the caller (fee_details.rs:189-244; doc 14).

## 6. Cross-pallet surface

Helpers other pallets rely on (all `pallets/identity/src/keys.rs` unless noted):

| Helper | Purpose | Ref |
|---|---|---|
| `get_identity(key)` | key → DID; `None` if frozen secondary / multisig signer | keys.rs:69 |
| `ensure_perms(origin)` / `ensure_origin_call_permissions(origin)` | full permission pipeline (doc 02) | keys.rs:735/719 |
| `ensure_did(origin)` | key → DID without extrinsic-permission check | keys.rs:706 |
| `ensure_primary_key(origin)` | primary-key-only gate | keys.rs:690 |
| `ensure_valid_origin(origin, must_be_primary_key)` | permission check with optional primary-only mode | keys.rs:776 |
| `add_auth` / `accept_auth_with` / `ensure_auth_by` | authorization machinery for other pallets | auth.rs:52/183/176 |
| `add_account_key_ref_count` / `remove_account_key_ref_count` | strong refs (asset/NFT account holdings) | keys.rs:175/180 |
| `add_key_record` / `remove_key_record` | used by multisig for signer keys | keys.rs:227/243 |
| `asset_holder_did(AssetHolder)` | resolve Portfolio/Account holder → DID | keys.rs:790 |
| `CheckAccountCallPermissions` impl | the permission `Checker` (doc 02) | keys.rs:807 |

## 7. Invariants & review checklist

When reviewing changes touching identity/keys, verify:

- [ ] No path links a key already present in `KeyRecords` (would break 1-key-1-DID);
      all insertions go through `add_key_record` / check `can_add_key_record`.
- [ ] Any path unlinking a key checks `AccountKeyRefCount == 0` (else asset balances become
      inaccessible-but-orphaned) and removes the per-key permission maps.
- [ ] `DidRecords[did].primary_key`, `KeyRecords[key]`, and `DidKeys[did][key]` stay mutually
      consistent (all three updated together in add/remove/rotate paths).
- [ ] New auth-accepting extrinsics must: use `accept_auth_with`, validate `auth.authorized_by`
      has authority over the object (e.g. via `ensure_auth_by`), and consider adding them to
      `fee_details.rs` if the caller may lack POLYX.
- [ ] Primary-key-only extrinsics use `ensure_primary_key`, not `ensure_perms`.
- [ ] Anything creating authorizations respects `MaxGivenAuths` (use `add_auth`, don't insert
      into `Authorizations` directly).
- [ ] Permission payloads validated with `ensure_perms_length_limited` (keys.rs:740).

## 8. Test map

- Unit: `pallets/runtime/tests/src/identity_test.rs` (rotation, join/leave, freeze, auths,
  off-chain key adds), `fee_details.rs` + `signed_extra.rs` (payer redirection).
- Integration: `integration/tests/` identity flows (secondary keys, portfolios custody use
  auth machinery heavily).
