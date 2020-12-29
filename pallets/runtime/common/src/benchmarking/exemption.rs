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

use polymesh_common_utilities::{
    benchs::user,
    traits::asset::{AssetName, AssetType, Trait as AssetTrait},
};
use polymesh_primitives::{ticker::TICKER_LEN, Ticker};

use frame_benchmarking::benchmarks;
use frame_support::ensure;
use sp_std::{convert::TryFrom, prelude::*};

benchmarks! {
    _ {}

    modify_exemption_list {
        let owner = user::<T>("owner", 0);
        let holder = user::<T>("holder", 0).did();

        let tm = 1u16;

        // Create the asset.
        let ticker = Ticker::try_from(vec![b'A'; TICKER_LEN as usize].as_slice()).unwrap();
        let name: AssetName = ticker.as_slice().into();
        let total_supply: T::Balance = 1_000_000.into();


        T::Asset::create_asset(
            owner.origin().into(),
            name,
            ticker.clone(),
            total_supply,
            true,
            AssetType::default(),
            vec![],
            None).expect("Asset cannot be created");

    }: _(owner.origin(), ticker, tm, holder.clone(), true)
    verify {
        let exemption_idx = (ticker, tm, holder);
        ensure!(
            Module::<T>::exemption_list(&exemption_idx) == true,
            "Exemption list was not updated");
    }
}
