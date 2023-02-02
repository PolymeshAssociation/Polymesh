An upgradable wrapper around the Polymesh Runtime API.

This allows contracts to use a stable API that can be updated
to support each major Polymesh release.

The `upgrade_tracker` contract is an optional feature for easier
upgrades.  It allows multiple contracts to use upgradable APIs
without having to have "admin" support in each contract.

## TODO

1. Add more useful APIs.
2. Add namespacing of the versioned APIs to allow suport multiple upgradable APIs (Asset, Portfolio, Settlement, etc..).
3. Support custom "Release" version in the tracker.  Right now the upgrade tracker uses the chain version (spec, tx) to trigger upgrades.  But with namescaped APIs, the tracker could be use to sync multiple contract upgrades.
4. Add events and errors.
5. Publish crates `polymesh-ink` and `upgrade_tracker`.

## Setup.

1. (Optional) Build and deploy the upgrade tracker contract `./upgrade_tracker/`.
2. Build and deploy the test contract `./example/`.  Use the contract address from step #1 if used.
3. (Optional) Build and upload (don't deploy) the `./runtime_v5/` contract for testing upgrades.

## Usable

The test contract's `system_remark` and `create_asset` calls can be used.

If the `runtime_v5` contract code was uploaded, then the upgrade can be tested using `update_code_hash` (admin only) or `update_polymesh_ink` (anyone, only works if the tracker is used).
