//! Runtime API definition for the protocol fee module.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::dispatch::Vec;
use polymesh_runtime_common::protocol_fee::ProtocolOp;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::traits::{SaturatedConversion, UniqueSaturatedInto};

/// A capped version of `Balance` which is normally a `u128`, fit into `u64` which is a serializable
/// type unlike `u128`. There are no fees that would not fit into `u64`.
#[derive(Eq, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct CappedFee(pub u64);

impl<T: UniqueSaturatedInto<u64>> From<T> for CappedFee {
    fn from(fee: T) -> CappedFee {
        CappedFee(fee.saturated_into())
    }
}

sp_api::decl_runtime_apis! {
    pub trait ProtocolFeeApi {
        fn compute_fee(op: ProtocolOp) -> CappedFee;
    }
}
