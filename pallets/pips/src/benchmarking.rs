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
use frame_benchmarking::benchmarks;
use frame_support::{dispatch::DispatchResult, traits::UnfilteredDispatchable};
use frame_system::RawOrigin;
use pallet_identity::{self as identity, benchmarking::make_account};
use polymesh_common_utilities::{MaybeBlock, SystematicIssuers};
use sp_std::{
    convert::{TryFrom, TryInto},
    prelude::*,
};

const DESCRIPTION_LEN: usize = 1000;
const URL_LEN: usize = 500;

/// Makes a proposal of a given length.
fn make_proposal<T: Trait>(content_len: usize) -> (Box<T::Proposal>, Url, PipDescription) {
    let content = vec![b'X'; content_len];
    let proposal = Box::new(frame_system::Call::<T>::remark(content).into());
    let url = Url::try_from(vec![b'X'; URL_LEN].as_slice()).unwrap();
    let description = PipDescription::try_from(vec![b'X'; DESCRIPTION_LEN].as_slice()).unwrap();
    (proposal, url, description)
}

/// Creates voters with seeds from 1 to `num_voters` inclusive.
fn make_voters<T: Trait>(
    num_voters: u32,
    prefix: &'static str,
) -> Vec<(RawOrigin<T::AccountId>, IdentityId)> {
    (1..=num_voters)
        .map(|i| {
            let (_, origin, did) = make_account::<T>(prefix, i);
            (origin, did)
        })
        .collect()
}

/// Casts an `aye_or_nay` vote from each of the given voters.
fn cast_votes<T: Trait>(
    id: PipId,
    voters: &[(RawOrigin<T::AccountId>, IdentityId)],
    aye_or_nay: bool,
) -> DispatchResult {
    for (origin, did) in voters {
        identity::CurrentDid::put(did);
        Module::<T>::vote(origin.clone().into(), id, aye_or_nay, 1.into())?;
    }
    Ok(())
}

/// Sets up snapshot and enact benches.
fn snapshot_setup<T: Trait>(
    h: u32,
    b: u32,
    p: u32,
    c: u32,
) -> Result<RawOrigin<T::AccountId>, DispatchError> {
    Module::<T>::set_proposal_cool_off_period(RawOrigin::Root.into(), 0.into())?;
    let hi_voters = make_voters::<T>(h, "hi");
    let bye_voters = make_voters::<T>(b, "bye");
    let (_, origin0, did0) = make_account::<T>("initial", 0);
    for i in 0..p {
        let (proposal, url, description) = make_proposal::<T>(c as usize);
        // Pick a proposer, diversifying like a poor man.
        let (proposer_origin, proposer_did) = if hi_voters.len() >= i as usize + 1 {
            hi_voters[i as usize].clone()
        } else {
            (origin0.clone(), did0)
        };
        identity::CurrentDid::put(proposer_did);
        Module::<T>::propose(
            proposer_origin.into(),
            proposal,
            42.into(),
            Some(url.clone()),
            Some(description.clone()),
        )?;
        // Alternate aye and nay voters with every iteration.
        cast_votes::<T>(i, hi_voters.as_slice(), i % 2 == 0)?;
        cast_votes::<T>(i, bye_voters.as_slice(), i % 2 != 0)?;
    }
    identity::CurrentDid::put(did0);
    T::GovernanceCommittee::bench_set_release_coordinator(did0);
    Ok(origin0)
}

benchmarks! {
    _ {}

    set_prune_historical_pips {
        let origin = RawOrigin::Root;
    }: _(origin, true)
    verify {
        assert_eq!(true, PruneHistoricalPips::get());
    }

    set_min_proposal_deposit {
        let origin = RawOrigin::Root;
        let deposit = 42.into();
    }: _(origin, deposit)
    verify {
        assert_eq!(deposit, MinimumProposalDeposit::<T>::get());
    }

    set_proposal_cool_off_period {
        let origin = RawOrigin::Root;
        let period = 42.into();
    }: _(origin, period)
    verify {
        assert_eq!(period, ProposalCoolOffPeriod::<T>::get());
    }

    set_default_enactment_period {
        let origin = RawOrigin::Root;
        let period = 42.into();
    }: _(origin, period)
    verify {
        assert_eq!(period, DefaultEnactmentPeriod::<T>::get());
    }

    set_pending_pip_expiry {
        let origin = RawOrigin::Root;
        let maybe_block = MaybeBlock::Some(42.into());
    }: _(origin, maybe_block)
    verify {
        assert_eq!(maybe_block, PendingPipExpiry::<T>::get());
    }

    set_max_pip_skip_count {
        let origin = RawOrigin::Root;
        let count = 42.try_into().unwrap();
    }: _(origin, count)
    verify {
        assert_eq!(count, MaxPipSkipCount::get());
    }

    set_active_pip_limit {
        let origin = RawOrigin::Root;
        let pip_limit = 42;
    }: _(origin, pip_limit)
    verify {
        assert_eq!(pip_limit, ActivePipLimit::get());
    }

    propose_from_community {
        // length of the proposal padding
        let c in 0 .. 10_000;

        let (account, origin, did) = make_account::<T>("proposer", 0);
        identity::CurrentDid::put(did);
        let (proposal, url, description) = make_proposal::<T>(c as usize);
    }: propose(origin, proposal, 42.into(), Some(url.clone()), Some(description.clone()))
    verify {
        let meta = Module::<T>::proposal_metadata(0).unwrap();
        assert_eq!(0, meta.id);
        assert_eq!(Some(url), meta.url);
        assert_eq!(Some(description), meta.description);
    }

    // `propose` from a committee origin.
    propose_from_committee {
        // length of the proposal padding
        let c in 0 .. 10_000;

        let (account, origin, did) = make_account::<T>("proposer", 0);
        identity::CurrentDid::put(did);
        let (proposal, url, description) = make_proposal::<T>(c as usize);
        let origin = T::UpgradeCommitteeVMO::successful_origin();
        Module::<T>::set_min_proposal_deposit(RawOrigin::Root.into(), 0.into())?;
        Module::<T>::set_proposal_cool_off_period(RawOrigin::Root.into(), 0.into())?;
        let call = Call::<T>::propose(proposal, 0.into(), Some(url.clone()), Some(description.clone()));
    }: {
        call.dispatch_bypass_filter(origin)?;
    }
    verify {
        let meta = Module::<T>::proposal_metadata(0).unwrap();
        assert_eq!(0, meta.id);
        assert_eq!(Some(url), meta.url);
        assert_eq!(Some(description), meta.description);
    }

    amend_proposal {
        // length of the proposal padding
        let c in 0 .. 10_000;

        let (account, origin, did) = make_account::<T>("proposer", 0);
        identity::CurrentDid::put(did);
        let (proposal, url, description) = make_proposal::<T>(c as usize);
        Module::<T>::propose(
            origin.clone().into(),
            proposal,
            42.into(),
            None,
            None,
        )?;
    }: _(origin, 0, Some(url.clone()), Some(description.clone()))
    verify {
        let meta = Module::<T>::proposal_metadata(0).unwrap();
        assert_eq!(0, meta.id);
        assert_eq!(Some(url), meta.url);
        assert_eq!(Some(description), meta.description);
    }

    cancel_proposal {
        // length of the proposal padding
        let c in 0 .. 10_000;

        let (account, origin, did) = make_account::<T>("proposer", 0);
        identity::CurrentDid::put(did);
        let (proposal, url, description) = make_proposal::<T>(c as usize);
        Module::<T>::propose(
            origin.clone().into(),
            proposal,
            42.into(),
            Some(url),
            Some(description)
        )?;
    }: _(origin, 0)
    verify {
        assert_eq!(
            Module::<T>::proposals(0).unwrap().proposer,
            Proposer::Community(account)
        );
    }

    vote {
        // length of the proposal padding
        let c in 0 .. 10_000;
        // number of ayes
        let a in 1 .. 10;
        // number of nays
        let n in 1 .. 10;

        let (proposer_account, proposer_origin, proposer_did) = make_account::<T>("proposer", 0);
        identity::CurrentDid::put(proposer_did);
        let (proposal, url, description) = make_proposal::<T>(c as usize);
        Module::<T>::set_proposal_cool_off_period(RawOrigin::Root.into(), 0.into())?;
        Module::<T>::propose(
            proposer_origin.into(),
            proposal,
            42.into(),
            Some(url),
            Some(description)
        )?;
        // Populate vote history.
        let aye_voters = make_voters::<T>(a, "aye");
        let nay_voters = make_voters::<T>(n, "nay");
        cast_votes::<T>(0, aye_voters.as_slice(), true)?;
        cast_votes::<T>(0, nay_voters.as_slice(), false)?;
        // Cast an opposite vote.
        let (account, origin, did) = make_account::<T>("voter", 0);
        identity::CurrentDid::put(did);
        let voter_deposit = 43.into();
        // Cast an opposite vote.
        Module::<T>::vote(origin.clone().into(), 0, false, voter_deposit)?;
    }: _(origin, 0, true, voter_deposit)
    verify {
        assert_eq!(voter_deposit, Deposits::<T>::get(0, &account).amount);
    }

    approve_committee_proposal {
        // length of the proposal padding
        let c in 0 .. 10_000;

        let (proposal, url, description) = make_proposal::<T>(c as usize);
        let proposer_origin = T::UpgradeCommitteeVMO::successful_origin();
        let proposer_did = SystematicIssuers::Committee.as_id();
        identity::CurrentDid::put(proposer_did);
        Module::<T>::set_min_proposal_deposit(RawOrigin::Root.into(), 0.into())?;
        Module::<T>::set_proposal_cool_off_period(RawOrigin::Root.into(), 0.into())?;
        let propose_call = Call::<T>::propose(proposal, 0.into(), Some(url.clone()), Some(description.clone()));
        propose_call.dispatch_bypass_filter(proposer_origin)?;
        let origin = T::VotingMajorityOrigin::successful_origin();
        let call = Call::<T>::approve_committee_proposal(0);
    }: {
        call.dispatch_bypass_filter(origin)?;
    }
    verify {
        assert_eq!(true, PipToSchedule::<T>::contains_key(&0));
    }

    reject_proposal {
        // length of the proposal padding
        let c in 0 .. 10_000;

        let (proposer_account, proposer_origin, proposer_did) = make_account::<T>("proposer", 0);
        identity::CurrentDid::put(proposer_did);
        let (proposal, url, description) = make_proposal::<T>(c as usize);
        Module::<T>::set_proposal_cool_off_period(RawOrigin::Root.into(), 0.into())?;
        let deposit = 42.into();
        Module::<T>::propose(
            proposer_origin.into(),
            proposal,
            deposit,
            Some(url),
            Some(description)
        )?;
        assert_eq!(deposit, Deposits::<T>::get(&0, &proposer_account).amount);
        let origin = T::VotingMajorityOrigin::successful_origin();
        let call = Call::<T>::reject_proposal(0);
    }: {
        call.dispatch_bypass_filter(origin)?;
    }
    verify {
        assert_eq!(false, Deposits::<T>::contains_key(&0, &proposer_account));
    }

    prune_proposal {
        // length of the proposal padding
        let c in 0 .. 10_000;

        Module::<T>::set_proposal_cool_off_period(RawOrigin::Root.into(), 0.into())?;
        let (proposer_account, proposer_origin, proposer_did) = make_account::<T>("proposer", 0);
        identity::CurrentDid::put(proposer_did);
        let (proposal, url, description) = make_proposal::<T>(c as usize);
        Module::<T>::propose(
            proposer_origin.into(),
            proposal,
            42.into(),
            Some(url),
            Some(description)
        )?;
        let origin = T::VotingMajorityOrigin::successful_origin();
        Module::<T>::set_prune_historical_pips(RawOrigin::Root.into(), false)?;
        let reject_call = Call::<T>::reject_proposal(0);
        reject_call.dispatch_bypass_filter(origin.clone())?;
        let call = Call::<T>::prune_proposal(0);
    }: {
        call.dispatch_bypass_filter(origin)?;
    }
    verify {
        assert_eq!(false, Proposals::<T>::contains_key(&0));
        assert_eq!(false, ProposalMetadata::<T>::contains_key(&0));
    }

    reschedule_execution {
        // length of the proposal padding
        let c in 0 .. 10_000;

        let (proposer_account, proposer_origin, proposer_did) = make_account::<T>("proposer", 0);
        identity::CurrentDid::put(proposer_did);
        let (proposal, url, description) = make_proposal::<T>(c as usize);
        Module::<T>::set_proposal_cool_off_period(RawOrigin::Root.into(), 0.into())?;
        Module::<T>::propose(
            proposer_origin.clone().into(),
            proposal,
            42.into(),
            Some(url.clone()),
            Some(description.clone())
        )?;
        T::GovernanceCommittee::bench_set_release_coordinator(proposer_did);
        Module::<T>::snapshot(proposer_origin.clone().into())?;
        let enact_origin = T::VotingMajorityOrigin::successful_origin();
        let enact_call = Call::<T>::enact_snapshot_results(vec![(0, SnapshotResult::Approve)]);
        enact_call.dispatch_bypass_filter(enact_origin)?;
        let future_block = frame_system::Module::<T>::block_number() + 100.into();
    }: _(proposer_origin, 0, Some(future_block))
    verify {
        assert_eq!(true, PipToSchedule::<T>::contains_key(&0));
    }

    clear_snapshot {
        // length of the proposal padding
        let c in 0 .. 10_000;

        let (account, origin, did) = make_account::<T>("proposer", 0);
        identity::CurrentDid::put(did);
        let (proposal, url, description) = make_proposal::<T>(c as usize);
        Module::<T>::set_proposal_cool_off_period(RawOrigin::Root.into(), 0.into())?;
        Module::<T>::propose(
            origin.clone().into(),
            proposal,
            42.into(),
            Some(url.clone()),
            Some(description.clone())
        )?;
        T::GovernanceCommittee::bench_set_release_coordinator(did);
        Module::<T>::snapshot(origin.clone().into())?;
        assert!(SnapshotMeta::<T>::get().is_some());
    }: _(origin)
    verify {
        assert!(SnapshotMeta::<T>::get().is_none());
    }

    snapshot {
        // length of the proposal padding
        let c in 0 .. 10_000;
        // number of proposals
        let p in 0 .. 10;
        // first group of voters
        let h in 0 .. 10;
        // second group of voters
        let b in 0 .. 10;

        let origin0 = snapshot_setup::<T>(h, b, p, c)?;
    }: _(origin0)
    verify {
        assert!(SnapshotMeta::<T>::get().is_some());
    }

    enact_snapshot_results {
        // length of the proposal padding
        let c in 0 .. 10_000;
        // number of proposals
        let p in 0 .. 10;
        // first group of voters
        let h in 0 .. 10;
        // second group of voters
        let b in 0 .. 10;

        let origin0 = snapshot_setup::<T>(h, b, p, c)?;
        Module::<T>::snapshot(origin0.into())?;
        let enact_origin = T::VotingMajorityOrigin::successful_origin();
        let enact_call = Call::<T>::enact_snapshot_results(
            Module::<T>::snapshot_queue()
                .iter()
                .map(|s| {
                    let id = s.id;
                    let action = if id % 2 == 0 {
                        SnapshotResult::Approve
                    } else {
                        SnapshotResult::Reject
                    };
                    (id, action)
                })
                .rev()
                .collect()
        );
    }: {
        enact_call.dispatch_bypass_filter(enact_origin)?;
    }
    verify {
        assert_eq!(true, PipToSchedule::<T>::contains_key(&0));
    }
}
