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
3. Enter the Polyx amount that would be available for the contract.

Top up contract by sending funds to the contract address.

## Usage

1. Call `release()` function to release Polyx.
2. Call `releasable()` function to show the amount of releasable Polyx.
3. Call `vested_amount(timestamp)` function to calculates the amount of Polyx that has already vested.
4. Call `released()` function to return the amount of Polyx already released.
5. Call `start()` function to return the start timestamp.
6. Call `duration()` function to return the vesting duration.
7. Call `beneficiary()` function to return the beneficiary address.

## Test

Test the contract:
`cargo +nightly contract test`