#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

/// Implementations of some helper traits passed into runtime modules as associated types.
pub mod impls;

pub mod statistics;

pub mod asset;

#[cfg(feature = "std")]
pub use pallet_staking::{Commission, StakerStatus};

pub use pallet_im_online::OfflineSlashingParams;

mod bridge;
mod contracts_wrapper;
mod dividend;
mod exemption;
mod general_tm;
mod multisig;
mod percentage_tm;
mod simple_token;

pub mod runtime;
mod sto_capped;
mod voting;
pub use runtime::{
    api, Asset, Authorship, AvailableBlockRatio, Balances, Bridge, Contracts, MaximumBlockWeight,
    NegativeImbalance, ProtocolFee, Runtime, RuntimeApi, SessionKeys, System, TargetBlockFullness,
    TransactionPayment,
};
#[cfg(feature = "std")]
pub use runtime::{native_version, WASM_BINARY};

#[cfg(feature = "std")]
pub mod config {

    use pallet_committee as committee;
    use polymesh_protocol_fee as protocol_fee;
    use polymesh_runtime_balances as balances;
    use polymesh_runtime_identity as identity;

    pub type AssetConfig = crate::asset::GenesisConfig<crate::Runtime>;
    pub type BalancesConfig = balances::GenesisConfig<crate::Runtime>;
    pub type BridgeConfig = crate::bridge::GenesisConfig<crate::Runtime>;
    pub type IdentityConfig = identity::GenesisConfig<crate::Runtime>;
    pub type SimpleTokenConfig = crate::simple_token::GenesisConfig<crate::Runtime>;
    pub type StakingConfig = pallet_staking::GenesisConfig<crate::Runtime>;
    pub type PolymeshCommitteeConfig =
        committee::GenesisConfig<crate::Runtime, committee::Instance1>;
    pub type MipsConfig = pallet_mips::GenesisConfig<crate::Runtime>;
    pub type ContractsConfig = pallet_contracts::GenesisConfig<crate::Runtime>;
    pub type IndicesConfig = pallet_indices::GenesisConfig<crate::Runtime>;
    pub type ImOnlineConfig = pallet_im_online::GenesisConfig<crate::Runtime>;
    pub type SudoConfig = pallet_sudo::GenesisConfig<crate::Runtime>;
    pub type SystemConfig = frame_system::GenesisConfig;
    pub type GenesisConfig = crate::runtime::GenesisConfig;
    pub type SessionConfig = pallet_session::GenesisConfig<crate::Runtime>;
    pub type ProtocolFeeConfig = protocol_fee::GenesisConfig<crate::Runtime>;
}

pub mod fee_details;
pub use fee_details::CddHandler;

pub use sp_runtime::{Perbill, Permill};

#[cfg(test)]
pub mod test;
