// This file is part of the Polymesh distribution (https://github.com/PolymathNetwork/Polymesh).
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

#![cfg(feature = "runtime-benchmarks")]
use crate::*;
use frame_benchmarking::benchmarks;
use pallet_identity::benchmarking::make_account;
use polymesh_primitives::PortfolioName;
use sp_std::{convert::TryFrom, prelude::*};

/// Given a number, this function generates a ticker with
/// A-Z, least number of characters in Lexicographic order
fn generate_ticker(n: u64) -> Ticker {
    fn calc_base26(n: u64, base_26: &mut Vec<u8>) {
        if n >= 26 {
            // Subtracting 1 is not required and shouldn't be done for a proper base_26 conversion
            // However, without this hack, B will be the first char after a bump in number of chars.
            // i.e. the sequence will go A,B...Z,BA,BB...ZZ,BAA. We want the sequence to start with A.
            // Subtracting 1 here means we are doing 1 indexing rather than 0.
            // i.e. A = 1, B = 2 instead of A = 0, B = 1
            calc_base26((n / 26) - 1, base_26);
        }
        let character = n % 26 + 65;
        base_26.push(character as u8);
    }
    let mut base_26 = Vec::new();
    calc_base26(n, &mut base_26);
    Ticker::try_from(base_26.as_slice()).unwrap()
}

benchmarks! {
    _ {}

    create_portfolio {
        // Length of portfolio name
        let i in 1 .. 500;

        let (_, target_origin, target_did) = make_account::<T>("target", 0);
        let portfolio_name = PortfolioName(vec![65u8; i as usize]);
        let next_portfolio_num = NextPortfolioNumber::get(&target_did);
    }: _(target_origin, portfolio_name.clone())
    verify {
        assert_eq!(Portfolios::get(&target_did, &next_portfolio_num), portfolio_name);
    }

    delete_portfolio {
        let (_, target_origin, target_did) = make_account::<T>("target", 0);
        let portfolio_name = PortfolioName(vec![65u8; 5]);
        let next_portfolio_num = NextPortfolioNumber::get(&target_did);
        Module::<T>::create_portfolio(target_origin.clone().into(), portfolio_name.clone())?;
        assert_eq!(Portfolios::get(&target_did, &next_portfolio_num), portfolio_name);
    }: _(target_origin, next_portfolio_num.clone())
    verify {
        assert!(!Portfolios::contains_key(&target_did, &next_portfolio_num));
    }

    move_portfolio_funds {
        // Number of assets being moved
        let i in 1 .. 500;
        let mut items = Vec::with_capacity(i as usize);
        let (_, target_origin, target_did) = make_account::<T>("target", 0);
        let first_ticker = generate_ticker(0u64);
        let amount = T::Balance::from(10);
        let portfolio_name = PortfolioName(vec![65u8; 5]);
        let next_portfolio_num = NextPortfolioNumber::get(&target_did);
        let default_portfolio = PortfolioId::default_portfolio(target_did);
        let user_portfolio = PortfolioId::user_portfolio(target_did, next_portfolio_num.clone());

        for x in 0..i as u64 {
            let ticker = generate_ticker(x);
            items.push(MovePortfolioItem {
                ticker,
                amount: amount,
            });
            <PortfolioAssetBalances<T>>::insert(&default_portfolio, &ticker, amount);
        }

        Module::<T>::create_portfolio(target_origin.clone().into(), portfolio_name.clone())?;

        assert_eq!(<PortfolioAssetBalances<T>>::get(&default_portfolio, &first_ticker), amount);
        assert_eq!(<PortfolioAssetBalances<T>>::get(&user_portfolio, &first_ticker), 0.into());
    }: _(target_origin, default_portfolio, user_portfolio, items)
    verify {
        assert_eq!(<PortfolioAssetBalances<T>>::get(&default_portfolio, &first_ticker), 0.into());
        assert_eq!(<PortfolioAssetBalances<T>>::get(&user_portfolio, &first_ticker), amount);
    }

    rename_portfolio {
        // Length of portfolio name
        let i in 1 .. 500;

        let (_, target_origin, target_did) = make_account::<T>("target", 0);
        let portfolio_name = PortfolioName(vec![65u8; i as usize]);
        let next_portfolio_num = NextPortfolioNumber::get(&target_did);
        Module::<T>::create_portfolio(target_origin.clone().into(), portfolio_name.clone())?;
        assert_eq!(Portfolios::get(&target_did, &next_portfolio_num), portfolio_name);
        let new_name = PortfolioName(vec![66u8; i as usize]);

    }: _(target_origin, next_portfolio_num.clone(), new_name.clone())
    verify {
        assert_eq!(Portfolios::get(&target_did, &next_portfolio_num), new_name);
    }
}
