An upgradable wrapper around the Polymesh Runtime API.

This allows contracts to use a stable API that can be updated
to support each major Polymesh release.

The `upgrade_tracker` contract is an optional feature for easier
upgrades.  It allows multiple contracts to use upgradable APIs
without having to have "admin" support in each contract.

## Setup.

1. (Optional) Build and deploy the upgrade tracker contract `./upgrade_tracker/`.
2. Build and upload (don't deploy) the `./polymesh_ink/` contract for testing upgrades.
3. Build and deploy the test contract `./example/`.  Use the code hash from step #2 and the tracker contract address from step #1 if used.

## Usable

The test contract's `system_remark` and `create_asset` calls can be used.

If the `polymesh_ink` contract code was uploaded, then the upgrade can be tested using `update_code_hash` (admin only) or `update_polymesh_ink` (anyone, only works if the tracker is used).
