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
use crate::exemption::*;

use polymesh_common_utilities::benchs::{make_asset, user};

use frame_benchmarking::benchmarks;
use frame_support::ensure;
use sp_std::prelude::*;

benchmarks! {
    _ {}

    modify_exemption_list {
        let owner = user::<T>("owner", 0);
        let holder = user::<T>("holder", 0).did();

        let ticker = make_asset::<T::Asset, T, T::Balance, T::AccountId, T::Origin>(&owner);
        let holder_exp = holder.clone();
        let tm = 1u16;
    }: _(owner.origin(), ticker, tm, holder, true)
    verify {
        let exemption_idx = (ticker, tm, holder_exp);
        ensure!(
            Module::<T>::exemption_list(&exemption_idx) == true,
            "Exemption list was not updated");
    }
}
