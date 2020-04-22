//! Runtime API definition for the protocol fee module.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use frame_support::dispatch::Vec;
use polymesh_runtime_common::protocol_fee::ProtocolOp;
use sp_runtime::traits::{MaybeDisplay, MaybeFromStr};

sp_api::decl_runtime_apis! {
    pub trait ProtocolFeeApi<Balance> where
        Balance: Codec + MaybeDisplay + MaybeFromStr,
    {
        fn compute_fee(op: ProtocolOp) -> Balance;
    }
}
