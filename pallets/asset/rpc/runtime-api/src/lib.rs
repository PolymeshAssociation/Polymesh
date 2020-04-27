//! Runtime API definition for Identity module.
#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use frame_support::traits::Currency;
use polymesh_primitives::{IdentityId, Ticker};
use sp_std::vec::Vec;

pub type Error = Vec<u8>;
pub type CanTransferResult = Result<u8, Error>;

pub trait Trait: frame_system::Trait {
    type Currency: Currency<Self::AccountId>;
}

pub type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
sp_api::decl_runtime_apis! {

     /// The API to interact with Asset.
     pub trait AssetApi<AccountId, T>
     where
         AccountId: Codec,
         T: Codec
     {
         /// Retrieve votes for a proposal for a given `mips_index`.
         fn can_transfer(
             sender: AccountId,
             ticker: Ticker,
             from_did: IdentityId,
             to_did: IdentityId,
             value: T) -> CanTransferResult;
    }
}
