# 14 — Transaction Extensions & Fee Payment

Sources: `pallets/transaction-payment/src/lib.rs` (polymesh-transaction-payment),
`pallets/protocol-fee/src/lib.rs`, `pallets/runtime/common/src/{runtime.rs,fee_details.rs,impls.rs,lib.rs}`,
`primitives/src/{transaction_payment.rs,protocol_fee.rs,traits.rs}`.
Related specs: [02-permissions](02-permissions.md) (StoreCallMetadata),
[15-relayer](15-relayer.md) (subsidies), [16-multisig](16-multisig.md) (fee redirection),
[21-revive-evm](21-revive-evm.md) (ETH path).
Note: the base `pallet-transaction-payment` is the **Polymesh fork** of polkadot-sdk — helpers
like `get_priority`, `remaining_txfee`, `deposit_txfee`, `ChargeFeesControl` come from there.

## 1. The TxExtension tuple (pallets/runtime/common/src/runtime.rs:901-917)

| # | Extension | Role |
|---|---|---|
| 0 | `AuthorizeCall`, `CheckNonZeroSender`, `CheckSpecVersion`, `CheckTxVersion`, `CheckGenesis` (:903-907) | standard validity/implicit payload |
| 1 | `CheckEra` (:909) | mortality |
| 2 | `CheckNonce` (:910) | nonce |
| 3 | `CheckWeight` (:911) | block limits — before money moves |
| 4 | **`polymesh_transaction_payment::ChargeTransactionPayment`** (:912) | fee logic (§2) — **tuple index 4 is hard-coded** by the ETH path (`tx_ext.4.set_storage_deposit`, runtime.rs:1037) |
| 5 | `pallet_permissions::StoreCallMetadata` (:913) | records pallet/extrinsic for permission checks (doc 02 §3) |
| 6 | `CheckMetadataHash` (:914) | disabled mode (`new(false)`) |
| 7 | `pallet_revive::evm::tx_extension::SetOrigin` (:915) | ETH-derived origin marking (doc 21); `new_from_eth_transaction()` on the ETH path (runtime.rs:941) |
| 8 | `WeightReclaim` (:916) | refunds over-estimated extension weight (last) |

Same order used for offchain-signed (runtime.rs:678-694), authorized (:726-743) and ETH
(`EthExtraImpl::get_eth_extension` :926-944) construction. Test replica:
`pallets/runtime/tests/src/signed_extra.rs:32-50`.

## 2. ChargeTransactionPayment lifecycle (pallets/transaction-payment/src/lib.rs)

Struct `{ tip (compact), storage_deposit (codec-skipped, ETH-only) }` (:161-177).

1. **validate** (:414-457): unsigned/root ⇒ `NoCharge` (:431-433). Else:
   - `ensure_valid_tip` (:324-345): **Normal class ⇒ tip must be 0**; Operational ⇒ tip allowed
     **only for Governance Committee members** (`is_gc_member` :314-318); violation ⇒
     `InvalidTransaction::Custom(ZeroTip)`.
   - `can_withdraw_fee` (:207-241): compute fee (base+len+weight+tip via forked base pallet,
     `Pays::No` ⇒ 0); resolve payer via `CurrentFeePayer::call_payment_info` (§3); check subsidy
     (§5); dry-run withdrawability; **set `CurrentPayer` context** (:236).
   - priority via forked `get_priority`.
2. **prepare** (:459-501): actually `withdraw_fee` (:243-290) from subsidiser-or-payer; set payer
   context again; if subsidised, **reserve** the full `fee_with_tip + storage_deposit` from the
   subsidy budget (:481-489) so protocol fees charged mid-dispatch can't exhaust it.
3. **post_dispatch** (:503-572): `CurrentPayer::take()` first (:510); on failed dispatch,
   decrement the authorization retry count (:533-536, doc 01 §5); compute actual fee, settle
   subsidy (refund unspent, `SubsidyDebited` event, :541-559); refund payer difference and route
   the fee via `DealWithFees` (:565-567); emit fee-paid event.

`CallPaymentInfo { paying_account, auth_id, ms_signatory }`
(primitives/src/transaction_payment.rs:5-42). `CurrentPayer` storage (:91-93) is readable during
dispatch — consumed by protocol fees (§4) and temporarily overridden by utility's
`dispatch_as`/`as_derivative` (`run_with_temporary_payer`, pallets/utility/src/lib.rs:587-612).

Dev/CI chains can disable fees entirely (`disable_fees` feature; storage `DisableFees` :87-89,
root-only `set_disable_fees` :121-131).

## 3. Payer resolution (`TxFeeHandler`, pallets/runtime/common/src/fee_details.rs)

Default: caller pays (:285). Special-cased calls (matched even when wrapped in
`Revive::eth_substrate_call`, runtime.rs:195-205):

| Call | Payer |
|---|---|
| `Identity::join_identity_as_key`, `accept_primary_key`, `rotate_primary_key_to_secondary` | **auth issuer's DID primary key** (:189-220) |
| `Identity::remove_authorization { auth_issuer_pays: true }` (target = caller) | auth issuer's primary key (:221-241) |
| `MultiSig::accept_multisig_signer` | `AddMultiSigSigner` auth issuer's primary key (:133-140) |
| `MultiSig::approve_join_identity` | JoinIdentity auth issuer's primary key; AlreadyVoted pre-check (:141-159) |
| `MultiSig::approve` / `reject` / `create_proposal` | multisig's `PayingDid` primary key, else the multisig account (:160-183, `get_multisig_payer` :92-111); duplicate votes rejected at pool level (`AlreadyVoted`) |
| `Relayer::accept_subsidy` (pending subsidy exists) | the prospective **paying key** (:247-255) |

Auth-based redirection requires a valid, unexpired auth with retries left
(`get_non_expired_auth`); failing dispatches burn a retry (doc 01 §5).

## 4. Protocol fees (pallets/protocol-fee/src/lib.rs)

- Storage: `BaseFees` (ProtocolOp → Balance, :103) and `Coefficient` (`PosRatio`, :107).
  fee = coefficient × base (:185-194). Genesis: only `AssetCreateAsset` = 2,500 POLYX and
  `AssetRegisterTicker` = 500 POLYX are non-zero (src/chain_spec/common.rs:217-227).
- Governance: `change_coefficient` / `change_base_fee` are **root-only** (:150-181, events
  attributed to `GC_DID`).
- Charging (`withdraw_from_payer` :252-258): the payer is **whoever `CurrentPayer` says** — i.e.
  the transaction-fee payer, including redirected payers and subsidisers. If no payer context
  (root/unsigned), the protocol fee is silently skipped. Subsidy consulted with `call=None` ⇒
  **no pallet-filter restriction for protocol fees** (:222-236; relayer lib.rs:649-652).
- Fee sink: `OnProtocolFeePayment = DealWithFees` (runtime.rs:266).
- `ProtocolOp` list (primitives/src/protocol_fee.rs:25-56): AssetRegisterTicker, AssetIssue,
  AssetAddDocuments, AssetCreateAsset, CheckpointCreateSchedule,
  ComplianceManagerAddComplianceRequirement, IdentityRegisterDid, IdentityAddClaim,
  IdentityAddSecondaryKeysWithAuthorization, PipsPropose, ContractsPutCode,
  CorporateBallotAttachBallot, CapitalDistributionDistribute, NFTCreateCollection, NFTMint
  (last two never charged — doc 04 §4).
- RPC: `protocolFee_computeFee` (pallets/protocol-fee/rpc/src/lib.rs:30-32).

## 5. Where fees go & fee sizing

- **100% of tx fees + tips + protocol fees go to the block author**: `DealWithFees =
  Author<Runtime>` (mainnet runtime.rs:43-44; impls.rs:42-53). No treasury split.
- Sizing (pallets/runtime/common/src/lib.rs): `TransactionByteFee` = 0.0001 POLYX/byte (:81),
  target base fee 3 CENTS per `ExtrinsicBaseWeight` = 650µs (:79-88); `WeightToFee` is
  revive's `BlockRatioFee<30_000, 650_000_000>` (runtime.rs:231, same ratio);
  `FeeMultiplierUpdate = ConstFeeMultiplier(1)` — **no congestion-based fee adjustment**
  (:233). `OperationalFeeMultiplier = 5`.
- ETH transactions: storage deposit is withdrawn up-front into the forked pallet's tx credit
  pool and threaded through `storage_deposit` (runtime.rs:1014-1037); no tips (:1060-1062).

## 6. Invariants & review checklist

- [ ] Tuple order: CheckWeight before ChargeTransactionPayment; StoreCallMetadata after fees;
      `tx_ext.4` index coupling with the ETH path (runtime.rs:1037) — reordering breaks revive.
- [ ] `CurrentPayer` must be set on every charged path and taken exactly once in post_dispatch;
      protocol fees depend on it — a path that charges protocol fees outside a signed
      transaction context charges nobody.
- [ ] Subsidy reserve/settle must bracket dispatch: reserve in prepare (:481-489), settle in
      post_dispatch (:541-559); protocol fees debit the same budget in between.
- [ ] Payer redirection must validate the auth *type* matches the call (fee_details
      `get_payers_account` :49-89) — otherwise anyone could drain an issuer via unrelated auths.
- [ ] Tip policy (Normal ⇒ 0; Operational ⇒ GC only) is consensus-critical for fair ordering.
- [ ] `Pays::No`/zero-fee short-circuits must keep skipping payer/subsidy machinery
      (:220-222, :263-266).

## 7. Test map

`pallets/runtime/tests/src/transaction_payment_test.rs` (lifecycle, refunds, tipping GC rules,
duplicate-vote rejection :775, auth-count decrement :854), `signed_extra.rs` (full-runtime
extension ordering & priorities), `fee_details.rs` (payer resolution matrix),
`protocol_fee.rs` (compute/batch).
