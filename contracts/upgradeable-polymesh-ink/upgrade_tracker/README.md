Upgrade tracker contract for easier upgrading of APIs.

## Build

Install [`cargo-contract`](https://github.com/paritytech/cargo-contract).
```
cargo install cargo-contract --force
```

Build the contract:
`cargo +nightly contract build --release`

Contract file needed for deployment `./target/ink/polymesh_ink_upgrade_tracker.contract`.

## Setup.

Deploy the contract `polymesh_ink_upgrade_tracker.contract`.  The account used will be the admin.

## Use

Call the `upgrade_wrapped_api(<api version>, Upgrade { chain_version: ChainVersion { spec: 5_002_001, tx: 3}, hash: <code hash of upgraded api>})`
to upgrade an API.
