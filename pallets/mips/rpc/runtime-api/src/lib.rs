//! Runtime API definition for mips module.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::Vec;
use sp_runtime::Perbill;

sp_api::decl_runtime_apis! {
    pub trait MipsApi {
        fn get_votes() -> Vec<u32>;
    }
}
