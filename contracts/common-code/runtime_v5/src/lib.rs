#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use ink_lang as ink;

use polymesh_api::{
    ink::{
        extension::{PolymeshEnvironment, PolymeshRuntimeErr},
        Error as PolymeshError,
    },
    polymesh::types::{
        polymesh_primitives::{
            asset::{AssetName, AssetType},
            ticker::Ticker,
        },
    },
    Api,
};

#[ink::contract(env = PolymeshEnvironment)]
mod runtime_v5 {
    use alloc::{vec, vec::Vec};

    use crate::*;

    /// Wrap Polymesh runtime v5.x calls.
    #[ink(storage)]
    pub struct RuntimeV5 {
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

    impl From<PolymeshRuntimeErr> for Error {
        fn from(_err: PolymeshRuntimeErr) -> Self {
            Self::PolymeshError
        }
    }

    /// The contract result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl RuntimeV5 {
        /// Creates a new contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            panic!("Only upload this contract, don't deploy it.");
        }

        /// Very simple create asset call.
        #[ink(message, selector = 0x00_00_00_01)]
        pub fn system_remark(&mut self, remark: Vec<u8>) -> Result<()> {
            let api = Api::new();
            api.call().system().remark(remark).submit()?;
            Ok(())
        }

        /// Very simple create asset call.
        #[ink(message, selector = 0x00_00_1a_01)]
        pub fn create_asset(&mut self, ticker: Ticker) -> Result<()> {
            let api = Api::new();
            // Create asset.
            api.call()
                .asset()
                .create_asset(
                    AssetName(b"".to_vec()),
                    ticker.into(),
                    true, // Divisible token.
                    AssetType::EquityCommon,
                    vec![],
                    None,
                    true, // Disable Investor uniqueness requirements.
                )
                .submit()?;
            Ok(())
        }
    }
}
