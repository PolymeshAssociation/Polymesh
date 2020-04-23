//! Runtime API definition for Identity module.
#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::vec::Vec;

pub type Error = Vec<u8>;
pub type CanTransferResult = Result<u8, Error>;

use frame_support::traits::Currency;
pub trait Trait: frame_system::Trait {
    type Currency: Currency<Self::AccountId>;
}

pub type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
sp_api::decl_runtime_apis! {

     /// The API to interact with Asset.
     pub trait AssetApi<IdentityId, Ticker, T>
     where
         IdentityId: Codec,
         Ticker: Codec,
         T: Codec
     {
         /// Retrieve votes for a proposal for a given `mips_index`.
         fn can_transfer(ticker: Ticker, from_did: IdentityId, to_did: IdentityId, value: T, data: Vec<u8>) -> CanTransferResult;
    }

}
