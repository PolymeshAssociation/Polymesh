#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod macros;

#[cfg(not(feature = "as-library"))]
use ink_lang as ink;

use alloc::{vec, vec::Vec};

#[cfg(feature = "tracker")]
pub use upgrade_tracker::UpgradeTrackerRef;

use polymesh_api::Api;
pub use polymesh_api::{
    ink::{
        basic_types::IdentityId,
        extension::PolymeshEnvironment,
    },
    polymesh::types::{
        pallet_portfolio::MovePortfolioItem,
        pallet_settlement::{Leg, SettlementType, VenueDetails, VenueId, VenueType},
        polymesh_primitives::{
            asset::{AssetName, AssetType},
            identity_id::{PortfolioId, PortfolioKind, PortfolioName},
            ticker::Ticker,
        },
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
    /// Missing Identity.  MultiSig's are not supported.
    MissingIdentity,
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
#[cfg(feature = "as-library")]
pub type AccountId = <PolymeshEnvironment as ink_env::Environment>::AccountId;

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

            /// Create a portfolio.
            #[ink(message)]
            pub fn create_portfolio(&self, name: Vec<u8>) -> PolymeshResult<PortfolioId> {
                let api = Api::new();
                // Get the contract's did.
                let did = self.get_key_did(ink_env::account_id::<PolymeshEnvironment>()).unwrap();
                // Get the next portfolio number.
                let num = api.query().portfolio().next_portfolio_number(did).map(|v| v.into())?;
                // Create Venue.
                api.call()
                    .portfolio()
                    .create_portfolio(
                        PortfolioName(name),
                    )
                    .submit()?;
                Ok(PortfolioId {
                  did,
                  kind: PortfolioKind::User(num),
                })
            }

            /// Create a Settlement Venue.
            #[ink(message)]
            pub fn create_venue(&self, details: VenueDetails, ty: VenueType) -> PolymeshResult<VenueId> {
                let api = Api::new();
                // Get the next venue id.
                let id = api.query().settlement().venue_counter().map(|v| v.into())?;
                // Create Venue.
                api.call()
                    .settlement()
                    .create_venue(
                        details,
                        vec![],
                        ty,
                    )
                    .submit()?;
                Ok(id)
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
            pub fn asset_create_and_issue(&self, name: AssetName, ticker: Ticker, asset_type: AssetType, divisible: bool, issue: Option<Balance>) -> PolymeshResult<()> {
                let api = Api::new();
                // Create asset.
                api.call()
                    .asset()
                    .create_asset(
                        name,
                        ticker,
                        divisible,
                        asset_type,
                        vec![],
                        None,
                        true, // Disable Investor uniqueness requirements.
                    )
                    .submit()?;
                // Mint some tokens.
                if let Some(amount) = issue {
                  api.call().asset().issue(ticker.into(), amount).submit()?;
                }
                // Pause compliance rules to allow transfers.
                api.call()
                    .compliance_manager()
                    .pause_asset_compliance(ticker.into())
                    .submit()?;
                Ok(())
            }

            /// Get the identity of a key.
            pub fn get_key_did(&self, acc: AccountId) -> PolymeshResult<IdentityId> {
                let api = Api::new();
                api.runtime()
                    .get_key_did(acc)?
                    .map(|did| did.into())
                    .ok_or(PolymeshError::MissingIdentity)
            }
        }
    }
}
