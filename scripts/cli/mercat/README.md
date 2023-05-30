# WASM bindings for MERCAT Library


This library provides WASM binding for MERCAT. The Rust code can be
found at [Polymesh mercat library][mercat-rust-lib] and
the source code for the wasm bindings can be found at
[WASM bindings][wasm-src].


## Build Instructions

For comprehensive build instructions, refer to the README.md file in the
root of the [repository][cryptography-rust-lib].

If you have all the necessary tools installed, you can build the wasm
bindings using the following commands.

```bash
# If your active toolchain is stable, then run
rustup run nightly wasm-pack build --release

# If your active toolchain is nightly, then you can use the simpler version and run
wasm-pack build --release
```

This will create the bindings in `./pkg/` directory. You can import
these into any javascript-based project using a wasm-loader.

## Publish

Note that the name in the `package.json` file will be "mercat".
But, in order to properly publish the package, the name should be changed to
`@polymeshassociation/mercat`.


[cryptography-rust-lib]: https://github.com/PolymeshAssociation/cryptography/tree/develop/README.md
[mercat-rust-lib]: https://github.com/PolymeshAssociation/cryptography/tree/develop/mercat
[mercat-wasm-src]: https://github.com/PolymeshAssociation/cryptography/blob/develop/mercat/wasm/src/lib.rs 
