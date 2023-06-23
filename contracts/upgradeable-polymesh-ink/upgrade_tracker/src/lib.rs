//! This contract is for managing upgradable APIs.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub use self::upgrade_tracker::{Error, UpgradeTracker, UpgradeTrackerRef};

use ink_lang as ink;

use polymesh_api::{ink::extension::PolymeshEnvironment, Api};

pub type Hash = <PolymeshEnvironment as ink_env::Environment>::Hash;

/// Chain versions.
pub type SpecVersion = u32;
pub type TxVersion = u32;
/// Wrapped api.
pub type WrappedApi = ([u8; 4], u32);

/// The chain version to apply an upgrade at.
#[derive(
    Default,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    scale::Encode,
    scale::Decode
)]
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
#[derive(
    Default,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    scale::Encode,
    scale::Decode
)]
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
    use crate::*;
    use alloc::vec::Vec;
    use ink_storage::{traits::SpreadAllocate, Mapping};
    use ink_prelude::collections::BTreeSet;

    /// Upgrade tracker contract for Polymesh Ink! API.
    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct UpgradeTracker {
        /// The `AccountId` of a privileged account that can upgrade apis.
        /// This address is set to the account that instantiated this contract.
        admin: AccountId,
        /// List of APIs.
        apis: Vec<WrappedApi>,
        /// Wrapped API upgrades.
        upgrades: Mapping<WrappedApi, BTreeSet<Upgrade>>,
    }

    /// Event emitted when the admin account is set.
    #[ink(event)]
    pub struct AdminSet {
        #[ink(topic)]
        old: Option<AccountId>,
        #[ink(topic)]
        new: AccountId,
    }

    /// Event emitted when a wrapped api is upgraded.
    #[ink(event)]
    pub struct UpgradeWrappedApi {
        #[ink(topic)]
        admin: AccountId,
        #[ink(topic)]
        api: WrappedApi,
        #[ink(topic)]
        upgrade: Upgrade,
    }

    /// Contract error type.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Only the current admin can make this call.
        NotAdmin,
        /// Failed to get current ChainVersion.
        NoChainVersion,
        /// No upgrade available for the API.
        NoUpgrade,
    }

    /// Contract result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl UpgradeTracker {
        /// Sets the privileged account to the caller. Only this account may
        /// later changed `update_code_hash`.
        #[ink(constructor)]
        pub fn new() -> Self {
            // This call is required in order to correctly initialize the
            // `Mapping`s of our contract.
            ink_lang::utils::initialize_contract(|contract| Self::new_init(contract))
        }

        fn new_init(&mut self) {
            self.admin = Self::env().caller();
            self.env().emit_event(AdminSet {
                old: None,
                new: self.admin,
            });
        }

        fn ensure_admin(&self) -> Result<()> {
            let caller = self.env().caller();
            if caller != self.admin {
                ink_env::debug_println!(
                    "caller {:?} does not have sufficient permissions, only {:?} does",
                    caller,
                    self.admin
                );
                Err(Error::NotAdmin)
            } else {
                Ok(())
            }
        }

        /// Change admin.
        #[ink(message)]
        pub fn set_admin(&mut self, new_admin: AccountId) -> Result<()> {
            self.ensure_admin()?;
            self.env().emit_event(AdminSet {
                old: Some(self.admin),
                new: new_admin,
            });
            self.admin = new_admin;
            Ok(())
        }

        /// Get current admin.
        #[ink(message)]
        pub fn get_admin(&self) -> AccountId {
          self.admin
        }

        /// Upgrade a wrapped api.
        #[ink(message)]
        pub fn upgrade_wrapped_api(&mut self, api: WrappedApi, upgrade: Upgrade) -> Result<()> {
            self.ensure_admin()?;
            // Get current upgrades.
            let mut upgrades = self.upgrades.get(&api).unwrap_or_default();
            // Remove old upgrade if it is for the same ChainVersion.
            upgrades.retain(|x| x.chain_version != upgrade.chain_version);
            // Add new upgrade.
            upgrades.insert(upgrade);
            // Only keep the latest 10 upgrades for each API.
            if upgrades.len() > 10 {
                // The upgrades are sorted oldest to newest.
                let old = upgrades.iter().next().copied();
                if let Some(old) = old {
                    upgrades.remove(&old);
                }
            }

            self.env().emit_event(UpgradeWrappedApi {
                admin: self.admin,
                api,
                upgrade,
            });
            // Store upgrades.
            self.upgrades.insert(api, &upgrades);
            if !self.apis.contains(&api) {
              self.apis.push(api);
            }
            Ok(())
        }

        /// Get the latest compatible API upgrade.
        #[ink(message)]
        pub fn get_latest_upgrade(&self, api: WrappedApi) -> Result<Hash> {
            let upgrades = self.upgrades.get(&api).ok_or(Error::NoUpgrade)?;

            // Search for a compatible upgrade to the wrapped api.
            let current = ChainVersion::current().ok_or(Error::NoChainVersion)?;
            // Do a reverse search, since the newest chain versions are last.
            upgrades.into_iter()
                .rfind(|u| u.chain_version <= current)
                .map(|u| u.hash)
                .ok_or(Error::NoUpgrade)
        }

        /// Get all apis.
        #[ink(message)]
        pub fn get_apis(&self) -> Vec<WrappedApi> {
            self.apis.clone()
        }

        /// Get all upgrades.
        #[ink(message)]
        pub fn get_api_upgrades(&self, api: WrappedApi) -> Result<BTreeSet<Upgrade>> {
            Ok(self.upgrades.get(&api).ok_or(Error::NoUpgrade)?)
        }
    }
}
