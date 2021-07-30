use crate::{
    benchs::User,
    constants::currency::POLY,
    traits::asset::{AssetFnTrait, Config},
};
use polymesh_primitives::{
    asset::{AssetName, AssetType},
    Ticker,
};
use sp_std::{convert::TryFrom, vec};

pub type ResultTicker = Result<Ticker, &'static str>;

/// Create a ticker and register it.
pub fn make_ticker<T: Config>(owner: T::Origin, opt_name: Option<&[u8]>) -> Ticker {
    let ticker = match opt_name {
        Some(name) => Ticker::try_from(name).expect("Invalid ticker name"),
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
        false,
    )
    .expect("Asset cannot be created");

    T::AssetFn::issue(owner.origin().into(), ticker, (1_000_000 * POLY).into()).unwrap();

    ticker
}
