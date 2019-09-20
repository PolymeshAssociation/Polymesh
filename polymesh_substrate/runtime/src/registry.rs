//! A runtime module providing a unique ticker registry

use crate::utils;
use codec::{Decode, Encode};
use rstd::prelude::*;
use srml_support::{decl_module, decl_storage, dispatch::Result, ensure, StorageMap};
use system::ensure_signed;

#[repr(u32)]
#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode)]
pub enum TokenType {
    AssetToken,
    ConfidentialAssetToken,
}

#[derive(Clone, Debug, Eq, PartialEq, Default, Encode, Decode)]
pub struct RegistryEntry {
    pub token_type: u32,
    pub owner_did: Vec<u8>,
}

/// Default on TokenType is there only to please the storage macro.
impl Default for TokenType {
    fn default() -> Self {
        TokenType::AssetToken
    }
}

/// The module's configuration trait.
pub trait Trait: system::Trait {
    // TODO: Add other types and constants required configure this module.
}

decl_storage! {
    trait Store for Module<T: Trait> as Registry {
        // Tokens by ticker. This represents the global namespace for tokens of all kinds. Entry
        // keys MUST be in full caps. To ensure this the storage item is private and using the
        // custom access methods is mandatory
        pub Tokens get(tokens): map Vec<u8> => RegistryEntry;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        pub fn printTickerAvailability(origin, ticker: Vec<u8>) -> Result {
            let sender = ensure_signed(origin)?;
            let ticker = utils::bytes_to_upper(ticker.as_slice());

            if <Tokens>::exists(ticker.clone()) {
                sr_primitives::print("Ticker not available");
            } else {
                sr_primitives::print("Ticker available");
            }

            Ok(())
        }


    }
}

impl<T: Trait> Module<T> {
    pub fn get(ticker: Vec<u8>) -> Option<RegistryEntry> {
        let ticker = utils::bytes_to_upper(ticker.as_slice());

        if <Tokens>::exists(ticker.clone()) {
            Some(<Tokens>::get(ticker))
        } else {
            None
        }
    }

    pub fn put(ticker: Vec<u8>, entry: &RegistryEntry) -> Result {
        let ticker = utils::bytes_to_upper(ticker.as_slice());

        ensure!(!<Tokens>::exists(ticker.clone()), "Token ticker exists");

        <Tokens>::insert(ticker.clone(), entry);

        Ok(())
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use chrono::{prelude::*, Duration};
    use lazy_static::lazy_static;
    use substrate_primitives::{Blake2Hasher, H256};
    use sr_io::with_externalities;
    use sr_primitives::{Perbill, traits::{BlakeTwo256, IdentityLookup, ConvertInto}, testing::Header};
    use srml_support::{impl_outer_origin, assert_ok, assert_err, assert_noop, parameter_types};
    use yaml_rust::{Yaml, YamlLoader};

    use std::{
        collections::HashMap,
        fs::read_to_string,
        path::PathBuf,
        sync::{Arc, Mutex},
    };

    use crate::identity::{self, IdentityTrait, Investor, InvestorList};

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
        system::GenesisConfig::default()
            .build_storage()
            .unwrap()
            .0
            .into()
    }

    #[test]
    fn registry_ignores_case() {
        with_externalities(&mut new_test_ext(), || {
            let entry = RegistryEntry {
                token_type: TokenType::AssetToken as u32,
                owner_did: "did:poly:some_did".as_bytes().to_vec(),
            };

            assert_ok!(Registry::put("SOMETOKEN".as_bytes().to_vec(), &entry));

            // Verify that the entry corresponds to what we intended to insert
            assert_eq!(
                Registry::get("SOMETOKEN".as_bytes().to_vec()),
                Some(entry.clone())
            );

            // Effectively treated as identical ticker
            assert!(Registry::put("sOmEtOkEn".as_bytes().to_vec(), &entry).is_err());
        });
    }
}
