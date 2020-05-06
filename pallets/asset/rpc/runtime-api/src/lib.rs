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

sp_api::decl_runtime_apis! {

    /// The API to interact with Asset.
    pub trait AssetApi<AccountId, Balance>
    where
        AccountId: Codec,
        Balance: Codec
    {
         /// Checks whether a transaction with given parameters can take place or not.
         ///
         /// # Example
         ///
         /// In this example we are checking if Alice can transfer 500 of ticket 0x01
         /// from herself (Id=0x2a) to Bob (Id=0x3905)
         ///
         /// ```ignore
         ///  curl
         ///    -H "Content-Type: application/json"
         ///    -d {
         ///        "id":1, "jsonrpc":"2.0",
         ///        "method": "asset_canTransfer",
         ///        "params":[
         ///            "5CoRaw9Ex4DUjGcnPbPBnc2nez5ZeTmM5WL3ZDVLZzM6eEgE",
         ///            "0x010000000000000000000000",
         ///            "0x2a00000000000000000000000000000000000000000000000000000000000000",
         ///            "0x3905000000000000000000000000000000000000000000000000000000000000",
         ///            500]}
         ///    http://localhost:9933 | python3 -m json.tool
         /// ```
        fn can_transfer(
            sender: AccountId,
            ticker: Ticker,
            from_did: IdentityId,
            to_did: IdentityId,
            value: Balance
        ) -> CanTransferResult;
    }
}
