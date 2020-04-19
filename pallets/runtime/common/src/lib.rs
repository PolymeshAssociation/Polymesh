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

#[cfg(test)]
pub mod test;

use pallet_balances as balances;
use polymesh_primitives::BlockNumber;
use frame_support::{
	parameter_types, traits::Currency,
	weights::Weight,
};
use frame_system::{self as system};

// #[cfg(feature = "std")]
// pub mod config {

//     use pallet_committee as committee;
//     use pallet_protocol_fee as protocol_fee;
//     use pallet_balances as balances;
//     use pallet_identity as identity;

//     pub type AssetConfig = crate::asset::GenesisConfig<crate::Runtime>;
//     pub type BalancesConfig = balances::GenesisConfig<crate::Runtime>;
//     pub type BridgeConfig = crate::bridge::GenesisConfig<crate::Runtime>;
//     pub type IdentityConfig = identity::GenesisConfig<crate::Runtime>;
//     pub type SimpleTokenConfig = crate::simple_token::GenesisConfig<crate::Runtime>;
//     pub type StakingConfig = pallet_staking::GenesisConfig<crate::Runtime>;
//     pub type PolymeshCommitteeConfig =
//         committee::GenesisConfig<crate::Runtime, committee::Instance1>;
//     pub type MipsConfig = pallet_mips::GenesisConfig<crate::Runtime>;
//     pub type ContractsConfig = pallet_contracts::GenesisConfig<crate::Runtime>;
//     pub type IndicesConfig = pallet_indices::GenesisConfig<crate::Runtime>;
//     pub type ImOnlineConfig = pallet_im_online::GenesisConfig<crate::Runtime>;
//     pub type SudoConfig = pallet_sudo::GenesisConfig<crate::Runtime>;
//     pub type SystemConfig = frame_system::GenesisConfig;
//     pub type GenesisConfig = crate::runtime::GenesisConfig;
//     pub type SessionConfig = pallet_session::GenesisConfig<crate::Runtime>;
//     pub type ProtocolFeeConfig = protocol_fee::GenesisConfig<crate::Runtime>;
// }

pub use impls::{CurrencyToVoteHandler, TargetedFeeAdjustment, Author};

pub type NegativeImbalance<T> = <balances::Module<T> as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;

parameter_types! {
	pub const BlockHashCount: BlockNumber = 250;
	pub const MaximumBlockWeight: Weight = 1_000_000_000;
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
	pub const MaximumBlockLength: u32 = 5 * 1024 * 1024;
}
