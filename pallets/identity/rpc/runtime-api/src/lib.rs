//! Runtime API definition for Identity module.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Codec, Decode, Encode};
use polymesh_primitives::{IdentityId, Ticker};

sp_api::decl_runtime_apis! {
    pub trait IdentityApi<IdentityId, Ticker> where
        IdentityId: Codec,
        Ticker: Codec,
    {
        fn is_identity_has_valid_cdd(did: IdentityId, buffer_time: Option<u64>) -> (bool, Option<IdentityId>);
        fn get_asset_did(ticker: Ticker) -> Option<IdentityId>;
    }
}
