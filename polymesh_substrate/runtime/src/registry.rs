//! A runtime module providing a unique ticker registry

use codec::{Decode, Encode};
use rstd::prelude::*;
use system::ensure_signed;
use crate::utils;
use srml_support::{decl_module, decl_storage, dispatch::Result, ensure, StorageMap};

#[repr(u32)]
#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode)]
pub enum TokenType {
    AssetToken,
    ConfidentialAssetToken,
}

#[derive(Clone, Debug, Eq, PartialEq, Default, Encode, Decode)]
pub struct RegistryEntry<U> {
    pub token_type: u32,
    pub owner: U,
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
        pub Tokens get(tokens): map Vec<u8> => RegistryEntry<T::AccountId>;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        pub fn printTickerAvailability(origin, ticker: Vec<u8>) -> Result {
            let sender = ensure_signed(origin)?;
            let ticker = utils::bytes_to_upper(ticker.as_slice());

            if <Tokens<T>>::exists(ticker.clone()) {
                sr_primitives::print("Ticker not available");
            } else {
                sr_primitives::print("Ticker available");
            }

            Ok(())
        }


    }
}

impl<T: Trait> Module<T> {
    pub fn get(ticker: Vec<u8>) -> Option<RegistryEntry<T::AccountId>> {
        let ticker = utils::bytes_to_upper(ticker.as_slice());

        if <Tokens<T>>::exists(ticker.clone()) {
            Some(<Tokens<T>>::get(ticker))
        } else {
            None
        }
    }

    pub fn put(ticker: Vec<u8>, entry: &RegistryEntry<T::AccountId>) -> Result {
        let ticker = utils::bytes_to_upper(ticker.as_slice());

        ensure!(!<Tokens<T>>::exists(ticker.clone()), "Token ticker exists");

        <Tokens<T>>::insert(ticker.clone(), entry);

        Ok(())
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use primitives::{Blake2Hasher, H256};
    use sr_io::with_externalities;
    use sr_primitives::{
        testing::{Digest, DigestItem, Header},
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage,
    };
    use srml_support::{assert_ok, impl_outer_origin};

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    impl system::Trait for Test {
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type Digest = Digest;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type Log = DigestItem;
    }
    impl Trait for Test {}
    type Registry = Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> sr_io::TestExternalities<Blake2Hasher> {
        system::GenesisConfig::<Test>::default()
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
                owner: 0,
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
