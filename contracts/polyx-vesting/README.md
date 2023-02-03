Example contract for Polyx vesting.

## Build

Install [`cargo-contract`](https://github.com/paritytech/cargo-contract).
```
cargo install cargo-contract --force
```

Build the contract:
`cargo +nightly contract build --release`

Contract file needed for deployment `./target/ink/polyx_vesting.contract`.

## Deployment and setup.

Needed:

1. Upload and deploy the contract file `polyx_vesting.contract`.
2. For deployment use the `new(beneficiary_address, start_timestamp, duration_seconds)` contructor with the beneficiary address, start and duration times.
