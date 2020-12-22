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
use frame_support::{dispatch::DispatchResult, traits::UnfilteredDispatchable, StorageValue};
use polymesh_common_utilities::{
    benchs::{User, UserBuilder},
    MaybeBlock,
};
use sp_std::prelude::*;

const COMMITTEE_MEMBERS_NUM: usize = 10;
const COMMITTEE_MEMBERS_MAX: u32 = 100;
const PROPOSAL_PADDING_LEN: usize = 10_000;
const PROPOSALS_NUM: u8 = 100;

fn make_proposals_and_vote<T, I>(users: &[User<T>]) -> DispatchResult
where
    I: Instance,
    T: Trait<I>,
{
    for i in 0..PROPOSALS_NUM {
        let index = Module::<T, I>::proposal_count();
        let proposal: <T as Trait<I>>::Proposal =
            frame_system::Call::<T>::remark(vec![i + 1; PROPOSAL_PADDING_LEN]).into();
        identity::CurrentDid::put(users[0].did());
        Module::<T, I>::vote_or_propose(users[0].origin.clone().into(), true, Box::new(proposal))?;
        if users.len() > 1 {
            let hash = *Module::<T, I>::proposals()
                .last()
                .ok_or("missing last proposal")?;
            // cast max N-1 additional votes for proposal #N
            for (j, user) in users.iter().skip(1).take(i as usize).enumerate() {
                // Vote for the proposal if it's not finalised.
                if Module::<T, I>::voting(&hash).is_some() {
                    identity::CurrentDid::put(user.did());
                    Module::<T, I>::vote(user.origin.clone().into(), hash, index, j % 2 == 0)?;
                }
            }
        }
    }
    Ok(())
}

fn make_members_and_proposals<T, I>(m: u32) -> Result<Vec<User<T>>, DispatchError>
where
    I: Instance,
    T: Trait<I>,
{
    let members: Vec<_> = (0..m)
        .map(|i| {
            UserBuilder::<T>::default()
                .generate_did()
                .seed(i)
                .build("member")
        })
        .collect();
    Members::<I>::put(members.iter().map(|m| m.did()).collect::<Vec<_>>());
    make_proposals_and_vote::<T, I>(members.as_slice())?;
    members
}

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
            .map(|i| UserBuilder::<T>::default()
                 .generate_did()
                 .seed(i as u32)
                 .build("member")
                 .did()
            ).collect();
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

    vote_or_propose_new_proposal {
        let m in 2 .. COMMITTEE_MEMBERS_MAX;

        let members = make_members_and_proposals::<T, I>(m)?;
        let last_proposal_num = ProposalCount::<I>::get();
        let proposal: <T as Trait<I>>::Proposal =
            frame_system::Call::<T>::remark(vec![0; PROPOSAL_PADDING_LEN]).into();
        let hash = <T as frame_system::Trait>::Hashing::hash_of(&proposal);
        identity::CurrentDid::put(members[0].did());
    }: vote_or_propose(members[0].origin.clone(), true, Box::new(proposal.clone()))
    verify {
        // The proposal was stored.
        ensure!(Proposals::<T, I>::get().contains(&hash), "new proposal hash not found");
        ensure!(ProposalOf::<T, I>::get(&hash) == Some(proposal), "new proposal not found");
    }

    vote_or_propose_existing_proposal {
        let m in 2 .. COMMITTEE_MEMBERS_MAX;

        let members = make_members_and_proposals::<T, I>(m)?;
        let proposal: <T as Trait<I>>::Proposal =
            frame_system::Call::<T>::remark(vec![1; PROPOSAL_PADDING_LEN]).into();
        let hash = <T as frame_system::Trait>::Hashing::hash_of(&proposal);
        let proposals = Proposals::<T, I>::get();
        ensure!(proposals.contains(&hash), "cannot find the first proposal");
        identity::CurrentDid::put(members[1].did());
    }: vote_or_propose(members[1].origin.clone(), true, Box::new(proposal.clone()))
    verify {
        if m <= 4 {
            // Proposal was executed.
            ensure!(
                Module::<T, _>::voting(&hash).is_none(),
                "votes are present on an executed existing proposal"
            );
        } else {
            // The proposal was stored.
            ensure!(Proposals::<T, I>::get().contains(&hash), "existing proposal hash not found");
            ensure!(ProposalOf::<T, I>::get(&hash) == Some(proposal), "existing proposal not found");
        }
    }

    vote {
        let m in 2 .. COMMITTEE_MEMBERS_MAX;
        // reject or approve
        let a in 0 .. 1;

        let members = make_members_and_proposals::<T, I>(m)?;
        let proposal: <T as Trait<I>>::Proposal =
            frame_system::Call::<T>::remark(vec![1; PROPOSAL_PADDING_LEN]).into();
        let hash = <T as frame_system::Trait>::Hashing::hash_of(&proposal);
        let first_proposal_num = 0;
        let origin = members[1].origin.clone();
        let did = members[1].did();
        identity::CurrentDid::put(did);
    }: _(origin, hash, first_proposal_num, a != 0)
    verify {
        if m > 4 || (m == 4 && a == 0) {
            // The proposal is not finalised because there is no quorum yet.
            if let Some(votes) = Voting::<T, I>::get(&hash) {
                ensure!(votes.index == first_proposal_num, "wrong first proposal index");
                if a != 0 {
                    ensure!(votes.ayes.contains(&did), "aye vote missing");
                } else {
                    ensure!(votes.nays.contains(&did), "nay vote missing");
                }
            } else {
                return Err("cannot get votes".into());
            }
        } else {
            // The proposal is finalised and removed from storage.
            // TODO: pattern-match an event emitted during proposal finalisation.
            ensure!(!Voting::<T, I>::contains_key(&hash), "executed proposal is not removed");
        }
    }

    close {
        let m in 2 .. COMMITTEE_MEMBERS_MAX;

        let members = make_members_and_proposals::<T, I>(m)?;
        let proposal: <T as Trait<I>>::Proposal =
            frame_system::Call::<T>::remark(vec![1; PROPOSAL_PADDING_LEN]).into();
        let hash = <T as frame_system::Trait>::Hashing::hash_of(&proposal);
        let first_proposal_num = 0;
        let origin = members[0].origin.clone();
        let did = members[0].did();
        identity::CurrentDid::put(did);
    }: _(origin, hash, first_proposal_num)
    verify {
        ensure!(!Proposals::<T, I>::get().contains(&hash), "closed proposal is not removed");
    }
}
