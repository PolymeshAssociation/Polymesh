An upgradable wrapper around the Polymesh Runtime API.

This allows contracts to use a stable API that can be updated
to support each major Polymesh release.

The `./upgrade_tracker` contract is an optional feature for easier
upgrades.  It allows multiple contracts to use upgradable APIs
without having to have "admin" support in each contract.

## Build the wrapped API contract.

Install [`cargo-contract`](https://github.com/paritytech/cargo-contract).
```
cargo install cargo-contract --force
```

Build the contract:
`cargo +nightly contract build --release`

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
ink_primitives = { version = "3.0", default-features = false }
ink_prelude = { version = "3.0", default-features = false }
ink_metadata = { version = "3.0", default-features = false, features = ["derive"], optional = true }
ink_env = { version = "3.0", default-features = false }
ink_storage = { version = "3.0", default-features = false }
ink_lang = { version = "3.0", default-features = false }
ink_lang_codegen = { version = "3.0", default-features = false }

scale = { package = "parity-scale-codec", version = "3", default-features = false, features = ["derive"] }
scale-info = { version = "2", default-features = false, features = ["derive"], optional = true }

polymesh-ink = { version = "0.5.1", path = "../../", default-features = false, features = ["as-library", "tracker", "always-delegate"] }

[lib]
name = "example_contract"
path = "lib.rs"
crate-type = ["cdylib"]

[features]
default = ["std"]
std = [
    "ink_primitives/std",
    "ink_metadata/std",
    "ink_env/std",
    "ink_storage/std",
    "ink_lang/std",
    "scale/std",
    "scale-info/std",
    "polymesh-ink/std",
]
ink-as-dependency = []
```

lib.rs:
```rust
//! Example contract for upgradable `polymesh-ink` API.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use ink_lang as ink;

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
        /// Upgrade error.
        UpgradeError(UpgradeError)
    }

    impl From<PolymeshError> for Error {
        fn from(err: PolymeshError) -> Self {
            Self::PolymeshInk(err)
        }
    }

    impl From<UpgradeError> for Error {
        fn from(err: UpgradeError) -> Self {
            Self::UpgradeError(err)
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
        pub fn new(tracker: UpgradeTrackerRef) -> Self {
            Self {
                api: PolymeshInk::new_tracker(tracker),
            }
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

1. (Optional) Build and deploy the upgrade tracker contract `./upgrade_tracker/`.
2. Build and upload (don't deploy) the wrapped API contract `./target/ink/polymesh_ink.contract`.
3. Build and deploy an example contract from `./examples/`.  Use the code hash from step #2 and the tracker contract address from step #1 if used.

## Usable

The `update_polymesh_ink` or `update_code_hash` calls can be used to update the code hash for the Polymesh Ink! API.
