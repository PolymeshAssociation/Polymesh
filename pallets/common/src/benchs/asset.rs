use sp_std::vec;

use polymesh_primitives::asset::{AssetName, AssetType};
use polymesh_primitives::{PortfolioKind, Ticker};

use crate::benchs::User;
use crate::constants::currency::POLY;
use crate::traits::asset::{AssetFnTrait, Config};

pub type ResultTicker = Result<Ticker, &'static str>;

/// Registers a unique ticker named `ticker_name` for `ticker_owner`.
pub fn reg_unique_ticker<T: Config>(
    ticker_owner: T::RuntimeOrigin,
    ticker_name: Option<&[u8]>,
) -> Ticker {
    let ticker = match ticker_name {
        Some(name) => Ticker::from_slice_truncated(name),
        None => Ticker::repeating(b'A'),
    };
    T::AssetFn::register_unique_ticker(ticker_owner, ticker).unwrap();
    ticker
}

//fn create_sample_asset<T: Config>(asset_owner: &User<T>, divisible: bool, asset_name: Option<&[u8]>) {
//    T::AssetFn::create_asset(
//        asset_owner.origin().into(),
//        asset_name.clone(),
//        divisible,
//        AssetType::default(),
//        vec![],
//        None,
//    )
//    .unwrap();
//
//    T::AssetFn::issue(
//        asset_owner.origin().into(),
//        ticker,
//        (1_000_000 * POLY).into(),
//        PortfolioKind::Default,
//    )
//    .unwrap();
//}
