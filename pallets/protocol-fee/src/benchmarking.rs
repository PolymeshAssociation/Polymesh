// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
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
use frame_system::RawOrigin;
use polymesh_common_utilities::protocol_fee::ProtocolOp;
use polymesh_primitives::PosRatio;

benchmarks! {
    change_coefficient {
        let origin = RawOrigin::Root;
        let coefficient = PosRatio::from((0, 0));
    }: _(origin, coefficient)

    change_base_fee {
        let origin = RawOrigin::Root;
        let op = ProtocolOp::AssetRegisterTicker;
    }: _(origin, op, 0)
}
