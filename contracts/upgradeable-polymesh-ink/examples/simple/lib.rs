//! Example contract for upgradable `polymesh-ink` API.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use ink_lang as ink;

use polymesh_ink::*;

#[ink::contract(env = PolymeshEnvironment)]
pub mod test_polymesh_ink {
    use crate::*;
    use alloc::vec::Vec;

    /// A simple proxy contract.
    #[ink(storage)]
    pub struct Proxy {
        /// The `AccountId` of a privileged account that override the
        /// code hash for `PolymeshInk`.
        ///
        /// This address is set to the account that instantiated this contract.
        admin: AccountId,
        /// Upgradable Polymesh Ink API.
        api: PolymeshInk,
    }

    /// The contract error types.
    #[derive(Debug, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// PolymeshInk errors.
        PolymeshInk(PolymeshError),
        /// Upgrade error.
        UpgradeError(UpgradeError)
    }

    impl From<PolymeshError> for Error {
        fn from(err: PolymeshError) -> Self {
            Self::PolymeshInk(err)
        }
    }

    impl From<UpgradeError> for Error {
        fn from(err: UpgradeError) -> Self {
            Self::UpgradeError(err)
        }
    }

    /// The contract result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl Proxy {
        /// Instantiate this contract with an address of the `logic` contract.
        ///
        /// Sets the privileged account to the caller. Only this account may
        /// later changed the `forward_to` address.
        #[ink(constructor)]
        pub fn new(hash: Hash, tracker: Option<UpgradeTrackerRef>) -> Self {
            Self {
                admin: Self::env().caller(),
                api: PolymeshInk::new(hash, tracker),
            }
        }

        /// Update the code hash of the polymesh runtime API.
        ///
        /// Only the `admin` is allowed to call this.
        #[ink(message)]
        pub fn update_code_hash(&mut self, hash: Hash) {
            assert_eq!(
                self.env().caller(),
                self.admin,
                "caller {:?} does not have sufficient permissions, only {:?} does",
                self.env().caller(),
                self.admin,
            );
            self.api.update_code_hash(hash);
        }

        /// Update the `polymesh-ink` API using the tracker.
        ///
        /// Anyone can pay the gas fees to do the update using the tracker.
        #[ink(message)]
        pub fn update_polymesh_ink(&mut self) -> Result<()> {
            self.api.check_for_upgrade()?;
            Ok(())
        }

        #[ink(message)]
        pub fn system_remark(&mut self, remark: Vec<u8>) -> Result<()> {
            self.api
                .system_remark(remark)?;
            Ok(())
        }

        #[ink(message)]
        pub fn get_our_did(&mut self) -> Result<IdentityId> {
            Ok(self.api
                .get_our_did()?)
        }

        #[ink(message)]
        pub fn get_caller_did(&mut self) -> Result<IdentityId> {
            Ok(self.api
                .get_caller_did()?)
        }

        #[ink(message)]
        pub fn create_venue(&mut self, details: Vec<u8>) -> Result<VenueId> {
            Ok(self.api
                .create_venue(VenueDetails(details), VenueType::Other)?)
        }

        /// Test creating and issueing an asset using the upgradable `polymesh-ink` API.
        #[ink(message)]
        pub fn create_asset(&mut self, name: Vec<u8>, ticker: Ticker, amount: Balance) -> Result<()> {
            self.api
                .asset_create_and_issue(AssetName(name), ticker, AssetType::EquityCommon, true, Some(amount))?;
            Ok(())
        }
    }
}
