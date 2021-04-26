use crate::{
    benchs::User,
    constants::currency::POLY,
    traits::{asset::AssetFnTrait, identity::Trait as IdentityTrait},
};

use frame_system::RawOrigin;
use polymesh_primitives::{
    asset::{AssetName, AssetType},
    Ticker,
};
use sp_std::{convert::TryFrom, vec};

pub type ResultTicker = Result<Ticker, &'static str>;

/// Create a ticker and register it.
pub fn make_ticker<Asset, Balance, Acc, O, N>(owner: O, opt_name: Option<N>) -> ResultTicker
where
    Asset: AssetFnTrait<Balance, Acc, O>,
    N: AsRef<[u8]>,
{
    let ticker = match &opt_name {
        Some(name) => Ticker::try_from(name.as_ref()).map_err(|_| "Invalid ticker name")?,
        _ => Ticker::repeating(b'A'),
    };
    Asset::register_ticker(owner, ticker).map_err(|_| "Ticker cannot be registered")?;

    Ok(ticker)
}

pub fn make_asset<Asset, Identity, Balance, Acc, Origin, N>(
    owner: &User<Identity>,
    name: Option<N>,
) -> ResultTicker
where
    Asset: AssetFnTrait<Balance, Acc, Origin>,
    Identity: IdentityTrait,
    Origin: From<RawOrigin<<Identity as frame_system::Config>::AccountId>>,
    Balance: From<u128>,
    N: AsRef<[u8]>,
{
    make_base_asset::<Asset, Identity, Balance, Acc, Origin, N>(owner, true, name)
}

pub fn make_indivisible_asset<Asset, Identity, Balance, Acc, Origin, N>(
    owner: &User<Identity>,
    name: Option<N>,
) -> ResultTicker
where
    Asset: AssetFnTrait<Balance, Acc, Origin>,
    Identity: IdentityTrait,
    Origin: From<RawOrigin<<Identity as frame_system::Config>::AccountId>>,
    Balance: From<u128>,
    N: AsRef<[u8]>,
{
    make_base_asset::<Asset, Identity, Balance, Acc, Origin, N>(owner, false, name)
}

fn make_base_asset<Asset, Identity, Balance, Acc, Origin, N>(
    owner: &User<Identity>,
    divisible: bool,
    name: Option<N>,
) -> ResultTicker
where
    Asset: AssetFnTrait<Balance, Acc, Origin>,
    Identity: IdentityTrait,
    Origin: From<RawOrigin<<Identity as frame_system::Config>::AccountId>>,
    Balance: From<u128>,
    N: AsRef<[u8]>,
{
    let ticker = make_ticker::<Asset, _, _, _, _>(owner.origin().into(), name)?;
    let name: AssetName = ticker.as_slice().into();
    let total_supply: Balance = (1_000_000 * POLY).into();

    Asset::create_asset(
        owner.origin().into(),
        name.clone(),
        ticker,
        divisible,
        AssetType::default(),
        vec![],
        None,
    )
    .map_err(|_| "Asset cannot be created")?;

    Asset::issue(owner.origin().into(), ticker, total_supply)
        .map_err(|_| "Asset cannot be issued")?;

    Ok(ticker)
}
