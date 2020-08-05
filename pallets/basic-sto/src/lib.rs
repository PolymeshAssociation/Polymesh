// Copyright (c) 2020 Polymath

//! # STO Module

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

use pallet_identity as identity;
use polymesh_common_utilities::{
    traits::{asset::Trait as AssetTrait, identity::Trait as IdentityTrait, CommonTrait},
    Context,
    SystematicIssuers::Settlement as SettlementDID,
};
use polymesh_primitives::{IdentityId, Ticker};

use codec::{Decode, Encode};
use frame_support::storage::{with_transaction, TransactionOutcome::*};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
    traits::Get, weights::Weight, IterableStorageDoubleMap,
};
use frame_system::{self as system, ensure_signed};
use polymesh_primitives_derive::VecU8StrongTyped;
use sp_runtime::{Perbill, traits::Verify};
use sp_std::{collections::btree_set::BTreeSet, convert::TryFrom, prelude::*};
type Identity<T> = identity::Module<T>;
type Settlement<T> = pallet_settlement::Module<T>;

pub trait Trait:
    frame_system::Trait + CommonTrait + IdentityTrait + pallet_timestamp::Trait + pallet_settlement::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    /// Asset module
    type Asset: AssetTrait<Self::Balance, Self::AccountId>;
}


decl_event!(
    pub enum Event<T>
    where
        Balance = <T as CommonTrait>::Balance
    {
        /// A new STO has been created
        StoCreated(IdentityId, Balance),
    }
);

decl_error! {
    /// Errors for the Settlement module.
    pub enum Error for Module<T: Trait> {
        /// Sender does not have required permissions
        Unauthorized
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as StoCapped {

    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        // type Asset = Asset<T>;
        // type Settlement = Settlement<T>;
        fn deposit_event() = default;

        /// Create a new offering.
        #[weight = 200_000]
        pub fn create_offering(
            origin,
            offering_token: Ticker,
            raise_token: Ticker,
            sell_amount: T::Balance,
            price_per_token: Perbill,
            venue_id: u128
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            ensure!(Asset<T>::is_owner(&raise_token, did), Error::Unauthorized);
            // TODO: Take custodial ownership of `offering_token` from treasury

            Ok(())
        }
    }

}

impl<T: Trait> Module<T> {
    /// Returns true if `sender_did` is the owner of `ticker` asset.
    fn is_owner(ticker: &Ticker, sender_did: IdentityId) -> bool {
        T::Asset::is_owner(ticker, sender_did)
    }


}
