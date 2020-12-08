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
use pallet_asset::benchmarking::make_asset;
use pallet_identity::benchmarking::{User, UserBuilder};
use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;

const SEED: u32 = 0;

fn setup<T: Trait>() -> (User<T>, Ticker) {
    let owner = UserBuilder::<T>::default().build_with_did("owner", SEED);
    let ticker = make_asset::<T>(&owner);
    (owner, ticker)
}

benchmarks! {
    _ {}

    set_max_details_length {}: _(RawOrigin::Root, 100)
    verify {
        ensure!(MaxDetailsLength::get() == 100, "Wrong length set");
    }

    reset_caa {
        let (owner, ticker) = setup::<T>();
        // Generally the code path for no CAA is more complex,
        // but in this case having a different CAA already could cause more storage writes.
        let caa = UserBuilder::<T>::default().build_with_did("caa", SEED);
        Agent::insert(ticker, caa.did());
    }: _(owner.origin(), ticker)
    verify {
        ensure!(Agent::get(ticker) == None, "CAA not reset.");
    }
}
