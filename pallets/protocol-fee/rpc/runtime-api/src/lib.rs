//! Runtime API definition for the protocol fee module.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::Vec;
use polymesh_runtime_common::protocol_fee::ProtocolOp;

sp_api::decl_runtime_apis! {
    pub trait ProtocolFeeApi<Balance> {
        fn get_fee(op: ProtocolOp) -> Balance;
    }
}
