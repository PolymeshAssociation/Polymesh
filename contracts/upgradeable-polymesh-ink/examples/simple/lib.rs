//! Example contract for upgradable `polymesh-ink` API.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;

use polymesh_ink::*;

#[ink::contract(env = PolymeshEnvironment)]
pub mod test_polymesh_ink {
    use crate::*;
    use alloc::vec::Vec;

    /// A simple test contract.
    #[ink(storage)]
    pub struct SimpleTest {
    }

    /// The contract error types.
    #[derive(Debug, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// PolymeshInk errors.
        PolymeshInk(PolymeshError),
    }

    impl From<PolymeshError> for Error {
        fn from(err: PolymeshError) -> Self {
            Self::PolymeshInk(err)
        }
    }

    /// The contract result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl SimpleTest {
        /// Instantiate this contract with an address of the `logic` contract.
        #[ink(constructor)]
        pub fn new() -> Result<Self> {
            Ok(Self {
            })
        }

        #[ink(message)]
        pub fn system_remark(&mut self, remark: Vec<u8>) -> Result<()> {
            let api = PolymeshInk::new()?;
            api.system_remark(remark)?;
            Ok(())
        }

        #[ink(message)]
        pub fn get_our_did(&mut self) -> Result<IdentityId> {
            Ok(PolymeshInk::get_our_did()?)
        }

        #[ink(message)]
        pub fn get_caller_did(&mut self) -> Result<IdentityId> {
            Ok(PolymeshInk::get_caller_did()?)
        }

        #[ink(message)]
        pub fn create_venue(&mut self, details: Vec<u8>) -> Result<VenueId> {
            let api = PolymeshInk::new()?;
            Ok(api
                .create_venue(VenueDetails(details), VenueType::Other)?)
        }

        /// Test creating and issueing an asset using the upgradable `polymesh-ink` API.
        #[ink(message)]
        pub fn create_asset(
            &mut self,
            name: Vec<u8>,
            ticker: Ticker,
            amount: Balance,
        ) -> Result<()> {
            let api = PolymeshInk::new()?;
            api.asset_create_and_issue(
                AssetName(name),
                ticker,
                AssetType::EquityCommon,
                true,
                Some(amount),
            )?;
            Ok(())
        }
    }
}
