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
use core::iter;

const SEED: u32 = 0;
const MAX_TARGET_IDENTITIES: u32 = 100;

// NOTE(Centril): A non-owner CAA is the less complex code path.
// Therefore, in general, we'll be using the owner as the CAA.

fn user<T: Trait>(prefix: &'static str, u: u32) -> User<T> {
    UserBuilder::<T>::default().build_with_did(prefix, u)
}

fn setup<T: Trait>() -> (User<T>, Ticker) {
    let owner = user("owner", SEED);
    let ticker = make_asset::<T>(&owner);
    (owner, ticker)
}

fn target_ids<T: Trait>(n: u32, treatment: TargetTreatment) -> TargetIdentities {
    let identities = (0..n)
        .flat_map(|i| iter::repeat(user::<T>("target", i).did()).take(2))
        .collect::<Vec<_>>();
    TargetIdentities { identities, treatment }
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

    set_default_targets {
        let (owner, ticker) = setup::<T>();
        let i in 0..MAX_TARGET_IDENTITIES;
        let targets = target_ids::<T>(i, TargetTreatment::Exclude);
        let targets2 = targets.clone();
    }: _(owner.origin(), ticker, targets)
    verify {
        ensure!(DefaultTargetIdentities::get(ticker) == targets2.dedup(), "Default targets not set");
    }

    set_default_withholding_tax {
        let (owner, ticker) = setup::<T>();
    }: _(owner.origin(), ticker, Tax::one())
    verify {
        ensure!(DefaultWithholdingTax::get(ticker) == Tax::one(), "Default WHT not set");
    }
}
