#![allow(missing_docs)]
// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymesh Association

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use frame_support::dispatch::{DispatchError, DispatchResult};
use sp_runtime::transaction_validity::InvalidTransaction;

use crate::{
    asset::AssetId, asset_metadata::AssetMetadataKey, compliance_manager::AssetComplianceResult,
    secondary_key::SecondaryKey, Balance, IdentityId, NFTId, PortfolioId, WeightMeter,
};

#[cfg(feature = "runtime-benchmarks")]
use crate::{asset::NonFungibleType, NFTCollectionKeys};

mod asset;
pub mod group;
pub use asset::*;

// Polymesh note: This was specifically added for Polymesh
pub trait CddAndFeeDetails<AccountId, Call> {
    /// Returns the account that will pay for the call.
    fn get_valid_payer(
        call: &Call,
        caller: &AccountId,
    ) -> Result<Option<AccountId>, InvalidTransaction>;
    /// Clears context. Should be called in post_dispatch
    fn clear_context();
    /// Sets payer in context. Should be called by the signed extension that first charges fee.
    fn set_payer_context(payer: Option<AccountId>);
    /// Fetches fee payer for further payments (forwarded calls)
    fn get_payer_from_context() -> Option<AccountId>;
}

pub trait CheckCdd<AccountId> {
    fn check_key_cdd(key: &AccountId) -> bool;
    fn get_key_cdd_did(key: &AccountId) -> Option<IdentityId>;
}

pub trait IdentityFnTrait<AccountId> {
    fn get_identity(key: &AccountId) -> Option<IdentityId>;
    fn current_payer() -> Option<AccountId>;
    fn set_current_payer(payer: Option<AccountId>);

    /// Provides the DID status for the given DID
    fn has_valid_cdd(target_did: IdentityId) -> bool;

    /// Creates a new did and attaches a CDD claim.
    fn testing_cdd_register_did(
        target: AccountId,
        secondary_keys: sp_std::vec::Vec<SecondaryKey<AccountId>>,
    ) -> Result<IdentityId, DispatchError>;
}

pub trait SubsidiserTrait<AccountId, RuntimeCall> {
    /// Check if a `user_key` has a subsidiser and that the subsidy can pay the `fee`.
    fn check_subsidy(
        user_key: &AccountId,
        fee: Balance,
        call: Option<&RuntimeCall>,
    ) -> Result<Option<AccountId>, InvalidTransaction>;

    /// Debit `fee` from the remaining balance of the subsidy for `user_key`.
    fn debit_subsidy(
        user_key: &AccountId,
        fee: Balance,
    ) -> Result<Option<AccountId>, InvalidTransaction>;
}

/// This trait is used to accept custody of a portfolio
pub trait PortfolioSubTrait<AccountId> {
    /// Checks that the custodian is authorized for the portfolio
    ///
    /// # Arguments
    /// * `portfolio` - Portfolio to check
    /// * `custodian` - DID of the custodian
    fn ensure_portfolio_custody(portfolio: PortfolioId, custodian: IdentityId) -> DispatchResult;

    /// Ensure that the `portfolio` exists.
    ///
    /// # Arguments
    /// * `portfolio` - Portfolio to check
    fn ensure_portfolio_validity(portfolio: &PortfolioId) -> DispatchResult;

    /// Locks some tokens of a portfolio
    ///
    /// # Arguments
    /// * `portfolio` - Portfolio to lock tokens
    /// * `asset_id` - [`AssetId`] of the token to lock
    /// * `amount` - Amount of tokens to lock

    fn lock_tokens(portfolio: &PortfolioId, asset_id: &AssetId, amount: Balance) -> DispatchResult;

    /// Unlocks some tokens of a portfolio
    ///
    /// # Arguments
    /// * `portfolio` - Portfolio to unlock tokens
    /// * asset_id` - [`AssetId`] of the token to unlock
    /// * `amount` - Amount of tokens to unlock
    fn unlock_tokens(
        portfolio: &PortfolioId,
        asset_id: &AssetId,
        amount: Balance,
    ) -> DispatchResult;

    /// Ensures that the portfolio's custody is with the provided identity
    /// And the secondary key has the relevant portfolio permission
    ///
    /// # Arguments
    /// * `portfolio` - PortfolioId of the portfolio to check
    /// * `custodian` - Identity of the custodian
    /// * `secondary_key` - Secondary key that is accessing the portfolio
    fn ensure_portfolio_custody_and_permission(
        portfolio: PortfolioId,
        custodian: IdentityId,
        secondary_key: Option<&SecondaryKey<AccountId>>,
    ) -> DispatchResult;

    /// Locks the given nft. This prevents transfering the same NFT more than once.
    ///
    /// # Arguments
    /// * `portfolio_id` - PortfolioId that contains the nft to be locked.
    /// asset_id` - [`AssetId`] of the NFT.
    /// * `nft_id` - the id of the nft to be unlocked.
    fn lock_nft(portfolio_id: &PortfolioId, asset_id: &AssetId, nft_id: &NFTId) -> DispatchResult;

    /// Unlocks the given nft.
    ///
    /// # Arguments
    /// * `portfolio_id` - PortfolioId that contains the locked nft.
    /// asset_id` - [`AssetId`] of the NFT.
    /// * `nft_id` - the id of the nft to be unlocked.
    fn unlock_nft(portfolio_id: &PortfolioId, asset_id: &AssetId, nft_id: &NFTId)
        -> DispatchResult;

    /// Returns `true` if the portfolio has pre-approved the receivement of `asset_id`, otherwise returns `false`.
    fn skip_portfolio_affirmation(portfolio_id: &PortfolioId, asset_id: &AssetId) -> bool;
}

pub trait ComplianceFnConfig {
    /// Returns `true` if there are no requirements or if any requirement is satisfied.
    /// Otherwise, returns `false`.
    fn is_compliant(
        asset_id: &AssetId,
        sender_did: IdentityId,
        receiver_did: IdentityId,
        weight_meter: &mut WeightMeter,
    ) -> Result<bool, DispatchError>;

    fn verify_restriction_granular(
        asset_id: &AssetId,
        from_did_opt: Option<IdentityId>,
        to_did_opt: Option<IdentityId>,
        weight_meter: &mut WeightMeter,
    ) -> Result<AssetComplianceResult, DispatchError>;

    #[cfg(feature = "runtime-benchmarks")]
    fn setup_asset_compliance(
        caler_did: IdentityId,
        asset_id: AssetId,
        n: u32,
        pause_compliance: bool,
    );
}

pub trait NFTTrait<Origin> {
    /// Returns `true` if the given `metadata_key` is a mandatory key for the `asset_id` NFT collection.
    fn is_collection_key(asset_id: &AssetId, metadata_key: &AssetMetadataKey) -> bool;
    /// Updates the NFTOwner storage after moving funds.
    fn move_portfolio_owner(asset_id: AssetId, nft_id: NFTId, new_owner_portfolio: PortfolioId);

    #[cfg(feature = "runtime-benchmarks")]
    fn create_nft_collection(
        origin: Origin,
        asset_id: Option<AssetId>,
        nft_type: Option<NonFungibleType>,
        collection_keys: NFTCollectionKeys,
    ) -> DispatchResult;
}

pub trait GovernanceGroupTrait<Moment: PartialOrd + Copy>: group::GroupTrait<Moment> {
    fn release_coordinator() -> Option<IdentityId>;

    #[cfg(feature = "runtime-benchmarks")]
    fn bench_set_release_coordinator(did: IdentityId);
}

/// A currency that has a block rewards reserve.
pub trait BlockRewardsReserveCurrency<NegativeImbalance> {
    /// An instance of `Drop` for positive imbalance.
    fn drop_positive_imbalance(amount: Balance);
    /// An instance of `Drop` for negative imbalance.
    fn drop_negative_imbalance(amount: Balance);
    /// Issues a given amount of currency from the block rewards reserve if possible.
    fn issue_using_block_rewards_reserve(amount: Balance) -> NegativeImbalance;
    /// Returns the balance of the block rewards reserve.
    fn block_rewards_reserve_balance() -> Balance;
}
