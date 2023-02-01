//! Example contract showing how to delegate calls to the `runtime_v5`
//! contract code to make Polymesh runtime calls.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use ink_lang as ink;

/// TODO: Create a better error type.
#[derive(Debug, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PolymeshError {
  /// Polymesh runtime error.
  PolymeshError,
}

pub type PolymeshResult<T> = core::result::Result<T, PolymeshError>;

use polymesh_api::{
    ink::{
        extension::PolymeshEnvironment,
    },
    Api,
};

#[ink::contract(env = PolymeshEnvironment)]
pub mod test_runtime_v5 {
    use ink_env::call::{DelegateCall, Selector, ExecutionInput};

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
            }
        }

        /// Update the code hash of the `runtime` contract code.
        #[ink(message)]
        pub fn update_runtime(&mut self, new_runtime: Hash) {
            assert_eq!(
                self.env().caller(),
                self.admin,
                "caller {:?} does not have sufficient permissions, only {:?} does",
                self.env().caller(),
                self.admin,
            );
            self.runtime = new_runtime;
        }

        /// Test direct calling `system.remark()` using the chain extension.
        #[ink(message)]
        pub fn direct_remark(&mut self, remark: Vec<u8>) -> Result<()> {
            let api = Api::new();
            api.call().system().remark(remark).submit().map_err(|_| Error::PolymeshError)?;
            Ok(())
        }

        /// Test calling `system.remark()` using the `runtime_v5` contract code.
        #[ink(message)]
        pub fn delegate_remark(&mut self, remark: Vec<u8>) -> Result<()> {
            ink_env::call::build_call::<ink_env::DefaultEnvironment>()
                .call_type(DelegateCall::new().code_hash(self.runtime))
                .exec_input(
                      ExecutionInput::new(Selector::new([0x00, 0x00, 0x00, 0x01]))
                          .push_arg(remark)
                )
                .returns::<PolymeshResult<()>>()
                .fire()
                .unwrap_or_else(|err| {
                    panic!(
                        "delegate call to {:?} failed due to {:?}",
                        self.runtime, err
                    )
                })?;
            Ok(())
        }

        /// Test calling `asset.create_asset()` using the `runtime_v5` contract code.
        #[ink(message)]
        pub fn create_asset(&mut self, ticker: [u8; 12]) -> Result<()> {
            ink_env::call::build_call::<ink_env::DefaultEnvironment>()
                .call_type(DelegateCall::new().code_hash(self.runtime))
                .exec_input(
                      ExecutionInput::new(Selector::new([0x00, 0x00, 0x1a, 0x01]))
                          .push_arg(ticker)
                )
                .returns::<PolymeshResult<()>>()
                .fire()
                .unwrap_or_else(|err| {
                    panic!(
                        "delegate call to {:?} failed due to {:?}",
                        self.runtime, err
                    )
                })?;
            Ok(())
        }
    }
}
