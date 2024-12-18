#![allow(missing_docs)]

use crate::{
    asset::{AssetId, AssetName, AssetType},
    constants::currency::POLY,
    traits::{AssetFnConfig, AssetFnTrait},
    PortfolioKind, Ticker,
};
use sp_std::vec;

/// Registers a unique ticker named `ticker_name` for `ticker_owner`.
#[cfg(feature = "runtime-benchmarks")]
pub fn reg_unique_ticker<T: AssetFnConfig>(
    ticker_owner: T::AccountId,
    ticker_name: Option<&[u8]>,
) -> Ticker {
    let ticker = match ticker_name {
        Some(name) => Ticker::from_slice_truncated(name),
        None => Ticker::repeating(b'A'),
    };
    T::AssetFn::register_unique_ticker(ticker_owner, ticker).unwrap();
    ticker
}

#[cfg(feature = "runtime-benchmarks")]
pub fn create_and_issue_sample_asset<T: AssetFnConfig>(
    asset_owner: T::AccountId,
    divisible: bool,
    asset_type: Option<AssetType>,
    asset_name: &[u8],
    issue_tokens: bool,
) -> AssetId {
    let asset_id = T::AssetFn::generate_asset_id(asset_owner.clone());

    T::AssetFn::create_asset(
        asset_owner.clone(),
        AssetName::from(asset_name),
        divisible,
        asset_type.unwrap_or_default(),
        vec![],
        None,
    )
    .unwrap();

    if issue_tokens {
        T::AssetFn::issue(
            asset_owner,
            asset_id,
            (1_000_000 * POLY).into(),
            PortfolioKind::Default,
        )
        .unwrap();
    }

    asset_id
}
