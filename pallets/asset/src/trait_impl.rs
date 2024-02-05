#[cfg(feature = "runtime-benchmarks")]
use polymesh_primitives::asset_metadata::{AssetMetadataName, AssetMetadataSpec};
#[cfg(feature = "runtime-benchmarks")]
use sp_std::collections::btree_set::BTreeSet;

use frame_support::dispatch::DispatchResult;
use frame_support::storage::{StorageDoubleMap, StorageMap};

use polymesh_common_utilities::asset::AssetFnTrait;
use polymesh_common_utilities::traits::asset::Config;
use polymesh_primitives::asset::{AssetName, AssetType, FundingRoundName};
use polymesh_primitives::{AssetIdentifier, Balance, IdentityId, PortfolioKind, Ticker};

use crate::{Module, PreApprovedTicker, TickersExemptFromAffirmation, Vec};

impl<T: Config> AssetFnTrait<T::AccountId, T::RuntimeOrigin> for Module<T> {
    fn ensure_granular(ticker: &Ticker, value: Balance) -> DispatchResult {
        Self::ensure_granular(ticker, value)
    }

    fn balance(ticker: &Ticker, who: IdentityId) -> Balance {
        Self::balance_of(ticker, &who)
    }

    fn create_asset(
        origin: T::RuntimeOrigin,
        name: AssetName,
        ticker: Ticker,
        divisible: bool,
        asset_type: AssetType,
        identifiers: Vec<AssetIdentifier>,
        funding_round: Option<FundingRoundName>,
    ) -> DispatchResult {
        Self::create_asset(
            origin,
            name,
            ticker,
            divisible,
            asset_type,
            identifiers,
            funding_round,
        )
    }

    fn register_ticker(origin: T::RuntimeOrigin, ticker: Ticker) -> DispatchResult {
        Self::base_register_ticker(origin, ticker)
    }

    fn issue(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        amount: Balance,
        portfolio_kind: PortfolioKind,
    ) -> DispatchResult {
        Self::issue(origin, ticker, amount, portfolio_kind)
    }

    fn skip_ticker_affirmation(identity_id: &IdentityId, ticker: &Ticker) -> bool {
        if TickersExemptFromAffirmation::get(ticker) {
            return true;
        }
        PreApprovedTicker::get(identity_id, ticker)
    }

    fn ticker_affirmation_exemption(ticker: &Ticker) -> bool {
        TickersExemptFromAffirmation::get(ticker)
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn register_asset_metadata_type(
        origin: T::RuntimeOrigin,
        ticker: Option<Ticker>,
        name: AssetMetadataName,
        spec: AssetMetadataSpec,
    ) -> DispatchResult {
        match ticker {
            Some(ticker) => Self::register_asset_metadata_local_type(origin, ticker, name, spec),
            None => Self::register_asset_metadata_global_type(origin, name, spec),
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn add_mandatory_mediators(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        mediators: BTreeSet<IdentityId>,
    ) -> DispatchResult {
        Self::add_mandatory_mediators(origin, ticker, mediators.try_into().unwrap_or_default())
    }
}
