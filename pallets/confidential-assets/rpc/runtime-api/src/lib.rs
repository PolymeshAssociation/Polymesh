//! Runtime API definition for confidential assets.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet_confidential_assets::RelayerSubmitBatchedFeeInfo;
use polymesh_dart::{BatchedProofs, PolymeshLimits};

sp_api::decl_runtime_apis! {
    pub trait ConfidentialAssetsApi {
        /// Returns the weight and fee for a signed `relayer_submit_batched_proofs` extrinsic.
        ///
        /// `len` is the anticipated SCALE-encoded length of the complete signed extrinsic.
        fn relayer_submit_batched_fee_info(
            batch: BatchedProofs<PolymeshLimits>,
            len: u32,
        ) -> RelayerSubmitBatchedFeeInfo;
    }
}
