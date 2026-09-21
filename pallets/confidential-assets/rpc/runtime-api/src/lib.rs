//! Runtime API definition for confidential assets.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use polymesh_dart::{BatchedProofs, PolymeshLimits};
use polymesh_primitives::Balance;
use scale_info::TypeInfo;
use sp_weights::Weight;

#[derive(Clone, Decode, Encode, Eq, PartialEq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct RelayerSubmitBatchedFeeInfo {
    pub weight: Weight,
    pub fee: Balance,
}

sp_api::decl_runtime_apis! {
    pub trait ConfidentialAssetsApi {
        fn relayer_submit_batched_fee_info(
            batch: BatchedProofs<PolymeshLimits>,
        ) -> RelayerSubmitBatchedFeeInfo;
    }
}
