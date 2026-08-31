# 04 — Asset Lifecycle (Fungible & NFT)

Sources: `pallets/asset/src/{lib.rs,types.rs}`, `pallets/nft/src/lib.rs`,
`primitives/src/asset.rs`, `primitives/src/nft.rs`, `primitives/src/asset_metadata.rs`.
Related specs: [05-external-agents](05-external-agents.md) (who may administer),
[09-asset-transfers](09-asset-transfers.md) (transfer paths), [06-compliance](06-compliance.md),
[07-statistics](07-statistics.md), [11-checkpoints](11-checkpoints.md).

"Agent" below = caller passing `ExternalAgents::ensure_perms` (permissioned external agent of the
asset — the owner is the initial `Full` agent; doc 05). "Permissioned DID" = caller passing the
generic identity pipeline (doc 02) with no asset-role requirement.

## 1. Data model

### Fungible assets

| Type | Shape | Ref |
|---|---|---|
| `AssetId` | `[u8;16]`, formatted as UUIDv8 | primitives/src/asset.rs:33, 35-43 |
| `AssetDetails` | `{ total_supply, owner_did, divisible, asset_type }` | pallets/asset/src/types.rs:19-28 |
| `AssetType` | Equity/Commodity/.../`Custom(CustomAssetTypeId)`/StableCoin/`NonFungible(NonFungibleType)` | primitives/src/asset.rs:93-131 |
| `TickerRegistration` | `{ owner, expiry: Option<Moment> }` | types.rs:59-62 |
| `AssetHolder` | `Portfolio(PortfolioId) \| Account(AccountId32)` — assets can be held by portfolios **or** raw account keys | primitives/src/asset.rs:194-199 |

**AssetId generation** (`generate_asset_id`, pallets/asset/src/lib.rs:3646-3660): deterministic
`blake2_128(("modlpy/pallet_asset", genesis_hash, caller_account, AssetNonce[caller]))`,
nonce post-incremented, then UUIDv8-normalized. Collision guard `ensure_new_asset_id`
(lib.rs:3632-3638).

### Asset storage highlights (pallets/asset/src/lib.rs)

| Item | Purpose | Ref |
|---|---|---|
| `Assets` | AssetId → `AssetDetails` | :389 |
| `BalanceOf` | (AssetId, DID) → aggregate per-identity balance | :402 |
| `AssetBalance` / `LockedBalance` / `FrozenBalance` / `FrozenAccounts` | per-**account-key** holdings, locks, frozen amounts, frozen flag | :602/:614/:645/:657 |
| `Frozen` | AssetId → bool (asset-wide freeze) | :443 |
| `Allowances` | (owner, spender, AssetId) → Balance; ERC-20 style (doc 09) | :632 |
| `UniqueTickerRegistration` / `TickerConfig` | ticker ownership + expiry / max len & duration | :379/:384 |
| `TickerAssetId` / `AssetIdTicker` | 1:1 ticker↔asset link | :593/:588 |
| `AssetDocuments`(+`IdSequence`) | attached documents | :447/:460 |
| metadata maps (local/global names, specs, values, details) | see §6 | :465-575 |
| `AssetsExemptFromAffirmation` / `PreApprovedAsset` | receiver-affirmation exemptions (doc 09 §5) | :547/:552 |
| `MandatoryMediators` | AssetId → bounded set of required mediator DIDs (doc 10) | :558 |
| `SecurityTokensOwnedByUser` / `TickersOwnedByUser` | owner indexes | :583/:578 |
| `AssetNonce` | per-account nonce for id generation | :598 |

Portfolio-held balances live in the portfolio pallet (doc 08); `BalanceOf` is the DID-level sum
used by compliance/statistics/checkpoints.

### NFTs (pallets/nft/src/lib.rs)

| Item | Purpose | Ref |
|---|---|---|
| `Collection` / `CollectionAsset` | NFTCollectionId → `NFTCollection { id, asset_id }`; asset → collection | :103/:98; primitives/src/nft.rs:31 |
| `CollectionKeys` | collection → mandatory `AssetMetadataKey` set | :108 |
| `MetadataValue` | ((collection, NFTId), key) → value | :114 |
| `NumberOfNFTs` / `NFTsInCollection` | per-DID count / total supply | :93/:127 |
| `NFTHolder` / `Owner` | account-key-held NFTs (`NFTOwnerStatus`: Owner/OwnerLocked) / reverse owner lookup | :141/:154 |
| `CurrentNFTId` / `CurrentCollectionId` | id sequences (start at 1) | :132/:137 |

## 2. Ticker system

- Validation: chars `A-Z 0-9 _ - . /`, ≤ `TickerConfig.max_ticker_length` (12 at genesis,
  `src/chain_spec/common.rs:128-131`), 60-day registration at genesis
  (`verify_ticker_characters` lib.rs:3088-3111; length lib.rs:3123-3129).
- `register_unique_ticker` (lib.rs:736 → 2033): any permissioned DID. Re-registration matrix
  `can_reregister_ticker` (lib.rs:3132-3161): free renewal of own live ticker; fee-charged
  takeover of unregistered/expired (`AssetRegisterTicker` fee, lib.rs:4033-4035); denial of
  another's live ticker.
- Transfer: `TransferTicker` authorization → `accept_ticker_transfer` (lib.rs:757 → 2057);
  issuer must still own the ticker at acceptance (lib.rs:2067); linked tickers can't transfer
  (lib.rs:2062).
- Link to asset: `link_ticker_to_asset_id` (lib.rs:1610 → 2814) — caller must be *agent AND
  ticker owner*; **linking clears expiry to `None`** (permanent, lib.rs:2837); 1:1 both ways
  (lib.rs:2845-2850). `unlink_ticker_from_asset_id` (lib.rs:1640 → 2858) **deletes the ticker
  registration entirely** (`take`, lib.rs:2867).

## 3. Fungible lifecycle

### Create (`create_asset`, lib.rs:810 → `validate_and_create_asset` 3580)

1. Generate AssetId (§1); validate name ≤ `AssetNameMaxLength` (128), funding-round name ≤ 128,
   custom type exists, identifiers valid (lib.rs:3164-3226).
2. Charge `AssetCreateAsset` protocol fee (lib.rs:4071). (Doc comment at lib.rs:4061 claiming two
   fees is stale — tickers are decoupled from creation.)
3. Insert `AssetDetails { total_supply: 0, owner_did: caller, divisible, asset_type }`
   (lib.rs:4073); **owner becomes `AgentGroup::Full` agent** (lib.rs:4097 →
   `unchecked_add_agent`).
- `create_asset_with_custom_type` (lib.rs:1195) registers the custom type first;
  `register_custom_asset_type` (lib.rs:1159) is idempotent (lib.rs:4249-4274).
- Divisibility: chosen at creation; indivisible assets require amounts in whole multiples of
  `ONE_UNIT = 1_000_000` (`ensure_asset_granular` lib.rs:3281-3286;
  primitives/src/constants.rs:29). One-way upgrade via `make_divisible` (lib.rs:994, agent).
- Ownership transfer: `TransferAssetOwnership` auth → `accept_asset_ownership_transfer`
  (lib.rs:778 → 2075): auth issuer must be permissioned agent at acceptance (lib.rs:2092);
  linked ticker registration moves too (lib.rs:2095-2099); new owner added as Full agent, old
  owner removed as agent (lib.rs:2107-2108).

### Issue / mint (`issue`, lib.rs:926 → `base_issue` 2190, `unverified_issue_tokens` 4113)

- Caller: agent + holding-destination permission (`ensure_asset_and_holding_permissions`
  lib.rs:3234-3272) — portfolio validity + secondary-key portfolio perms; **custody NOT
  required** for issuance destination (lib.rs:2197).
- Rules (lib.rs:3612-3629): fungible only, granularity, `total_supply + amount ≤ MAX_SUPPLY`
  (=10¹² × ONE_UNIT, constants.rs:30).
- Effects, in order: `AssetIssue` fee (lib.rs:4122); **checkpoint pre-update with pre-change
  balance** (`Checkpoint::advance_update_balances`, lib.rs:4127-4131, doc 11); `BalanceOf` +=,
  `total_supply` += (lib.rs:4133-4137); holder balance += (portfolio pallet or `AssetBalance`,
  lib.rs:4140-4143); **statistics update** (from=None, lib.rs:4145-4153, doc 07);
  `IssuedInFundingRound` += (lib.rs:4155). **No compliance check on issuance.**

### Redeem / burn (`redeem`, lib.rs:958 → `base_redeem` 2226)

- Caller: agent + holding permission **with custody** over the source (lib.rs:2233-2234 —
  asymmetric with issue, which doesn't require custody).
- Checks: fungible, sufficient *available* balance (net of locked+frozen,
  `ensure_sufficient_balance` lib.rs:3838-3867).
- Effects: checkpoint pre-update (lib.rs:2252); supply/balances −= (lib.rs:2244-2260);
  statistics update (to=None, lib.rs:2264-2272). No protocol fee, **no compliance check**.

### Freeze layers (three distinct mechanisms)

| Layer | Set by | Blocks | Ref |
|---|---|---|---|
| `Frozen` (asset-wide) | agent `freeze`/`unfreeze` (lib.rs:848/871) | all settlement transfers (`ensure_asset_is_not_frozen` lib.rs:3434, checked lib.rs:3406); **not** issue/redeem; controller transfers exempt | :443 |
| `FrozenBalance` (amount per holder) | agent `set_frozen_tokens` (lib.rs:1840 → 4431-4460) | reduces available balance: available = balance − locked − frozen (lib.rs:3853-3858); controller transfers bypass and *reduce* it (lib.rs:4207-4218) | :645 |
| `FrozenAccounts` (bool per holder) | agent `set_holder_frozen` (lib.rs:1852 → 3026) | holder as **sender** only (`ensure_holder_is_not_frozen` lib.rs:3443-3459) | :657 |

### Documents & funding rounds

- `add_documents` (lib.rs:1017, agent, `AssetAddDocuments` fee per doc lib.rs:2324) /
  `remove_documents` (lib.rs:1041, agent). Sequenced `DocumentId` (lib.rs:2317-2321).
- `set_funding_round` (lib.rs:1068, agent); `rename_asset` (lib.rs:895, agent);
  `update_identifiers` (lib.rs:1095, agent); `update_asset_type` (lib.rs:1397, agent —
  cannot cross fungible↔non-fungible, lib.rs:2571-2574).

## 4. NFT lifecycle (pallets/nft/src/lib.rs)

### Collection creation (`create_nft_collection`, :199 → base :383)

- Existing asset: must exist, be `AssetType::NonFungible`, caller agent (:392-402). **Not
  auto-created in this branch.** With `asset_id = None`: auto-creates the asset (type must be
  `NonFungible(nft_type)`, :407-452; `AssetCreateAsset` fee via asset pallet).
- Collection keys = mandatory metadata attributes for every NFT: ≤ `MaxNumberOfCollectionKeys`
  (u8::MAX in all runtimes, e.g. `pallets/runtime/develop/src/runtime.rs:152`), deduped, each key
  must be a registered metadata type (:419-439).
- `NFTCreateCollection`/`NFTMint` protocol ops exist (primitives/src/protocol_fee.rs:53,55) but
  are **never charged** by the NFT pallet.

### Mint (`issue_nft`, :227 → base :469)

Caller: agent + holding perms (custody not required, :484). Metadata attributes must exactly
match the collection keys (count :489, dedup :495, membership :505). Supply/balance overflow
guards (:513-518). Holder placement: portfolio (via portfolio pallet) or account key
(`NFTHolder`, + `AccountKeyRefCount` strong ref on first NFT, :1004-1008).
**No compliance/statistics on mint.**

### Burn (`redeem_nft`, :258 → base :540)

Caller: agent + holding perms **with custody** (:556). NFT must be held and not locked
(:560-567). Metadata drained (:581); account-key strong ref removed when holdings empty
(:1026-1030).

### NFT transfers — see doc 09 §6 (validation `validate_nft_transfer` :631; **no statistics for
NFTs**, compliance checked :688; per-leg cap `MaxNumberOfNFTsCount` = 10).

## 5. Controller transfer (forced transfer)

- Fungible: `controller_transfer` (lib.rs:1127 → 2375). Caller: agent; destination = caller's
  chosen portfolio/account (perms lib.rs:2383-2384). `validate_asset_transfer(...,
  is_controller_transfer=true)` checks fungibility/balances/holdings/locks then **early-returns
  before frozen/receiver-active/statistics/compliance checks** (lib.rs:3401-3404). Also bypasses
  holder-frozen and frozen-balance limits, decrementing `FrozenBalance` if needed
  (lib.rs:3849-3858, 4207-4218). Statistics *updates* still applied (lib.rs:4224-4232).
- NFT: `controller_transfer` (nft :281 → 799). Ownership/limit checks still apply; compliance and
  frozen checks skipped (:671-674).
- Purpose: regulatory forced recovery — powerful; watch that only agent-group-permissioned
  callers reach it (`PolymeshV1PIA` group includes it; doc 05 §2).

## 6. Asset metadata

- **Global** types: root-only registration/spec-update (`register_asset_metadata_global_type`
  lib.rs:1365 origin-checked at 1371; `update_global_metadata_spec` lib.rs:1665 → 2894).
  **Local** types: agent (`register_asset_metadata_local_type` lib.rs:1338; combined
  register+set lib.rs:1303).
- Values: `set_asset_metadata` (lib.rs:1237, agent) — key must exist, value ≤ 8 KiB
  (`AssetMetadataValueMaxLength`, runtime.rs), not locked (lib.rs:4287-4296).
- Locking: `AssetMetadataValueDetail { expire, lock_status: Unlocked|Locked|LockedUntil(t) }`
  (primitives/src/asset_metadata.rs:74-79, 99-117); set via `set_asset_metadata_details`
  (lib.rs:1269); locking an empty value is forbidden (lib.rs:2492-2495).
- Removal: `remove_local_metadata_key` (lib.rs:1426) — refused if value locked (lib.rs:2594) or
  key is an NFT collection key (lib.rs:2601-2604); `remove_metadata_value` (lib.rs:1454) —
  refused if locked (lib.rs:2638).

## 7. Extrinsic authorization summary

Agent-gated (via `ensure_agent_asset_perms`): freeze/unfreeze, rename, issue, redeem,
make_divisible, documents add/remove, funding round, identifiers, controller_transfer, metadata
set/register-local/details/remove, update_asset_type, mediators add/remove, ticker link/unlink
(+ owner), set_frozen_tokens, set_holder_frozen, checkpoint create (doc 11), compliance config
(doc 06), statistics config (doc 07), NFT collection-on-existing-asset/mint/burn/controller.

Permissionless (any permissioned DID): register_unique_ticker, create_asset(+custom type),
register_custom_asset_type, pre_approve_asset / remove_asset_pre_approval, approve (allowance),
transfer_asset / receiver_affirm_asset_transfer / reject_asset_transfer (doc 09).

Root-only: register_asset_metadata_global_type, update_global_metadata_spec,
exempt_asset_affirmation / remove_asset_affirmation_exemption (lib.rs:2660/2671).

Auth-accepting: accept_ticker_transfer, accept_asset_ownership_transfer.

## 8. Invariants & review checklist

- [ ] Every `BalanceOf` mutation must be preceded by `Checkpoint::advance_update_balances` with
      **pre-change** balances (issue lib.rs:4127, redeem lib.rs:2252, transfer lib.rs:4193) and
      followed by `Statistics::update_asset_stats` — new balance-mutating paths must do both.
- [ ] Account-key holdings must maintain `AccountKeyRefCount` (0↔nonzero transitions,
      lib.rs:3673-3695; NFT :1004/:1026) or keys could be unlinked while holding assets.
- [ ] `total_supply` changes only in issue/redeem; must stay ≤ `MAX_SUPPLY` and consistent with
      Σ balances.
- [ ] Fungibility boundary: fungible entry points reject NFT assets and vice versa
      (lib.rs:3616-3619, 2237-2240; nft collection checks) — check any new entry point.
- [ ] Controller-transfer skip-list (frozen/compliance/stats) must not leak into normal paths —
      the `is_controller_transfer` flag gates it (lib.rs:3401).
- [ ] Ticker link/unlink keeps `TickerAssetId`/`AssetIdTicker` in 1:1 sync.
- [ ] NFT collection keys are immutable-in-practice (removal blocked lib.rs:2601) — metadata
      integrity of issued NFTs depends on it.

## 9. Test map

`pallets/runtime/tests/src/asset_pallet/*` (setup, register_ticker, accept_ticker_transfer,
link/unlink_ticker, asset_ownership_transfer, issue, controller_transfer, allowances,
base_transfer, asset_transfer, register_metadata), `asset_test.rs` (incl. checkpoint fuzz :354),
`asset_metadata_test.rs`, `nft.rs` (collection/mint/burn/transfer/controller),
`external_agents_test.rs`.
