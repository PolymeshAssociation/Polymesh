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
use polymesh_common_utilities::{
    benchs::{cdd_provider, uid_from_name_and_idx, user, AccountIdOf, UserBuilder},
    TestUtilsFn,
};

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
    where_clause { where T: TestUtilsFn<AccountIdOf<T>> }

    register_did {
        // Number of secondary items.
        let i in 0 .. MAX_SECONDARY_KEYS;

        let _cdd =  cdd_provider::<T>("cdd", SEED);
        let caller = UserBuilder::<T>::default().build("caller");
        let uid = uid_from_name_and_idx("caller", SEED);
        let secondary_keys = generate_secondary_keys::<T>(i as usize);
    }: _(caller.origin, uid, secondary_keys)
    verify {

    }

    mock_cdd_register_did {
        let cdd =  cdd_provider::<T>("cdd", SEED);
        let target: T::AccountId = account("target", SEED, SEED);
    }: _(cdd.origin, target)
    verify {
    }

    get_my_did {
        let user = user::<T>("user", SEED);
    }: _(user.origin)

    get_cdd_of {
        let cdd =  cdd_provider::<T>("cdd", SEED);
        let user = UserBuilder::<T>::default().build("user");

        Module::<T>::mock_cdd_register_did(cdd.origin().into(), user.account())
            .expect("CDD provider cannot generate a DID for that user");
    }: _(user.origin, user.account)
}
