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
use frame_system::RawOrigin;
use polymesh_common_utilities::{
    benchs::{User, UserBuilder},
    MaybeBlock,
};
use sp_std::prelude::*;

const COMMITTEE_MEMBERS_NUM: usize = 10;
const COMMITTEE_MEMBERS_MAX: u32 = 100;
const PROPOSAL_PADDING_LEN: usize = 10_000;
const PROPOSALS_NUM: u8 = 100;

fn make_proposals<T, I>(origin: RawOrigin<T::AccountId>) -> DispatchResult
where
    I: Instance,
    T: Trait<I>,
{
    let origin: <T as frame_system::Trait>::Origin = origin.into();
    for i in 0..PROPOSALS_NUM {
        let proposal: <T as Trait<I>>::Proposal =
            frame_system::Call::<T>::remark(vec![i + 1; PROPOSAL_PADDING_LEN]).into();
        Module::<T, I>::vote_or_propose(origin.clone(), true, Box::new(proposal))?;
    }
    Ok(())
}

fn add_votes<T, I>(users: Vec<User<T>>) -> DispatchResult
where
    I: Instance,
    T: Trait<I>,
{
    let proposals = Proposals::<T, I>::get();
    for (i, proposal_hash) in proposals.iter().enumerate() {
        // cast N votes for proposal #N
        for (j, user) in users.iter().take(i).enumerate() {
            Module::<T, I>::vote(
                user.origin.clone().into(),
                *proposal_hash,
                i as u32,
                j % 2 == 0,
            )?;
        }
    }
    Ok(())
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
        let m in 1 .. COMMITTEE_MEMBERS_MAX;

        let proposer = UserBuilder::<T>::default().generate_did().build("proposer");
        let mut members: Vec<_> = (1..m)
            .map(|i| UserBuilder::<T>::default().generate_did().seed(i).build("member"))
            .collect();
        members.push(proposer.clone());
        Members::<I>::put(members.iter().map(|m| m.did()).collect::<Vec<_>>());
        identity::CurrentDid::put(proposer.did());
        make_proposals::<T, I>(proposer.origin.clone())?;
        // members.retain(|u| u != &proposer);
        // add_votes(members)?;
        let last_proposal_num = ProposalCount::<I>::get();
        let proposal: <T as Trait<I>>::Proposal =
            frame_system::Call::<T>::remark(vec![0; PROPOSAL_PADDING_LEN]).into();
        let hash = <T as frame_system::Trait>::Hashing::hash_of(&proposal);
    }: vote_or_propose(proposer.origin.clone(), true, Box::new(proposal.clone()))
    verify {
        if m == 1 {
            // // The proposal was executed and execution was logged.
            // let pallet_event: <T as Trait<I>>::Event =
            //     RawEvent::Executed(
            //         proposer.did(),
            //         hash,
            //         Ok(())
            //     ).into();
            // let system_event: <T as frame_system::Trait>::Event = pallet_event.into();
            // ensure!(frame_system::Module::<T>::events().iter().any(|e| {
            //     e.event == system_event
            // }), "new proposal was not executed");
        } else {
            // The proposal was stored.
            ensure!(Proposals::<T, I>::get().contains(&hash), "new proposal hash not found");
            ensure!(ProposalOf::<T, I>::get(&hash) == Some(proposal), "new proposal not found");
        }
    }

    // vote_or_propose_existing_proposal {
    //     let m in 2 .. COMMITTEE_MEMBERS_MAX;
    //     let p in 1 .. PROPOSAL_PADDING_MAX;

    //     let proposer = UserBuilder::<T>::default().build_with_did("proposer", 0);
    //     let mut members: Vec<_> = (1..m)
    //         .map(|i| UserBuilder::<T>::default().build_with_did("member", i))
    //         .collect();
    //     members.push(proposer.clone());
    //     Members::<I>::put(members.iter().map(|m| m.did()).collect::<Vec<_>>());
    //     identity::CurrentDid::put(proposer.did());
    //     make_proposals::<T, I>(proposer.origin.clone(), p as usize)?;
    //     add_votes(members.clone())?;
    //     let proposal: <T as Trait<I>>::Proposal =
    //         frame_system::Call::<T>::remark(vec![1; p as usize]).into();
    //     let hash = <T as frame_system::Trait>::Hashing::hash_of(&proposal);
    //     let proposals = Proposals::<T, I>::get();
    //     ensure!(proposals.contains(&hash), "cannot find the first proposal");
    //     let first_proposal_num = proposals.binary_search(&hash).unwrap();
    //     identity::CurrentDid::put(members[0].did());
    // }: vote_or_propose(members[0].origin.clone(), true, Box::new(proposal.clone()))
    // verify {
    //     if m <= 3 {
    //         // The proposal was executed and execution was logged.
    //         let pallet_event: <T as Trait<I>>::Event =
    //             RawEvent::FinalVotes(
    //                 members[0].did(),
    //                 first_proposal_num as u32,
    //                 hash,
    //                 vec![proposer.did(), members[0].did()],
    //                 vec![]
    //             ).into();
    //         let system_event: <T as frame_system::Trait>::Event = pallet_event.into();
    //         ensure!(frame_system::Module::<T>::events().iter().any(|e| {
    //             e.event == system_event
    //         }), "existing proposal was not executed");
    //     } else {
    //         // The proposal was stored.
    //         ensure!(Proposals::<T, I>::get().contains(&hash), "existing proposal hash not found");
    //         ensure!(ProposalOf::<T, I>::get(&hash) == Some(proposal), "existing proposal not found");
    //     }
    // }

    vote {
        let m in 2 .. COMMITTEE_MEMBERS_MAX;
        // reject or approve
        let a in 0 .. 1;

        let proposer = UserBuilder::<T>::default().generate_did().build("proposer");
        let mut members: Vec<_> = (1..m)
            .map(|i| UserBuilder::<T>::default().generate_did().seed(i).build("member"))
            .collect();
        members.push(proposer.clone());
        Members::<I>::put(members.iter().map(|m| m.did()).collect::<Vec<_>>());
        identity::CurrentDid::put(proposer.did());
        make_proposals::<T, I>(proposer.origin.clone())?;
        let proposal: <T as Trait<I>>::Proposal =
            frame_system::Call::<T>::remark(vec![1; PROPOSAL_PADDING_LEN]).into();
        let hash = <T as frame_system::Trait>::Hashing::hash_of(&proposal);
        // let proposals = Proposals::<T, I>::get();
        // ensure!(proposals.contains(&hash), "cannot find the first proposal for voting");
        let first_proposal_num = 0; // proposals.binary_search(&hash).unwrap() as u32;
        let origin = members[0].origin.clone();
        let did = members[0].did();
        identity::CurrentDid::put(did);
    }: _(origin, hash, first_proposal_num, a != 0)
    verify {
        if m > 4 {
            // The proposal is not finalised because there is no quorum yet.
            let voting = Voting::<T, I>::get(&hash);
            ensure!(voting.is_some(), "cannot get votes");
            let votes = voting.unwrap();
            ensure!(votes.index == first_proposal_num, "wrong first proposal index");
            if a != 0 {
                ensure!(votes.ayes.contains(&did), "aye vote missing");
            } else {
                ensure!(votes.nays.contains(&did), "nay vote missing");
            }
        } else {
            // The proposal is finalised and removed from storage.
            // TODO: pattern-match an event emitted during proposal finalisation.
            ensure!(!Voting::<T, I>::contains_key(&hash), "executed proposal is not removed");
        }
    }

    close {
        let m in 2 .. COMMITTEE_MEMBERS_MAX;
        // reject or approve
        let a in 0 .. 1;

        let proposer = UserBuilder::<T>::default().generate_did().build("proposer");
        let mut members: Vec<_> = (1..m)
            .map(|i| UserBuilder::<T>::default().generate_did().seed(i).build("member"))
            .collect();
        members.push(proposer.clone());
        Members::<I>::put(members.iter().map(|m| m.did()).collect::<Vec<_>>());
        identity::CurrentDid::put(proposer.did());
        make_proposals::<T, I>(proposer.origin.clone())?;
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
