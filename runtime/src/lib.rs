#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

/// Implementations of some helper traits passed into runtime modules as associated types.
pub mod impls;

mod asset;
pub mod balances;
/// Constant values used within the runtime.
pub mod constants;

mod contracts_wrapper;
mod dividend;
mod exemption;
mod general_tm;
mod identity;
mod percentage_tm;
mod registry;
mod simple_token;

pub mod staking;
#[cfg(feature = "std")]
pub use staking::StakerStatus;

pub mod runtime;
mod sto_capped;
mod utils;
mod voting;
pub use runtime::{Authorship, Balances, MaximumBlockWeight, NegativeImbalance, Runtime};

pub mod update_did_signed_extension;
pub use update_did_signed_extension::UpdateDid;

#[cfg(test)]
pub mod test;
