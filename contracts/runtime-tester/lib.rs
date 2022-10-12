#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use ink_lang as ink;
use alloc::vec::Vec;

use polymesh_api::{
  Api,
  ink::{
    extension::PolymeshEnvironment,
    basic_types::IdentityId,
    Error as PolymeshError,
  },
  polymesh::types::{
    polymesh_primitives::{
      ticker::Ticker,
      asset::{
        AssetName,
        AssetType,
      },
      identity_id::PortfolioId,
    },
  },
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
        PolymeshError(PolymeshError),
        //PolymeshError(PolymeshRuntimeErr),
        /// Scale decode failed.
        ScaleError,
    }

    // hard-code protocol fees.
    pub const POLYX: Balance = 1_000_000u128;
    pub const CREATE_ASSET_FEE: Balance = (500 * POLYX) + (2_500 * POLYX);

    /// The contract result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl RuntimeTester {
        /// Creates a new contract.
        #[ink(constructor)]
        pub fn new() -> Self { Self {} }

        #[ink(message)]
        pub fn call_runtime(&mut self, call: Vec<u8>) -> Result<()> {
            Self::env().extension().call_runtime(call.into())
              .map_err(|err| Error::PolymeshError(PolymeshError::RuntimeError(err)))
        }

        #[ink(message)]
        pub fn read_storage(&self, key: Vec<u8>) -> Result<Option<Vec<u8>>> {
            Self::env().extension().read_storage(key.into())
              .map_err(|err| Error::PolymeshError(PolymeshError::RuntimeError(err)))
        }

        #[ink(message)]
        pub fn get_spec_version(&self) -> Result<u32> {
            Self::env().extension().get_spec_version()
              .map_err(|err| Error::PolymeshError(PolymeshError::RuntimeError(err)))
        }

        #[ink(message)]
        pub fn get_transaction_version(&self) -> Result<u32> {
            Self::env().extension().get_transaction_version()
              .map_err(|err| Error::PolymeshError(PolymeshError::RuntimeError(err)))
        }

        #[ink(message)]
        pub fn asset_balance_of(&self, ticker: Ticker, did: IdentityId) -> Result<u128> {
            let api = Api::new();
            api.query().asset().balance_of(ticker, did)
              .map_err(|err| Error::PolymeshError(err))
        }

        #[ink(message)]
        pub fn portfolio_asset_balance(&self, portfolio: PortfolioId, ticker: Ticker) -> Result<u128> {
            let api = Api::new();
            api.query().portfolio().portfolio_asset_balances(portfolio, ticker)
              .map_err(|err| Error::PolymeshError(err))
        }

        #[ink(message)]
        pub fn register_ticker(&mut self, ticker: Ticker) -> Result<()> {
            let api = Api::new();
            api.call().asset().register_ticker(ticker).submit()
              .map_err(|err| Error::PolymeshError(err))
        }

        #[ink(message)]
        pub fn accept_ticker_transfer(&mut self, auth_id: u64) -> Result<()> {
            let api = Api::new();
            api.call().asset().accept_ticker_transfer(auth_id).submit()
              .map_err(|err| Error::PolymeshError(err))
        }

        #[ink(message)]
        pub fn accept_asset_ownership_transfer(&mut self, auth_id: u64) -> Result<()> {
            let api = Api::new();
            api.call().asset().accept_asset_ownership_transfer(auth_id).submit()
              .map_err(|err| Error::PolymeshError(err))
        }

        fn create_asset(&mut self, name: AssetName, ticker: Ticker, asset_type: AssetType, supply: u128) -> Result<()> {
            let api = Api::new();
            api.call().asset().create_asset(name, ticker.clone(), true, asset_type, vec![], None, true).submit()
              .map_err(|err| Error::PolymeshError(err))?;
            api.call().asset().issue(ticker, supply).submit()
              .map_err(|err| Error::PolymeshError(err))
        }

        #[ink(message)]
        /// Create asset where the contract pays the asset creation fees (3k POLYX).
        pub fn create_asset_and_issue(&mut self, name: AssetName, ticker: Ticker, asset_type: AssetType, supply: u128) -> Result<()> {
            self.create_asset(name, ticker, asset_type, supply)
        }

        #[ink(message, payable)]
        /// Create asset where the caller need to pay the asset creation fees (3k POLYX).
        pub fn payable_create_asset_and_issue(&mut self, name: AssetName, ticker: Ticker, asset_type: AssetType, supply: u128) -> Result<()> {
            let transferred = Self::env().transferred_value();
            if transferred < CREATE_ASSET_FEE {
              return Err(Error::InsufficientTransferValue(CREATE_ASSET_FEE));
            }
            self.create_asset(name, ticker, asset_type, supply)
        }

        #[ink(message)]
        pub fn register_custom_asset_type(&mut self, ty: Vec<u8>) -> Result<()> {
            let api = Api::new();
            api.call().asset().register_custom_asset_type(ty).submit()
              .map_err(|err| Error::PolymeshError(err))
        }
    }
}
