//! Runtime API definition for transaction payment module.

#![cfg_attr(not(feature = "std"), no_std)]

sp_api::decl_runtime_apis! {
    pub trait StakingApi {
        fn get_curve() -> u32;
    }
}
