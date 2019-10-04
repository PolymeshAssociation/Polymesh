# polymesh-simple-sto

Simple STO implemented in Ink! smart contract.

Designed to be attached to the Settlement module to primary offerings of an asset.

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