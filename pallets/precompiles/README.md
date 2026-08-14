# pallet-precompiles

Polymesh precompiles for `pallet-revive`: Solidity-callable entry points that map ERC-20 /
ERC-7943 calls onto Polymesh runtime calls.

- `src/lib.rs` — the crate's [`Config`] trait, implemented by every runtime.
- `src/common.rs` — [`Common<T>`], the helpers shared by all precompiles.
- `src/interface/` — the `FungibleAssetInterface` precompile (`erc20.rs`, `erc7943.rs`,
  `polymesh_specific.rs`), one module per ABI group.

The Solidity ABI types (`IFungibleAsset`, `IFungibleAssetEvents`, `FUNGIBLE_ASSET_CODE`) come from
the `polymesh-precompiles-api` crate.

## Design

### Everything goes through a runtime call

Polymesh checks secondary key permissions with `pallet_permissions`, which reads the *current call
metadata* (`CurrentPalletName` / `CurrentDispatchableName`) that the `StoreCallMetadata` transaction
extension set for the outer extrinsic. Inside a precompile that metadata is `Revive.call` /
`Revive.eth_transact`, so calling a pallet function directly
(`pallet_asset::Pallet::<T>::issue(...)`) would check the caller's permissions against the wrong
extrinsic.

Precompiles therefore never call pallet functions directly. They build a runtime call and hand it to
[`Common::call_runtime`], which dispatches it inside `pallet_permissions::with_call_metadata`. This
also means the runtime's `BaseCallFilter` is applied, and the declared weight is charged and
refunded automatically.

### The `Config` trait

```rust
pub trait Config:
    pallet_revive::Config
    + pallet_permissions::Config
    + pallet_asset::Config
    + pallet_asset::checkpoint::Config
    + pallet_settlement::Config
{
    type RuntimeCall: Dispatchable<RuntimeOrigin = OriginFor<Self>, PostInfo = PostDispatchInfo>
        + GetDispatchInfo
        + GetCallMetadata
        + IsType<<Self as frame_system::Config>::RuntimeCall>
        + From<pallet_asset::Call<Self>>
        + From<pallet_settlement::Call<Self>>;
}
```

It gives the precompiles the aggregated runtime call and lets every precompile bound itself with a
single `T: crate::Config`. Each runtime implements it next to its `Precompiles` tuple:

```rust
impl pallet_precompiles::Config for Runtime {
    type RuntimeCall = RuntimeCall;
}
```

Add a `From<pallet_x::Call<Self>>` bound when a precompile needs to dispatch calls of a pallet that
isn't listed yet. Use the `CallOf<T>` alias in code: `RuntimeCall` is also an associated type of
`pallet_revive::Config`, so the bare name is ambiguous.

## Using the helpers

All helpers live on `Common<T>` (`use crate::common::Common;`), except `revert` and
`extrinsic_error`, which are free functions.

### Dispatching a runtime call

```rust
let caller = Common::<T>::caller(env)?;
let amount = Common::<T>::to_balance(call.value)?;

Common::<T>::call_runtime(
    env,
    caller.runtime_origin(),
    pallet_asset::Call::<T>::issue {
        asset_id,
        amount,
        asset_holder_kind: AssetHolderKind::Account,
    },
)?;
```

`call_runtime` charges `DispatchInfo::call_weight` plus the cost of swapping the call metadata,
dispatches with the call's own metadata, refunds the difference against `PostDispatchInfo`, and
converts a `DispatchError` into a revert carrying the module error message. **Don't call
`env.charge(...)` for the extrinsic weight yourself** — that would charge it twice.

### Dispatching something that isn't an extrinsic

Some paths need the return value of an internal function rather than the extrinsic's
`PostDispatchInfo` (for example `SettlementFn::transfer_funds`, which needs a `WeightMeter` and
returns an `InstructionId`). Build the equivalent runtime call anyway and wrap the work in
[`Common::with_runtime_call`]: it checks the `BaseCallFilter` and sets the call metadata, but leaves
the weight accounting to the closure.

```rust
let worst_case = <T as pallet_asset::Config>::SettlementFn::transfer_funds_weight_limit(None, &fund);
let charged = env.charge(worst_case)?;
let mut weight_meter = WeightMeter::from_limit_unchecked(Weight::zero(), worst_case);

let result = Common::<T>::with_runtime_call(
    env,
    pallet_settlement::Call::<T>::transfer_funds {
        from: None,
        to: to.clone(),
        fund: fund.clone(),
    },
    || {
        <T as pallet_asset::Config>::SettlementFn::transfer_funds(
            caller.runtime_origin(),
            None,
            to,
            fund,
            &mut weight_meter,
        )
    },
)?;

env.adjust_gas(charged, weight_meter.consumed());
```

### The caller

`Common::<T>::caller(env)` returns a `Caller<T>` with all three representations, so nothing has to
round-trip an address through the `AddressMapper`:

| Field / method     | Use                                                     |
| ------------------ | ------------------------------------------------------- |
| `origin`           | The `pallet_revive` origin (`ExecOrigin<T>`).            |
| `account_id`       | The substrate account, e.g. for storage lookups.         |
| `address`          | The `H160` to put in emitted events.                     |
| `runtime_origin()` | The `OriginFor<T>` to dispatch runtime calls with.       |

A root caller is rejected with `ERR_INVALID_CALLER`.

### Conversions

| Helper                          | Converts                                                        |
| ------------------------------- | --------------------------------------------------------------- |
| `account_id(Address)`           | ABI address → `T::AccountId`.                                    |
| `account_id32(Address)`         | ABI address → `polymesh_primitives::AccountId` (storage keys).   |
| `asset_holder(Address)`         | ABI address → `AssetHolder`.                                     |
| `account_holder(&T::AccountId)` | Account → `AssetHolder`.                                         |
| `to_balance(U256)`              | ABI value → `Balance`, reverting on overflow.                    |
| `to_u256(Balance)`              | `Balance` → ABI value.                                           |

### Events

`Common::<T>::deposit_event(env, event)` accepts any `alloy` event (`IntoLogData`), charges
`RuntimeCosts::DepositEvent` and emits it as a contract log.

### Errors

- `revert(reason)` — a plain Solidity `revert("...")`, catchable by `try`/`catch`.
- `revert_err(err, reason)` — the same, but logs `err` first; use it instead of
  `.map_err(|_| revert(reason))` so the original error isn't lost.
- `extrinsic_error(err)` — a revert that includes the dispatch error's module message.
- `Common::<T>::state_change_denied()` — for state-changing calls in a read-only (`eth_call`)
  context.
- `Common::<T>::ensure_direct_call(env)?` — reject delegate calls; call it first in `Precompile::call`.

## Adding a precompile

1. Add the ABI to `precompiles-api` and a module under `src/`.
2. Declare `pub struct MyInterface<T>(PhantomData<T>);` and `impl<T: Config> Precompile for MyInterface<T>`
   with a unique `MATCHER` id.
3. Start `call()` with `Common::<T>::ensure_direct_call(env)?` and reject state-changing calls when
   `env.is_read_only()`.
4. Charge for the reads you do (`env.charge(T::DbWeight::get().reads(n))`) and route every
   state change through `call_runtime` / `with_runtime_call`.
5. Register it in the `Precompiles` tuple of each runtime, and extend `Config::RuntimeCall`'s `From`
   bounds if it dispatches calls of a new pallet.

## Building and testing

`cargo check -p pallet-precompiles` on its own fails on an unrelated feature-unification problem in
the dependency graph. Check through a runtime instead:

```sh
SKIP_WASM_BUILD=1 cargo check -p polymesh-runtime-develop
```

The precompiles are covered by the integration tests (`integration/tests/revive_erc20.rs`,
`revive_erc7943.rs`, `revive_permissions.rs`), which need a running dev node and `eth-rpc`.
