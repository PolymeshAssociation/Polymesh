#[cfg(feature = "runtime-benchmarks")]
use sp_std::{collections::btree_set::BTreeSet, prelude::Vec};

use frame_support::dispatch::DispatchResult;
use frame_support::pallet_prelude::DispatchError;
use frame_support::weights::Weight;

#[cfg(feature = "runtime-benchmarks")]
use frame_support::pallet_prelude::DispatchResultWithPostInfo;

#[cfg(feature = "runtime-benchmarks")]
use crate::{
    asset::{AssetHolderKind, AssetName, AssetType, FundingRoundName},
    asset_metadata::{AssetMetadataName, AssetMetadataSpec},
    AssetIdentifier, Ticker,
};

use crate::asset::AssetId;
use crate::{Balance, IdentityId};

pub trait AssetFnConfig: frame_system::Config {
    type AssetFn: AssetFnTrait<Self::AccountId>;
}

pub trait AssetFnTrait<AccountId> {
    /// Returns `Ok` if [`AssetDetails::divisible`] or `value` % ONE_UNIT == 0.
    fn ensure_granular(asset_id: &AssetId, value: Balance) -> DispatchResult;

    /// Returns `true` if the given `identity_id` is exempt from affirming the receivement of `asset_id`, otherwise returns `false`.
    fn skip_asset_affirmation(identity_id: &IdentityId, asset_id: &AssetId) -> bool;

    /// Returns `true` if the receivement of `asset_id` is exempt from being affirmed, otherwise returns `false`.
    fn asset_affirmation_exemption(asset_id: &AssetId) -> bool;

    /// Returns the `did` balance for the given `asset_id`.
    fn asset_balance(asset_id: &AssetId, did: &IdentityId) -> Balance;

    /// Returns the total supply for the given `asset_id`.
    fn asset_total_supply(asset_id: &AssetId) -> Result<Balance, DispatchError>;

    /// Returns the next [`AssetID`] for the `caller_acc`.
    fn generate_asset_id(caller_acc: AccountId) -> AssetId;

    /// Worst-case weight for `spend_allowance` (check + decrement).
    /// Charged dynamically only when the caller is a spender (not the asset owner).
    fn spend_allowance_weight() -> Weight;

    /// Returns `Ok` if the asset is not frozen.
    fn asset_is_not_frozen(asset_id: &AssetId) -> DispatchResult;

    #[cfg(feature = "runtime-benchmarks")]
    fn register_unique_ticker(caller: AccountId, ticker: Ticker) -> DispatchResult;

    #[cfg(feature = "runtime-benchmarks")]
    fn create_asset(
        caller: AccountId,
        asset_name: AssetName,
        divisible: bool,
        asset_type: AssetType,
        asset_identifiers: Vec<AssetIdentifier>,
        funding_round: Option<FundingRoundName>,
    ) -> DispatchResult;

    #[cfg(feature = "runtime-benchmarks")]
    fn issue(
        caller: AccountId,
        asset_id: AssetId,
        amount: Balance,
        portfolio_kind: AssetHolderKind,
    ) -> DispatchResultWithPostInfo;

    #[cfg(feature = "runtime-benchmarks")]
    fn register_asset_metadata_type(
        asset_and_caller: Option<(AssetId, AccountId)>,
        name: AssetMetadataName,
        spec: AssetMetadataSpec,
    ) -> DispatchResult;

    #[cfg(feature = "runtime-benchmarks")]
    fn add_mandatory_mediators(
        caller: AccountId,
        asset_id: AssetId,
        mediators: BTreeSet<IdentityId>,
    ) -> DispatchResult;
}
