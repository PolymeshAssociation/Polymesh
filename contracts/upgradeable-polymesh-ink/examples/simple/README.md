Example contract that uses the upgradable `polymesh-ink` API.

## Build

Install [`cargo-contract`](https://github.com/paritytech/cargo-contract).
```
cargo install cargo-contract --force
```

Build the contract:
`cargo +nightly contract build --release`

Contract file needed for deployment `./target/ink/test_polymesh_ink.contract`.

## Setup.

Deploy the contract `test_polymesh_ink.contract`.  The account used will be the admin.
(Optional) pass the `new` contractor the address for the upgrade tracker.
