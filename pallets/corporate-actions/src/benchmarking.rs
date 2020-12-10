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
use core::convert::TryFrom;
use core::iter;
use frame_benchmarking::benchmarks;
use frame_support::assert_ok;
use frame_system::RawOrigin;
use pallet_asset::benchmarking::{make_asset, make_document};
use pallet_identity::benchmarking::{User, UserBuilder};
use pallet_timestamp::Module as Timestamp;

const TAX: Tax = Tax::one();
const SEED: u32 = 0;
const MAX_TARGET_IDENTITIES: u32 = 100;
const MAX_DID_WHT_IDS: u32 = 100;
const MAX_DETAILS_LEN: u32 = 100;
const MAX_DOCS: u32 = 100;

crate const RD_SPEC: Option<RecordDateSpec> = Some(RecordDateSpec::Scheduled(2000));
const RD_SPEC2: Option<RecordDateSpec> = Some(RecordDateSpec::Scheduled(3000));

// NOTE(Centril): A non-owner CAA is the less complex code path.
// Therefore, in general, we'll be using the owner as the CAA.

fn user<T: Trait>(prefix: &'static str, u: u32) -> User<T> {
    UserBuilder::<T>::default().build_with_did(prefix, u)
}

fn setup<T: Trait>() -> (User<T>, Ticker) {
    <Timestamp<T>>::set_timestamp(1000.into());
    let owner = user("owner", SEED);
    let ticker = make_asset::<T>(&owner);
    (owner, ticker)
}

fn target<T: Trait>(u: u32) -> IdentityId {
    user::<T>("target", u).did()
}

crate fn target_ids<T: Trait>(n: u32, treatment: TargetTreatment) -> TargetIdentities {
    let identities = (0..n)
        .map(target::<T>)
        .flat_map(|did| iter::repeat(did).take(2))
        .collect::<Vec<_>>();
    TargetIdentities {
        identities,
        treatment,
    }
}

fn did_whts<T: Trait>(n: u32) -> Vec<(IdentityId, Tax)> {
    (0..n)
        .map(target::<T>)
        .map(|did| (did, TAX))
        .collect::<Vec<_>>()
}

fn init_did_whts<T: Trait>(ticker: Ticker, n: u32) -> Vec<(IdentityId, Tax)> {
    let mut whts = did_whts::<T>(n);
    whts.sort_by_key(|(did, _)| *did);
    DidWithholdingTax::insert(ticker, whts.clone());
    whts
}

fn details(len: u32) -> CADetails {
    iter::repeat(b'a')
        .take(len as usize)
        .collect::<Vec<_>>()
        .into()
}

fn add_docs<T: Trait>(origin: &T::Origin, ticker: Ticker, n: u32) -> Vec<DocumentId> {
    let ids = (0..n).map(DocumentId).collect::<Vec<_>>();
    let docs = (0..n).map(|_| make_document()).collect::<Vec<_>>();
    assert_ok!(<Asset<T>>::add_documents(origin.clone(), docs, ticker));
    ids
}

crate fn setup_ca<T: Trait>(kind: CAKind) -> (User<T>, CAId) {
    let (owner, ticker) = setup::<T>();
    <Timestamp<T>>::set_timestamp(1000.into());
    let origin: T::Origin = owner.origin().into();
    assert_ok!(<Module<T>>::initiate_corporate_action(
        origin.clone(),
        ticker,
        kind,
        1000,
        RD_SPEC,
        "".into(),
        None,
        None,
        None
    ));
    let ca_id = CAId {
        ticker,
        local_id: LocalCAId(0),
    };
    let ids = add_docs::<T>(&origin, ticker, 1);
    assert_ok!(<Module<T>>::link_ca_doc(origin.clone(), ca_id, ids));
    (owner, ca_id)
}

fn attach<T: Trait>(owner: &User<T>, ca_id: CAId) {
    let range = ballot::BallotTimeRange {
        start: 4000,
        end: 5000,
    };
    let motion = ballot::Motion {
        title: "".into(),
        info_link: "".into(),
        choices: vec!["".into()],
    };
    let meta = ballot::BallotMeta {
        title: "".into(),
        motions: vec![motion],
    };
    assert_ok!(<Ballot<T>>::attach_ballot(
        owner.origin().into(),
        ca_id,
        range,
        meta,
        true
    ));
}

crate fn currency<T: Trait>(owner: &User<T>) -> Ticker {
    let currency = Ticker::try_from(b"B" as &[_]).unwrap();
    Asset::<T>::create_asset(
        owner.origin().into(),
        currency.as_slice().into(),
        currency,
        1_000_000.into(),
        true,
        <_>::default(),
        vec![],
        None,
    )
    .expect("Asset cannot be created");
    currency
}

fn distribute<T: Trait>(owner: &User<T>, ca_id: CAId) {
    let currency = currency::<T>(owner);
    assert_ok!(<Distribution<T>>::distribute(
        owner.origin().into(),
        ca_id,
        None,
        currency,
        1000.into(),
        4000,
        None
    ));
}

fn check_ca_created<T: Trait>(ca_id: CAId) -> DispatchResult {
    ensure!(CAIdSequence::get(ca_id.ticker).0 == 1, "CA not created");
    Ok(())
}

fn check_ca_exists<T: Trait>(ca_id: CAId) -> DispatchResult {
    ensure!(
        CorporateActions::get(ca_id.ticker, ca_id.local_id) == None,
        "CA not removed"
    );
    Ok(())
}

fn check_rd<T: Trait>(ca_id: CAId) -> DispatchResult {
    let rd = CorporateActions::get(ca_id.ticker, ca_id.local_id)
        .unwrap()
        .record_date
        .unwrap()
        .date;
    ensure!(rd == 3000, "CA not removed");
    Ok(())
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
        let details = details(i);
        let whts = init_did_whts::<T>(ticker, j);
        let targets = target_ids::<T>(k, TargetTreatment::Exclude).dedup();
        DefaultTargetIdentities::insert(ticker, targets);
    }: initiate_corporate_action(
        owner.origin(), ticker, CAKind::Other, 1000, RD_SPEC, details, None, None, None
    )
    verify {
        ensure!(CAIdSequence::get(ticker).0 == 1, "CA not created");
    }

    initiate_corporate_action_provided {
        let i in 0..MAX_DETAILS_LEN;
        let j in 0..MAX_DID_WHT_IDS;
        let k in 0..MAX_TARGET_IDENTITIES;

        let (owner, ticker) = setup::<T>();
        let details = details(i);
        let whts = Some(did_whts::<T>(j));
        let targets = Some(target_ids::<T>(k, TargetTreatment::Exclude));
    }: initiate_corporate_action(
        owner.origin(), ticker, CAKind::Other, 1000, RD_SPEC, details, targets, Some(TAX), whts
    )
    verify {
        ensure!(CAIdSequence::get(ticker).0 == 1, "CA not created");
    }

    link_ca_doc {
        let i in 0..MAX_DOCS;

        let (owner, ticker) = setup::<T>();
        let origin: T::Origin = owner.origin().into();
        let ids = add_docs::<T>(&origin, ticker, i);
        let ids2 = ids.clone();
        assert_ok!(<Module<T>>::initiate_corporate_action(
            origin, ticker, CAKind::Other, 1000, None, "".into(), None, None, None
        ));
        let ca_id = CAId { ticker, local_id: LocalCAId(0) };
    }: _(owner.origin(), ca_id, ids)
    verify {
        ensure!(CADocLink::get(ca_id) == ids2, "Docs not linked")
    }

    remove_ca_with_ballot {
        let (owner, ca_id) = setup_ca::<T>(CAKind::IssuerNotice);
        attach(&owner, ca_id);
    }: remove_ca(owner.origin(), ca_id)
    verify {
        check_ca_created::<T>(ca_id)?;
        check_ca_exists::<T>(ca_id)?;
    }

    remove_ca_with_dist {
        let (owner, ca_id) = setup_ca::<T>(CAKind::UnpredictableBenefit);
        distribute(&owner, ca_id);
    }: remove_ca(owner.origin(), ca_id)
    verify {
        check_ca_created::<T>(ca_id)?;
        check_ca_exists::<T>(ca_id)?;
    }

    change_record_date_with_ballot {
        let (owner, ca_id) = setup_ca::<T>(CAKind::IssuerNotice);
        attach(&owner, ca_id);
    }: change_record_date(owner.origin(), ca_id, RD_SPEC2)
    verify {
        check_ca_created::<T>(ca_id)?;
        check_rd::<T>(ca_id)?;
    }

    change_record_date_with_dist {
        let (owner, ca_id) = setup_ca::<T>(CAKind::UnpredictableBenefit);
        distribute(&owner, ca_id);
    }: change_record_date(owner.origin(), ca_id, RD_SPEC2)
    verify {
        check_ca_created::<T>(ca_id)?;
        check_rd::<T>(ca_id)?;
    }
}
