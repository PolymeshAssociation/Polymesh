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

//! Storage migrations for `pallet-pips`.

use frame_support::traits::Get;
use frame_support::weights::Weight;

use crate::{Config, ProposalProposers, Proposals};

pub(crate) fn migrate_to_v3<T: Config>() -> Weight {
    let mut count: u64 = 0;
    for (id, pip) in Proposals::<T>::iter() {
        ProposalProposers::<T>::insert(id, pip.proposer);
        count = count.saturating_add(1);
    }

    log::info!(
        target: "runtime::pips",
        "migrate_to_v3: {} ProposalProposers entries",
        count,
    );

    T::DbWeight::get().reads_writes(count, count)
}
