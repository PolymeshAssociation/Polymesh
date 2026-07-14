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
use frame_benchmarking::benchmarks;
use pallet_identity::benchmarking::User;

use super::*;
use crate::ballot::{MAX_CHOICES_PER_MOTION, MAX_MOTIONS};
use crate::benchmarking::{set_ca_targets, setup_ca};
use crate::CAConfig;

const MAX_TARGETS: u32 = 1000;

const RANGE: BallotTimeRange = BallotTimeRange {
    start: 3000,
    end: 4000,
};

fn meta<T: CAConfig>(n_motions: u32, n_choices: u32) -> BallotMeta {
    let max_len = T::MaxLen::get();

    let choices = (0..n_choices)
        .map(|_| vec![0u8; max_len as usize].into())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    let single_motion = Motion {
        title: vec![0u8; max_len as usize].into(),
        info_link: vec![0u8; max_len as usize].into(),
        choices,
    };

    let motions = (0..n_motions)
        .map(|_| single_motion.clone())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    BallotMeta {
        title: vec![0u8; max_len as usize].into(),
        motions,
    }
}

fn attach<T: CAConfig>(n_motions: u32, n_choices: u32) -> (User<T>, CAId) {
    let meta = meta::<T>(n_motions, n_choices);
    let (owner, ca_id) = setup_ca::<T>(CAKind::IssuerNotice);
    <Pallet<T>>::attach_ballot(owner.origin().into(), ca_id, RANGE, meta, true).unwrap();
    (owner, ca_id)
}

benchmarks! {
    where_clause {  where T: CAConfig }

    attach_ballot {
        let m in 1..MAX_MOTIONS as u32;
        let c in 0..MAX_CHOICES_PER_MOTION as u32;

        let meta = meta::<T>(m, c);
        let (owner, ca_id) = setup_ca::<T>(CAKind::IssuerNotice);
    }: _(owner.origin(), ca_id, RANGE, meta, true)
    verify {
        assert_eq!(TimeRanges::<T>::get(ca_id), Some(RANGE), "ballot not created");
    }

    vote {
        let c in 0..MAX_CHOICES_PER_MOTION as u32;
        let t in 0..MAX_TARGETS;

        // Attach and prepare to vote.
        let (owner, ca_id) = attach::<T>(1, c);
        <pallet_timestamp::Now<T>>::set(3000u32.into());

        // Change targets, as they are read in voting.
        set_ca_targets::<T>(ca_id, t);

        // Construct the voting list.
        let votes = (0..c)
            .map(|c| BallotVote {
                power: 0u32.into(),
                fallback: (c as u16).checked_sub(1),
            })
            .collect::<Vec<_>>();

        // Vote already to force a longer code path.
        <Pallet<T>>::vote(owner.origin().into(), ca_id, votes.clone()).unwrap();
        let results = votes.iter().map(|v| v.power).collect::<Vec<_>>();
    }: _(owner.origin(), ca_id, votes)
    verify {
        assert_eq!(Results::<T>::get(ca_id), results, "voting results are wrong")
    }

    change_end {
        let (owner, ca_id) = attach::<T>(1, 1);
    }: _(owner.origin(), ca_id, 5000)
    verify {
        assert_eq!(TimeRanges::<T>::get(ca_id).unwrap().end, 5000, "range not changed");
    }

    change_meta {
        let m in 1..MAX_MOTIONS as u32;
        let c in 0..MAX_CHOICES_PER_MOTION as u32;

        let (owner, ca_id) = attach::<T>(1, 1);
        let meta = meta::<T>(m, c);
        let meta2 = meta.clone();
    }: _(owner.origin(), ca_id, meta)
    verify {
        assert_eq!(Metas::<T>::get(ca_id).unwrap(), meta2, "meta not changed");
    }

    change_rcv {
        let (owner, ca_id) = attach::<T>(1, 1);
    }: _(owner.origin(), ca_id, false)
    verify {
        assert!(!RCV::<T>::get(ca_id), "RCV not changed");
    }

    remove_ballot {
        let (owner, ca_id) = attach::<T>(1, 1);
    }: _(owner.origin(), ca_id)
    verify {
        assert_eq!(TimeRanges::<T>::get(ca_id), None, "ballot not removed");
    }
}
