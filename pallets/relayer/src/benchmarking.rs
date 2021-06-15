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

use frame_benchmarking::benchmarks;
use polymesh_common_utilities::{
    benchs::{user, AccountIdOf},
    traits::{relayer::Trait, TestUtilsFn},
};
use sp_std::prelude::*;

pub(crate) const SEED: u32 = 0;

benchmarks! {
    where_clause { where T: Trait, T: TestUtilsFn<AccountIdOf<T>> }

    _ {}

    set_paying_key {
        let payer = user::<T>("payer", SEED);
        let user = user::<T>("user", SEED);
    }: _(payer.origin(), user.account())
}
