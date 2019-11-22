//! # Percentage Transfer Manager Module
//!
//! The PTM module provides functionality for restricting transfers based on an investors ownership percentage of the asset
//!
//! ## Overview
//!
//! The PTM module provides functions for:
//!
//! - Setting a percentage based transfer restriction
//! - Removing a percentage based transfer restriction
//!
//! ### Use case
//!
//! An asset issuer can restrict token transfers that would breach a single investor owning more than a set percentage of the issued asset.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `toggle_maximum_percentage_restriction` - Sets a percentage restriction on a ticker - set to 0 to remove
//!
//! ### Public Functions
//!
//! - `verify_restriction` - Checks if a transfer is a valid transfer and returns the result

use crate::{asset::AssetTrait, balances, constants::*, exemption, identity, utils};
use primitives::{IdentityId, Key};

use codec::Encode;
use core::result::Result as StdResult;
use rstd::{convert::TryFrom, prelude::*};
use sr_primitives::traits::{CheckedAdd, CheckedDiv, CheckedMul};
use srml_support::{decl_event, decl_module, decl_storage, dispatch::Result, ensure};
use system::{self, ensure_signed};

/// The module's configuration trait.
pub trait Trait: system::Trait + utils::Trait + exemption::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_event!(
    pub enum Event<T>
    where
        Balance = <T as balances::Trait>::Balance,
    {
        TogglePercentageRestriction(Vec<u8>, u16, bool),
        DoSomething(Balance),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as PercentageTM {
        MaximumPercentageEnabledForToken get(maximum_percentage_enabled_for_token): map Vec<u8> => u16;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Set a maximum percentage that can be owned by a single investor
        fn toggle_maximum_percentage_restriction(origin, did: IdentityId, _ticker: Vec<u8>, max_percentage: u16) -> Result  {
            let upper_ticker = utils::bytes_to_upper(_ticker.as_slice());
            let sender = ensure_signed(origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");

            ensure!(Self::is_owner(&upper_ticker, did),"Sender DID must be the token owner");
            // if max_percentage == 0 then it means we are disallowing the percentage transfer restriction to that ticker.

            //PABLO: TODO: Move all the max % logic to a new module and call that one instead of holding all the different logics in just one module.
            //SATYAM: TODO: Add the decimal restriction
            <MaximumPercentageEnabledForToken>::insert(&upper_ticker, max_percentage);
            // Emit an event with values (Ticker of asset, max percentage, restriction enabled or not)
            Self::deposit_event(RawEvent::TogglePercentageRestriction(upper_ticker, max_percentage, max_percentage != 0));

            if max_percentage != 0 {
                sr_primitives::print("Maximum percentage restriction enabled!");
            } else {
                sr_primitives::print("Maximum percentage restriction disabled!");
            }

            Ok(())
        }

    }
}

impl<T: Trait> Module<T> {
    pub fn is_owner(ticker: &Vec<u8>, sender_did: IdentityId) -> bool {
        let upper_ticker = utils::bytes_to_upper(ticker);
        T::Asset::is_owner(&upper_ticker, sender_did)
    }

    /// Transfer restriction verification logic
    pub fn verify_restriction(
        ticker: &[u8],
        _from_did_opt: Option<IdentityId>,
        to_did_opt: Option<IdentityId>,
        value: T::Balance,
    ) -> StdResult<u8, &'static str> {
        let upper_ticker = utils::bytes_to_upper(ticker);
        let max_percentage = Self::maximum_percentage_enabled_for_token(&upper_ticker);
        // check whether the to address is in the exemption list or not
        // 2 refers to percentageTM
        // TODO: Mould the integer into the module identity
        if let Some(to_did) = to_did_opt.clone() {
            let is_exempted = <exemption::Module<T>>::is_exempted(&upper_ticker, 2, to_did);
            if max_percentage != 0 && !is_exempted {
                let new_balance = (T::Asset::balance(&upper_ticker, to_did))
                    .checked_add(&value)
                    .ok_or("Balance of to will get overflow")?;
                let total_supply = T::Asset::total_supply(&upper_ticker);

                let percentage_balance = (new_balance
                    .checked_mul(&((10 as u128).pow(18)).into())
                    .ok_or("unsafe multiplication")?)
                .checked_div(&total_supply)
                .ok_or("unsafe division")?;

                let allowed_token_amount = (max_percentage as u128)
                    .checked_mul((10 as u128).pow(16))
                    .ok_or("unsafe percentage multiplication")?;

                if percentage_balance > allowed_token_amount.into() {
                    sr_primitives::print(
                        "It is failing because it is not validating the PercentageTM restrictions",
                    );
                    return Ok(APP_FUNDS_LIMIT_REACHED);
                }
            }
            sr_primitives::print("It is passing thorugh the PercentageTM");
            Ok(ERC1400_TRANSFER_SUCCESS)
        } else {
            sr_primitives::print("to account is not active");
            Ok(ERC1400_INVALID_RECEIVER)
        }
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    // use super::*;

    // use crate::asset::SecurityToken;
    // use lazy_static::lazy_static;
    // use substrate_primitives::{Blake2Hasher, H256};
    // use sr_io::with_externalities;
    // use sr_primitives::{
    //     testing::{Digest, DigestItem, Header},
    //     traits::{BlakeTwo256, IdentityLookup},
    //     BuildStorage,
    // };
    // use srml_support::{assert_noop, assert_ok, impl_outer_origin};

    // use std::{
    //     collections::HashMap,
    //     sync::{Arc, Mutex},
    // };

    // impl_outer_origin! {
    //     pub enum Origin for Test {}
    // }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
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
    //     type OnFreeBalanceZero = ();
    //     type OnNewAccount = ();
    //     type TransactionPayment = ();
    //     type TransferPayment = ();
    // }

    // impl utils::Trait for Test {
    //     type Balance = u128;
    // }

    // impl timestamp::Trait for Test {
    //     type Moment = u64;
    //     type OnTimestampSet = ();
    // }

    // impl asset::HasOwner<<Test as system::Trait>::AccountId> for Module<Test> {
    //     fn is_owner(_ticker: Vec<u8>, sender: <Test as system::Trait>::AccountId) -> bool {
    //         if let Some(token) = TOKEN_MAP.lock().unwrap().get(&_ticker) {
    //             token.owner == sender
    //         } else {
    //             false
    //         }
    //     }
    // }

    // impl Trait for Test {
    //     type Event = ();
    //     type Asset = Module<Test>;
    // }
    // // This function basically just builds a genesis storage key/value store according to
    // // our desired mockup.
    // fn new_test_ext() -> sr_io::TestExternalities<Blake2Hasher> {
    //     system::GenesisConfig::default()
    //         .build_storage()
    //         .unwrap()
    //         .0
    //         .into()
    // }
    //type PercentageTM = Module<Test>;

    // lazy_static! {
    //     static ref TOKEN_MAP: Arc<
    //         Mutex<
    //             HashMap<
    //                 Vec<u8>,
    //                 SecurityToken<
    //                     <Test as balances::Trait>::Balance,
    //                     <Test as system::Trait>::AccountId,
    //                 >,
    //             >,
    //         >,
    //     > = Arc::new(Mutex::new(HashMap::new()));
    //     /// Because Rust's Mutex is not recursive a second symbolic lock is necessary
    //     static ref TOKEN_MAP_OUTER_LOCK: Arc<Mutex<()>> = Arc::new(Mutex::new(()));
    // }

    // #[test]
    // fn discards_non_owners() {
    //     with_externalities(&mut new_test_ext(), || {
    //         let ticker = vec![0x01];

    //         // We need the lock to exist till assertions are done
    //         let outer = TOKEN_MAP_OUTER_LOCK.lock().unwrap();

    //         // Prepare a token entry with mismatched owner
    //         *TOKEN_MAP.lock().unwrap() = {
    //             let mut map = HashMap::new();
    //             let token = SecurityToken {
    //                 name: ticker.clone(),
    //                 owner: 1337,
    //                 total_supply: 1_000_000,
    //             };
    //             map.insert(ticker.clone(), token);
    //             map
    //         };

    //         // But look up against 1
    //         assert_noop!(
    //             PercentageTM::toggle_maximum_percentage_restriction(
    //                 Origin::signed(1),
    //                 ticker,
    //                 true,
    //                 15
    //             ),
    //             "Sender must be the token owner"
    //         );
    //         drop(outer);
    //     });
    // }

    // #[test]
    // fn accepts_real_owners() {
    //     with_externalities(&mut new_test_ext(), || {
    //         let ticker = vec![0x01];
    //         let matching_owner = 1;

    //         // We need the lock to exist till assertions are done
    //         let outer = TOKEN_MAP_OUTER_LOCK.lock().unwrap();

    //         *TOKEN_MAP.lock().unwrap() = {
    //             let mut map = HashMap::new();
    //             let token = SecurityToken {
    //                 name: ticker.clone(),
    //                 owner: matching_owner,
    //                 total_supply: 1_000_000,
    //             };
    //             map.insert(ticker.clone(), token);
    //             map
    //         };

    //         assert_ok!(PercentageTM::toggle_maximum_percentage_restriction(
    //             Origin::signed(matching_owner),
    //             ticker.clone(),
    //             true,
    //             15
    //         ));
    //         assert_eq!(
    //             PercentageTM::maximum_percentage_enabled_for_token(ticker.clone()),
    //             (true, 15)
    //         );
    //         drop(outer);
    //     });
    // }
}
