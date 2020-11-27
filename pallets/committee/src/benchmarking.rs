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
use frame_benchmarking::benchmarks_instance;
use frame_system::RawOrigin;
use pallet_identity::benchmarking::UserBuilder;
use sp_std::{convert::TryFrom, prelude::*};

const COMMITTEE_MEMBERS_NUM: usize = 10;

benchmarks_instance! {
    _ {}

    set_vote_threshold {
        let n = 1;
        let d = 2;
    }: _(RawOrigin::Root, n, d)
    verify {
        ensure!(Module::<T, _>::vote_threshold() == (n, d), "incorrect vote threshold");
    }

    set_release_coordinator {
        let dids: Vec<_> = (0..COMMITTEE_MEMBERS_NUM)
            .map(|i| UserBuilder::<T>::default().build_with_did("member", i as u32).did())
            .collect();
        let coordinator = dids[COMMITTEE_MEMBERS_NUM / 2].clone();
        Members::<I>::put(dids);
    }: _(RawOrigin::Root, coordinator)
    verify {
        ensure!(
            Module::<T, _>::release_coordinator() == Some(coordinator),
            "incorrect release coordinator"
        );
    }
}
