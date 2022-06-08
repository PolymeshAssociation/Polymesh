Example contracts.

## Preparation

For building the example smart contracts found in this folder you will need to have [`cargo-contract`](https://github.com/paritytech/cargo-contract) installed.

```
cargo install cargo-contract --force
```

## Building a contract

`cargo +nightly contract build --release`

That will produce the following files in `./target/ink/`:

- `<contract-name>.contract` - Combined wasm and metadata.json
- `<contract-name.wasm` - Contract code.
- `metadata.json` - Contract metadata.

