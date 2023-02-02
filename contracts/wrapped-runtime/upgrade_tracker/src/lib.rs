//! This contract is for managing upgradable APIs.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub use self::upgrade_tracker::{
    UpgradeTracker,
    UpgradeTrackerRef,
};

use ink_lang as ink;

use polymesh_api::{
    ink::extension::PolymeshEnvironment,
    Api,
};

pub type Hash = <PolymeshEnvironment as ink_env::Environment>::Hash;

/// Chain versions.
pub type SpecVersion = u32;
pub type TxVersion = u32;
/// Wrapped api.
/// TODO: Change to `(String, u32)` to allow multiple smaller APIs?
pub type WrappedApi = u32;

/// The chain version to apply an upgrade at.
#[derive(Default, Eq, PartialEq, Ord, PartialOrd, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
#[derive(ink_storage::traits::SpreadLayout)]
#[derive(ink_storage::traits::PackedLayout)]
#[cfg_attr(feature = "std", derive(ink_storage::traits::StorageLayout))]
pub struct ChainVersion {
    pub spec: SpecVersion,
    pub tx: TxVersion,
}

impl ChainVersion {
    pub fn current() -> Option<Self> {
        let api = Api::new();
        Some(Self {
            spec: api.runtime().get_spec_version().ok()?,
            tx: api.runtime().get_transaction_version().ok()?,
        })
    }
}

/// The chain version and contract hash of the upgrade.
///
/// This upgrade is applied when the current chain version 
/// is greater then or equal to `chain_version`.
#[derive(Default, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
#[derive(ink_storage::traits::SpreadLayout)]
#[derive(ink_storage::traits::PackedLayout)]
#[cfg_attr(feature = "std", derive(ink_storage::traits::StorageLayout))]
pub struct Upgrade {
    pub chain_version: ChainVersion,
    pub hash: Hash,
}

#[ink::contract(env = PolymeshEnvironment)]
pub mod upgrade_tracker {
    use alloc::vec::Vec;
    use crate::*;
    use ink_storage::{
        traits::SpreadAllocate,
        Mapping,
    };

    /// A simple proxy contract.
    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct UpgradeTracker {
        /// The `AccountId` of a privileged account that can upgrade apis.
        /// This address is set to the account that instantiated this contract.
        admin: AccountId,
        /// Wrapped API upgrades.
        upgrades: Mapping<WrappedApi, Vec<Upgrade>>,
    }

    impl UpgradeTracker {
        /// Sets the privileged account to the caller. Only this account may
        /// later changed `update_code_hash`.
        #[ink(constructor)]
        pub fn new() -> Self {
            // This call is required in order to correctly initialize the
            // `Mapping`s of our contract.
            ink_lang::utils::initialize_contract(|contract| {
                Self::new_init(contract)
            })
        }

        fn new_init(&mut self) {
            self.admin = Self::env().caller();
        }

        fn ensure_admin(&self) {
            assert_eq!(
                self.env().caller(),
                self.admin,
                "caller {:?} does not have sufficient permissions, only {:?} does",
                self.env().caller(),
                self.admin,
            );
        }

        /// Change admin.
        #[ink(message)]
        pub fn set_admin(&mut self, new_admin: AccountId) {
            self.ensure_admin();
            self.admin = new_admin;
        }

        /// Upgrade a wrapped api.
        #[ink(message)]
        pub fn upgrade_wrapped_api(&mut self, api: WrappedApi, upgrade: Upgrade) {
            self.ensure_admin();
            // Get current upgrades.
            let mut upgrades = self.upgrades.get(&api).unwrap_or_default();
            // Remove old upgrade if it is for the same ChainVersion.
            upgrades.retain(|x| x.chain_version != upgrade.chain_version);
            // Add new upgrade.
            upgrades.push(upgrade);
            // Sort by upgrade chain_version.
            upgrades.sort_by(|a, b| a.chain_version.cmp(&b.chain_version));
            // Only keep the latest 10 upgrades for each API.
            upgrades.truncate(10);

            // Store upgrades.
            self.upgrades.insert(api, &upgrades);
        }

        /// Get the latest compatible API upgrade.
        #[ink(message)]
        pub fn get_latest_upgrade(&self, api: WrappedApi) -> Option<Hash> {
            let upgrades = self.upgrades.get(&api)?;

            // Search for a compatible upgrade to the wrapped api.
            let current = ChainVersion::current()?;
            for upgrade in upgrades.into_iter() {
                if upgrade.chain_version <= current {
                    return Some(upgrade.hash);
                }
            }

            // No upgrades found for `version`.
            None
        }
    }
}
