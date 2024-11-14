#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;

mod macros;

use alloc::collections::BTreeSet;
#[cfg(not(feature = "as-library"))]
use alloc::vec;

use alloc::vec::Vec;

pub use polymesh_api::ink::basic_types::{AssetId, IdentityId};
pub use polymesh_api::ink::extension::PolymeshEnvironment;
pub use polymesh_api::ink::Error as PolymeshInkError;
pub use polymesh_api::polymesh::types::pallet_corporate_actions;
pub use polymesh_api::polymesh::types::pallet_corporate_actions::CAId;
pub use polymesh_api::polymesh::types::polymesh_contracts::Api as ContractRuntimeApi;
pub use polymesh_api::polymesh::types::polymesh_primitives::asset::{
    AssetName, AssetType, CheckpointId,
};
pub use polymesh_api::polymesh::types::polymesh_primitives::asset_metadata::{
    AssetMetadataKey, AssetMetadataLocalKey, AssetMetadataName, AssetMetadataValue,
};
pub use polymesh_api::polymesh::types::polymesh_primitives::identity_id::{
    PortfolioId, PortfolioKind, PortfolioName, PortfolioNumber,
};
pub use polymesh_api::polymesh::types::polymesh_primitives::nft::{NFTId, NFTs};
pub use polymesh_api::polymesh::types::polymesh_primitives::portfolio::{Fund, FundDescription};
pub use polymesh_api::polymesh::types::polymesh_primitives::settlement::{
    InstructionId, Leg, SettlementType, VenueDetails, VenueId, VenueType,
};
pub use polymesh_api::polymesh::Api;

pub const API_VERSION: ContractRuntimeApi = ContractRuntimeApi {
    desc: *b"POLY",
    major: 7,
};

/// Contract Errors.
#[derive(Debug, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PolymeshError {
    /// Polymesh runtime error.
    PolymeshRuntime(PolymeshInkError),
    /// [`IdentityId`] not found for the contract caller. MultiSig's are not supported.
    MissingIdentity,
    /// No portfolio was found for the given [`PortfolioId`].
    InvalidPortfolioAuthorization,
    /// Ink! Delegate call error.
    InkDelegateCallError {
        selector: [u8; 4],
        err: Option<InkEnvError>,
    },
    /// Not the NFT owner.
    NotNftOwner,
}

/// Encodable `ink::env::Error`.
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
    pub fn from_delegate_error(err: ink::env::Error, selector: ink::env::call::Selector) -> Self {
        use ink::env::Error::*;
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

impl From<PolymeshInkError> for PolymeshError {
    fn from(err: PolymeshInkError) -> Self {
        Self::PolymeshRuntime(err)
    }
}

/// The contract result type.
pub type PolymeshResult<T> = core::result::Result<T, PolymeshError>;

#[cfg(feature = "as-library")]
pub type Hash = <PolymeshEnvironment as ink::env::Environment>::Hash;
#[cfg(feature = "as-library")]
pub type AccountId = <PolymeshEnvironment as ink::env::Environment>::AccountId;
pub type Balance = <PolymeshEnvironment as ink::env::Environment>::Balance;
pub type Timestamp = <PolymeshEnvironment as ink::env::Environment>::Timestamp;

#[derive(Debug, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct DistributionSummary {
    pub currency: AssetId,
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
    pub asset_id: AssetId,
    pub decl_date: Timestamp,
    pub record_date: Timestamp,
    pub portfolio: Option<PortfolioNumber>,
    pub currency: AssetId,
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

            /// Creates a portfolio with the given `name`.
            #[ink(message)]
            pub fn create_portfolio(&self, name: Vec<u8>) -> PolymeshResult<PortfolioId> {
                let api = Api::new();

                // Get the contract's did
                let contracts_did = Self::get_our_did()?;
                // Get the next portfolio number
                let portfolio_number = api
                    .query()
                    .portfolio()
                    .next_portfolio_number(contracts_did)?;

                api.call()
                    .portfolio()
                    .create_portfolio(PortfolioName(name))
                    .submit()?;

                Ok(PortfolioId {
                    did: contracts_did,
                    kind: PortfolioKind::User(portfolio_number),
                })
            }

            /// Accepts custody of a portfolio.
            #[ink(message)]
            pub fn accept_portfolio_custody(
                &self,
                auth_id: u64,
                portfolio_kind: PortfolioKind
            ) -> PolymeshResult<PortfolioId> {
                let api = Api::new();

                // Get the caller's identity.
                let caller_did = Self::get_caller_did()?;
                // Get the contract's did
                let contracts_did = Self::get_our_did()?;

                api.call()
                    .portfolio()
                    .accept_portfolio_custody(auth_id)
                    .submit()?;

                let portfolio_id = PortfolioId {
                    did: caller_did,
                    kind: portfolio_kind,
                };

                if !api
                    .query()
                    .portfolio()
                    .portfolios_in_custody(contracts_did, portfolio_id)?
                {
                    return Err(PolymeshError::InvalidPortfolioAuthorization);
                }

                Ok(portfolio_id)
            }

            /// Quits custodianship of a portfolio returning control back to the owner.
            #[ink(message)]
            pub fn quit_portfolio_custody(&self, portfolio_id: PortfolioId) -> PolymeshResult<()> {
                let api = Api::new();

                api.call()
                    .portfolio()
                    .quit_portfolio_custody(portfolio_id)
                    .submit()?;
                Ok(())
            }

            /// Moves the given `funds` from `source_portfolio_id` to `destination_portfolio_id`.
            #[ink(message)]
            pub fn move_portfolio_funds(
                &self,
                source_portfolio_id: PortfolioId,
                destination_portfolio_id: PortfolioId,
                funds: Vec<Fund>
            ) -> PolymeshResult<()> {
                let api = Api::new();

                api.call()
                    .portfolio()
                    .move_portfolio_funds(source_portfolio_id, destination_portfolio_id, funds)
                    .submit()?;
                Ok(())
            }

            /// Returns the [`Balance`] for the `asset_id` in the given `portfolio_id`.
            #[ink(message)]
            pub fn portfolio_asset_balances(
                &self,
                portfolio_id: PortfolioId,
                asset_id: AssetId
            ) -> PolymeshResult<Balance> {
                let api = Api::new();

                let balance = api
                    .query()
                    .portfolio()
                    .portfolio_asset_balances(portfolio_id, asset_id)?;
                Ok(balance)
            }

            /// Returns `true` if `portfolio_id` is in custody of `custodian_did`, otherwise returns `false`.
            #[ink(message)]
            pub fn check_portfolios_in_custody(
                &self,
                custodian_did: IdentityId,
                portfolio_id: PortfolioId
            ) -> PolymeshResult<bool> {
                let api = Api::new();

                let is_custodian = api.query()
                    .portfolio()
                    .portfolios_in_custody(custodian_did, portfolio_id)?;
                Ok(is_custodian)
            }

            /// Creates a Settlement Venue.
            #[ink(message)]
            pub fn create_venue(
                &self,
                venue_details: VenueDetails,
                venue_type: VenueType
            ) -> PolymeshResult<VenueId> {
                let api = Api::new();

                // Get the next venue id.
                let venue_id = api.query().settlement().venue_counter()?;

                api.call()
                    .settlement()
                    .create_venue(venue_details, vec![], venue_type)
                    .submit()?;
                Ok(venue_id)
            }

            /// Creates and manually executes a settlement to transfer assets.
            #[ink(message)]
            pub fn settlement_execute(
                &self,
                venue_id: Option<VenueId>,
                legs: Vec<Leg>,
                portfolios: BTreeSet<PortfolioId>
            ) -> PolymeshResult<()> {
                let api = Api::new();

                // Counts the number of each asset type
                let (fungible, nfts, offchain) =
                    legs.iter()
                        .fold((0, 0, 0), |(fungible, nfts, offchain), leg| match leg {
                            Leg::Fungible { .. } => (fungible + 1, nfts, offchain),
                            Leg::NonFungible { .. } => (fungible, nfts + 1, offchain),
                            Leg::OffChain { .. } => (fungible, nfts, offchain + 1),
                        });

                // Get the next instruction id.
                let instruction_id = api
                    .query()
                    .settlement()
                    .instruction_counter()?;

                api.call()
                    .settlement()
                    .add_and_affirm_instruction(
                        venue_id,
                        SettlementType::SettleManual(0),
                        None,
                        None,
                        legs,
                        portfolios,
                        None,
                    )
                    .submit()?;

                api.call()
                    .settlement()
                    .execute_manual_instruction(instruction_id, None, fungible, nfts, offchain, None)
                    .submit()?;
                Ok(())
            }

            /// Issues `amount_to_issue` new tokens to `portfolio_kind`.
            #[ink(message)]
            pub fn asset_issue(
                &self,
                asset_id: AssetId,
                amount_to_issue: Balance,
                portfolio_kind: PortfolioKind
            ) -> PolymeshResult<()> {
                let api = Api::new();
                api.call().asset().issue(asset_id, amount_to_issue, portfolio_kind).submit()?;
                Ok(())
            }

            /// Redeems `amount_to_redeem` tokens from `portfolio_kind`.
            #[ink(message)]
            pub fn asset_redeem(
                &self,
                asset_id: AssetId,
                amount_to_redeem: Balance,
                portfolio_kind: PortfolioKind
            ) -> PolymeshResult<()> {
                let api = Api::new();
                api.call().asset().redeem(asset_id, amount_to_redeem, portfolio_kind).submit()?;
                Ok(())
            }

            /// Creates a new asset and issues `amount_to_issue` tokens of that asset to the default portfolio of the caller.
            #[ink(message)]
            pub fn asset_create_and_issue(
                &self,
                asset_name: AssetName,
                asset_type: AssetType,
                divisible: bool,
                amount_to_issue: Option<Balance>
            ) -> PolymeshResult<()> {
                let api = Api::new();

                let asset_id = Self::get_asset_id(ink::env::account_id::<PolymeshEnvironment>())?;

                api.call()
                    .asset()
                    .create_asset(asset_name, divisible, asset_type, vec![], None)
                    .submit()?;

                if let Some(amount_to_issue) = amount_to_issue {
                    api.call()
                        .asset()
                        .issue(asset_id, amount_to_issue, PortfolioKind::Default)
                        .submit()?;
                }

                Ok(())
            }

            /// Returns the `asset_id` [`Balance`] for the given `did`.
            #[ink(message)]
            pub fn asset_balance_of(
                &self,
                asset_id: AssetId,
                did: IdentityId
            ) -> PolymeshResult<Balance> {
                let api = Api::new();
                let balance = api.query().asset().balance_of(asset_id, did)?;
                Ok(balance)
            }

            /// Returns the total supply of `asset_id`.
            #[ink(message)]
            pub fn asset_total_supply(
                &self,
                asset_id: AssetId
            ) -> PolymeshResult<Balance> {
                let api = Api::new();

                let asset_details = api.query().asset().assets(asset_id)?;
                let total_supply = asset_details
                    .map(|asset| asset.total_supply)
                    .unwrap_or_default();
                Ok(total_supply)
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

            /// Claims dividends from a distribution.
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
                    asset_id: dividend.asset_id,
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

            /// Adds and affirms an instruction.
            #[ink(message)]
            pub fn add_and_affirm_instruction(
                &self,
                venue_id: Option<VenueId>,
                legs: Vec<Leg>,
                portfolios: BTreeSet<PortfolioId>
            ) -> PolymeshResult<InstructionId> {
                let api = Api::new();

                let instruction_id = api
                    .query()
                    .settlement()
                    .instruction_counter()?;

                api.call()
                    .settlement()
                    .add_and_affirm_instruction(
                        venue_id,
                        SettlementType::SettleOnAffirmation,
                        None,
                        None,
                        legs,
                        portfolios,
                        None,
                    )
                    .submit()?;
                Ok(instruction_id)
            }

            /// Creates a portoflio owned by `portfolio_owner_id` and transfer its custody to the smart contract.
            /// Returns the [`PortfolioId`] of the new portfolio.
            #[ink(message)]
            pub fn create_custody_portfolio(
                &self,
                portfolio_owner_id: IdentityId,
                portfolio_name: PortfolioName
            ) -> PolymeshResult<PortfolioId> {
                let api = Api::new();

                let portfolio_number = api
                    .query()
                    .portfolio()
                    .next_portfolio_number(portfolio_owner_id)?;
                let portfolio_id = PortfolioId {
                    did: portfolio_owner_id,
                    kind: PortfolioKind::User(portfolio_number),
                };

                api.call()
                    .portfolio()
                    .create_custody_portfolio(portfolio_owner_id, portfolio_name)
                    .submit()?;
                Ok(portfolio_id)
            }

            /// Returns the [`AssetMetadataLocalKey`] for the given `asset_id` and `asset_metadata_name`.
            #[ink(message)]
            pub fn asset_metadata_local_name_to_key(
                &self,
                asset_id: AssetId,
                asset_metadata_name: AssetMetadataName
            ) -> PolymeshResult<Option<AssetMetadataLocalKey>> {
                Ok(Api::new()
                    .query()
                    .asset()
                    .asset_metadata_local_name_to_key(asset_id, asset_metadata_name)?)
            }

            /// Returns the [`AssetMetadataValue`] for the given `asset_id` and `asset_metadata_key`.
            #[ink(message)]
            pub fn asset_metadata_value(
                &self,
                asset_id: AssetId,
                asset_metadata_key: AssetMetadataKey
            ) -> PolymeshResult<Option<AssetMetadataValue>> {
                Ok(Api::new()
                    .query()
                    .asset()
                    .asset_metadata_values(asset_id, asset_metadata_key)?)
            }

            /// Returns the [`PortfolioId`] that holds the NFT.
            #[ink(message)]
            pub fn nft_owner(
              &self,
              asset_id: AssetId,
              nft: NFTId,
            ) -> PolymeshResult<Option<PortfolioId>> {
                let api = Api::new();
                Ok(api.query().nft().nft_owner(asset_id, nft)?)
            }

            /// Returns `Ok` if `portfolio_id` holds all `nfts`. Otherwise, returns [`PolymeshError::NotNftOwner`].
            #[ink(message)]
            pub fn holds_nfts(
              &self,
              portfolio_id: PortfolioId,
              asset_id: AssetId,
              nfts: Vec<NFTId>,
            ) -> PolymeshResult<()> {
                let api = Api::new();
                for nft_id in nfts {
                    let nft_holder = api
                        .query()
                        .nft()
                        .nft_owner(asset_id, nft_id)?
                        .ok_or(PolymeshError::NotNftOwner)?;

                    if nft_holder != portfolio_id {
                        return Err(PolymeshError::NotNftOwner);
                    }
                }
                Ok(())
            }
        }

        // Non-upgradable api.
        impl PolymeshInk {
            /// Get the identity of the caller.
            pub fn get_caller_did() -> PolymeshResult<IdentityId> {
                Self::get_key_did(ink::env::caller::<PolymeshEnvironment>())
            }

            /// Get the identity of the contract.
            pub fn get_our_did() -> PolymeshResult<IdentityId> {
                Self::get_key_did(ink::env::account_id::<PolymeshEnvironment>())
            }

            /// Get the identity of a key.
            pub fn get_key_did(acc: AccountId) -> PolymeshResult<IdentityId> {
                let api = Api::new();
                api.runtime()
                    .get_key_did(acc)?
                    .ok_or(PolymeshError::MissingIdentity)
            }

            /// Returns the next [`AssetId`] for the given `account_id`.
            pub fn get_asset_id(account_id: AccountId) -> PolymeshResult<AssetId> {
                let api = Api::new();
                Ok(api.runtime().get_next_asset_id(account_id)?)
            }
        }
    }
}
