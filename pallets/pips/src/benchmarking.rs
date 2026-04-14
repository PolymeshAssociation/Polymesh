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

use crate::*;

use frame_benchmarking::benchmarks;
use frame_support::{dispatch::DispatchResult, traits::UnfilteredDispatchable};
use frame_system::RawOrigin;
use rand::{seq::SliceRandom, SeedableRng};
use rand_chacha::ChaCha20Rng;
use scale_info::prelude::format;
use sp_std::convert::{TryFrom, TryInto};
use sp_std::iter;
use sp_std::prelude::*;

use pallet_identity::benchmarking::{user, User, UserBuilder};
use polymesh_primitives::{MaybeBlock, SystematicIssuers, GC_DID};

#[cfg(feature = "running-ci")]
mod limits {
    pub const DESCRIPTION_LEN: usize = 10;
    pub const URL_LEN: usize = 50;
    pub const PROPOSAL_PADDING_LEN: usize = 100;
    pub const VOTERS_A_NUM: usize = 10;
    pub const VOTERS_B_NUM: usize = 10;
    pub const PROPOSALS_NUM: u32 = 5;
}

#[cfg(not(feature = "running-ci"))]
mod limits {
    pub const DESCRIPTION_LEN: usize = 1_000;
    pub const URL_LEN: usize = 500;
    pub const PROPOSAL_PADDING_LEN: usize = 10_000;
    pub const VOTERS_A_NUM: usize = 200;
    pub const VOTERS_B_NUM: usize = 200;
    pub const PROPOSALS_NUM: u32 = 100;
}

use limits::*;

pub const MAX_SKIPPED_COUNT: u8 = 255;

fn zeroize_deposit<T: Config>() {
    Pallet::<T>::set_min_proposal_deposit(RawOrigin::Root.into(), 0u32.into()).unwrap();
}

/// Makes a proposal.
fn make_proposal<T: Config>() -> (Box<T::Proposal>, Url, PipDescription) {
    let content = vec![b'X'; PROPOSAL_PADDING_LEN];
    let proposal = Box::new(frame_system::Call::<T>::remark { remark: content }.into());
    let url = Url::try_from(vec![b'X'; URL_LEN].as_slice()).unwrap();
    let description = PipDescription::try_from(vec![b'X'; DESCRIPTION_LEN].as_slice()).unwrap();
    (proposal, url, description)
}

/// Creates voters with seeds from 1 to `num_voters` inclusive.
fn make_voters<T: Config>(
    num_voters: usize,
    prefix: &'static str,
) -> Vec<(T::AccountId, RawOrigin<T::AccountId>, IdentityId)> {
    (1..=num_voters)
        .map(|i| {
            let User {
                account,
                origin,
                did,
                ..
            } = user::<T>(prefix, i as u32);
            (account, origin, did.unwrap())
        })
        .collect()
}

/// Casts an `aye_or_nay` vote from each of the given voters.
fn cast_votes<T: Config>(
    id: PipId,
    voters: &[(T::AccountId, RawOrigin<T::AccountId>, IdentityId)],
    aye_or_nay: bool,
) -> DispatchResult {
    for (_, origin, _) in voters {
        Pallet::<T>::vote(origin.clone().into(), id, aye_or_nay, 1u32.into()).unwrap();
    }
    Ok(())
}

/// Creates `number_of_proposals` proposals and casts `number_of_voter` votes for each.
fn vote_setup<T: Config>(number_of_proposals: u32, number_of_voter: u32) {
    Pallet::<T>::set_active_pip_limit(RawOrigin::Root.into(), PROPOSALS_NUM as u32).unwrap();
    Pallet::<T>::set_min_proposal_deposit(RawOrigin::Root.into(), 1_000).unwrap();
    for i in 0..number_of_proposals {
        let (proposal, url, description) = make_proposal::<T>();
        let proposer = UserBuilder::<T>::default()
            .generate_did()
            .build(&format!("Proposer{}", i));
        Pallet::<T>::propose(
            proposer.origin().into(),
            proposal,
            1_000,
            Some(url),
            Some(description),
        )
        .unwrap();
        for j in 0..number_of_voter {
            let voter = UserBuilder::<T>::default()
                .generate_did()
                .build(&format!("User{}", j));
            Pallet::<T>::vote(voter.origin().into(), PipId(i as u32), true, 1_000).unwrap();
        }
    }
}

fn enact_call<T: Config>(num_approves: usize, num_rejects: usize, num_skips: usize) -> Call<T> {
    let seed = [42; 32];
    let mut rng = ChaCha20Rng::from_seed(seed);
    let mut snapshot_results: Vec<_> = iter::repeat(SnapshotResult::Approve)
        .take(num_approves)
        .chain(iter::repeat(SnapshotResult::Reject).take(num_rejects))
        .chain(iter::repeat(SnapshotResult::Skip).take(num_skips))
        .collect();
    snapshot_results.shuffle(&mut rng);
    let results = SnapshotQueue::<T>::get()
        .iter()
        .rev()
        .map(|s| s.id)
        .zip(snapshot_results.into_iter())
        .collect();
    Call::<T>::enact_snapshot_results { results }
}

fn propose_verify<T: Config>(url: Url, description: PipDescription) -> DispatchResult {
    let meta = ProposalMetadata::<T>::get(PipId(0)).unwrap();
    assert_eq!(PipId(0), meta.id, "incorrect meta.id");
    assert_eq!(Some(url), meta.url, "incorrect meta.url");
    assert_eq!(
        Some(description),
        meta.description,
        "incorrect meta.description"
    );
    Ok(())
}

benchmarks! {
    set_prune_historical_pips {
        let origin = RawOrigin::Root;
    }: _(origin, true)
    verify {
        assert!(PruneHistoricalPips::<T>::get(), "set_prune_historical_pips didn't work");
    }

    set_min_proposal_deposit {
        let origin = RawOrigin::Root;
        let deposit = 42u32.into();
    }: _(origin, deposit)
    verify {
        assert_eq!(deposit, MinimumProposalDeposit::<T>::get(), "incorrect MinimumProposalDeposit");
    }

    set_default_enactment_period {
        let origin = RawOrigin::Root;
        let period = 42u32.into();
    }: _(origin, period)
    verify {
        assert_eq!(period, DefaultEnactmentPeriod::<T>::get(), "incorrect DefaultEnactmentPeriod");
    }

    set_pending_pip_expiry {
        let origin = RawOrigin::Root;
        let maybe_block = MaybeBlock::Some(42u32.into());
    }: _(origin, maybe_block)
    verify {
        assert_eq!(maybe_block, PendingPipExpiry::<T>::get(), "incorrect PendingPipExpiry");
    }

    set_max_pip_skip_count {
        let origin = RawOrigin::Root;
        let count = 42.try_into().unwrap();
    }: _(origin, count)
    verify {
        assert_eq!(count, MaxPipSkipCount::<T>::get(), "incorrect MaxPipSkipCount");
    }

    set_active_pip_limit {
        let origin = RawOrigin::Root;
        let pip_limit = 42;
    }: _(origin, pip_limit)
    verify {
        assert_eq!(pip_limit, ActivePipLimit::<T>::get(), "incorrect ActivePipLimit");
    }

    propose_from_community {
        let user = user::<T>("proposer", 0);
        let (proposal, url, description) = make_proposal::<T>();
        let some_url = Some(url.clone());
        let some_desc = Some(description.clone());
        let origin = user.origin();
        zeroize_deposit::<T>();
    }: propose(origin, proposal, 42u32.into(), some_url, some_desc)
    verify {
        propose_verify::<T>(url, description).unwrap();
    }

    // `propose` from a committee origin.
    propose_from_committee {
        let user = user::<T>("proposer", 0);
        let (proposal, url, description) = make_proposal::<T>();
        let origin = T::UpgradeCommitteeVMO::try_successful_origin().unwrap();
        zeroize_deposit::<T>();
        let some_url = Some(url.clone());
        let some_desc = Some(description.clone());
        let call = Call::<T>::propose { proposal, deposit: 0u32.into(), url: some_url, description: some_desc };
    }: {
        call.dispatch_bypass_filter(origin).unwrap();
    }
    verify {
        propose_verify::<T>(url, description).unwrap();
    }

    vote {
        let proposer = user::<T>("proposer", 0);
        let (proposal, url, description) = make_proposal::<T>();
        zeroize_deposit::<T>();
        Pallet::<T>::propose(
            proposer.origin().into(),
            proposal,
            42u32.into(),
            Some(url),
            Some(description)
        ).unwrap();
        // Populate vote history.
        let aye_voters = make_voters::<T>(VOTERS_A_NUM, "aye");
        let nay_voters = make_voters::<T>(VOTERS_B_NUM, "nay");
        let id = PipId(0);
        cast_votes::<T>(id, aye_voters.as_slice(), true).unwrap();
        cast_votes::<T>(id, nay_voters.as_slice(), false).unwrap();
        // Cast an opposite vote.
        let voter = user::<T>("voter", 0);
        let voter_deposit = 43u32.into();
        // Cast an opposite vote.
        Pallet::<T>::vote(voter.origin().into(), id, false, voter_deposit).unwrap();
        let origin = voter.origin();
    }: _(origin, id, true, voter_deposit)
    verify {
        assert!(voter_deposit == Deposits::<T>::get(id, &voter.account()).expect("Deposit").amount, "incorrect voter deposit");
    }

    approve_committee_proposal {
        let (proposal, url, description) = make_proposal::<T>();
        let proposer_origin = T::UpgradeCommitteeVMO::try_successful_origin().unwrap();
        let proposer_did = SystematicIssuers::Committee.as_id();
        zeroize_deposit::<T>();
        let propose_call = Call::<T>::propose {
            proposal,
            deposit: 0u32.into(),
            url: Some(url.clone()),
            description: Some(description.clone())
        };
        propose_call.dispatch_bypass_filter(proposer_origin).unwrap();
        let origin = T::VotingMajorityOrigin::try_successful_origin().unwrap();
        let id = PipId(0);
        let call = Call::<T>::approve_committee_proposal { id };
    }: {
        call.dispatch_bypass_filter(origin).unwrap();
    }
    verify {
        assert!(PipToSchedule::<T>::contains_key(id), "approved committee proposal is not in the schedule");
    }

    reject_proposal {
        vote_setup::<T>(1, T::MaxRefundsAndVotesPruned::get());

        Pallet::<T>::set_prune_historical_pips(RawOrigin::Root.into(), true).unwrap();

        let origin = T::VotingMajorityOrigin::try_successful_origin().unwrap();
        let call = Call::<T>::reject_proposal { id: PipId(0) };
    }: {
        call.dispatch_bypass_filter(origin).unwrap();
    }
    verify {
        assert!(!Proposals::<T>::contains_key(PipId(0)));
    }

    prune_proposal {
        vote_setup::<T>(1, T::MaxRefundsAndVotesPruned::get());

        let origin = T::VotingMajorityOrigin::try_successful_origin().unwrap();
        let reject_call = Call::<T>::reject_proposal { id: PipId(0) };
        reject_call.dispatch_bypass_filter(origin.clone()).unwrap();
        let call = Call::<T>::prune_proposal { id: PipId(0) };
    }: {
        call.dispatch_bypass_filter(origin).unwrap();
    }
    verify {
        assert!(!Proposals::<T>::contains_key(PipId(0)));
        assert!(!ProposalMetadata::<T>::contains_key(PipId(0)));
    }

    reschedule_execution {
        let user = user::<T>("proposer", 0);
        zeroize_deposit::<T>();
        let (proposal, url, description) = make_proposal::<T>();
        Pallet::<T>::propose(
            user.origin().into(),
            proposal,
            42u32.into(),
            Some(url.clone()),
            Some(description.clone())
        ).unwrap();
        let id = PipId(0);
        T::GovernanceCommittee::bench_set_release_coordinator(user.did());
        Pallet::<T>::snapshot(user.origin().into(), 1).unwrap();
        let vmo_origin = T::VotingMajorityOrigin::try_successful_origin().unwrap();
        let enact_call = Call::<T>::enact_snapshot_results { results: vec![(id, SnapshotResult::Approve)] };
        enact_call.dispatch_bypass_filter(vmo_origin).unwrap();
        let future_block = frame_system::Pallet::<T>::block_number() + 100u32.into();
        let origin = user.origin();
    }: _(origin, id, Some(future_block))
    verify {
        assert!(PipToSchedule::<T>::contains_key(id), "rescheduled proposal is missing in the schedule");
    }

    clear_snapshot {
        let user = user::<T>("proposer", 0);
        zeroize_deposit::<T>();
        let (proposal, url, description) = make_proposal::<T>();
        Pallet::<T>::propose(
            user.origin().into(),
            proposal,
            42u32.into(),
            Some(url.clone()),
            Some(description.clone())
        ).unwrap();
        T::GovernanceCommittee::bench_set_release_coordinator(user.did());
        Pallet::<T>::snapshot(user.origin().into(), 1).unwrap();
        assert!(SnapshotMeta::<T>::get().is_some(), "missing a snapshot before clear_snapshot");
        let origin = user.origin();
    }: _(origin)
    verify {
        assert!(SnapshotMeta::<T>::get().is_none(), "snapshot was not cleared by clear_snapshot");
    }

    snapshot {
        let q in 0..PROPOSALS_NUM;
        vote_setup::<T>(q, T::MaxRefundsAndVotesPruned::get());
        let user = user::<T>("release_coordinator", 0);
        T::GovernanceCommittee::bench_set_release_coordinator(user.did());
    }: _(user.origin(), q)
    verify {
        assert!(SnapshotMeta::<T>::get().is_some());
        assert_eq!(SnapshotQueue::<T>::get().len(), q as usize);
    }

    enact_snapshot_results {
        // The number of Approve results.
        let a in 0..PROPOSALS_NUM / 3;
        // The number of Reject results.
        let r in 0..PROPOSALS_NUM / 3;
        // The number of Skip results.
        let s in 0..PROPOSALS_NUM / 3;

        Pallet::<T>::set_max_pip_skip_count(RawOrigin::Root.into(), MAX_SKIPPED_COUNT).unwrap();
        vote_setup::<T>(PROPOSALS_NUM, T::MaxRefundsAndVotesPruned::get());

        // Adds a user to the governance committee and captures a snapshot
        let user = user::<T>("release_coordinator", 0);
        T::GovernanceCommittee::bench_set_release_coordinator(user.did());
        Pallet::<T>::snapshot(user.origin().into(), PROPOSALS_NUM).unwrap();

        // Set up the results parameter based on the number of Approve, Reject, and Skip results.
        let origin = T::VotingMajorityOrigin::try_successful_origin().unwrap();
        let call = enact_call::<T>(a as usize, r as usize, s as usize);
    }: {
        call.dispatch_bypass_filter(origin).unwrap();
    }
    verify {
        assert_eq!(SnapshotQueue::<T>::get().len(), (PROPOSALS_NUM - (a + r + s)) as usize);
    }

    execute_scheduled_pip {
        Pallet::<T>::set_prune_historical_pips(RawOrigin::Root.into(), true).unwrap();
        vote_setup::<T>(PROPOSALS_NUM, T::MaxRefundsAndVotesPruned::get());

        // Adds a user to the governance committee and captures a snapshot
        let user = user::<T>("release_coordinator", 0);
        T::GovernanceCommittee::bench_set_release_coordinator(user.did());
        Pallet::<T>::snapshot(user.origin().into(), PROPOSALS_NUM).unwrap();
        assert!(SnapshotQueue::<T>::get().len() == PROPOSALS_NUM as usize);

        // Set up the results parameter where all proposals are approved
        let origin = T::VotingMajorityOrigin::try_successful_origin().unwrap();
        let call = enact_call::<T>(PROPOSALS_NUM as usize, 0, 0);
        call.dispatch_bypass_filter(origin).unwrap();
    }: _(RawOrigin::Root, PipId(0))
    verify {
        assert!(!Proposals::<T>::contains_key(PipId(0)));
    }

    expire_scheduled_pip {
        Pallet::<T>::set_prune_historical_pips(RawOrigin::Root.into(), true).unwrap();
        vote_setup::<T>(PROPOSALS_NUM, T::MaxRefundsAndVotesPruned::get());

        // Adds a user to the governance committee and captures a snapshot
        let user = user::<T>("release_coordinator", 0);
        T::GovernanceCommittee::bench_set_release_coordinator(user.did());
        Pallet::<T>::snapshot(user.origin().into(), PROPOSALS_NUM).unwrap();

        assert_eq!(ProposalState::Pending, ProposalStates::<T>::get(PipId(0)).unwrap());
    }: _(RawOrigin::Root, GC_DID, PipId(0))
    verify {
        assert!(!Proposals::<T>::contains_key(PipId(0)));
    }

    remove_pending_storage {
        let r in 0..T::MaxRefundsAndVotesPruned::get() as u32;
        let v in 0..T::MaxRefundsAndVotesPruned::get() as u32;

        for i in 0..r {
            let user = UserBuilder::<T>::default().generate_did().build(&format!("Voter{}", i));
            Pallet::<T>::increase_lock(&user.account(), 1_000).unwrap();
            Deposits::<T>::insert(PipId(0), user.account(), DepositInfo { owner: user.account(), amount: 1_000 });
        }

        for i in 0..v {
            let user = UserBuilder::<T>::default().generate_did().build(&format!("Voter{}", i));
            ProposalVotes::<T>::insert(PipId(0), user.account(), Vote(true, 1_000));
        }

        if r > 0 {
            PendingRefunds::<T>::insert(PipId(0), true);
        }

        if v > 0 {
            VotesToBePruned::<T>::insert(PipId(0), true);
        }
    }: {
        Pallet::<T>::remove_pending_storage();
    }
}
