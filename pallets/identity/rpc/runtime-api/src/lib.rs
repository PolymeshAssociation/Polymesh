//! Runtime API definition for Identity module.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use polymesh_primitives::{IdentityId, Ticker};
use frame_support::dispatch::Vec;

pub type Error = Vec<u8>;
pub type CddStatus = Result<IdentityId, Error>;
pub type AssetDidResult = Result<IdentityId, Error>;

sp_api::decl_runtime_apis! {
    pub trait IdentityApi<IdentityId, Ticker> where
        IdentityId: Codec,
        Ticker: Codec,
    {
        fn is_identity_has_valid_cdd(did: IdentityId, buffer_time: Option<u64>) -> CddStatus;
        fn get_asset_did(ticker: Ticker) -> AssetDidResult;
    }
}

