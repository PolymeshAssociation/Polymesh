#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

/// Implementations of some helper traits passed into runtime modules as associated types.
pub mod impls;

pub mod asset;
pub mod balances;
/// Constant values used within the runtime.
pub mod constants;

mod contracts_wrapper;
mod dividend;
mod exemption;
mod general_tm;
mod group;
mod identity;
mod mips;
mod percentage_tm;
mod simple_token;

pub mod committee;
pub mod staking;
#[cfg(feature = "std")]
pub use staking::StakerStatus;

pub mod runtime;
mod sto_capped;
mod utils;
mod voting;
pub use runtime::{
    api, Asset, Authorship, Balances, MaximumBlockWeight, NegativeImbalance, Runtime, RuntimeApi,
    SessionKeys,
};
#[cfg(feature = "std")]
pub use runtime::{native_version, WASM_BINARY};

#[cfg(feature = "std")]
pub mod config {
    pub type AssetConfig = crate::asset::GenesisConfig<crate::Runtime>;
    pub type BalancesConfig = crate::balances::GenesisConfig<crate::Runtime>;
    pub type IdentityConfig = crate::identity::GenesisConfig<crate::Runtime>;
    pub type SimpleTokenConfig = crate::simple_token::GenesisConfig<crate::Runtime>;
    pub type StakingConfig = crate::staking::GenesisConfig<crate::Runtime>;
    pub type PolymeshCommitteeConfig =
        crate::committee::GenesisConfig<crate::Runtime, crate::committee::Instance1>;
    pub type MIPSConfig = crate::mips::GenesisConfig<crate::Runtime>;
    pub type ContractsConfig = contracts::GenesisConfig<crate::Runtime>;
    pub type IndicesConfig = indices::GenesisConfig<crate::Runtime>;
    pub type SudoConfig = sudo::GenesisConfig<crate::Runtime>;
    pub type SystemConfig = system::GenesisConfig;
    pub type GenesisConfig = crate::runtime::GenesisConfig;
    pub type SessionConfig = session::GenesisConfig<crate::Runtime>;
}

pub mod update_did_signed_extension;
pub use update_did_signed_extension::UpdateDid;

pub use sr_primitives::{Perbill, Permill};

#[cfg(test)]
pub mod test;
