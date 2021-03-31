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
use frame_support::StorageValue;
use frame_system::RawOrigin;
use polymesh_common_utilities::{
    benchs::{self, AccountIdOf, User, UserBuilder},
    constants::currency::POLY,
    TestUtilsFn,
};
use polymesh_contracts::ExtensionInfo;
use polymesh_primitives::{
    asset::AssetName, ticker::TICKER_LEN, ExtensionAttributes, SmartExtension, Ticker,
};
use sp_io::hashing::keccak_256;
use sp_std::{convert::TryInto, iter, prelude::*};

benchmarks! {
    _ {}

    todo {
    }: {

    }
    verify {
    }
}
