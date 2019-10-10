# polymesh-count-transfer-manager

Count Transfer Manager implemented in Ink! smart contract.

Designed to be attached to the Asset module to allow the total number of investors to be capped.

# Pre-Requisites

  - Rust (nightly and stable)
  - Contract Crate
  `cargo install --force --git https://github.com/paritytech/ink cargo-contract`

# To build WASM

```
cargo contract build
```

# To build ABI

```
cargo +nightly build --features ink-generate-abi
```