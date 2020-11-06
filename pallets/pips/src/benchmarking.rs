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
use frame_benchmarking::{account, benchmarks};
use frame_system::RawOrigin;
use pallet_committee as committee;
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

    set_min_proposal_deposit {
        let b in 0 .. u32::MAX;

        let origin = RawOrigin::Root;
    }: _(origin, b.into())

    set_proposal_cool_off_period {
        let b in 0 .. u32::MAX;

        let origin = RawOrigin::Root;
    }: _(origin, b.into())

    set_default_enactment_period {
        let b in 0 .. u32::MAX;

        let origin = RawOrigin::Root;
    }: _(origin, b.into())

    set_pending_pip_expiry {
        let b in 0 .. u32::MAX;

        let origin = RawOrigin::Root;
    }: _(origin, MaybeBlock::Some(b.into()))

    set_max_pip_skip_count {
        let n in 0 .. 255;

        let origin = RawOrigin::Root;
    }: _(origin, n.try_into().unwrap())

    set_active_pip_limit {
        let n in 0 .. u32::MAX;

        let origin = RawOrigin::Root;
    }: _(origin, n)

    propose_from_community {
        let a in 0 .. u32::MAX;
        let d in 0 .. 1000;
        let u in 0 .. 500;
        let c in 0 .. 100_000;

        let account: T::AccountId = account("signer", 1_000_000, 0);
        let origin = RawOrigin::Signed(account.clone());
        let content = vec![b'X'; c as usize];
        let proposal = Box::new(frame_system::Call::<T>::remark(content).into());
        let url = Url::try_from(vec![b'X'; u as usize].as_slice()).unwrap();
        let desc = PipDescription::try_from(vec![b'X'; d as usize].as_slice()).unwrap();
    }: propose(origin, proposal, a.into(), Some(url), Some(desc))

    // propose_from_committee {
    //     let a in 0 .. u32::MAX;
    //     let d in 0 .. 1000;
    //     let u in 0 .. 500;
    //     let c in 0 .. 100_000;

    //     let origin =
    //         committee::Origin::<T, committee::Instance4>::Members(0, 0).into();
    //     let content = vec![b'X'; c as usize];
    //     let proposal = Box::new(frame_system::Call::<T>::remark(content).into());
    //     let url = Url::try_from(vec![b'X'; u as usize].as_slice()).unwrap();
    //     let desc = PipDescription::try_from(vec![b'X'; d as usize].as_slice()).unwrap();
    // }: propose(origin, proposal, a.into(), Some(url), Some(desc))
}
