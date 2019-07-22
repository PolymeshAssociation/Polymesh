//! A runtime module providing a unique ticker registry

use parity_codec::{Decode, Encode};
use rstd::prelude::*;

use crate::utils;
use support::{decl_module, decl_storage, dispatch::Result, ensure, StorageMap};

/// The module's configuration trait.
pub trait Trait: system::Trait {
    // TODO: Add other types and constants required configure this module.
}

decl_storage! {
    trait Store for Module<T: Trait> as Settlement {

    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

    }
}

impl<T: Trait> Module<T> {

}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use primitives::{Blake2Hasher, H256};
    use runtime_io::with_externalities;
    use runtime_primitives::{
        testing::{Digest, DigestItem, Header},
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage,
    };
    use support::{assert_ok, impl_outer_origin};

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

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
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
