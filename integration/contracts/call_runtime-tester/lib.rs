#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;

use alloc::vec::Vec;

use polymesh_api::{
    ink::{basic_types::IdentityId, extension::{PolymeshEnvironment, CallRuntimeError}, Error as PolymeshInkError},
    polymesh::types::{
        polymesh_contracts::Api as ContractRuntimeApi,
        polymesh_primitives::{
            asset::{AssetName, AssetType},
            identity_id::{PortfolioId, PortfolioKind},
            ticker::Ticker,
        },
    },
    Api,
};

#[ink::contract(env = PolymeshEnvironment)]
mod runtime_tester {
    use alloc::vec;

    use crate::*;

    /// A simple ERC-20 contract.
    #[ink(storage)]
    pub struct RuntimeTester {}

    /// The contract error types.
    #[derive(Debug, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Caller needs to pay the contract for the protocol fee.
        /// (Amount needed)
        InsufficientTransferValue(Balance),
        /// Polymesh runtime error.
        PolymeshRuntime(PolymeshInkError),
        /// Scale decode failed.
        ScaleError,
    }

    /// The contract result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl RuntimeTester {
        /// Creates a new contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn call_runtime(&mut self, call: Vec<u8>) -> Result<()> {
            Self::env()
                .extension()
                .call_runtime(call.into())
                .map_err(|err| Error::PolymeshRuntime(err))
        }

        #[ink(message)]
        pub fn call_runtime_with_error(&mut self, call: Vec<u8>) -> Result<()> {
            Self::env()
                .extension()
                .call_runtime_with_error(call.into())
                .map_err(|err| Error::PolymeshRuntime(err))?
                .map_err(|err| Error::PolymeshRuntime(err.into()))
        }
    }
}
