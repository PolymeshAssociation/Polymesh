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
use frame_benchmarking::benchmarks_instance;
use frame_support::{
    dispatch::DispatchResult,
    traits::{ChangeMembers, UnfilteredDispatchable},
    StorageValue,
};
use frame_system::RawOrigin as SystemOrigin;
use polymesh_common_utilities::{
    benchs::{user, AccountIdOf, User},
    MaybeBlock, TestUtilsFn,
};
use polymesh_primitives::committee::COMMITTEE_MEMBERS_MAX;
use sp_std::prelude::*;

const PROPOSAL_PADDING_WORDS: usize = 1_000;
// The number of the proposal that has ceil(1/2)-1 aye votes after make_proposals_and_vote. One more
// aye vote leads to acceptance of that proposal.
const PROPOSAL_ALMOST_APPROVED: u32 = COMMITTEE_MEMBERS_MAX - 3;

fn make_proposal<T, I>(
    n: u32,
) -> (
    <T as Config<I>>::Proposal,
    <T as frame_system::Config>::Hash,
)
where
    I: Instance,
    T: Config<I>,
{
    let bytes: [u8; 4] = n.to_be_bytes();
    let padding = bytes.repeat(PROPOSAL_PADDING_WORDS);
    let proposal = frame_system::Call::<T>::remark { remark: padding }.into();
    let hash = <T as frame_system::Config>::Hashing::hash_of(&proposal);
    (proposal, hash)
}

fn make_proposals_and_vote<T, I>(users: &[User<T>]) -> DispatchResult
where
    I: Instance,
    T: Config<I>,
    <T as Config<I>>::RuntimeOrigin: From<SystemOrigin<T::AccountId>>,
{
    assert!(
        users.len() > 0,
        "make_proposals_and_vote requires non-empty users"
    );
    // Leave space for one additional proposal to be created
    for i in 0..(PROPOSALS_MAX - 1) {
        let index = Module::<T, I>::proposal_count();
        let proposal = make_proposal::<T, I>(i).0;
        identity::CurrentDid::put(users[0].did());
        Module::<T, I>::vote_or_propose(users[0].origin.clone().into(), true, Box::new(proposal))
            .unwrap();
        if users.len() > 1 {
            let hash = *Module::<T, I>::proposals().last().unwrap();
            // cast min(user.len(), N) - 1 additional votes for proposal #N
            // alternating nay, aye, nay, aye...
            for (j, user) in users.iter().skip(1).take(i as usize).enumerate() {
                // Vote for the proposal if it's not finalised.
                if Module::<T, I>::voting(&hash).is_some() {
                    identity::CurrentDid::put(user.did());
                    Module::<T, I>::vote(user.origin.clone().into(), hash, index, j % 2 != 0)
                        .unwrap();
                }
            }
        }
    }
    Ok(())
}

fn make_members_and_proposals<T, I>() -> Result<Vec<User<T>>, DispatchError>
where
    I: Instance,
    T: Config<I> + TestUtilsFn<AccountIdOf<T>>,
    <T as Config<I>>::RuntimeOrigin: From<SystemOrigin<T::AccountId>>,
{
    let members: Vec<_> = (0..COMMITTEE_MEMBERS_MAX)
        .map(|i| user::<T>("member", i))
        .collect();
    let mut dids: Vec<_> = members.iter().map(|m| m.did()).collect();
    dids.sort();
    Module::<T, I>::change_members_sorted(&dids, &[], &dids);
    make_proposals_and_vote::<T, I>(&members).unwrap();
    Ok(members)
}

fn vote_verify<T, I>(
    did: &IdentityId,
    hash: <T as frame_system::Config>::Hash,
    proposal_num: u32,
    vote: bool,
) -> DispatchResult
where
    I: Instance,
    T: Config<I>,
{
    if COMMITTEE_MEMBERS_MAX > 4 || (COMMITTEE_MEMBERS_MAX == 4 && !vote) {
        // The proposal is not finalised because there is no quorum yet.
        let votes = Voting::<T, I>::get(&hash).unwrap();
        assert_eq!(votes.index, proposal_num, "wrong proposal_num");
        assert_eq!(vote, votes.ayes.contains(did), "aye vote missing");
        assert_ne!(vote, votes.nays.contains(did), "nay vote missing");
    } else {
        // The proposal is finalised and removed from storage.
        // TODO: pattern-match an event emitted during proposal finalisation.
        assert!(
            !Voting::<T, I>::contains_key(&hash),
            "executed proposal is not removed"
        );
    }
    Ok(())
}

benchmarks_instance! {
    where_clause {
        where
            T: TestUtilsFn<AccountIdOf<T>>,
            <T as Config<I>>::RuntimeOrigin: From<SystemOrigin<T::AccountId>>,
    }

    set_vote_threshold {
        let n = 1;
        let d = 2;
        let origin = T::CommitteeOrigin::try_successful_origin().unwrap();
        let call = Call::<T, I>::set_vote_threshold { n, d };
    }: {
        call.dispatch_bypass_filter(origin).unwrap();
    }
    verify {
        assert_eq!(Module::<T, _>::vote_threshold(), (n, d), "incorrect vote threshold");
    }

    set_release_coordinator {
        let mut dids: Vec<_> = (0..COMMITTEE_MEMBERS_MAX)
            .map(|i| user::<T>("member", i).did())
            .collect();
        dids.sort();
        let coordinator = dids.last().unwrap().clone();
        Module::<T, I>::change_members_sorted(&dids, &[], &dids);
        let origin = T::CommitteeOrigin::try_successful_origin().unwrap();
        let call = Call::<T, I>::set_release_coordinator { id: coordinator };
    }: {
        call.dispatch_bypass_filter(origin).unwrap();
    }
    verify {
        assert_eq!(
            Module::<T, _>::release_coordinator(), Some(coordinator),
            "incorrect release coordinator"
        );
    }

    set_expires_after {
        let maybe_block = MaybeBlock::Some(1u32.into());
        let origin = T::CommitteeOrigin::try_successful_origin().unwrap();
        let call = Call::<T, I>::set_expires_after { expiry: maybe_block };
    }: {
        call.dispatch_bypass_filter(origin).unwrap();
    }
    verify {
        assert_eq!(Module::<T, _>::expires_after(), maybe_block, "incorrect expiration");
    }

    vote_or_propose_new_proposal {
        let members = make_members_and_proposals::<T, I>().unwrap();
        let last_proposal_num = ProposalCount::<I>::get();
        let (proposal, hash) = make_proposal::<T, I>(PROPOSALS_MAX);
        identity::CurrentDid::put(members[0].did());
    }: vote_or_propose(members[0].origin.clone(), true, Box::new(proposal.clone()))
    verify {
        // The proposal was stored.
        assert!(Proposals::<T, I>::get().contains(&hash), "new proposal hash not found");
        assert_eq!(ProposalOf::<T, I>::get(&hash), Some(proposal), "new proposal not found");
    }

    vote_or_propose_existing_proposal {
        let members = make_members_and_proposals::<T, I>().unwrap();
        let (proposal, hash) = make_proposal::<T, I>(0);
        let proposals = Proposals::<T, I>::get();
        assert!(proposals.contains(&hash), "cannot find the first proposal");
        identity::CurrentDid::put(members[1].did());
        let member1 = members[1].origin.clone();
        let boxed_proposal = Box::new(proposal.clone());
    }: vote_or_propose(member1, true, boxed_proposal)
    verify {
        if COMMITTEE_MEMBERS_MAX <= 4 {
            // Proposal was executed.
            assert!(
                Module::<T, _>::voting(&hash).is_none(),
                "votes are present on an executed existing proposal"
            );
        } else {
            // The proposal was stored.
            assert!(Proposals::<T, I>::get().contains(&hash), "existing proposal hash not found");
            assert_eq!(ProposalOf::<T, I>::get(&hash), Some(proposal), "existing proposal not found");
        }
    }

    vote_aye {
        let members = make_members_and_proposals::<T, I>().unwrap();
        let hash = make_proposal::<T, I>(PROPOSAL_ALMOST_APPROVED).1;
        assert!(Proposals::<T, I>::get().contains(&hash), "vote_aye target proposal not found");
        let member = &members[PROPOSAL_ALMOST_APPROVED as usize + 1];
        let origin = member.origin.clone();
        let did = member.did();
        identity::CurrentDid::put(did);
    }: vote(origin, hash, PROPOSAL_ALMOST_APPROVED, true)
    verify {
        assert!(!Proposals::<T, I>::get().contains(&hash), "vote_aye target proposal not executed");
    }

    vote_nay {
        let members = make_members_and_proposals::<T, I>().unwrap();
        let first_proposal_num = 0;
        let hash = make_proposal::<T, I>(first_proposal_num).1;
        let member = &members[1];
        let origin = member.origin.clone();
        let did = member.did();
        identity::CurrentDid::put(did);
    }: vote(origin, hash, first_proposal_num, false)
    verify {
        vote_verify::<T, I>(&did, hash, first_proposal_num, false).unwrap();
    }
}
