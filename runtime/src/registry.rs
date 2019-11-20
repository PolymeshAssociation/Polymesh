//! # Registry Module - WIP
//!
//! The Registry module provides functionality for tracking asset issuers globally
//!
//! ## Overview
//!
//! The Registry module provides functions for:
//!
//! - Checkig whether a ticker has already been taken (across any asset class)
//! - Registering the issuance of a new ticker with its asset class and owner
//!
//! ### Use case
//!
//! This module is called by the asset module when new assets are created.
//! Ensures that tickers are unique in the global namespace
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `print_ticker_availability` - Prints ticker availability to console [TESTING]
//!
//! ### Public Functions
//!
//! - `get` - Returns token details if the ticker is registered, None otherwise.
//! - `put` - Checks if a transfer is a valid transfer and returns the result

use crate::utils;
use codec::{Decode, Encode};
use primitives::IdentityId;
use rstd::prelude::*;
use srml_support::{decl_module, decl_storage, dispatch::Result, ensure};
use system::ensure_signed;

#[repr(u32)]
#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode)]
pub enum TokenType {
    AssetToken,
    ConfidentialAssetToken,
}

#[derive(Clone, Debug, Eq, PartialEq, Default, Encode, Decode)]
/// Asset class and owning identity
pub struct RegistryEntry {
    pub token_type: u32,
    pub owner_did: IdentityId,
}

/// Default on TokenType is there only to please the storage macro.
impl Default for TokenType {
    fn default() -> Self {
        TokenType::AssetToken
    }
}

/// The module's configuration trait.
pub trait Trait: system::Trait {}

decl_storage! {
    trait Store for Module<T: Trait> as Registry {
        /// Tokens by ticker. This represents the global namespace for tokens of all kinds. Entry
        /// keys MUST be in full caps. To ensure this the storage item is private and using the
        /// custom access methods is mandatory
        pub Tokens get(tokens): map Vec<u8> => RegistryEntry;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        /// DEBUG: prints ticker availablilty to console
        pub fn print_ticker_availability(origin, ticker: Vec<u8>) -> Result {
            let _sender = ensure_signed(origin)?;
            let upper_ticker = utils::bytes_to_upper(&ticker);

            if <Tokens>::exists(&upper_ticker) {
                sr_primitives::print("Ticker not available");
            } else {
                sr_primitives::print("Ticker available");
            }

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    /// Returns whether a ticker has been taken as an Option
    pub fn get(ticker: &Vec<u8>) -> Option<RegistryEntry> {
        let upper_ticker = utils::bytes_to_upper(ticker);

        if <Tokens>::exists(&upper_ticker) {
            Some(<Tokens>::get(upper_ticker))
        } else {
            None
        }
    }

    /// Stores a new ticker with associated asset class and owner
    pub fn put(ticker: &Vec<u8>, entry: &RegistryEntry) -> Result {
        let upper_ticker = utils::bytes_to_upper(ticker);

        ensure!(!<Tokens>::exists(&upper_ticker), "Token ticker exists");

        <Tokens>::insert(upper_ticker, entry);

        Ok(())
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use sr_io::with_externalities;
    use sr_primitives::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
        Perbill,
    };
    use srml_support::{assert_ok, impl_outer_origin, parameter_types};
    use substrate_primitives::{Blake2Hasher, H256};

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    parameter_types! {
        pub const BlockHashCount: u32 = 250;
        pub const MaximumBlockWeight: u32 = 4 * 1024 * 1024;
        pub const MaximumBlockLength: u32 = 4 * 1024 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    }
    impl system::Trait for Test {
        type Origin = Origin;
        type Call = ();
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<u64>;
        type WeightMultiplierUpdate = ();
        type Header = Header;
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type AvailableBlockRatio = AvailableBlockRatio;
        type MaximumBlockLength = MaximumBlockLength;
        type Version = ();
    }

    impl Trait for Test {}
    type Registry = Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> sr_io::TestExternalities<Blake2Hasher> {
        let t = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        sr_io::TestExternalities::new(t)
    }

    #[test]
    fn registry_ignores_case() {
        with_externalities(&mut new_test_ext(), || {
            let entry = RegistryEntry {
                token_type: TokenType::AssetToken as u32,
                owner_did: IdentityId::from(42),
            };

            assert_ok!(Registry::put(&"SOMETOKEN".as_bytes().to_vec(), &entry));

            // Verify that the entry corresponds to what we intended to insert
            assert_eq!(
                Registry::get(&"SOMETOKEN".as_bytes().to_vec()),
                Some(entry.clone())
            );

            // Effectively treated as identical ticker
            assert!(Registry::put(&"sOmEtOkEn".as_bytes().to_vec(), &entry).is_err());
        });
    }
}
