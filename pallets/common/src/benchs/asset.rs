use sp_std::vec;

use polymesh_primitives::asset::{AssetName, AssetType};
use polymesh_primitives::{PortfolioKind, Ticker};

use crate::benchs::User;
use crate::constants::currency::POLY;
use crate::traits::asset::{AssetFnTrait, Config};

pub type ResultTicker = Result<Ticker, &'static str>;

/// Create a ticker and register it.
pub fn make_ticker<T: Config>(owner: T::RuntimeOrigin, opt_name: Option<&[u8]>) -> Ticker {
    let ticker = match opt_name {
        Some(name) => Ticker::from_slice_truncated(name),
        _ => Ticker::repeating(b'A'),
    };
    T::AssetFn::register_ticker(owner, ticker).expect("Ticker cannot be registered");
    ticker
}

pub fn make_asset<T: Config>(owner: &User<T>, name: Option<&[u8]>) -> Ticker {
    make_base_asset::<T>(owner, true, name)
}

pub fn make_indivisible_asset<T: Config>(owner: &User<T>, name: Option<&[u8]>) -> Ticker {
    make_base_asset::<T>(owner, false, name)
}

fn make_base_asset<T: Config>(owner: &User<T>, divisible: bool, name: Option<&[u8]>) -> Ticker {
    let ticker = make_ticker::<T>(owner.origin().into(), name);
    let name: AssetName = ticker.as_slice().into();

    T::AssetFn::create_asset(
        owner.origin().into(),
        name.clone(),
        ticker,
        divisible,
        AssetType::default(),
        vec![],
        None,
    )
    .expect("Asset cannot be created");

    T::AssetFn::issue(
        owner.origin().into(),
        ticker,
        (1_000_000 * POLY).into(),
        PortfolioKind::Default,
    )
    .unwrap();

    ticker
}
