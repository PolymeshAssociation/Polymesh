# 15 — Relayer (Fee Subsidies) & relay_tx

Sources: `pallets/relayer/src/lib.rs`, `pallets/runtime/common/src/runtime.rs` (SubsidyFilter),
`primitives/src/traits.rs` (SubsidiserTrait), `primitives/src/crypto.rs` (relay_tx signatures).
Related specs: [14-fees-and-extensions](14-fees-and-extensions.md) (how subsidies are consumed).

## 1. Purpose

A **paying key** can subsidise another **user key**'s transaction and protocol fees up to a POLYX
budget. Also hosts `relay_tx`: dispatching a call *as* another account using that account's
off-chain signature.

## 2. Data model & storage (pallets/relayer/src/lib.rs)

- `Subsidy { paying_key, remaining }` (:183-188) — `remaining` is the POLYX budget left.
- `Subsidies`: user_key → `Subsidy` (:212-214) — **one subsidy per user key**; accepting a new
  one replaces the old (:483-489).
- `PendingSubsidies`: (user_key, paying_key) → initial limit (:221-230) — offered, not yet
  accepted.
- `RelayTxNonces`: target account → nonce (:232-235).

## 3. Subsidy lifecycle extrinsics

| Extrinsic (call_index) | Who | Behavior | Ref |
|---|---|---|---|
| `approve_subsidy(0)` | paying key | create/overwrite pending offer (plain storage — **no identity authorization object**) | :245 → :441 |
| `revoke_subsidy(1)` | paying key | cancel pending offer | :259 → :459 |
| `accept_subsidy(2)` | user key | consume pending → active `Subsidy`; **the paying key pays this tx's fee** (fee_details.rs:247-255 via `has_pending_subsidy` :601-603) | :269 → :475 |
| `remove_subsidy(3)` | user key **or** paying key | end active subsidy (`NotAuthorized` otherwise :517-520) | :286 → :509 |
| `update_polyx_limit(4)` / `increase_polyx_limit(5)` / `decrease_polyx_limit(6)` | paying key | Set/Add/Sub `remaining` (checked, `Overflow`) | :305/:325/:345 |
| `relay_tx(7)` | any signed caller | §5 | :366-418 |

## 4. Subsidy consumption (`SubsidiserTrait` impl, :630-721)

Wired as `type Subsidiser = Relayer` for **both** transaction-payment and protocol-fee
(runtime.rs:241/268).

- `check_subsidy(user, fee, call?)` (:635-655): no subsidy ⇒ not subsidised; insufficient
  `remaining` ⇒ `InvalidTransaction::Payment` (a subsidised key with an exhausted budget can't
  fall back to self-paying at the pool level for filtered calls). With `Some(call)`:
  `ensure_subsidy_call` (:610-628) — the call must pass the **`SubsidyFilter`**; calls to the
  `Relayer` pallet itself make the user pay their own fee (so `remove_subsidy` always works);
  any other non-whitelisted call ⇒ `InvalidTransaction::Custom(PalletNotSubsidised)`. With
  `None` (protocol fees): **no filter** (:649-652).
- `reserve_subsidy` (:680-695) / `settle_subsidy` (:697-720) / `debit_subsidy` (:657-678):
  reserve full fee+deposit at prepare; settle (refund unspent + `SubsidyDebited` event) at
  post-dispatch; protocol fees debit directly mid-dispatch (protocol-fee lib.rs:222-236).
  Settle only refunds if the paying key is unchanged (:703-710).

**SubsidyFilter whitelist** (runtime.rs:383-452): Asset, CapitalDistribution, Checkpoint,
ComplianceManager, CorporateAction, CorporateBallot, ExternalAgents, Portfolio, Settlement,
Statistics, Sto, Balances, Identity, Nft, Staking, MultiSig (:402-417). `Revive::
eth_substrate_call` recurses into the inner call (:418-423). `Utility::{batch, batch_all,
force_batch}` allowed non-nested with **≤ 7 inner calls**, each individually filtered
(:386-398, :424-437). Everything else (PIPs, committees, treasury, utility-other, ...) is not
subsidisable.

## 5. relay_tx (:366-418)

Dispatch `call` as `target`, authorized by the target's off-chain signature:
1. Nonce = `RelayTxNonces[target]`, read-and-increment (:385-389).
2. Message = `ChainScopedMessage { genesis_hash, nonce, "Polymesh Relay Transaction",
   expires_at, call }` (crypto.rs:83,92-99); expiry checked at construction (`ExpiredRelayTx`);
   sr25519/ed25519 signature over the `<Bytes>`-wrapped SCALE encoding (`InvalidSignature`,
   :395-398).
3. Dispatch via `pallet_utility::dispatch_call(Signed(target), false, call)` (:402-406) —
   target-origin, call filters apply, **call metadata swapped** so permission checks evaluate
   the inner call against the *target's* key permissions.
4. **Fees are paid by the caller** (the extrinsic signer), not the target (doc :360) — including
   the caller's own subsidy if any. Event `RelayedTx { caller, target, result }`.

Replay protection = genesis hash + per-target nonce + expiry.

## 6. Invariants & review checklist

- [ ] Subsidy budget accounting: reserve/settle/debit must never double-refund; `remaining`
      mutations are checked/saturating and event-logged (`SubsidyDebited`).
- [ ] The filter must stay call-deep: wrappers (utility batches, revive eth calls) must recurse;
      adding a new wrapper pallet requires updating `SubsidyFilter` or subsidised users can
      escape the whitelist.
- [ ] `Relayer`-pallet calls must remain self-paid (`Ok(false)` path :618-620) so users can
      always detach from a subsidy.
- [ ] `relay_tx` must keep nonce-increment *before* signature failure paths it guards, and the
      permission context must be the target's (utility `dispatch_call` handles it).
- [ ] `accept_subsidy` fee redirection: pool-level `has_pending_subsidy` check must match
      dispatch-time behavior or the paying key can be griefed.

## 7. Test map

`pallets/runtime/tests/src/relayer_test.rs` (subsidy lifecycle, tx+protocol fee consumption
:422, batched subsidised calls :540, reserve/settle :689, relay happy/unhappy :797/:842).
Integration: `integration/tests/relayer_negative.rs`, `offchain_signatures.rs:287-325`.
