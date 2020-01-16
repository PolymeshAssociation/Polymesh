#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

/// Implementations of some helper traits passed into runtime modules as associated types.
pub mod impls;

pub mod asset;
pub mod balances;
/// Constant values used within the runtime.
pub mod constants;

#[cfg(feature = "std")]
pub use pallet_staking::StakerStatus;

mod contracts_wrapper;
mod dividend;
mod exemption;
mod general_tm;
mod identity;
mod percentage_tm;
mod simple_token;

pub mod runtime;
mod sto_capped;
mod utils;
mod voting;
pub use runtime::{
    api, Asset, Authorship, Balances, MaximumBlockWeight, NegativeImbalance, Runtime, RuntimeApi,
    SessionKeys, System, Contracts, TransactionPayment,
};
#[cfg(feature = "std")]
pub use runtime::{native_version, WASM_BINARY};

#[cfg(feature = "std")]
pub mod config {
    pub type AssetConfig = crate::asset::GenesisConfig<crate::Runtime>;
    pub type BalancesConfig = crate::balances::GenesisConfig<crate::Runtime>;
    pub type IdentityConfig = crate::identity::GenesisConfig<crate::Runtime>;
    pub type SimpleTokenConfig = crate::simple_token::GenesisConfig<crate::Runtime>;
    pub type StakingConfig = pallet_staking::GenesisConfig<crate::Runtime>;
    pub type GovernanceCommitteeConfig =
        pallet_collective::GenesisConfig<crate::Runtime, pallet_collective::Instance1>;
    pub type ContractsConfig = pallet_contracts::GenesisConfig<crate::Runtime>;
    pub type IndicesConfig = pallet_indices::GenesisConfig<crate::Runtime>;
    pub type SudoConfig = pallet_sudo::GenesisConfig<crate::Runtime>;
    pub type SystemConfig = frame_system::GenesisConfig;
    pub type GenesisConfig = crate::runtime::GenesisConfig;
    pub type SessionConfig = pallet_session::GenesisConfig<crate::Runtime>;
}

pub mod update_did_signed_extension;
pub use update_did_signed_extension::UpdateDid;

pub use sp_runtime::{Perbill, Permill};

#[cfg(test)]
pub mod test;
