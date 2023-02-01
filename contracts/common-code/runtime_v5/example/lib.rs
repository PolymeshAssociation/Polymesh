//! Example contract showing how to delegate calls to the `runtime_v5`
//! contract code to make Polymesh runtime calls.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use ink_lang as ink;

use polymesh_ink::*;

#[ink::contract(env = PolymeshEnvironment)]
pub mod test_runtime_v5 {
    use alloc::vec::Vec;
    use crate::*;

    /// A simple proxy contract.
    #[ink(storage)]
    pub struct Proxy {
        /// The `Hash` of the current `runtime_v5` contract code.
        runtime: Hash,
        /// The `AccountId` of a privileged account that can update the
        /// runtime code hash. This address is set to the account that
        /// instantiated this contract.
        admin: AccountId,
        /// Upgradable Polymesh Ink API.
        api: PolymeshInk,
    }

    /// The contract error types.
    #[derive(Debug, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Polymesh runtime error.
        PolymeshError,
    }

    impl From<PolymeshError> for Error {
        fn from(_err: PolymeshError) -> Self {
            Self::PolymeshError
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
        pub fn new(runtime: Hash) -> Self {
            Self {
                runtime,
                admin: Self::env().caller(),
                api: Default::default(),
            }
        }

        /// Update the code hash of the polymesh runtime API.
        #[ink(message)]
        pub fn update_code_hash(&mut self, hash: Option<Hash>) {
            assert_eq!(
                self.env().caller(),
                self.admin,
                "caller {:?} does not have sufficient permissions, only {:?} does",
                self.env().caller(),
                self.admin,
            );
            self.api.update_code_hash(hash);
        }

        #[ink(message)]
        pub fn system_remark(&mut self, remark: Vec<u8>) -> Result<()> {
            self.api.system_remark(remark).map_err(|_| Error::PolymeshError)?;
            Ok(())
        }

        /// Test calling `asset.create_asset()` using the `runtime_v5` contract code.
        #[ink(message)]
        pub fn create_asset(&mut self, ticker: Ticker, amount: Balance) -> Result<()> {
            self.api.create_simple_asset(ticker, amount).map_err(|_| Error::PolymeshError)?;
            Ok(())
        }
    }
}
