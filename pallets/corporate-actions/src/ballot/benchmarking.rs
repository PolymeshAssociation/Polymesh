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

use super::*;
use core::iter;
use crate::benchmarking::setup_ca;
use frame_benchmarking::benchmarks;

const MAX_MOTIONS: u32 = 10;
const MAX_CHOICES: u32 = 10;

benchmarks! {
    _ {}

    attach_ballot {
        let i in 0..MAX_MOTIONS;
        let j in 0..MAX_CHOICES;

        let motion = Motion {
            title: "".into(),
            info_link: "".into(),
            choices: iter::repeat("".into()).take(j as usize).collect(),
        };
        let motions = iter::repeat(motion).take(i as usize).collect();

        let meta = BallotMeta { title: "".into(), motions };
        let range = BallotTimeRange { start: 3000, end: 4000 };
        let (owner, ca_id) = setup_ca::<T>(CAKind::IssuerNotice);
    }: _(owner.origin(), ca_id, range, meta, true)
    verify {
        ensure!(TimeRanges::get(ca_id) == Some(range), "ballot not created");
    }
}
