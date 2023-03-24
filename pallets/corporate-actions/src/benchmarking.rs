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
use core::iter;
use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;
use pallet_asset::benchmarking::make_document;
use polymesh_common_utilities::{
    benchs::{make_asset, user, AccountIdOf, User},
    TestUtilsFn,
};

const TAX: Tax = Tax::one();
pub(crate) const SEED: u32 = 0;
const MAX_TARGET_IDENTITIES: u32 = 500;
const MAX_DID_WHT_IDS: u32 = 1000;
const DETAILS_LEN: u32 = 1000;
const MAX_DOCS: u32 = 1000;

pub(crate) const RD_SPEC: Option<RecordDateSpec> = Some(RecordDateSpec::Scheduled(2000));
const RD_SPEC2: Option<RecordDateSpec> = Some(RecordDateSpec::Scheduled(3000));

// NOTE(Centril): A non-owner CAA is the less complex code path.
// Therefore, in general, we'll be using the owner as the CAA.

fn setup<T: Config + TestUtilsFn<AccountIdOf<T>>>() -> (User<T>, Ticker) {
    <pallet_timestamp::Now<T>>::set(1000u32.into());

    let owner = user("owner", SEED);
    let ticker = make_asset::<T>(&owner, None);
    (owner, ticker)
}

fn target<T: Config + TestUtilsFn<AccountIdOf<T>>>(u: u32) -> IdentityId {
    user::<T>("target", u).did()
}

pub(crate) fn target_ids<T: Config + TestUtilsFn<AccountIdOf<T>>>(
    n: u32,
    treatment: TargetTreatment,
) -> TargetIdentities {
    let identities = (0..n)
        .map(target::<T>)
        .flat_map(|did| iter::repeat(did).take(2))
        .collect::<Vec<_>>();
    TargetIdentities {
        identities,
        treatment,
    }
}

pub(crate) fn did_whts<T: Config + TestUtilsFn<AccountIdOf<T>>>(n: u32) -> Vec<(IdentityId, Tax)> {
    (0..n)
        .map(target::<T>)
        .map(|did| (did, TAX))
        .collect::<Vec<_>>()
}

fn init_did_whts<T: Config + TestUtilsFn<AccountIdOf<T>>>(
    ticker: Ticker,
    n: u32,
) -> Vec<(IdentityId, Tax)> {
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

fn add_docs<T: Config>(origin: &T::RuntimeOrigin, ticker: Ticker, n: u32) -> Vec<DocumentId> {
    let ids = (0..n).map(DocumentId).collect::<Vec<_>>();
    let docs = (0..n).map(|_| make_document()).collect::<Vec<_>>();
    <Asset<T>>::add_documents(origin.clone(), docs, ticker).unwrap();
    ids
}

pub(crate) fn setup_ca<T: Config + TestUtilsFn<AccountIdOf<T>>>(kind: CAKind) -> (User<T>, CAId) {
    let (owner, ticker) = setup::<T>();

    <pallet_timestamp::Now<T>>::set(1000u32.into());

    let origin: T::RuntimeOrigin = owner.origin().into();
    <Module<T>>::initiate_corporate_action(
        origin.clone(),
        ticker,
        kind,
        1000,
        RD_SPEC,
        "".into(),
        None,
        None,
        None,
    )
    .unwrap();
    let ca_id = CAId {
        ticker,
        local_id: LocalCAId(0),
    };
    let ids = add_docs::<T>(&origin, ticker, 1);
    <Module<T>>::link_ca_doc(origin.clone(), ca_id, ids).unwrap();
    (owner, ca_id)
}

fn attach<T: Config>(owner: &User<T>, ca_id: CAId) {
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
    <Ballot<T>>::attach_ballot(owner.origin().into(), ca_id, range, meta, true).unwrap();
}

pub(crate) fn currency<T: Config>(owner: &User<T>) -> Ticker {
    let currency = Ticker::from_slice_truncated(b"B" as &[_]);
    Asset::<T>::create_asset(
        owner.origin().into(),
        currency.as_slice().into(),
        currency,
        true,
        <_>::default(),
        vec![],
        None,
        true,
    )
    .expect("Asset cannot be created");
    Asset::<T>::issue(owner.origin().into(), currency, 1_000_000u32.into())
        .expect("Could not mint for asset");
    currency
}

fn distribute<T: Config>(owner: &User<T>, ca_id: CAId) {
    let currency = currency::<T>(owner);
    <Distribution<T>>::distribute(
        owner.origin().into(),
        ca_id,
        None,
        currency,
        2u32.into(),
        1000u32.into(),
        4000,
        None,
    )
    .unwrap();
}

pub(crate) fn set_ca_targets<T: Config + TestUtilsFn<AccountIdOf<T>>>(ca_id: CAId, k: u32) {
    CorporateActions::mutate(ca_id.ticker, ca_id.local_id, |ca| {
        let mut ids = target_ids::<T>(k, TargetTreatment::Exclude);
        ids.identities.sort();
        ca.as_mut().unwrap().targets = ids;
    });
}

fn check_ca_created<T: Config>(ca_id: CAId) -> DispatchResult {
    assert_eq!(CAIdSequence::get(ca_id.ticker).0, 1, "CA not created");
    Ok(())
}

fn check_ca_exists<T: Config>(ca_id: CAId) -> DispatchResult {
    assert_eq!(
        CorporateActions::get(ca_id.ticker, ca_id.local_id),
        None,
        "CA not removed"
    );
    Ok(())
}

fn check_rd<T: Config>(ca_id: CAId) -> DispatchResult {
    let rd = CorporateActions::get(ca_id.ticker, ca_id.local_id)
        .unwrap()
        .record_date
        .unwrap()
        .date;
    assert_eq!(rd, 3000, "CA not removed");
    Ok(())
}

benchmarks! {
    where_clause { where T: TestUtilsFn<AccountIdOf<T>> }

    set_max_details_length {}: _(RawOrigin::Root, 100)
    verify {
        assert_eq!(MaxDetailsLength::get(), 100, "Wrong length set");
    }

    set_default_targets {
        let t in 0..MAX_TARGET_IDENTITIES;

        let (owner, ticker) = setup::<T>();
        let targets = target_ids::<T>(t, TargetTreatment::Exclude);
        let targets2 = targets.clone();
    }: _(owner.origin(), ticker, targets)
    verify {
        assert_eq!(DefaultTargetIdentities::get(ticker), targets2.dedup(), "Default targets not set");
    }

    set_default_withholding_tax {
        let (owner, ticker) = setup::<T>();
    }: _(owner.origin(), ticker, TAX)
    verify {
        assert_eq!(DefaultWithholdingTax::get(ticker), TAX, "Default WHT not set");
    }

    set_did_withholding_tax {
        let w in 0..(MAX_DID_WHT_IDS - 1);

        let (owner, ticker) = setup::<T>();
        let mut whts = init_did_whts::<T>(ticker, w);
        let last = target::<T>(w + 1);
    }: _(owner.origin(), ticker, last, Some(TAX))
    verify {
        whts.push((last, TAX));
        whts.sort_by_key(|(did, _)| *did);
        assert_eq!(DidWithholdingTax::get(ticker), whts, "Wrong DID WHTs");
    }

    initiate_corporate_action_use_defaults {
        let w in 0..MAX_DID_WHT_IDS;
        let t in 0..MAX_TARGET_IDENTITIES;

        let (owner, ticker) = setup::<T>();
        let details = details(DETAILS_LEN);
        let whts = init_did_whts::<T>(ticker, w);
        let targets = target_ids::<T>(t, TargetTreatment::Exclude).dedup();
        DefaultTargetIdentities::insert(ticker, targets);
    }: initiate_corporate_action(
        owner.origin(), ticker, CAKind::Other, 1000, RD_SPEC, details, None, None, None
    )
    verify {
        assert_eq!(CAIdSequence::get(ticker).0, 1, "CA not created");
    }

    initiate_corporate_action_provided {
        let w in 0..MAX_DID_WHT_IDS;
        let t in 0..MAX_TARGET_IDENTITIES;

        let (owner, ticker) = setup::<T>();
        let details = details(DETAILS_LEN);
        let whts = Some(did_whts::<T>(w));
        let targets = Some(target_ids::<T>(t, TargetTreatment::Exclude));
    }: initiate_corporate_action(
        owner.origin(), ticker, CAKind::Other, 1000, RD_SPEC, details, targets, Some(TAX), whts
    )
    verify {
        assert_eq!(CAIdSequence::get(ticker).0, 1, "CA not created");
    }

    link_ca_doc {
        let d in 0..MAX_DOCS;

        let (owner, ticker) = setup::<T>();
        let origin: T::RuntimeOrigin = owner.origin().into();
        let ids = add_docs::<T>(&origin, ticker, d);
        let ids2 = ids.clone();
        <Module<T>>::initiate_corporate_action(
            origin, ticker, CAKind::Other, 1000, None, "".into(), None, None, None
        ).unwrap();
        let ca_id = CAId { ticker, local_id: LocalCAId(0) };
    }: _(owner.origin(), ca_id, ids)
    verify {
        assert_eq!(CADocLink::get(ca_id), ids2, "Docs not linked")
    }

    remove_ca_with_ballot {
        let (owner, ca_id) = setup_ca::<T>(CAKind::IssuerNotice);
        attach(&owner, ca_id);
    }: remove_ca(owner.origin(), ca_id)
    verify {
        check_ca_created::<T>(ca_id).unwrap();
        check_ca_exists::<T>(ca_id).unwrap();
    }

    remove_ca_with_dist {
        let (owner, ca_id) = setup_ca::<T>(CAKind::UnpredictableBenefit);
        distribute(&owner, ca_id);
    }: remove_ca(owner.origin(), ca_id)
    verify {
        check_ca_created::<T>(ca_id).unwrap();
        check_ca_exists::<T>(ca_id).unwrap();
    }

    change_record_date_with_ballot {
        let (owner, ca_id) = setup_ca::<T>(CAKind::IssuerNotice);
        attach(&owner, ca_id);
    }: change_record_date(owner.origin(), ca_id, RD_SPEC2)
    verify {
        check_ca_created::<T>(ca_id).unwrap();
        check_rd::<T>(ca_id).unwrap();
    }

    change_record_date_with_dist {
        let (owner, ca_id) = setup_ca::<T>(CAKind::UnpredictableBenefit);
        distribute(&owner, ca_id);
    }: change_record_date(owner.origin(), ca_id, RD_SPEC2)
    verify {
        check_ca_created::<T>(ca_id).unwrap();
        check_rd::<T>(ca_id).unwrap();
    }
}
