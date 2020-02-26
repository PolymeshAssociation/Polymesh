//! Runtime API definition for Identity module.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Codec, Decode, Encode};
use polymesh_primitives::{IdentityId, Ticker};
use sp_runtime::RuntimeDebug;

/// Data structure returned to provide the cdd status
#[derive(Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum CddStatus {
    Success {
        /// Cdd claim provider
        cdd_claim_provider: IdentityId,
    },
    Error,
}

/// Data structure returned to provide the cdd status
#[derive(Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum AssetDidResult {
    Success {
        /// asset DID
        asset_did: IdentityId,
    },
    Error,
}

sp_api::decl_runtime_apis! {
    pub trait IdentityApi<IdentityId, Ticker> where
        IdentityId: Codec,
        Ticker: Codec,
    {
        fn is_identity_has_valid_cdd(did: IdentityId, buffer_time: Option<u64>) -> CddStatus;
        fn get_asset_did(ticker: Ticker) -> AssetDidResult;
    }
}
