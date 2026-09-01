# 21 — Revive (EVM/ETH Support) & Precompiles

Sources: `pallets/runtime/common/src/runtime.rs` (RTC — config, `EthExtraImpl`),
`pallets/precompiles/` (runtime-side precompile), `precompiles/` (Solidity interface crate),
forked `pallet-revive` (FORK = `substrate/frame/revive` in the pinned polkadot-sdk checkout).
Related specs: [14-fees-and-extensions](14-fees-and-extensions.md) (ETH fee path),
[02-permissions](02-permissions.md) (call-metadata swaps), [09-asset-transfers](09-asset-transfers.md)
(what ERC-20 ops map to).

## 1. Purpose

`pallet-revive` (index 80, all three runtimes) provides EVM & PolkaVM smart contracts plus an
Ethereum-compatible transaction path. Polymesh integrates it with the identity/permission system
via a **dispatch hook** that swaps call metadata, and exposes the runtime to contracts through
three **precompiles**: two asset ones — a fungible (ERC-20/2612/3643/7943 surface) and a
non-fungible (ERC-721/165/7943 surface), each addressed per asset — plus a general-purpose
`IPolymeshRuntime` one at a single fixed address, for extrinsics that are not scoped to an asset.

## 2. Runtime configuration (RTC:485-514)

| Item | Value |
|---|---|
| `AddressMapper` | stock `AccountId32Mapper` (:497): eth address → AccountId32 = 20 bytes + 12×`0xEE` suffix (FORK/src/address.rs:130-135). **No identity linkage in the mapper** — the fallback account must be onboarded to a DID like any account before holding assets |
| `ChainId` | develop 1_641_818 / testnet 1_641_819 / mainnet 1_641_820 (`develop/src/runtime.rs:79` etc.) |
| `NativeToEthRatio` | `10^12` (:505) — POLYX 6 decimals ↔ ETH 18 decimals |
| Deposits | `DepositPerItem` 0.15 POLYX, `DepositPerByte` 0.06 POLYX, lockup 30% (`common/src/lib.rs:97-103`) |
| `UploadOrigin`/`InstantiateOrigin` | `EnsureSigned` (:500-501) — **contract deployment is open to any signed account, not identity-gated** |
| `AllowEVMBytecode` | true (:507); `DebugEnabled` false (:510) |
| **`DispatchHook`** | `pallet_precompiles::common::DispatchWithCallMetadata` (:513) — Polymesh-specific (§4) |
| `Precompiles` | `(FungibleAssetInterface, NonFungibleAssetInterface, PolymeshRuntimeInterface)` (per-runtime, e.g. develop:202-206) |

## 3. The ETH transaction path

- eth-rpc sidecar (external `paritypr/eth-rpc` docker, see AGENTS.md) maps Ethereum JSON-RPC to
  the runtime's `ReviveApi` and submits raw RLP as bare `eth_transact` extrinsics.
- `UncheckedExtrinsic` is revive's EVM wrapper (RTC:1073); decoding is Polymesh-customized in
  `EthExtraImpl::try_into_checked_extrinsic` (RTC:946-1069):
  1. legacy/2930/1559 accepted; 7702/4844 rejected (:978-992);
  2. signer recovered → 0xEE fallback AccountId32 (:994-999);
  3. **subsidy support**: `check_subsidy_conditions(&signer, &call, storage_deposit)` — a
     relayer subsidy can pay for ETH transactions too (:1014-1021, doc 15);
  4. **storage deposit pre-charged** from the fee key into the forked tx-credit pool, threaded
     via `tx_ext.4.set_storage_deposit(...)` (:1023-1037, doc 14 §2). This charge lives in
     `check()`, which also runs during read-only pool validation — safe because each
     `validate_transaction` call mutates a throwaway overlay that is never persisted (sp-api
     contract; executive docs: "Changes made to storage should be discarded"). The deposit is
     charged exactly once, from committed state, at apply time, and `post_dispatch` always
     settles/refunds it (doc 14 §2). Mirrors upstream pallet-revive's default impl verbatim;
  5. no tips (:1060-1062); extension tuple built with `SetOrigin::new_from_eth_transaction()`
     (RTC:926-944).
- `SetOrigin` (upstream, FORK/src/evm/tx_extension.rs:48-99): a runtime-only-settable flag that
  swaps the origin to `Origin::EthTransaction(signer)` — required by `eth_call`/
  `eth_instantiate_with_code`/`eth_substrate_call` and preventing their invocation from plain
  signed extrinsics. It does **not** set identity context.
- Routing by `to` address: `to == RUNTIME_PALLETS_ADDR` (PalletId `py/paddr`,
  FORK/src/lib.rs:2757-2762) ⇒ calldata SCALE-decodes to a `RuntimeCall` wrapped in
  **`eth_substrate_call`** (FORK/src/evm/call.rs:143-158; zero value enforced) — i.e. an ETH
  wallet can dispatch arbitrary Polymesh extrinsics. Otherwise ⇒ contract call / instantiate.
- Fee mapping: `WeightToFee = BlockRatioFee<30_000, 650_000_000>` shared with substrate txs
  (RTC:225-231); eth gas × price splits into weight fee + storage deposit
  (FORK/src/evm/call.rs:199-257). `ReviveApi` reports balances in 18-decimals
  (`evm_balance`, macro at RTC:1102-1106 → FORK/src/lib.rs:2974-3235).

## 4. Identity & permission integration (the critical part)

Two Polymesh mechanisms close the "calls entering via revive carry `Revive.*` call metadata"
hole (doc 02 §3):

1. **`DispatchHook`** (fork delta: `DispatchRuntimeCall` trait + `Config::DispatchHook`,
   FORK/src/lib.rs:148-172, :243, wired into `eth_substrate_call` :1456-1477).
   Polymesh's impl `DispatchWithCallMetadata` (pallets/precompiles/src/common.rs:86-104)
   dispatches the inner call inside `with_call_metadata` (:102) — secondary-key permission
   checks evaluate the **inner** extrinsic.
2. **Precompile dispatch**: `Common::call_runtime` / `with_runtime_call`
   (common.rs:217-244/:250-263) also swap metadata (:227/:262), apply the `BaseCallFilter`
   (:258), meter weight, and convert `DispatchError` into Solidity reverts (:64-74).
   Root callers rejected, delegate-calls rejected, state changes in read-only context rejected
   (:106-159).

Fee-payer redirection (`fee_details`) and the relayer `SubsidyFilter` both **unwrap
`eth_substrate_call`** to inspect the inner call (RTC:195-205, :418-423).

ETH-side accounts must still be onboarded to a DID before doing identity-gated things (the
integration tests onboard the 0xEE fallback account first — `integration/tests/revive_erc20.rs:92-94`).

## 5. The precompiles

Each precompile lives in its own module under `pallets/precompiles/src/interface/`:
`fungible_asset/`, `nft/` and `polymesh/`. All three reject delegate-calls and root callers, and
reject state-changing calls made in a read-only (`STATICCALL`/`eth_call`) context.

### 5.1 The fungible-asset precompile

- Interface crate `precompiles/`: `sol!`-generated bindings + committed stub bytecode
  (`FungibleAssetStub.sol`/`.bin`; regenerate with `scripts/build_precompile_stub.sh`, solc
  0.8.33 exactly). Surface = ERC-20 + ERC20Metadata + EIP-2612 permit + mint/burn +
  ERC-7943 (canTransfer/forcedTransfer/frozen tokens) + ERC-3643 (pause/freeze/naming).
- Runtime side `interface/fungible_asset/mod.rs:51-138`: **address scheme** —
  `asset_id (16 bytes) ‖ zeros ‖ prefix-id 8` (`AddressMatcher::VarPrefix`, :55-58; fork delta
  enabling multi-address precompiles). The trailing bytes are the **precompile selector**: the
  matcher validates them against the registered prefix id *before* any precompile code runs;
  only then does `asset_id_from_address` decode the leading 16 bytes (:142-160). An address
  whose suffix doesn't match never reaches this pallet, so each asset has exactly one valid
  address per precompile interface — the suffix is not an aliasing surface. One precompile
  instance serves *every* fungible asset; decimals fixed at 6 (:46).
  The NFT precompile is the same scheme with **prefix-id 9**
  (`interface/nft/mod.rs:70-73`), so each asset id yields two distinct, non-overlapping
  addresses. `asset_id_from_address` asserts the *opposite* fungibility on each side, so an
  asset is only reachable through the matching interface (`interface/fungible_asset/mod.rs:153`,
  `interface/nft/mod.rs:157`).
- Call mapping (each dispatches the real extrinsic under the caller's account, with metadata
  swap ⇒ full permission/compliance enforcement):

| Solidity | Runtime call | Ref |
|---|---|---|
| `transfer`/`transferFrom` | `Settlement::transfer_funds` (doc 09 §4, incl. allowance spend) | fungible_asset/erc20.rs:66, 215 |
| `approve` | `Asset::approve` | fungible_asset/erc20.rs:166 |
| `mint`/`burn` | `Asset::issue`/`redeem` (agent-gated) | fungible_asset/polymesh_specific.rs:27, 58 |
| ERC-7943 forcedTransfer / setFrozenTokens | `Asset::controller_transfer` / `set_frozen_tokens` | fungible_asset/erc7943.rs:83, 118 |
| ERC-3643 pause/unpause, setName, freeze wallet, ... | `Asset::freeze`/`unfreeze`/`rename_asset`/`set_holder_frozen`/ticker calls | fungible_asset/erc3643.rs:40-136 |

### 5.2 The non-fungible-asset precompile

Stub `NonFungibleAssetStub.sol`/`.bin`, built by the same `scripts/build_precompile_stub.sh`
(which loops over both stubs). ERC-721 `tokenId` == on-chain `NFTId`; one precompile address ==
one NFT collection.

| Solidity | Runtime call / storage | Ref |
|---|---|---|
| `balanceOf` | `Nft::NFTAccountCount` (account-scoped, not per-DID) | nft/erc721.rs:44 |
| `ownerOf` / `totalSupply` | `Nft::Owner` / `Nft::NFTsInCollection` | nft/erc721.rs:60, nft/polymesh_specific.rs:44 |
| `transferFrom` / `safeTransferFrom` | `Settlement::transfer_funds` with `FundDescription::NonFungible` (doc 09 §4, incl. approval spend); the `safe` variants additionally reject receivers with code | nft/erc721.rs:78, 265 |
| `approve` / `setApprovalForAll` | `Nft::approve` / `Nft::set_approval_for_all` (doc 04 §4) | nft/erc721.rs:107, 143 |
| `mint` / `burn` | `Nft::issue_nft` / `Nft::redeem_nft` (agent-gated) | nft/polymesh_specific.rs:62, 116 |
| `tokenURI` | `Nft::MetadataValue[tokenUri]`, falling back to `Asset::AssetMetadataValues[baseTokenUri]`, with `{tokenId}` substitution | nft/metadata.rs:80 |
| `supportsInterface` | constant match on ERC-165 / 721 / 721Metadata ids | nft/metadata.rs:113 |
| ERC-7943 canTransfer / forcedTransfer | `Nft::nft_transfer_report` / `Nft::controller_transfer` | nft/erc7943.rs:37, 73 |

**Deliberate deviations from ERC-721**, documented in the `.sol` NatSpec:

- `safeTransferFrom` **only accepts externally-owned accounts**. A precompile cannot re-enter the
  EVM to invoke the required `onERC721Received` callback, so instead of skipping the check —
  which would silently drop the guarantee the method exists to provide — `ensure_eoa_receiver`
  (nft/erc721.rs:105) rejects any receiver with code, precompiles included. Strictly stronger
  than ERC-721, never weaker: a token cannot be stranded. Contracts that knowingly handle NFTs
  use `transferFrom`. Supporting `onERC721Received` properly is deferred.
- `ownerOf` reverts for portfolio-held NFTs, which have no EVM address.

`redeem_nft` prices itself on `number_of_keys`, defaulting to the worst case of 255 when `None`;
the precompile reads the collection's real key count and passes it, or the upfront weight charge
exceeds any sane EVM gas limit (nft/polymesh_specific.rs:126).

`tokenUri`/`baseTokenUri` global metadata keys are supplied as per-runtime constants
(`TokenUriMetadataKey`, `BaseTokenUriMetadataKey`) rather than looked up by name, since the GC
assigns global key ids per network. `integration/tests/revive_erc721.rs::erc721_token_uri`
asserts the constants still match the chain.

### 5.3 The general-purpose runtime precompile

`IPolymeshRuntime` (`PolymeshRuntime.sol`/`.bin`, `interface/polymesh/mod.rs`) exposes runtime
extrinsics that are **not scoped to a single asset**, so it needs no address data:
`AddressMatcher::Fixed(65_535)` puts it at the one fixed address `0x…FFFF0000` (:42). Every call
it exposes is state-changing, so the whole interface is rejected in a read-only context.

| Solidity | Runtime call | Ref |
|---|---|---|
| `assetCreateAsset` | `Asset::create_asset`; pre-computes the id with `generate_asset_id(caller, false)` so it can return/emit it | polymesh/asset.rs:37 |
| `assetRegisterTicker` | `Asset::register_unique_ticker` | polymesh/asset.rs:89 |
| `identityRegisterDid` | `Identity::register_did` (caller must be an active DID registrar) | polymesh/identity.rs:31 |
| `identitySelfRegisterDid` | `Identity::self_register_did` | polymesh/identity.rs:63 |

`identitySelfRegisterDid` is the notable capability here: a contract cannot sign extrinsics, so
ordinarily it needs a DID registrar to onboard it before it can hold assets. This call lets it
onboard **itself**, under its own contract account, without any registrar — see
`integration/contracts/Onboarder.sol` for the intended pattern (self-register, then create an
asset and register a ticker attributed to the contract, all in one call).

`AssetType`/`AssetIdentifier` cross the ABI boundary as `(enum kind, uint32 customTypeId)` and
`(enum identifierType, bytes value)` pairs, converted in `common.rs:213-255`. The enum orderings
must stay in lockstep with `polymesh_primitives::asset::AssetType` and
`polymesh_primitives::AssetIdentifier`; the identifier `value` lengths are validated on the way
in (CUSIP 9, CINS 9, ISIN 12, LEI 20, FIGI 12).

## 6. Fork deltas (vs upstream stable2603) — exactly four commits

1. **DispatchHook** for `eth_substrate_call` (identity integration point) — FORK/src/lib.rs:148-172.
2. **Precompile improvements**: `AddressMatcher::VarPrefix`, block deployment at precompile
   addresses, custom stub `CODE`.
3. No-account-reaping fix (`exec.rs`).
4. `NativeToEthRatio` as u64 (enables 10^12 for 6-decimal POLYX).

`SetOrigin`, `eth_substrate_call` itself, and `BlockRatioFee` are upstream Parity features;
Polymesh customization of the fee/subsidy path lives in the runtime's `EthExtraImpl` override.

## 7. Invariants & review checklist

- [ ] Every path dispatching runtime calls from revive (hook, precompile, future additions)
      must swap call metadata — regression-tested by `integration/tests/revive_permissions.rs`
      (`erc20_mint_checks_secondary_key_permissions`, `substrate_call_checks_...`).
- [ ] `eth_substrate_call` unwrapping must stay in sync across: fee_details payer matching,
      SubsidyFilter, and the dispatch hook.
- [ ] Precompile address decoding must validate the asset exists & has the right fungibility
      (interface/fungible_asset/mod.rs:142-160, interface/nft/mod.rs:146-164) — collisions with
      contract addresses are prevented by the matcher prefix + deploy-block fork delta, and the
      asset precompiles must keep distinct prefix ids (8 / 9), asserted by
      `pallets/runtime/tests/src/precompiles.rs::precompile_matchers_are_distinct`.
- [ ] Every precompile must set `const CODE` to its own stub blob. The `Precompile` default is
      a bare revert stub shared by every precompile that forgets it, which makes explorers
      attribute one precompile's verified ABI to all of them.
- [ ] Adding a call to a `.sol` interface means updating **three** places that the compiler does
      not check: the read-only guard in that precompile's `call()`, the upfront `env.charge` for
      any storage the handler reads outside the dispatched extrinsic, and this spec's call table.
- [ ] `supportsInterface` must only claim interfaces really implemented with standard
      signatures — asserted by `precompiles.rs::erc165_interface_ids_match_our_selectors`,
      which recomputes each id from the generated selectors.
- [ ] Balance conversions must use `NativeToEthRatio` consistently (18↔6 decimals); dust
      handling via `new_balance_with_dust`.
- [ ] `SetOrigin` flag must remain non-codec (`#[codec(skip)]`) — a user-settable variant would
      let anyone forge eth origins.
- [ ] Tuple index coupling `tx_ext.4` (doc 14 §1) on any TxExtension change.

## 8. Test map

Integration (need node + eth-rpc): `integration/tests/revive_erc20.rs`, `revive_erc3643.rs`,
`revive_erc7943.rs`, `revive_erc721.rs`, `revive_polymesh_runtime.rs`, `revive_onboarder.rs`,
`revive_permissions.rs`, `revive_contracts.rs`, `revive_swap.rs`; helpers
`integration/src/{eth_helper,revive_helper}.rs`; fixtures `integration/contracts/` (regenerate
via `build.sh`; the `.sol` fixtures import the precompile interfaces straight out of
`precompiles/src/interfaces/` rather than keeping a second copy). Unit tests:
`pallets/runtime/tests/src/precompiles.rs`. Fork unit tests: FORK/src/tests/sol.rs:525-614
(dispatch hook, eth origin).
