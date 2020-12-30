use crate::{
    benchs::User,
    traits::{
        asset::{AssetFnTrait, AssetName, AssetType},
        identity::Trait as IdentityTrait,
    },
};

use polymesh_primitives::{ticker::TICKER_LEN, Ticker};

use frame_system::RawOrigin;
use sp_std::{convert::TryFrom, vec};

/// Create a ticker and register it.
pub fn make_ticker<Asset, Balance, Acc, O>(owner: O) -> Ticker
where
    Asset: AssetFnTrait<Balance, Acc, O>,
{
    let ticker = Ticker::try_from(vec![b'A'; TICKER_LEN as usize].as_slice()).unwrap();
    Asset::register_ticker(owner, ticker).unwrap();

    ticker
}

pub fn make_asset<Asset, Identity, Balance, Acc, Origin>(owner: &User<Identity>) -> Ticker
where
    Asset: AssetFnTrait<Balance, Acc, Origin>,
    Identity: IdentityTrait,
    Origin: From<RawOrigin<<Identity as frame_system::Trait>::AccountId>>,
    Balance: From<u128>,
{
    make_base_asset::<Asset, Identity, Balance, Acc, Origin>(owner, true)
}

pub fn make_indivisible_asset<Asset, Identity, Balance, Acc, Origin>(
    owner: &User<Identity>,
) -> Ticker
where
    Asset: AssetFnTrait<Balance, Acc, Origin>,
    Identity: IdentityTrait,
    Origin: From<RawOrigin<<Identity as frame_system::Trait>::AccountId>>,
    Balance: From<u128>,
{
    make_base_asset::<Asset, Identity, Balance, Acc, Origin>(owner, false)
}

fn make_base_asset<Asset, Identity, Balance, Acc, Origin>(
    owner: &User<Identity>,
    divisible: bool,
) -> Ticker
where
    Asset: AssetFnTrait<Balance, Acc, Origin>,
    Identity: IdentityTrait,
    Origin: From<RawOrigin<<Identity as frame_system::Trait>::AccountId>>,
    Balance: From<u128>,
{
    let ticker = make_ticker::<Asset, _, _, _>(owner.origin().into());
    let name: AssetName = ticker.as_slice().into();
    let total_supply: Balance = 1_000_000u128.into();

    Asset::create_asset(
        owner.origin().into(),
        name,
        ticker,
        total_supply,
        divisible,
        AssetType::default(),
        vec![],
        None,
    )
    .expect("Asset cannot be created");

    ticker
}
