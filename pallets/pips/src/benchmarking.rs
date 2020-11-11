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
use polymesh_common_utilities::MaybeBlock;
use sp_std::{
    convert::{TryFrom, TryInto},
    prelude::*,
};

pub fn make_proposal<T: Trait>(
    content_len: usize,
    url_len: usize,
    desc_len: usize,
) -> (Box<T::Proposal>, Url, PipDescription) {
    let content = vec![b'X'; content_len];
    let proposal = Box::new(frame_system::Call::<T>::remark(content).into());
    let url = Url::try_from(vec![b'X'; url_len].as_slice()).unwrap();
    let description = PipDescription::try_from(vec![b'X'; desc_len as usize].as_slice()).unwrap();
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
        // description length
        let d in 0 .. 1_000;
        // URL length
        let u in 0 .. 500;
        // length of the proposal padding
        let c in 0 .. 100_000;

        let (account, origin, did) = make_account::<T>("proposer", 0);
        identity::CurrentDid::put(did);
        let (proposal, url, description) = make_proposal::<T>(c as usize, u as usize, d as usize);
    }: propose(origin, proposal, a.into(), Some(url.clone()), Some(description.clone()))
    verify {
        let meta = Module::<T>::proposal_metadata(0).unwrap();
        assert_eq!(0, meta.id);
        assert_eq!(Some(url), meta.url);
        assert_eq!(Some(description), meta.description);
    }

    // `propose` from a committee origin.
    #[extra]
    propose {
        // description length
        let d in 0 .. 1_000;
        // URL length
        let u in 0 .. 500;
        // length of the proposal padding
        let c in 0 .. 100_000;

        let (account, origin, did) = make_account::<T>("proposer", 0);
        identity::CurrentDid::put(did);
        let (proposal, url, description) = make_proposal::<T>(c as usize, u as usize, d as usize);
        let origin = T::UpgradeCommitteeVMO::successful_origin();
        Module::<T>::set_min_proposal_deposit(RawOrigin::Root.into(), 0.into()).unwrap();
        Module::<T>::set_proposal_cool_off_period(RawOrigin::Root.into(), 0.into()).unwrap();
        let call = Call::<T>::propose(proposal, 0.into(), Some(url.clone()), Some(description.clone()));
    }: {
        call.dispatch_bypass_filter(origin)?;
        let meta = Module::<T>::proposal_metadata(0).unwrap();
        assert_eq!(0, meta.id);
        assert_eq!(Some(url), meta.url);
        assert_eq!(Some(description), meta.description);
    }

    amend_proposal {
        // description length
        let d in 0 .. 1_000;
        // URL length
        let u in 0 .. 500;
        // length of the proposal padding
        let c in 0 .. 100_000;

        let (account, origin, did) = make_account::<T>("proposer", 0);
        identity::CurrentDid::put(did);
        let (proposal, url, description) = make_proposal::<T>(c as usize, u as usize, d as usize);
        let propose_result = Module::<T>::propose(
            origin.clone().into(),
            proposal,
            42.into(),
            None,
            None,
        );
    }: _(origin, 0, Some(url.clone()), Some(description.clone()))
    verify {
        assert!(propose_result.is_ok());
        let meta = Module::<T>::proposal_metadata(0).unwrap();
        assert_eq!(0, meta.id);
        assert_eq!(Some(url), meta.url);
        assert_eq!(Some(description), meta.description);
    }

    cancel_proposal_community {
        // description length
        let d in 0 .. 1_000;
        // URL length
        let u in 0 .. 500;
        // length of the proposal padding
        let c in 0 .. 100_000;

        let (account, origin, did) = make_account::<T>("proposer", 0);
        identity::CurrentDid::put(did);
        let (proposal, url, description) = make_proposal::<T>(c as usize, u as usize, d as usize);
        let propose_result = Module::<T>::propose(
            origin.clone().into(),
            proposal,
            42.into(),
            Some(url),
            Some(description)
        );
    }: cancel_proposal(origin, 0)
    verify {
        assert!(propose_result.is_ok());
        assert_eq!(
            Module::<T>::proposals(0).unwrap().proposer,
            Proposer::Community(account)
        );
    }

    vote {
        // description length
        let d in 0 .. 1_000;
        // URL length
        let u in 0 .. 500;
        // length of the proposal padding
        let c in 0 .. 100_000;
        // aye or nay
        let v in 0 .. 1;

        let (proposer_account, proposer_origin, proposer_did) =
            make_account::<T>("proposer", 0);
        identity::CurrentDid::put(proposer_did);
        let (proposal, url, description) = make_proposal::<T>(c as usize, u as usize, d as usize);
        Module::<T>::set_proposal_cool_off_period(RawOrigin::Root.into(), 0.into()).unwrap();
        let propose_result = Module::<T>::propose(
            proposer_origin.into(),
            proposal,
            42.into(),
            Some(url),
            Some(description)
        );
        let (account, origin, did) = make_account::<T>("voter", 0);
        identity::CurrentDid::put(did);
        let voter_deposit = 43.into();
    }: _(origin, 0, v != 0, voter_deposit)
    verify {
        assert!(propose_result.is_ok());
        assert_eq!(voter_deposit, Deposits::<T>::get(0, &account).amount);
    }

    #[extra]
    approve_committee_proposal {
        // description length
        let d in 0 .. 1_000;
        // URL length
        let u in 0 .. 500;
        // length of the proposal padding
        let c in 0 .. 100_000;

        let (proposal, url, description) = make_proposal::<T>(c as usize, u as usize, d as usize);
        let proposer_origin = T::UpgradeCommitteeVMO::successful_origin();
        Module::<T>::set_min_proposal_deposit(RawOrigin::Root.into(), 0.into()).unwrap();
        Module::<T>::set_proposal_cool_off_period(RawOrigin::Root.into(), 0.into()).unwrap();
        Module::<T>::propose(
            proposer_origin,
            proposal,
            0.into(),
            Some(url),
            Some(description)
        ).unwrap();
        let origin = T::VotingMajorityOrigin::successful_origin();
        let call = Call::<T>::approve_committee_proposal(0);
    }: {
        call.dispatch_bypass_filter(origin)?;
        assert_eq!(true, PipToSchedule::<T>::contains_key(&0));
    }

    // reject_proposal {
    // }: _(origin, id)
    // verify {
    // }

    // prune_proposal {
    // }: _(origin, id)
    // verify {
    // }

    // reschedule_execution {
    // }: _(origin, id, until)
    // verify {
    // }

    // clear_snapshot {
    // }: _(origin)
    // verify {
    // }

    // snapshot {
    // }: _(origin)
    // verify {
    // }

    // enact_snapshot_results {
    // }: _(origin, results)
    // verify {
    // }
}
