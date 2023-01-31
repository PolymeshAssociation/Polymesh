Wrap Polymesh v5.x runtime API.

## Build

Install [`cargo-contract`](https://github.com/paritytech/cargo-contract).
```
cargo install cargo-contract --force
```

Build the contract:
`cargo +nightly contract build --release`

Contract file needed for upload `./target/ink/runtime_v5.contract`.

## Setup.

1. Upload contract file `runtime_v5.contract`.  Don't deploy.
