# 17 — Utility (Batching & Call Wrappers)

Sources: `pallets/utility/src/lib.rs` (Substrate fork "+ permissions checks").
Related specs: [02-permissions](02-permissions.md) §3 (nested-call metadata),
[14-fees-and-extensions](14-fees-and-extensions.md) (payer context), [15-relayer](15-relayer.md)
(`relay_tx` lives in Relayer in v8; subsidised batch limits).

## 1. Purpose

Standard utility batching, forked to preserve Polymesh's permission model: every inner call is
dispatched under `with_call_metadata`, so secondary-key extrinsic permissions and agent-group
checks evaluate **each inner call individually**, not the outer `utility.*` wrapper.

## 2. Extrinsics

| Extrinsic (call_index) | Origin | Semantics | Ref |
|---|---|---|---|
| `batch(0)` | any except None (:224-226) | stop on first error but return `Ok`; events `BatchInterrupted { index, error }` / `ItemCompleted` / `BatchCompleted` | :213-259 |
| `batch_all(2)` | any except None | **atomic** — first error rolls back the whole batch (:317-324); nested `batch_all` filtered for non-root (:300-307) | :274-330 |
| `force_batch(4)` | any except None | continue on error; `ItemFailed`; `BatchCompletedWithErrors` or `BatchCompleted` | :368-414 |
| `dispatch_as(3)` | **root** (:522) | dispatch under arbitrary origin, bypasses filters; **temporary fee-payer = the as-origin account** (:526-534) | :338 → :517-544 |
| `with_weight(5)` | **root** (:429) | dispatch with caller-declared weight | :422-436 |
| `as_derivative(9)` | signed | pseudonym account = `blake2_256("modlpy/utilisuba", who, index)` (:578-585); dispatches as Signed(derivative); **payer = derivative account** (:556-562) | :444 → :546-575 |

Batch size cap: `batched_calls_limit` extra-constant (:157-171, `TooManyCalls`). Root callers
bypass call filters inside batches (:228, :239-240). Call indices 1/6/7/8 are gaps — `relay_tx`,
`UniqueCall`, `batch_old`/`batch_atomic`/`batch_optimistic` were removed (relay_tx now in
Relayer, doc 15 §5; the old variants survive only in `previous_release` integration tests).

The pallet is stateless (no storage, :106-107).

## 3. Permission & payer semantics for nested calls

- `dispatch_call` (:489-502) wraps every inner dispatch in `with_call_metadata` (:494); so does
  `run_with_temporary_payer` (:589-612 at :601) used by `dispatch_as`/`as_derivative`. Multisig
  proposal execution uses the same wrapper (multisig lib.rs:1125).
- Consequence: a secondary key with permission for `utility.batch` but not `asset.issue` cannot
  smuggle an `asset.issue` inside a batch — the inner check fails that item (batch: interrupt;
  batch_all: rollback; force_batch: item failure).
- `run_with_temporary_payer` swaps `CurrentPayer` around the inner dispatch, so protocol fees
  charged by inner calls hit the derivative/as-origin account (doc 14 §2).
- Subsidised users: relayer's `SubsidyFilter` allows non-nested `batch`/`batch_all`/`force_batch`
  with ≤ 7 inner calls, each individually whitelisted (runtime.rs:386-398, doc 15 §4).

## 4. Invariants & review checklist

- [ ] Every dispatch path in this pallet must wrap inner calls in `with_call_metadata` — new
      wrapper extrinsics too.
- [ ] `batch_all` nesting restriction for non-root must stay (recursion/filter-bypass guard).
- [ ] `dispatch_as`/`with_weight` must remain root-only; `as_derivative` payer swap must restore
      the previous payer (RAII pattern in `run_with_temporary_payer`).
- [ ] `batch` returns `Ok` even when interrupted — callers/tests must check events, not just
      dispatch success.

## 5. Test map

`pallets/runtime/tests/src/utility_test.rs` (26 tests: early exit :95, secondary-key
permissions :207, batch_all revert/nesting :498/:605, size limit :657, force_batch :673,
committee origins :779-847, with_weight :881, as_derivative :908).
