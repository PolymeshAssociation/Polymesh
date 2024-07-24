use sp_std::vec;

use polymesh_primitives::asset::{AssetName, AssetType};
use polymesh_primitives::{PortfolioKind, Ticker};

use crate::benchs::User;
use crate::constants::currency::POLY;
use crate::traits::asset::{AssetFnTrait, Config};

pub type ResultTicker = Result<Ticker, &'static str>;

/// Registers a unique ticker for `ticker_owner`.
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

//pub fn make_asset<T: Config>(owner: &User<T>, name: Option<&[u8]>) -> Ticker {
//    make_base_asset::<T>(owner, true, name)
//}
//
//pub fn make_indivisible_asset<T: Config>(owner: &User<T>, name: Option<&[u8]>) -> Ticker {
//    make_base_asset::<T>(owner, false, name)
//}
//
//fn make_base_asset<T: Config>(owner: &User<T>, divisible: bool, name: Option<&[u8]>) -> Ticker {
//    let ticker = make_ticker::<T>(owner.origin().into(), name);
//    let name: AssetName = ticker.as_slice().into();
//
//    T::AssetFn::create_asset(
//        owner.origin().into(),
//        name.clone(),
//        ticker,
//        divisible,
//        AssetType::default(),
//        vec![],
//        None,
//    )
//    .expect("Asset cannot be created");
//
//    T::AssetFn::issue(
//        owner.origin().into(),
//        ticker,
//        (1_000_000 * POLY).into(),
//        PortfolioKind::Default,
//    )
//    .unwrap();
//
//    ticker
//}
