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
use frame_support::assert_ok;
use frame_system::RawOrigin;
use pallet_identity::{self as identity, benchmarking::make_account};
use polymesh_common_utilities::MaybeBlock;
use sp_std::{
    convert::{TryFrom, TryInto},
    prelude::*,
};

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
        let a in 0 .. u32::MAX;
        // description length
        let d in 0 .. 1_000;
        // URL length
        let u in 0 .. 500;
        // length of the proposal padding
        let c in 0 .. 100_000;

        let (account, origin, did) = make_account::<T>("signer", 0);
        identity::CurrentDid::put(did);
        let content = vec![b'X'; c as usize];
        let proposal = Box::new(frame_system::Call::<T>::remark(content).into());
        let url = Url::try_from(vec![b'X'; u as usize].as_slice()).unwrap();
        let description = PipDescription::try_from(vec![b'X'; d as usize].as_slice()).unwrap();
    }: propose(origin, proposal, a.into(), Some(url.clone()), Some(description.clone()))
    verify {
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

        let (account, origin, did) = make_account::<T>("signer", 0);
        identity::CurrentDid::put(did);
        let content = vec![b'X'; c as usize];
        let proposal = Box::new(frame_system::Call::<T>::remark(content).into());
        let url = Url::try_from(vec![b'X'; u as usize].as_slice()).unwrap();
        let description = PipDescription::try_from(vec![b'X'; d as usize].as_slice()).unwrap();
        let propose_result = Module::<T>::propose(
            frame_system::RawOrigin::Signed(account).into(),
            proposal,
            42.into(),
            None,
            None,
        );
    }: _(origin, 0, Some(url.clone()), Some(description.clone()))
    verify {
        assert_ok!(propose_result);
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

        let (account, origin, did) = make_account::<T>("signer", 0);
        identity::CurrentDid::put(did);
        let content = vec![b'X'; c as usize];
        let proposal = Box::new(frame_system::Call::<T>::remark(content).into());
        let url = Url::try_from(vec![b'X'; u as usize].as_slice()).unwrap();
        let desc = PipDescription::try_from(vec![b'X'; d as usize].as_slice()).unwrap();
        let propose_result = Module::<T>::propose(
            frame_system::RawOrigin::Signed(account.clone()).into(),
            proposal,
            42.into(),
            Some(url),
            Some(desc)
        );
    }: cancel_proposal(origin, 0)
    verify {
        assert_ok!(propose_result);
        assert_eq!(
            Module::<T>::proposals(0).unwrap().proposer,
            Proposer::Community(account)
        );
    }
}
