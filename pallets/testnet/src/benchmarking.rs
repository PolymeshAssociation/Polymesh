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

use crate::*;

use pallet_identity::benchmarking::generate_secondary_keys;
use polymesh_common_utilities::benchs::{uid_from_name_and_idx, UserBuilder};

use frame_benchmarking::{account, benchmarks};
use sp_std::prelude::*;

const SEED: u32 = 0;
#[cfg(feature = "running-ci")]
mod limits {
    pub const MAX_SECONDARY_KEYS: u32 = 2;
}

#[cfg(not(feature = "running-ci"))]
mod limits {
    pub const MAX_SECONDARY_KEYS: u32 = 100;
}

use limits::*;

benchmarks! {
    _ {}

    register_did {
        // Number of secondary items.
        let i in 0 .. MAX_SECONDARY_KEYS;

        let _cdd = UserBuilder::<T>::default().generate_did().become_cdd_provider().build("cdd");
        let caller = UserBuilder::<T>::default().build("caller");
        let uid = uid_from_name_and_idx("caller", SEED);
        let secondary_keys = generate_secondary_keys::<T>(i as usize);
    }: _(caller.origin, uid, secondary_keys)

    mock_cdd_register_did {
        let cdd = UserBuilder::<T>::default().generate_did().become_cdd_provider().build("cdd");
        let target: T::AccountId = account("target", SEED, SEED);
    }: _(cdd.origin, target)
}
