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
use pallet_identity::{
    self as identity,
    benchmarking::{User, UserBuilder},
};
use polymesh_common_utilities::{MaybeBlock, SystematicIssuers, GC_DID};
use sp_std::{
    convert::{TryFrom, TryInto},
    prelude::*,
};

const PIPS_PREFIX: &'static [u8] = b"Pips";
const DESCRIPTION_LEN: usize = 1_000;
const URL_LEN: usize = 500;
const PROPOSAL_PADDING_LEN: usize = 10_000;
const VOTERS_A_NUM: usize = 200;
const VOTERS_B_NUM: usize = 200;
const PROPOSALS_NUM: usize = 100;

/// Makes a proposal.
fn make_proposal<T: Trait>() -> (Box<T::Proposal>, Url, PipDescription) {
    let content = vec![b'X'; PROPOSAL_PADDING_LEN];
    let proposal = Box::new(frame_system::Call::<T>::remark(content).into());
    let url = Url::try_from(vec![b'X'; URL_LEN].as_slice()).unwrap();
    let description = PipDescription::try_from(vec![b'X'; DESCRIPTION_LEN].as_slice()).unwrap();
    (proposal, url, description)
}

/// Creates voters with seeds from 1 to `num_voters` inclusive.
fn make_voters<T: Trait>(
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
            } = UserBuilder::<T>::default().build_with_did(prefix, i as u32);
            (account, origin, did.unwrap())
        })
        .collect()
}

/// Casts an `aye_or_nay` vote from each of the given voters.
fn cast_votes<T: Trait>(
    id: PipId,
    voters: &[(T::AccountId, RawOrigin<T::AccountId>, IdentityId)],
    aye_or_nay: bool,
) -> DispatchResult {
    for (account, origin, did) in voters {
        identity::CurrentDid::put(did);
        Module::<T>::vote(origin.clone().into(), id, aye_or_nay, 1.into())?;
    }
    Ok(())
}

/// Sets up PIPs and votes.
fn pips_and_votes_setup<T: Trait>(
    approve_only: bool,
) -> Result<(RawOrigin<T::AccountId>, IdentityId), DispatchError> {
    Module::<T>::set_proposal_cool_off_period(RawOrigin::Root.into(), 0.into())?;
    Module::<T>::set_active_pip_limit(RawOrigin::Root.into(), PROPOSALS_NUM as u32)?;
    let (voters_a_num, voters_b_num) = if approve_only {
        (VOTERS_A_NUM + VOTERS_B_NUM, 0)
    } else {
        (VOTERS_A_NUM, VOTERS_B_NUM)
    };
    let hi_voters = make_voters::<T>(voters_a_num, "hi");
    let bye_voters = make_voters::<T>(voters_b_num, "bye");
    let User { origin, did, .. } = UserBuilder::<T>::default().build_with_did("initial", 0);
    let did = did.expect("no did in pips_and_votes_setup");
    for i in 0..PROPOSALS_NUM {
        let (proposal, url, description) = make_proposal::<T>();
        // Pick a proposer, diversifying like a poor man.
        let (proposer_origin, proposer_did) = if hi_voters.len() >= i + 1 {
            (hi_voters[i].1.clone(), hi_voters[i].2.clone())
        } else {
            (origin.clone(), did)
        };
        identity::CurrentDid::put(proposer_did);
        Module::<T>::propose(
            proposer_origin.into(),
            proposal,
            42.into(),
            Some(url.clone()),
            Some(description.clone()),
        )?;
        // Alternate aye and nay voters with every iteration unless only approve votes are cast.
        cast_votes::<T>(i as u32, hi_voters.as_slice(), approve_only || i % 2 == 0)?;
        cast_votes::<T>(i as u32, bye_voters.as_slice(), i % 2 != 0)?;
    }
    identity::CurrentDid::kill();
    Ok((origin, did))
}

/// Sets up snapshot and enact benches.
fn snapshot_setup<T: Trait>() -> Result<RawOrigin<T::AccountId>, DispatchError> {
    let (origin, did) = pips_and_votes_setup::<T>(false)?;
    identity::CurrentDid::put(did);
    T::GovernanceCommittee::bench_set_release_coordinator(did);
    Ok(origin)
}

fn enact_call<T: Trait>() -> Call<T> {
    Call::<T>::enact_snapshot_results(
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
            .collect(),
    )
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
        let User { account, origin, did, .. } =
            UserBuilder::<T>::default().build_with_did("proposer", 0);
        identity::CurrentDid::put(did.unwrap());
        let (proposal, url, description) = make_proposal::<T>();
    }: propose(origin, proposal, 42.into(), Some(url.clone()), Some(description.clone()))
    verify {
        let meta = Module::<T>::proposal_metadata(0).unwrap();
        assert_eq!(0, meta.id);
        assert_eq!(Some(url), meta.url);
        assert_eq!(Some(description), meta.description);
    }

    // `propose` from a committee origin.
    propose_from_committee {
        let User { account, origin, did, .. } =
            UserBuilder::<T>::default().build_with_did("proposer", 0);
        identity::CurrentDid::put(did.unwrap());
        let (proposal, url, description) = make_proposal::<T>();
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

    vote {
        let User { account, origin, did, .. } =
            UserBuilder::<T>::default().build_with_did("proposer", 0);
        identity::CurrentDid::put(did.unwrap());
        let (proposal, url, description) = make_proposal::<T>();
        Module::<T>::set_proposal_cool_off_period(RawOrigin::Root.into(), 0.into())?;
        Module::<T>::propose(
            origin.into(),
            proposal,
            42.into(),
            Some(url),
            Some(description)
        )?;
        // Populate vote history.
        let aye_voters = make_voters::<T>(VOTERS_A_NUM, "aye");
        let nay_voters = make_voters::<T>(VOTERS_B_NUM, "nay");
        cast_votes::<T>(0, aye_voters.as_slice(), true)?;
        cast_votes::<T>(0, nay_voters.as_slice(), false)?;
        // Cast an opposite vote.
        let User { account, origin, did, .. } =
            UserBuilder::<T>::default().build_with_did("voter", 0);
        identity::CurrentDid::put(did.unwrap());
        let voter_deposit = 43.into();
        // Cast an opposite vote.
        Module::<T>::vote(origin.clone().into(), 0, false, voter_deposit)?;
    }: _(origin, 0, true, voter_deposit)
    verify {
        assert_eq!(voter_deposit, Deposits::<T>::get(0, &account).amount);
    }

    approve_committee_proposal {
        let (proposal, url, description) = make_proposal::<T>();
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
        let User { account, origin, did, .. } =
            UserBuilder::<T>::default().build_with_did("proposer", 0);
        identity::CurrentDid::put(did.unwrap());
        let (proposal, url, description) = make_proposal::<T>();
        Module::<T>::set_proposal_cool_off_period(RawOrigin::Root.into(), 0.into())?;
        let deposit = 42.into();
        Module::<T>::propose(
            origin.into(),
            proposal,
            deposit,
            Some(url),
            Some(description)
        )?;
        assert_eq!(deposit, Deposits::<T>::get(&0, &account).amount);
        let vmo_origin = T::VotingMajorityOrigin::successful_origin();
        let call = Call::<T>::reject_proposal(0);
    }: {
        call.dispatch_bypass_filter(vmo_origin)?;
    }
    verify {
        assert_eq!(false, Deposits::<T>::contains_key(&0, &account));
    }

    prune_proposal {
        Module::<T>::set_proposal_cool_off_period(RawOrigin::Root.into(), 0.into())?;
        let User { account, origin, did, .. } =
            UserBuilder::<T>::default().build_with_did("proposer", 0);
        identity::CurrentDid::put(did.unwrap());
        let (proposal, url, description) = make_proposal::<T>();
        Module::<T>::propose(
            origin.into(),
            proposal,
            42.into(),
            Some(url),
            Some(description)
        )?;
        let vmo_origin = T::VotingMajorityOrigin::successful_origin();
        Module::<T>::set_prune_historical_pips(RawOrigin::Root.into(), false)?;
        let reject_call = Call::<T>::reject_proposal(0);
        reject_call.dispatch_bypass_filter(vmo_origin.clone())?;
        let call = Call::<T>::prune_proposal(0);
    }: {
        call.dispatch_bypass_filter(vmo_origin)?;
    }
    verify {
        assert_eq!(false, Proposals::<T>::contains_key(&0));
        assert_eq!(false, ProposalMetadata::<T>::contains_key(&0));
    }

    reschedule_execution {
        let User { account, origin, did, .. } =
            UserBuilder::<T>::default().build_with_did("proposer", 0);
        let did = did.expect("missing did in reschedule_execution");
        identity::CurrentDid::put(did);
        let (proposal, url, description) = make_proposal::<T>();
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
        let vmo_origin = T::VotingMajorityOrigin::successful_origin();
        let enact_call = Call::<T>::enact_snapshot_results(vec![(0, SnapshotResult::Approve)]);
        enact_call.dispatch_bypass_filter(vmo_origin)?;
        let future_block = frame_system::Module::<T>::block_number() + 100.into();
    }: _(origin, 0, Some(future_block))
    verify {
        assert_eq!(true, PipToSchedule::<T>::contains_key(&0));
    }

    clear_snapshot {
        let User { account, origin, did, .. } =
            UserBuilder::<T>::default().build_with_did("proposer", 0);
        let did = did.expect("missing did in clear_snapshot");
        identity::CurrentDid::put(did);
        let (proposal, url, description) = make_proposal::<T>();
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

    // TODO reduce fn complexity
    snapshot {
        // let origin0 = snapshot_setup::<T>()?;
    }: {} // _(origin0)
    verify {
        // assert!(SnapshotMeta::<T>::get().is_some());
    }

    // TODO reduce fn complexity
    enact_snapshot_results {
        // let origin0 = snapshot_setup::<T>()?;
        // Module::<T>::snapshot(origin0.into())?;
        // let enact_origin = T::VotingMajorityOrigin::successful_origin();
        // let enact_call = enact_call::<T>();
    }: {
        // enact_call.dispatch_bypass_filter(enact_origin)?;
    }
    verify {
        // assert_eq!(true, PipToSchedule::<T>::contains_key(&0));
    }

    execute_scheduled_pip {
        // set up
        let (origin0, did0) = pips_and_votes_setup::<T>(true)?;

        // snapshot
        identity::CurrentDid::put(did0);
        T::GovernanceCommittee::bench_set_release_coordinator(did0);
        Module::<T>::snapshot(origin0.into())?;

        // enact
        let enact_origin = T::VotingMajorityOrigin::successful_origin();
        let enact_call = enact_call::<T>();
        enact_call.dispatch_bypass_filter(enact_origin)?;

        // execute
        identity::CurrentDid::kill();
        let origin = RawOrigin::Root;
    }: _(origin, 0)
    verify {
        if Proposals::<T>::contains_key(&0) {
            assert_eq!(ProposalState::Failed, Module::<T>::proposals(&0).unwrap().state);
        }
    }

    expire_scheduled_pip {
        pips_and_votes_setup::<T>(true)?;
        identity::CurrentDid::kill();
        let origin = RawOrigin::Root;
    }: _(origin, GC_DID, 0)
    verify {
        assert_eq!(ProposalState::Expired, Module::<T>::proposals(&0).unwrap().state);
    }
}
