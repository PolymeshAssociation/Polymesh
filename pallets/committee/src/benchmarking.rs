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
use frame_support::traits::UnfilteredDispatchable;
use frame_system::RawOrigin;
use pallet_identity::benchmarking::UserBuilder;
use polymesh_common_utilities::MaybeBlock;
use sp_std::prelude::*;

const COMMITTEE_MEMBERS_NUM: usize = 10;
const PROPOSAL_PADDING_MAX: u32 = 10_000;

benchmarks_instance! {
    _ {}

    set_vote_threshold {
        let n = 1;
        let d = 2;
        let origin = T::CommitteeOrigin::successful_origin();
        let call = Call::<T, I>::set_vote_threshold(n, d);
    }: {
        call.dispatch_bypass_filter(origin)?;
    }
    verify {
        ensure!(Module::<T, _>::vote_threshold() == (n, d), "incorrect vote threshold");
    }

    set_release_coordinator {
        let dids: Vec<_> = (0..COMMITTEE_MEMBERS_NUM)
            .map(|i| UserBuilder::<T>::default().build_with_did("member", i as u32).did())
            .collect();
        let coordinator = dids[COMMITTEE_MEMBERS_NUM / 2].clone();
        Members::<I>::put(dids);
        let origin = T::CommitteeOrigin::successful_origin();
        let call = Call::<T, I>::set_release_coordinator(coordinator);
    }: {
        call.dispatch_bypass_filter(origin)?;
    }
    verify {
        ensure!(
            Module::<T, _>::release_coordinator() == Some(coordinator),
            "incorrect release coordinator"
        );
    }

    set_expires_after {
        let maybe_block = MaybeBlock::Some(1.into());
        let origin = T::CommitteeOrigin::successful_origin();
        let call = Call::<T, I>::set_expires_after(maybe_block);
    }: {
        call.dispatch_bypass_filter(origin)?;
    }
    verify {
        ensure!(Module::<T, _>::expires_after() == maybe_block, "incorrect expiration");
    }

    vote_or_propose {
        let p in 1 .. PROPOSAL_PADDING_MAX;

        let user = UserBuilder::<T>::default().build_with_did("proposer", 0);
//        ReleaseCoordinator::<I>::put(user.did());
        Members::<I>::put(vec![user.did()]);
        let proposal: <T as Trait<I>>::Proposal =
            frame_system::Call::<T>::remark(vec![1; p as usize]).into();
        let hash = <T as frame_system::Trait>::Hashing::hash_of(&proposal);
    }: _(user.origin, true, Box::new(proposal.clone()))
    verify {
        ensure!(ProposalOf::<T, I>::get(&hash) == Some(proposal), "didn't propose");
    }

    vote {
        let p in 1 .. PROPOSAL_PADDING_MAX;
        let proposal: <T as Trait<I>>::Proposal =
            frame_system::Call::<T>::remark(vec![1; p as usize]).into();
    }: {}
    verify {
    }

    close {
    }: {}
    verify {
    }
}
