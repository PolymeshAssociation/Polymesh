// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymath

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use core::convert::TryInto;
use frame_benchmarking::benchmarks;
use polymesh_common_utilities::{
    asset::Config as AssetConfig,
    benchs::{make_asset, user, AccountIdOf, User, UserBuilder},
    constants::currency::ONE_UNIT,
    TestUtilsFn,
};
use polymesh_primitives::{AuthorizationData, NFTs, PortfolioName, Signatory};
use scale_info::prelude::format;
use sp_api_hidden_includes_decl_storage::hidden_include::traits::Get;
use sp_std::prelude::*;

use crate::*;

const PORTFOLIO_NAME_LEN: usize = 500;

fn make_worst_memo() -> Option<Memo> {
    Some(Memo([7u8; 32]))
}

fn owner_portfolio<T: Config + TestUtilsFn<<T as frame_system::Config>::AccountId>>(
) -> (User<T>, PortfolioId) {
    let owner = user::<T>("owner", 0);

    let name = PortfolioName(vec![65u8; PORTFOLIO_NAME_LEN as usize]);
    let num = NextPortfolioNumber::get(&owner.did());
    Module::<T>::create_portfolio(owner.origin.clone().into(), name.clone()).unwrap();
    let pid = PortfolioId::user_portfolio(owner.did(), num.clone());

    (owner, pid)
}

fn add_auth<T: Config>(owner: &User<T>, custodian: &User<T>, pid: PortfolioId) -> u64 {
    identity::Module::<T>::add_auth(
        owner.did(),
        Signatory::from(custodian.did()),
        AuthorizationData::PortfolioCustody(pid),
        None,
    )
}

fn assert_custodian<T: Config>(pid: PortfolioId, custodian: &User<T>, holds: bool) {
    assert_eq!(
        PortfolioCustodian::get(&pid),
        holds.then(|| custodian.did())
    );
    assert_eq!(PortfoliosInCustody::get(&custodian.did(), &pid), holds);
}

benchmarks! {
    where_clause { where T: TestUtilsFn<AccountIdOf<T>> + AssetConfig }

    create_portfolio {
        let target = user::<T>("target", 0);
        let did = target.did();
        let portfolio_name = PortfolioName(vec![65u8; PORTFOLIO_NAME_LEN]);
        let next_portfolio_num = NextPortfolioNumber::get(&did);
    }: _(target.origin, portfolio_name.clone())
    verify {
        assert_eq!(Portfolios::get(&did, &next_portfolio_num), portfolio_name);
    }

    delete_portfolio {
        let target = user::<T>("target", 0);
        let did = target.did();
        let portfolio_name = PortfolioName(vec![65u8; 5]);
        let next_portfolio_num = NextPortfolioNumber::get(&did);
        Module::<T>::create_portfolio(target.origin.clone().into(), portfolio_name.clone()).unwrap();
        assert_eq!(Portfolios::get(&did, &next_portfolio_num), portfolio_name);
    }: _(target.origin, next_portfolio_num.clone())
    verify {
        assert!(!Portfolios::contains_key(&did, &next_portfolio_num));
    }

    move_portfolio_funds {
        // Number of assets being moved
        let a in 1 .. 500;
        let mut items = Vec::with_capacity(a as usize);
        let target = user::<T>("target", 0);
        let first_ticker = Ticker::generate_into(0u64);
        let amount = Balance::from(10u32);
        let portfolio_name = PortfolioName(vec![65u8; 5]);
        let next_portfolio_num = NextPortfolioNumber::get(&target.did());
        let default_portfolio = PortfolioId::default_portfolio(target.did());
        let user_portfolio = PortfolioId::user_portfolio(target.did(), next_portfolio_num.clone());

        for x in 0..a as u64 {
            let ticker = make_asset::<T>(&target, Some(&Ticker::generate(x)));
            items.push(MovePortfolioItem {
                ticker,
                amount: amount,
                memo: make_worst_memo(),
            });
            PortfolioAssetBalances::insert(&default_portfolio, &ticker, amount);
        }

        Module::<T>::create_portfolio(target.origin.clone().into(), portfolio_name.clone()).unwrap();

        assert_eq!(PortfolioAssetBalances::get(&default_portfolio, &first_ticker), amount);
        assert_eq!(PortfolioAssetBalances::get(&user_portfolio, &first_ticker), 0u32.into());
    }: _(target.origin, default_portfolio, user_portfolio, items)
    verify {
        assert_eq!(PortfolioAssetBalances::get(&default_portfolio, &first_ticker), 0u32.into());
        assert_eq!(PortfolioAssetBalances::get(&user_portfolio, &first_ticker), amount);
    }

    rename_portfolio {
        // Length of portfolio name
        let i in 1 .. PORTFOLIO_NAME_LEN.try_into().unwrap();

        let target = user::<T>("target", 0);
        let did = target.did();
        let portfolio_name = PortfolioName(vec![65u8; i as usize]);
        let next_portfolio_num = NextPortfolioNumber::get(&did);
        Module::<T>::create_portfolio(target.origin.clone().into(), portfolio_name.clone()).unwrap();
        assert_eq!(Portfolios::get(&did, &next_portfolio_num), portfolio_name);
        let new_name = PortfolioName(vec![66u8; i as usize]);

    }: _(target.origin, next_portfolio_num.clone(), new_name.clone())
    verify {
        assert_eq!(Portfolios::get(&did, &next_portfolio_num), new_name);
    }

    quit_portfolio_custody {
        let (owner, user_portfolio) = owner_portfolio::<T>();

        // Transfer the custody of the portfolio from `owner` to `custodian`.
        let custodian = user::<T>("custodian", 0);
        let auth_id = add_auth::<T>(&owner, &custodian, user_portfolio);
        Module::<T>::accept_portfolio_custody(custodian.origin.clone().into(), auth_id)?;

        assert_custodian::<T>(user_portfolio, &custodian, true);
    }: _(custodian.origin.clone(), user_portfolio)
    verify {
        assert_custodian::<T>(user_portfolio, &custodian, false);
    }

    accept_portfolio_custody {
        let (owner, user_portfolio) = owner_portfolio::<T>();

        let custodian = user::<T>("custodian", 0);
        let auth_id = add_auth::<T>(&owner, &custodian, user_portfolio);
        assert_custodian::<T>(user_portfolio, &custodian, false);
    }: _(custodian.origin.clone(), auth_id)
    verify {
        assert_custodian::<T>(user_portfolio, &custodian, true);
    }

    move_portfolio_funds_v2 {
        let f in 1..T::MaxNumberOfFungibleMoves::get() as u32;
        let n in 1..T::MaxNumberOfNFTsMoves::get() as u32;

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let alice_default_portfolio = PortfolioId { did: alice.did(), kind: PortfolioKind::Default };
        let alice_custom_portfolio = PortfolioId { did: alice.did(), kind: PortfolioKind::User(PortfolioNumber(1)) };
        let nft_ticker: Ticker = b"TICKERNFT".as_ref().try_into().unwrap();
        Module::<T>::create_portfolio(alice.clone().origin().into(), PortfolioName(b"MyOwnPortfolio".to_vec())).unwrap();
        // Simulates minting - Adding the NFT pallet causes cyclic dependency
        (1..n + 1).for_each(|id| PortfolioNFT::insert(alice_default_portfolio, (nft_ticker, NFTId(id.into())), true));

        let nfts = NFTs::new_unverified(nft_ticker, (1..n + 1).map(|id| NFTId(id.into())).collect());
        let mut funds = vec![Fund { description: FundDescription::NonFungible(nfts), memo: None }];
        for i in 0..f {
            let ticker = make_asset(&alice, Some(format!("TICKER{}", i).as_bytes()));
            funds.push(Fund { description: FundDescription::Fungible{ ticker, amount: ONE_UNIT }, memo: None })
        }
    }: _(alice.origin, alice_default_portfolio.clone(), alice_custom_portfolio.clone(), funds)
    verify {
        for i in 1..n + 1 {
            assert_eq!(PortfolioNFT::get(&alice_default_portfolio, (&nft_ticker, NFTId(i as u64))), false);
            assert_eq!(PortfolioNFT::get(&alice_custom_portfolio, (&nft_ticker, NFTId(i as u64))), true);
        }
    }
}
