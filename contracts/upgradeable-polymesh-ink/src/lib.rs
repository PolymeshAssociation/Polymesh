#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod macros;

#[cfg(not(feature = "as-library"))]
use ink_lang as ink;

use alloc::{vec, vec::Vec};

#[cfg(feature = "tracker")]
pub use upgrade_tracker::{Error as UpgradeError, UpgradeTrackerRef, WrappedApi};

// Polymesh V6 API.
use polymesh_api::polymesh::Api;

// Re-export Old V5 types.  Changed in V6.
pub use polymesh_api::v5_to_v6::{
    Leg, MovePortfolioItem,
};

use polymesh_api::polymesh::types::pallet_corporate_actions;
// Re-export V6 types that haven't changed.
pub use polymesh_api::polymesh::types;
pub use polymesh_api::{
    ink::{basic_types::IdentityId, extension::PolymeshEnvironment},
    polymesh::types::{
        pallet_corporate_actions::CAId,
        polymesh_primitives::{
            asset::{AssetName, AssetType, CheckpointId},
            identity_id::{PortfolioId, PortfolioKind, PortfolioName, PortfolioNumber},
            ticker::Ticker,
            settlement::{SettlementType, VenueDetails, VenueId, VenueType},
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
    /// Ink! Delegate call error.
    InkDelegateCallError {
        selector: [u8; 4],
        err: Option<InkEnvError>,
    },
}

/// Encodable `ink_env::Error`.
#[derive(Debug, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum InkEnvError {
    /// Error upon decoding an encoded value.
    ScaleDecodeError,
    /// The call to another contract has trapped.
    CalleeTrapped,
    /// The call to another contract has been reverted.
    CalleeReverted,
    /// The queried contract storage entry is missing.
    KeyNotFound,
    /// Transfer failed for other not further specified reason. Most probably
    /// reserved or locked balance of the sender that was preventing the transfer.
    TransferFailed,
    /// Deprecated and no longer returned: Endowment is no longer required.
    _EndowmentTooLow,
    /// No code could be found at the supplied code hash.
    CodeNotFound,
    /// The account that was called is no contract, but a plain account.
    NotCallable,
    /// The call to `seal_debug_message` had no effect because debug message
    /// recording was disabled.
    LoggingDisabled,
    /// ECDSA pubkey recovery failed. Most probably wrong recovery id or signature.
    EcdsaRecoveryFailed,
}

impl PolymeshError {
    pub fn from_delegate_error(err: ink_env::Error, selector: ink_env::call::Selector) -> Self {
        use ink_env::Error::*;
        Self::InkDelegateCallError {
            selector: selector.to_bytes(),
            err: match err {
                Decode(_) => Some(InkEnvError::ScaleDecodeError),
                CalleeTrapped => Some(InkEnvError::CalleeTrapped),
                CalleeReverted => Some(InkEnvError::CalleeReverted),
                KeyNotFound => Some(InkEnvError::KeyNotFound),
                TransferFailed => Some(InkEnvError::TransferFailed),
                _EndowmentTooLow => Some(InkEnvError::_EndowmentTooLow),
                CodeNotFound => Some(InkEnvError::CodeNotFound),
                NotCallable => Some(InkEnvError::NotCallable),
                LoggingDisabled => Some(InkEnvError::LoggingDisabled),
                EcdsaRecoveryFailed => Some(InkEnvError::EcdsaRecoveryFailed),
                _ => None,
            },
        }
    }
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
pub type Hash = <PolymeshEnvironment as ink_env::Environment>::Hash;
#[cfg(feature = "as-library")]
pub type AccountId = <PolymeshEnvironment as ink_env::Environment>::AccountId;
pub type Balance = <PolymeshEnvironment as ink_env::Environment>::Balance;
pub type Timestamp = <PolymeshEnvironment as ink_env::Environment>::Timestamp;

#[derive(Debug, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct DistributionSummary {
    pub currency: Ticker,
    pub per_share: Balance,
    pub reclaimed: bool,
    pub payment_at: Timestamp,
    pub expires_at: Option<Timestamp>,
}

impl From<pallet_corporate_actions::distribution::Distribution> for DistributionSummary {
    fn from(distribution: pallet_corporate_actions::distribution::Distribution) -> Self {
        Self {
            currency: distribution.currency,
            per_share: distribution.per_share,
            reclaimed: distribution.reclaimed,
            payment_at: distribution.payment_at,
            expires_at: distribution.expires_at,
        }
    }
}

#[derive(Debug, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct SimpleDividend {
    pub ticker: Ticker,
    pub decl_date: Timestamp,
    pub record_date: Timestamp,
    pub portfolio: Option<PortfolioNumber>,
    pub currency: Ticker,
    pub per_share: Balance,
    pub amount: Balance,
    pub payment_at: Timestamp,
    pub expires_at: Option<Timestamp>,
}

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
                let num = api.query().portfolio().next_portfolio_number(did)?;
                // Create Venue.
                api.call()
                    .portfolio()
                    .create_portfolio(
                        PortfolioName(name),
                    )
                    .submit()?;
                Ok(PortfolioId {
                  did,
                  kind: PortfolioKind::User(PortfolioNumber(num.0)),
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
                    .portfolios_in_custody(did, portfolio.into())?
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
                    .quit_portfolio_custody(portfolio.into())
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
                        src.into(),
                        dest.into(),
                        funds.into_iter().map(|f| f.into()).collect(),
                    )
                    .submit()?;
                Ok(())
            }

            /// Get portfolio balance.
            #[ink(message)]
            pub fn portfolio_asset_balances(
                &self,
                portfolio: PortfolioId,
                ticker: Ticker
            ) -> PolymeshResult<Balance> {
                let api = Api::new();
                let balance = api.query().portfolio().portfolio_asset_balances(portfolio.into(), ticker.into())?;
                Ok(balance)
            }

            /// Check portfolios_in_custody.
            #[ink(message)]
            pub fn check_portfolios_in_custody(
                &self,
                did: IdentityId,
                portfolio: PortfolioId
            ) -> PolymeshResult<bool> {
                let api = Api::new();
                Ok(api
                    .query()
                    .portfolio()
                    .portfolios_in_custody(did, portfolio.into())?)
            }

            /// Create a Settlement Venue.
            #[ink(message)]
            pub fn create_venue(&self, details: VenueDetails, ty: VenueType) -> PolymeshResult<VenueId> {
                let api = Api::new();
                // Get the next venue id.
                let id = api.query().settlement().venue_counter()?;
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
                    .instruction_counter()?;
                // Create settlement.
                api.call()
                    .settlement()
                    .add_and_affirm_instruction(
                        venue,
                        SettlementType::SettleManual(0),
                        None,
                        None,
                        legs.into_iter().map(|l| l.into()).collect(),
                        portfolios.into_iter().map(|p| p.into()).collect(),
                        None,
                    )
                    .submit()?;

                // Create settlement.
                api.call()
                    .settlement()
                    .execute_manual_instruction(instruction_id, None, leg_count, 0, 0, None)
                    .submit()?;
                Ok(())
            }

            /// Asset issue tokens.
            #[ink(message)]
            pub fn asset_issue(&self, ticker: Ticker, amount: Balance) -> PolymeshResult<()> {
                let api = Api::new();
                // Mint tokens.
                api.call().asset().issue(ticker.into(), amount, PortfolioKind::Default.into()).submit()?;
                Ok(())
            }

            /// Asset redeem tokens.
            #[ink(message)]
            pub fn asset_redeem_from_portfolio(&self, ticker: Ticker, amount: Balance, portfolio: PortfolioKind) -> PolymeshResult<()> {
                let api = Api::new();
                // Redeem tokens.
                api.call().asset().redeem_from_portfolio(ticker.into(), amount, portfolio.into()).submit()?;
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
                        ticker.into(),
                        divisible,
                        asset_type,
                        vec![],
                        None,
                    )
                    .submit()?;
                // Mint some tokens.
                if let Some(amount) = issue {
                  api.call().asset().issue(ticker.into(), amount, PortfolioKind::Default.into()).submit()?;
                }
                // Pause compliance rules to allow transfers.
                api.call()
                    .compliance_manager()
                    .pause_asset_compliance(ticker.into())
                    .submit()?;
                Ok(())
            }

            /// Get an identity's asset balance.
            #[ink(message)]
            pub fn asset_balance_of(
                &self,
                ticker: Ticker,
                did: IdentityId
            ) -> PolymeshResult<Balance> {
                let api = Api::new();
                let balance = api.query().asset().balance_of(ticker, did)?;
                Ok(balance)
            }

            /// Get the `total_supply` of an asset.
            #[ink(message)]
            pub fn asset_total_supply(
                &self,
                ticker: Ticker
            ) -> PolymeshResult<Balance> {
                let api = Api::new();
                let token = api.query().asset().tokens(ticker)?;
                Ok(token.map(|t| t.total_supply).unwrap_or_default())
            }

            /// Get corporate action distribution summary.
            #[ink(message)]
            pub fn distribution_summary(
                &self,
                ca_id: CAId
            ) -> PolymeshResult<Option<DistributionSummary>> {
                let api = Api::new();
                let distribution = api.query().capital_distribution().distributions(ca_id)?;
                Ok(distribution.map(|d| d.into()))
            }

            /// Cliam dividends from a distribution.
            #[ink(message)]
            pub fn dividend_claim(
                &self,
                ca_id: CAId
            ) -> PolymeshResult<()> {
                let api = Api::new();
                api.call().capital_distribution().claim(ca_id).submit()?;
                Ok(())
            }

            /// Create a simple dividend distribution.
            #[ink(message)]
            pub fn create_dividend(
                &self,
                dividend: SimpleDividend
            ) -> PolymeshResult<()> {
                let api = Api::new();
                // Corporate action args.
                let ca_args = pallet_corporate_actions::InitiateCorporateActionArgs {
                    ticker: dividend.ticker,
                    kind: pallet_corporate_actions::CAKind::PredictableBenefit,
                    decl_date: dividend.decl_date,
                    record_date: Some(pallet_corporate_actions::RecordDateSpec::Scheduled(dividend.record_date)),
                    details: pallet_corporate_actions::CADetails(vec![]),
                    targets: None,
                    default_withholding_tax: None,
                    withholding_tax: None,
                };
                // Create corporate action & distribution.
                api.call()
                    .corporate_action()
                    .initiate_corporate_action_and_distribute(
                        ca_args,
                        dividend.portfolio,
                        dividend.currency,
                        dividend.per_share,
                        dividend.amount,
                        dividend.payment_at,
                        dividend.expires_at,
                    )
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
                    .ok_or(PolymeshError::MissingIdentity)
            }
        }
    }
}
