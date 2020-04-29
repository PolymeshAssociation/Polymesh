#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

pub mod asset;
pub mod bridge;
pub mod cdd_check;
pub mod contracts_wrapper;
pub mod dividend;
pub mod exemption;
pub mod general_tm;
pub mod impls;
pub mod percentage_tm;
pub mod simple_token;
pub mod statistics;
pub mod sto_capped;
pub mod voting;

pub use cdd_check::CddChecker;
pub use sp_runtime::{Perbill, Permill};

use frame_support::{parameter_types, traits::Currency, weights::Weight};
use frame_system::{self as system};
use pallet_balances as balances;
use polymesh_primitives::BlockNumber;

pub use impls::{Author, CurrencyToVoteHandler, TargetedFeeAdjustment};

pub type NegativeImbalance<T> =
    <balances::Module<T> as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;

parameter_types! {
    pub const BlockHashCount: BlockNumber = 250;
    pub const MaximumBlockWeight: Weight = 1_000_000_000;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    pub const MaximumBlockLength: u32 = 5 * 1024 * 1024;
}

#[cfg(feature = "runtime-benchmarks")]
pub mod benches;
