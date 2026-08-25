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
via a **dispatch hook** that swaps call metadata, and exposes native assets to contracts through
a **fungible-asset precompile** (ERC-20/2612/3643/7943 surface).

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
| `Precompiles` | `(pallet_precompiles::FungibleAssetInterface,)` (per-runtime, e.g. develop:201) |

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
     via `tx_ext.4.set_storage_deposit(...)` (:1023-1037, doc 14 §2);
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

## 5. The fungible-asset precompile

- Interface crate `precompiles/`: `sol!`-generated bindings + committed stub bytecode
  (`FungibleAssetStub.sol`/`.bin`; regenerate with `scripts/build_precompile_stub.sh`, solc
  0.8.33 exactly). Surface = ERC-20 + ERC20Metadata + EIP-2612 permit + mint/burn +
  ERC-7943 (canTransfer/forcedTransfer/frozen tokens) + ERC-3643 (pause/freeze/naming).
- Runtime side `pallets/precompiles/src/interface/mod.rs:49-138`: **address scheme** —
  `asset_id (16 bytes) ‖ zeros ‖ prefix-id 8` (`AddressMatcher::VarPrefix`, :55-58; fork delta
  enabling multi-address precompiles). One precompile instance serves *every* fungible asset;
  decimals fixed at 6 (:46).
- Call mapping (each dispatches the real extrinsic under the caller's account, with metadata
  swap ⇒ full permission/compliance enforcement):

| Solidity | Runtime call | Ref |
|---|---|---|
| `transfer`/`transferFrom` | `Settlement::transfer_funds` (doc 09 §4, incl. allowance spend) | interface/erc20.rs:66, 215 |
| `approve` | `Asset::approve` | erc20.rs:166 |
| `mint`/`burn` | `Asset::issue`/`redeem` (agent-gated) | polymesh_specific.rs:27, 58 |
| ERC-7943 forcedTransfer / setFrozenTokens | `Asset::controller_transfer` / `set_frozen_tokens` | erc7943.rs:83, 118 |
| ERC-3643 pause/unpause, setName, freeze wallet, ... | `Asset::freeze`/`unfreeze`/`rename_asset`/`set_holder_frozen`/ticker calls | erc3643.rs:40-136 |

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
- [ ] Precompile address decoding must validate the asset exists & is fungible
      (interface/mod.rs:142-160) — collisions with contract addresses are prevented by the
      matcher prefix + deploy-block fork delta.
- [ ] Balance conversions must use `NativeToEthRatio` consistently (18↔6 decimals); dust
      handling via `new_balance_with_dust`.
- [ ] `SetOrigin` flag must remain non-codec (`#[codec(skip)]`) — a user-settable variant would
      let anyone forge eth origins.
- [ ] Tuple index coupling `tx_ext.4` (doc 14 §1) on any TxExtension change.

## 8. Test map

Integration (need node + eth-rpc): `integration/tests/revive_erc20.rs`, `revive_erc3643.rs`,
`revive_erc7943.rs`, `revive_permissions.rs`, `revive_contracts.rs`, `revive_swap.rs`; helpers
`integration/src/{eth_helper,revive_helper}.rs`; fixtures `integration/contracts/` (regenerate
via `build.sh`). Fork unit tests: FORK/src/tests/sol.rs:525-614 (dispatch hook, eth origin).
