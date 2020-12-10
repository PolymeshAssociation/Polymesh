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

use super::*;
use core::iter;
use crate::{CorporateActions, TargetTreatment, CAKind};
use crate::benchmarking::{setup_ca, target_ids, currency};
use pallet_identity::benchmarking::User;
use frame_benchmarking::benchmarks;
use frame_support::assert_ok;
use pallet_timestamp::Module as Timestamp;
use pallet_portfolio::MovePortfolioItem;

benchmarks! {
    _ {}

    distribute {
        let (owner, ca_id) = setup_ca::<T>(CAKind::UnpredictableBenefit);
        let currency = currency::<T>(&owner);

        let amount = 1000.into();
        let did = owner.did();
        let origin: T::Origin = owner.origin().into();
        let pnum = 1.into();
        assert_ok!(<Portfolio<T>>::create_portfolio(origin.clone(), "portfolio".into()));
        assert_ok!(<Portfolio<T>>::move_portfolio_funds(
            origin,
            PortfolioId::default_portfolio(did),
            PortfolioId::user_portfolio(did, pnum),
            vec![MovePortfolioItem { ticker: currency, amount }],
        ));
    }: _(owner.origin(), ca_id, Some(pnum), currency, amount, 3000, Some(4000))
    verify {
        ensure!(<Distributions<T>>::get(ca_id).is_some(), "distribution not created");
    }
}
