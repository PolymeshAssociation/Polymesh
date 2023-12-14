An upgradable wrapper around the Polymesh Runtime API.

This allows contracts to use a stable API that can be updated
to support each major Polymesh release.

## Build the wrapped API contract.

Install [`cargo-contract`](https://github.com/paritytech/cargo-contract).
```
cargo install cargo-contract --force
```

Build the contract:
`cargo contract build --release`

Contract file needed for upload `./target/ink/polymesh_ink.contract`.

## Example contract that uses the Polymesh Ink! API

Cargo.toml:
```toml
[package]
name = "example_contract"
version = "1.0.0"
authors = ["<author>"]
edition = "2021"
publish = false

[dependencies]
ink = { version = "4.3", default-features = false }

scale = { package = "parity-scale-codec", version = "3", default-features = false, features = ["derive"] }
scale-info = { version = "2", default-features = false, features = ["derive"], optional = true }

polymesh-ink = { version = "3.0", default-features = false, features = ["as-library"] }

[lib]
path = "lib.rs"

[features]
default = ["std"]
std = [
    "ink/std",
    "scale/std",
    "scale-info/std",
    "polymesh-ink/std",
]
ink-as-dependency = []
```

lib.rs:
```rust
//! Example contract for upgradable `polymesh-ink` API.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;

use polymesh_ink::*;

#[ink::contract(env = PolymeshEnvironment)]
pub mod example_contract {
    use crate::*;
    use alloc::vec::Vec;

    /// Exchange contract using the Polymesh Ink! API.
    #[ink(storage)]
    pub struct ExampleContract {
        /// Upgradable Polymesh Ink API.
        api: PolymeshInk,
    }

    /// The contract error types.
    #[derive(Debug, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// PolymeshInk errors.
        PolymeshInk(PolymeshError),
    }

    impl From<PolymeshError> for Error {
        fn from(err: PolymeshError) -> Self {
            Self::PolymeshInk(err)
        }
    }

    /// The contract result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl ExampleContract {
        /// Instantiate this contract with an address of the `logic` contract.
        ///
        /// Sets the privileged account to the caller. Only this account may
        /// later changed the `forward_to` address.
        #[ink(constructor)]
        pub fn new() -> Result<Self> {
            Ok(Self {
                api: PolymeshInk::new()?,
            })
        }

        /// Update the `polymesh-ink` API using the tracker.
        ///
        /// Anyone can pay the gas fees to do the update using the tracker.
        #[ink(message)]
        pub fn update_polymesh_ink(&mut self) -> Result<()> {
            self.api.check_for_upgrade()?;
            Ok(())
        }

				// Simple example of using the Polymesh Ink! API.
        #[ink(message)]
        pub fn create_asset(&mut self, name: Vec<u8>, ticker: Ticker, amount: Balance) -> Result<()> {
            self.api
                .asset_create_and_issue(AssetName(name), ticker, AssetType::EquityCommon, true, Some(amount))?;
            Ok(())
        }
    }
}
```

## Setup.

1. Build and upload (don't deploy) the wrapped API contract `./target/ink/polymesh_ink.contract`.
2. Build and deploy an example contract from `./examples/`.

## Usable

The `update_polymesh_ink` calls can be used to update the code hash for the Polymesh Ink! API.
