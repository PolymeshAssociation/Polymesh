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
use pallet_asset::benchmarking::make_asset;
use pallet_identity::benchmarking::{User, UserBuilder};
use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;
use core::iter;
use pallet_timestamp::Module as Timestamp;

const TAX: Tax = Tax::one();
const SEED: u32 = 0;
const MAX_TARGET_IDENTITIES: u32 = 100;
const MAX_DID_WHT_IDS: u32 = 100;
const MAX_DETAILS_LEN: u32 = 100;

// NOTE(Centril): A non-owner CAA is the less complex code path.
// Therefore, in general, we'll be using the owner as the CAA.

fn user<T: Trait>(prefix: &'static str, u: u32) -> User<T> {
    UserBuilder::<T>::default().build_with_did(prefix, u)
}

fn setup<T: Trait>() -> (User<T>, Ticker) {
    let owner = user("owner", SEED);
    let ticker = make_asset::<T>(&owner);
    (owner, ticker)
}

fn target<T: Trait>(u: u32) -> IdentityId {
    user::<T>("target", u).did()
}

fn target_ids<T: Trait>(n: u32, treatment: TargetTreatment) -> TargetIdentities {
    let identities = (0..n)
        .map(target::<T>)
        .flat_map(|did| iter::repeat(did).take(2))
        .collect::<Vec<_>>();
    TargetIdentities { identities, treatment }
}

fn did_whts<T: Trait>(n: u32) -> Vec<(IdentityId, Tax)> {
    (0..n).map(target::<T>).map(|did| (did, TAX)).collect::<Vec<_>>()
}

fn init_did_whts<T: Trait>(ticker: Ticker, n: u32) -> Vec<(IdentityId, Tax)> {
    let mut whts = did_whts::<T>(n);
    whts.sort_by_key(|(did, _)| *did);
    DidWithholdingTax::insert(ticker, whts.clone());
    whts
}

fn details(len: u32) -> CADetails {
    iter::repeat(b'a').take(len as usize).collect::<Vec<_>>().into()
}

benchmarks! {
    _ {}

    set_max_details_length {}: _(RawOrigin::Root, 100)
    verify {
        ensure!(MaxDetailsLength::get() == 100, "Wrong length set");
    }

    reset_caa {
        let (owner, ticker) = setup::<T>();
        // Generally the code path for no CAA is more complex,
        // but in this case having a different CAA already could cause more storage writes.
        let caa = UserBuilder::<T>::default().build_with_did("caa", SEED);
        Agent::insert(ticker, caa.did());
    }: _(owner.origin(), ticker)
    verify {
        ensure!(Agent::get(ticker) == None, "CAA not reset.");
    }

    set_default_targets {
        let i in 0..MAX_TARGET_IDENTITIES;

        let (owner, ticker) = setup::<T>();
        let targets = target_ids::<T>(i, TargetTreatment::Exclude);
        let targets2 = targets.clone();
    }: _(owner.origin(), ticker, targets)
    verify {
        ensure!(DefaultTargetIdentities::get(ticker) == targets2.dedup(), "Default targets not set");
    }

    set_default_withholding_tax {
        let (owner, ticker) = setup::<T>();
    }: _(owner.origin(), ticker, TAX)
    verify {
        ensure!(DefaultWithholdingTax::get(ticker) == TAX, "Default WHT not set");
    }

    set_did_withholding_tax {
        let i in 0..MAX_DID_WHT_IDS;

        let (owner, ticker) = setup::<T>();
        let mut whts = init_did_whts::<T>(ticker, i);
        let last = target::<T>(i + 1);
    }: _(owner.origin(), ticker, last, Some(TAX))
    verify {
        whts.push((last, TAX));
        whts.sort_by_key(|(did, _)| *did);
        ensure!(DidWithholdingTax::get(ticker) == whts, "Wrong DID WHTs");
    }

    initiate_corporate_action_use_defaults {
        let i in 0..MAX_DETAILS_LEN;
        let j in 0..MAX_DID_WHT_IDS;
        let k in 0..MAX_TARGET_IDENTITIES;

        let (owner, ticker) = setup::<T>();
        <Timestamp<T>>::set_timestamp(1000.into());
        let details = details(i);
        let whts = init_did_whts::<T>(ticker, j);
        let targets = target_ids::<T>(k, TargetTreatment::Exclude).dedup();
        DefaultTargetIdentities::insert(ticker, targets);
    }: initiate_corporate_action(
        owner.origin(), ticker, CAKind::Other, 1000,
        Some(RecordDateSpec::Scheduled(2000)),
        details, None, None, None
    )
    verify {
        ensure!(CAIdSequence::get(ticker).0 == 1, "CA not created");
    }

    initiate_corporate_action_provided {
        let i in 0..MAX_DETAILS_LEN;
        let j in 0..MAX_DID_WHT_IDS;
        let k in 0..MAX_TARGET_IDENTITIES;

        let (owner, ticker) = setup::<T>();
        <Timestamp<T>>::set_timestamp(1000.into());
        let details = details(i);
        let whts = Some(did_whts::<T>(j));
        let targets = Some(target_ids::<T>(k, TargetTreatment::Exclude));
    }: initiate_corporate_action(
        owner.origin(), ticker, CAKind::Other, 1000,
        Some(RecordDateSpec::Scheduled(2000)),
        details, targets, Some(TAX), whts
    )
    verify {
        ensure!(CAIdSequence::get(ticker).0 == 1, "CA not created");
    }
}
