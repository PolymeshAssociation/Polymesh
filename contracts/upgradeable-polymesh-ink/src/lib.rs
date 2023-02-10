#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod macros;

#[cfg(not(feature = "as-library"))]
use ink_lang as ink;

use alloc::{vec, vec::Vec};

#[cfg(feature = "tracker")]
pub use upgrade_tracker::{Error as UpgradeError, UpgradeTrackerRef, WrappedApi};

use polymesh_api::Api;
pub use polymesh_api::{
    ink::{basic_types::IdentityId, extension::PolymeshEnvironment},
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

#[cfg(feature = "tracker")]
pub const API_VERSION: WrappedApi = (*b"POLY", 5);

/// The contract error types.
#[derive(Debug, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PolymeshError {
    /// Polymesh runtime error.
    PolymeshError,
    /// Missing Identity.  MultiSig's are not supported.
    MissingIdentity,
    /// Invalid portfolio authorization.
    InvalidPortfolioAuthorization,
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
                let did = self.get_our_did()?;
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

            /// Accept custody of a portfolio.
            #[ink(message)]
            pub fn accept_portfolio_custody(&self, auth_id: u64, portfolio: PortfolioKind) -> PolymeshResult<()> {
                // Get the caller's identity.
                let caller_did = self.get_caller_did()?;

                let portfolio = PortfolioId {
                    did: caller_did,
                    kind: portfolio,
                };
                let api = Api::new();
                // Accept authorization.
                api.call()
                    .portfolio()
                    .accept_portfolio_custody(auth_id)
                    .submit()?;
                // Check that we are the custodian.
                let did = self.get_our_did()?;
                if !api
                    .query()
                    .portfolio()
                    .portfolios_in_custody(did, portfolio)?
                {
                    return Err(PolymeshError::InvalidPortfolioAuthorization);
                }
                Ok(())
            }

            /// Quit custodianship of a portfolio returning control back to the owner.
            #[ink(message)]
            pub fn quit_portfolio_custody(&self, portfolio: PortfolioId) -> PolymeshResult<()> {
                let api = Api::new();
                // Remove our custodianship.
                api.call()
                    .portfolio()
                    .quit_portfolio_custody(portfolio)
                    .submit()?;
                Ok(())
            }

            /// Move funds between portfolios.
            #[ink(message)]
            pub fn move_portfolio_funds(
                &self,
                src: PortfolioId,
                dest: PortfolioId,
                funds: Vec<MovePortfolioItem>
            ) -> PolymeshResult<()> {
                let api = Api::new();
                // Move funds out of the contract controlled portfolio.
                api.call()
                    .portfolio()
                    .move_portfolio_funds(
                        src,
                        dest,
                        funds,
                    )
                    .submit()?;
                Ok(())
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

            /// Create and execute a settlement to transfer assets.
            #[ink(message)]
            pub fn settlement_execute(&self, venue: VenueId, legs: Vec<Leg>, portfolios: Vec<PortfolioId>) -> PolymeshResult<()> {
                let leg_count = legs.len() as u32;
                let api = Api::new();
                // Get the next instruction id.
                let instruction_id = api
                    .query()
                    .settlement()
                    .instruction_counter()
                    .map(|v| v.into())?;
                // Create settlement.
                api.call()
                    .settlement()
                    .add_and_affirm_instruction(
                        venue,
                        SettlementType::SettleManual(0),
                        None,
                        None,
                        legs,
                        portfolios,
                    )
                    .submit()?;

                // Create settlement.
                api.call()
                    .settlement()
                    .execute_manual_instruction(instruction_id, leg_count, None)
                    .submit()?;
                Ok(())
            }

            /// Asset issue tokens.
            #[ink(message)]
            pub fn asset_issue(&self, ticker: Ticker, amount: Balance) -> PolymeshResult<()> {
                let api = Api::new();
                // Mint tokens.
                api.call().asset().issue(ticker.into(), amount).submit()?;
                Ok(())
            }

            /// Asset redeem tokens.
            #[ink(message)]
            pub fn asset_redeem_from_portfolio(&self, ticker: Ticker, amount: Balance, portfolio: PortfolioKind) -> PolymeshResult<()> {
                let api = Api::new();
                // Redeem tokens.
                api.call().asset().redeem_from_portfolio(ticker.into(), amount, portfolio).submit()?;
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

            /// Get the identity of the caller.
            pub fn get_caller_did(&self) -> PolymeshResult<IdentityId> {
                self.get_key_did(ink_env::caller::<PolymeshEnvironment>())
            }

            /// Get the identity of the contract.
            pub fn get_our_did(&self) -> PolymeshResult<IdentityId> {
                self.get_key_did(ink_env::account_id::<PolymeshEnvironment>())
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
