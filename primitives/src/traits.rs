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

use frame_support::pallet_prelude::DispatchError;
use sp_runtime::transaction_validity::InvalidTransaction;

use crate::asset::AssetId;
use crate::asset_metadata::AssetMetadataKey;
use crate::{Balance, IdentityId, NFTId, PortfolioId, WeightMeter};

#[cfg(feature = "runtime-benchmarks")]
use crate::{asset::NonFungibleType, NFTCollectionKeys};

#[cfg(feature = "runtime-benchmarks")]
use frame_support::pallet_prelude::DispatchResult;

mod asset;
pub mod group;
mod settlement;
pub use asset::*;
pub use settlement::*;

// Polymesh note: This was specifically added for Polymesh
pub trait CurrentFeePayer<AccountId, Call> {
    /// Returns the account that will pay for the call.
    fn get_valid_payer(
        call: &Call,
        caller: AccountId,
    ) -> Result<Option<AccountId>, InvalidTransaction>;
    /// Sets payer in context. Should be called by the signed extension that first charges fee.
    fn set_payer_context(payer: Option<AccountId>);
    /// Fetches fee payer for further payments (forwarded calls)
    fn get_payer_from_context() -> Option<AccountId>;
    /// Decreases the authorization count if any of the following extrinsics failed:
    /// - pallet-identity (accept_primary_key, join_identity_as_key, rotate_primary_key_to_secondary)
    /// - pallet-multisig (accept_multisig_signer)
    fn decrease_authorization_count(caller: &AccountId, auth_id: Option<u64>);
    /// Returns Some(auth_id) of the call if any of the following extrinsics are called:
    /// - pallet-identity (accept_primary_key, join_identity_as_key, rotate_primary_key_to_secondary)
    /// - pallet-multisig (accept_multisig_signer)
    fn get_authorization_id(call: &Call) -> Option<u64>;
}

pub trait IdentityFnTrait<AccountId> {
    fn get_identity(key: &AccountId) -> Option<IdentityId>;

    /// Provides the DID status for the given DID
    fn is_did_active(target_did: IdentityId) -> bool;

    /// Check if a DID is locked.
    /// TODO: Implement DID locking. For now, always returns false.
    fn is_did_locked(target_did: IdentityId) -> bool;

    /// Creates a new did.
    fn testing_register_did(target: AccountId) -> Result<IdentityId, DispatchError>;
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

pub trait PortfolioFnTrait {
    /// Returns `Ok(())` if `custodian` has custody over the portfolio.
    /// The portfolio owner is the default custodian when none is assigned.
    fn ensure_portfolio_custody(
        portfolio: &PortfolioId,
        custodian: IdentityId,
    ) -> Result<(), DispatchError>;
}

/// Supertrait config for pallets that need portfolio custody queries.
pub trait PortfolioFnConfig: frame_system::Config {
    type PortfolioFn: PortfolioFnTrait;
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
    /// Updates the Owner storage after moving funds between portfolios.
    fn update_nft_owner(asset_id: AssetId, nft_id: NFTId, new_owner_portfolio: PortfolioId);

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
