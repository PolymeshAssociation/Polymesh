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
//use pallet_identity::{self as identity, benchmarking::{uid_from_name_and_idx, make_account, make_account_without_did}};
use pallet_identity::benchmarking::make_account;
use polymesh_primitives::PortfolioName;
use sp_std::prelude::*;

benchmarks! {
    _ {}

    create_portfolio {
        // Length of portfolio name
        let i in 0 .. 500;

        let (_, target_origin, target_did) = make_account::<T>("target", 0);
        let portfolio_name = PortfolioName(vec![65u8; i as usize]);
        let next_portfolio_num = NextPortfolioNumber::get(&target_did);

    }: _(target_origin, portfolio_name.clone())
    verify {
        assert_eq!(Portfolios::get(&target_did, &next_portfolio_num), portfolio_name);
    }
}
