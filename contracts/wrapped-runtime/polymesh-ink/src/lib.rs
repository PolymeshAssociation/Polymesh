#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod macro_rule;

#[cfg(not(feature = "as-library"))]
use ink_lang as ink;

use alloc::{vec, vec::Vec};

#[cfg(feature = "tracker")]
pub use upgrade_tracker::UpgradeTrackerRef;

use polymesh_api::Api;
pub use polymesh_api::{
    ink::extension::PolymeshEnvironment,
    polymesh::types::polymesh_primitives::{
        asset::{AssetName, AssetType},
        ticker::Ticker,
    },
};

#[cfg(feature = "as-library")]
pub const API_VERSION: u32 = 5;

/// The contract error types.
#[derive(Debug, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PolymeshError {
    /// Polymesh runtime error.
    PolymeshError,
}

impl From<polymesh_api::ink::Error> for PolymeshError {
    fn from(_err: polymesh_api::ink::Error) -> Self {
        Self::PolymeshError
    }
}

impl From<polymesh_api::ink::extension::PolymeshRuntimeErr> for PolymeshError {
    fn from(_err: polymesh_api::ink::extension::PolymeshRuntimeErr) -> Self {
        Self::PolymeshError
    }
}

/// The contract result type.
pub type PolymeshResult<T> = core::result::Result<T, PolymeshError>;

#[cfg(feature = "as-library")]
pub type Balance = <PolymeshEnvironment as ink_env::Environment>::Balance;
#[cfg(feature = "as-library")]
pub type Hash = <PolymeshEnvironment as ink_env::Environment>::Hash;

upgradable_api! {
    mod polymesh_ink {
        impl PolymeshInk {
            /// Wrap the `system.remark` extrinsic.  Only useful for testing.
            #[ink(message)]
            pub fn system_remark(&self, remark: Vec<u8>) -> PolymeshResult<()> {
                let api = Api::new();
                api.call().system().remark(remark).submit()?;
                Ok(())
            }

            /// Asset issue.
            #[ink(message)]
            pub fn asset_issue(&self, ticker: Ticker, amount: Balance) -> PolymeshResult<()> {
                let api = Api::new();
                // Mint some tokens.
                api.call().asset().issue(ticker.into(), amount).submit()?;
                Ok(())
            }

            /// Very simple create asset and issue.
            #[ink(message)]
            pub fn asset_create_and_issue(&self, ticker: Ticker, amount: Balance) -> PolymeshResult<()> {
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
                // Mint some tokens.
                api.call().asset().issue(ticker.into(), amount).submit()?;
                // Pause compliance rules to allow transfers.
                api.call()
                    .compliance_manager()
                    .pause_asset_compliance(ticker.into())
                    .submit()?;
                Ok(())
            }
        }
    }
}
