#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

pub mod fee_details;
pub mod runtime;
pub use fee_details::CddHandler;
pub mod constants;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarks;

#[cfg(feature = "std")]
pub use pallet_staking::StakerStatus;

#[cfg(feature = "std")]
pub use runtime::{native_version, WASM_BINARY};

pub use runtime::{
    api, Asset, Authorship, Balances, BalancesCall, Bridge, Call, CheckedExtrinsic, MinimumPeriod,
    ProtocolFee, Runtime, RuntimeApi, SessionKeys, System, SystemCall, TransactionPayment,
    UncheckedExtrinsic,
};

pub use sp_runtime::{Perbill, Permill};
