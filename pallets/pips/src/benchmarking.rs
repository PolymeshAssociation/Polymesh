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
use frame_support::traits::UnfilteredDispatchable;
use frame_system::RawOrigin;
use pallet_identity::{self as identity, benchmarking::make_account};
use polymesh_common_utilities::{MaybeBlock, SystematicIssuers};
use sp_std::{
    convert::{TryFrom, TryInto},
    prelude::*,
};

const DESCRIPTION_LEN: usize = 1000;
const URL_LEN: usize = 500;

pub fn make_proposal<T: Trait>(content_len: usize) -> (Box<T::Proposal>, Url, PipDescription) {
    let content = vec![b'X'; content_len];
    let proposal = Box::new(frame_system::Call::<T>::remark(content).into());
    let url = Url::try_from(vec![b'X'; URL_LEN].as_slice()).unwrap();
    let description = PipDescription::try_from(vec![b'X'; DESCRIPTION_LEN].as_slice()).unwrap();
    (proposal, url, description)
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
        let b in 0 .. u32::MAX;

        let origin = RawOrigin::Root;
        let deposit = b.into();
    }: _(origin, deposit)
    verify {
        assert_eq!(deposit, MinimumProposalDeposit::<T>::get());
    }

    set_proposal_cool_off_period {
        let b in 0 .. u32::MAX;

        let origin = RawOrigin::Root;
        let period = b.into();
    }: _(origin, period)
    verify {
        assert_eq!(period, ProposalCoolOffPeriod::<T>::get());
    }

    set_default_enactment_period {
        let b in 0 .. u32::MAX;

        let origin = RawOrigin::Root;
        let period = b.into();
    }: _(origin, period)
    verify {
        assert_eq!(period, DefaultEnactmentPeriod::<T>::get());
    }

    set_pending_pip_expiry {
        let b in 0 .. u32::MAX;

        let origin = RawOrigin::Root;
        let maybe_block = MaybeBlock::Some(b.into());
    }: _(origin, maybe_block)
    verify {
        assert_eq!(maybe_block, PendingPipExpiry::<T>::get());
    }

    set_max_pip_skip_count {
        let n in 0 .. 255;

        let origin = RawOrigin::Root;
        let count = n.try_into().unwrap();
    }: _(origin, count)
    verify {
        assert_eq!(count, MaxPipSkipCount::get());
    }

    set_active_pip_limit {
        let n in 0 .. u32::MAX;

        let origin = RawOrigin::Root;
    }: _(origin, n)
    verify {
        assert_eq!(n, ActivePipLimit::get());
    }

    propose_from_community {
        // deposit
        let a in 0 .. 500_000;
        // length of the proposal padding
        let c in 0 .. 100_000;

        let (account, origin, did) = make_account::<T>("proposer", 0);
        identity::CurrentDid::put(did);
        let (proposal, url, description) = make_proposal::<T>(c as usize);
    }: propose(origin, proposal, a.into(), Some(url.clone()), Some(description.clone()))
    verify {
        let meta = Module::<T>::proposal_metadata(0).unwrap();
        assert_eq!(0, meta.id);
        assert_eq!(Some(url), meta.url);
        assert_eq!(Some(description), meta.description);
    }

    // `propose` from a committee origin.
    propose_from_committee {
        // length of the proposal padding
        let c in 0 .. 100_000;

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
        let c in 0 .. 100_000;

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
        let c in 0 .. 100_000;

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
        let c in 0 .. 100_000;
        // aye or nay
        //
        // TODO: The backend produces n - 1 samples of `1` and 1 sample of `0` for any n. This has
        // to be fixed since the number of `0` and `1` samples should be roughly the same.
        let v in 0 .. 1;

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
        let (account, origin, did) = make_account::<T>("voter", 0);
        identity::CurrentDid::put(did);
        let voter_deposit = 43.into();
    }: _(origin, 0, v != 0, voter_deposit)
    verify {
        assert_eq!(voter_deposit, Deposits::<T>::get(0, &account).amount);
    }

    approve_committee_proposal {
        // length of the proposal padding
        let c in 0 .. 100_000;

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
        let c in 0 .. 100_000;

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
        let c in 0 .. 100_000;

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
        let c in 0 .. 100_000;

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
        let c in 0 .. 100_000;

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
        let c in 0 .. 100_000;

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
    }: _(origin)
    verify {
        assert!(SnapshotMeta::<T>::get().is_some());
    }

    enact_snapshot_results {
        // length of the proposal padding
        let c in 0 .. 100_000;

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
    }: {
        enact_call.dispatch_bypass_filter(enact_origin)?;
    }
    verify {
        assert_eq!(true, PipToSchedule::<T>::contains_key(&0));
    }
}
