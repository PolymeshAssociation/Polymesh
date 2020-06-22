# polymesh-general-transfer-manager

General Transfer Manager implemented in Ink! smart contract.

Designed to be attached to a regulated asset to allow transfers of the asset to be controlled through an exemption list.

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