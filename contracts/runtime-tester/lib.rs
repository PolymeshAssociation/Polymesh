#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use ink_lang as ink;
use ink_env::Environment;

use ink_storage::traits::{
    PackedLayout, SpreadLayout,
};
#[cfg(feature = "std")]
use ink_storage::traits::StorageLayout;
use scale::{Decode, Encode};
use alloc::vec::Vec;

pub const TICKER_LEN: usize = 12;

pub type Ticker = [u8; TICKER_LEN];
pub type AssetName = Vec<u8>;
pub type FundingRoundName = Vec<u8>;

#[derive(Clone, Copy, Decode, Encode, PackedLayout, SpreadLayout)]
#[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
pub enum AssetType {
  EquityCommon,
  EquityPreferred,
  // TODO: More.
}

#[derive(Clone, Copy, Decode, Encode, PackedLayout, SpreadLayout)]
#[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
pub enum AssetIdentifier {
  CUSIP([u8; 9]),
  CINS([u8; 9]),
  // TODO: More.
}

#[ink::chain_extension]
pub trait PolymeshRuntime {
    type ErrorCode = PolymeshRuntimeErr;

    // V5.0.0-rc1
    //#[ink(extension = 0x00_01_03_00, returns_result = false)]
    #[ink(extension = 0x00_1A_03_00, returns_result = false)]
    fn create_asset(
      name: AssetName,
      ticker: Ticker,
      divisible: bool,
      asset_type: AssetType,
      identifiers: Vec<AssetIdentifier>,
      funding_round: Option<FundingRoundName>,
      disable_iu: bool,
    );
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PolymeshRuntimeErr {
    Unknown,
}

impl ink_env::chain_extension::FromStatusCode for PolymeshRuntimeErr {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::Unknown),
            _ => panic!("encountered unknown status code"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PolymeshEnvironment {}

impl Environment for PolymeshEnvironment {
    const MAX_EVENT_TOPICS: usize =
        <ink_env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = <ink_env::DefaultEnvironment as Environment>::AccountId;
    type Balance = <ink_env::DefaultEnvironment as Environment>::Balance;
    type Hash = <ink_env::DefaultEnvironment as Environment>::Hash;
    type BlockNumber = <ink_env::DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <ink_env::DefaultEnvironment as Environment>::Timestamp;

    type ChainExtension = PolymeshRuntime;
}

#[ink::contract(env = crate::PolymeshEnvironment)]
mod runtime_tester {
    use alloc::vec;

    use crate::*;

    /// A simple ERC-20 contract.
    #[ink(storage)]
    pub struct RuntimeTester {}

    /// The contract error types.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Caller needs to pay the contract for the protocol fee.
        /// (Amount needed)
        InsufficientTransferValue(Balance),
        /// Polymesh runtime error.
        PolymeshError(PolymeshRuntimeErr),
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
        pub fn create_asset(&mut self, name: AssetName, ticker: Ticker, asset_type: AssetType) -> Result<()> {
            Self::env().extension().create_asset(name, ticker, true, asset_type, vec![], None, true)
              .map_err(|err| Error::PolymeshError(err))
        }

        #[ink(message, payable)]
        pub fn payable_create_asset(&mut self, name: AssetName, ticker: Ticker, asset_type: AssetType) -> Result<()> {
            let transferred = Self::env().transferred_value();
            if transferred < CREATE_ASSET_FEE {
              return Err(Error::InsufficientTransferValue(CREATE_ASSET_FEE));
            }
            Self::env().extension().create_asset(name, ticker, true, asset_type, vec![], None, true)
              .map_err(|err| Error::PolymeshError(err))
        }
    }
}
