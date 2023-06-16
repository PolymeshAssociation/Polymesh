// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymesh Association

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use crate::traits::group::GroupTrait;
use polymesh_primitives::IdentityId;

pub trait GovernanceGroupTrait<Moment: PartialOrd + Copy>: GroupTrait<Moment> {
    fn release_coordinator() -> Option<IdentityId>;

    #[cfg(feature = "runtime-benchmarks")]
    fn bench_set_release_coordinator(did: IdentityId);
}
