use crate::asset;
use crate::asset::HasOwner;
use crate::utils;

use rstd::prelude::*;

use support::{decl_event, decl_module, decl_storage, dispatch::Result, ensure, StorageMap};
use system::{self, ensure_signed};

/// The module's configuration trait.
pub trait Trait: timestamp::Trait + system::Trait + utils::Trait {
    // TODO: Add other types and constants required configure this module.

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Asset: asset::HasOwner<Self::AccountId>;
}

decl_storage! {
    trait Store for Module<T: Trait> as PercentageTM {
        MaximumPercentageEnabledForToken get(maximum_percentage_enabled_for_token): map Vec<u8> => (bool,u16);
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event<T>() = default;

        fn toggle_maximum_percentage_restriction(origin, _ticker: Vec<u8>, enable:bool, max_percentage: u16) -> Result  {
            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            let sender = ensure_signed(origin)?;
            ensure!(Self::is_owner(ticker.clone(), sender.clone()),"Sender must be the token owner");

            //PABLO: TODO: Move all the max % logic to a new module and call that one instead of holding all the different logics in just one module.
            <MaximumPercentageEnabledForToken<T>>::insert(ticker.clone(),(enable,max_percentage));

            if enable{
                runtime_io::print("Maximum percentage restriction enabled!");
            }else{
                runtime_io::print("Maximum percentage restriction disabled!");
            }

            Ok(())
        }
    }

}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
    {
        Example(u32, AccountId, AccountId),
    }
);

impl<T: Trait> Module<T> {
    pub fn is_owner(_ticker: Vec<u8>, sender: T::AccountId) -> bool {
        let ticker = utils::bytes_to_upper(_ticker.as_slice());
        T::Asset::is_owner(ticker.clone(), sender)
        // let token = T::Asset::token_details(token_id);
        // token.owner == sender
    }

    // Transfer restriction verification logic
    pub fn verify_restriction(
        _ticker: Vec<u8>,
        _from: T::AccountId,
        _to: T::AccountId,
        _value: T::TokenBalance,
    ) -> Result {
        let ticker = utils::bytes_to_upper(_ticker.as_slice());
        let mut _can_transfer = Self::maximum_percentage_enabled_for_token(ticker.clone());
        let enabled = _can_transfer.0;
        // If the restriction is enabled, then we need to make the calculations, otherwise all good
        if enabled {
            Err("Cannot Transfer: Percentage TM restrictions not satisfied")
        } else {
            Ok(())
        }
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use crate::asset::SecurityToken;
    use lazy_static::lazy_static;
    use primitives::{Blake2Hasher, H256};
    use runtime_io::with_externalities;
    use runtime_primitives::{
        testing::{Digest, DigestItem, Header},
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage,
    };
    use support::{assert_noop, assert_ok, impl_outer_origin};

    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

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

    impl balances::Trait for Test {
        type Balance = u128;
        type DustRemoval = ();
        type Event = ();
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type TransactionPayment = ();
        type TransferPayment = ();
    }

    impl utils::Trait for Test {
        type TokenBalance = u128;
    }

    impl timestamp::Trait for Test {
        type Moment = u64;
        type OnTimestampSet = ();
    }

    impl asset::HasOwner<<Test as system::Trait>::AccountId> for Module<Test> {
        fn is_owner(_ticker: Vec<u8>, sender: <Test as system::Trait>::AccountId) -> bool {
            if let Some(token) = TOKEN_MAP.lock().unwrap().get(&_ticker) {
                token.owner == sender
            } else {
                false
            }
        }
    }

    impl Trait for Test {
        type Event = ();
        type Asset = Module<Test>;
    }
    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap()
            .0
            .into()
    }
    type PercentageTM = Module<Test>;

    lazy_static! {
        static ref TOKEN_MAP: Arc<
            Mutex<
                HashMap<
                    Vec<u8>,
                    SecurityToken<
                        <Test as utils::Trait>::TokenBalance,
                        <Test as system::Trait>::AccountId,
                    >,
                >,
            >,
        > = Arc::new(Mutex::new(HashMap::new()));
        /// Because Rust's Mutex is not recursive a second symbolic lock is necessary
        static ref TOKEN_MAP_OUTER_LOCK: Arc<Mutex<()>> = Arc::new(Mutex::new(()));
    }

    #[test]
    fn discards_non_owners() {
        with_externalities(&mut new_test_ext(), || {
            let ticker = vec![0x01];

            // We need the lock to exist till assertions are done
            let outer = TOKEN_MAP_OUTER_LOCK.lock().unwrap();

            // Prepare a token entry with mismatched owner
            *TOKEN_MAP.lock().unwrap() = {
                let mut map = HashMap::new();
                let token = SecurityToken {
                    name: ticker.clone(),
                    owner: 1337,
                    total_supply: 1_000_000,
                };
                map.insert(ticker.clone(), token);
                map
            };

            // But look up against 1
            assert_noop!(
                PercentageTM::toggle_maximum_percentage_restriction(
                    Origin::signed(1),
                    ticker,
                    true,
                    15
                ),
                "Sender must be the token owner"
            );
            drop(outer);
        });
    }

    #[test]
    fn accepts_real_owners() {
        with_externalities(&mut new_test_ext(), || {
            let ticker = vec![0x01];
            let matching_owner = 1;

            // We need the lock to exist till assertions are done
            let outer = TOKEN_MAP_OUTER_LOCK.lock().unwrap();

            *TOKEN_MAP.lock().unwrap() = {
                let mut map = HashMap::new();
                let token = SecurityToken {
                    name: ticker.clone(),
                    owner: matching_owner,
                    total_supply: 1_000_000,
                };
                map.insert(ticker.clone(), token);
                map
            };

            assert_ok!(PercentageTM::toggle_maximum_percentage_restriction(
                Origin::signed(matching_owner),
                ticker.clone(),
                true,
                15
            ));
            assert_eq!(
                PercentageTM::maximum_percentage_enabled_for_token(ticker.clone()),
                (true, 15)
            );
            drop(outer);
        });
    }
}
