use crate::{
    asset::{self, AssetTrait},
    balances, identity, utils,
};
use primitives::{IdentityId, Key};

use codec::Encode;
use rstd::{convert::TryFrom, prelude::*};
use srml_support::{decl_event, decl_module, decl_storage, dispatch::Result, ensure};
use system::ensure_signed;

/// The module's configuration trait.
pub trait Trait: system::Trait + utils::Trait + balances::Trait + identity::Trait {
    /// The overarching event type.
    type Event: From<Event> + Into<<Self as system::Trait>::Event>;
    type Asset: asset::AssetTrait<Self::Balance>;
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as exemption {
        // Mapping -> ExemptionList[ticker][TM][DID] = true/false
        ExemptionList get(exemption_list): map (Vec<u8>, u16, IdentityId) => bool;
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event() = default;

        fn modify_exemption_list(origin, did: IdentityId, ticker: Vec<u8>, _tm: u16, asset_holder_did: IdentityId, exempted: bool) -> Result {
            let upper_ticker = utils::bytes_to_upper(&ticker);
            let sender = ensure_signed(origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");

            ensure!(Self::is_owner(&upper_ticker, did), "Sender must be the token owner");
            let ticker_asset_holder_did = (ticker.clone(), _tm, asset_holder_did.clone());
            let is_exempted = Self::exemption_list(&ticker_asset_holder_did);
            ensure!(is_exempted != exempted, "No change in the state");

            <ExemptionList>::insert(&ticker_asset_holder_did, exempted);
            Self::deposit_event(Event::ModifyExemptionList(ticker, _tm, asset_holder_did, exempted));

            Ok(())
        }
    }
}

decl_event!(
    pub enum Event {
        ModifyExemptionList(Vec<u8>, u16, IdentityId, bool),
    }
);

impl<T: Trait> Module<T> {
    pub fn is_owner(ticker: &[u8], sender_did: IdentityId) -> bool {
        let upper_ticker = utils::bytes_to_upper(ticker);
        T::Asset::is_owner(&upper_ticker, sender_did)
    }

    pub fn is_exempted(ticker: &[u8], tm: u16, did: IdentityId) -> bool {
        let upper_ticker = utils::bytes_to_upper(ticker);
        Self::exemption_list((upper_ticker, tm, did))
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    // use super::*;

    // use substrate_primitives::{Blake2Hasher, H256};
    // use sr_io::with_externalities;
    // use sr_primitives::{
    //     testing::{Digest, DigestItem, Header},
    //     traits::{BlakeTwo256, IdentityLookup},
    //     BuildStorage,
    // };
    // use srml_support::{assert_ok, impl_outer_origin};

    // impl_outer_origin! {
    //     pub enum Origin for Test {}
    // }

    // // For testing the module, we construct most of a mock runtime. This means
    // // first constructing a configuration type (`Test`) which `impl`s each of the
    // // configuration traits of modules we want to use.
    // #[derive(Clone, Eq, PartialEq)]
    // pub struct Test;
    // impl system::Trait for Test {
    //     type Origin = Origin;
    //     type Index = u64;
    //     type BlockNumber = u64;
    //     type Hash = H256;
    //     type Hashing = BlakeTwo256;
    //     type Digest = H256;
    //     type AccountId = u64;
    //     type Lookup = IdentityLookup<Self::AccountId>;
    //     type Header = Header;
    //     type Event = ();
    //     type Log = DigestItem;
    // }
    // impl Trait for Test {
    //     type Event = ();
    // }
    // type exemption = Module<Test>;

    // // This function basically just builds a genesis storage key/value store according to
    // // our desired mockup.
    // fn new_test_ext() -> sr_io::TestExternalities<Blake2Hasher> {
    //     system::GenesisConfig::default()
    //         .build_storage()
    //         .unwrap()
    //         .0
    //         .into()
    // }

    // #[test]
    // fn it_works_for_default_value() {
    //     with_externalities(&mut new_test_ext(), || {
    //         // Just a dummy test for the dummy funtion `do_something`
    //         // calling the `do_something` function with a value 42
    //         assert_ok!(exemption::do_something(Origin::signed(1), 42));
    //         // asserting that the stored value is equal to what we stored
    //         assert_eq!(exemption::something(), Some(42));
    //     });
    // }
}
