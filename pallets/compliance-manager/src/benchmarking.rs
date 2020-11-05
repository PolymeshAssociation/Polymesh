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
use pallet_asset::SecurityToken;
use pallet_balances as balances;
use polymesh_common_utilities::{asset::AssetType, group::GroupTrait};
use polymesh_primitives::{IdentityId, InvestorUid, TrustedFor, TrustedIssuer};

use frame_benchmarking::{account, benchmarks};
use frame_support::traits::Currency;
use frame_system::RawOrigin;

const SEED: u32 = 1;

pub struct Account<T: Trait> {
    pub account: T::AccountId,
    pub origin: RawOrigin<T::AccountId>,
    pub did: IdentityId,
}

pub fn uid_from_name_and_idx(name: &'static str, u: u32) -> InvestorUid {
    InvestorUid::from((name, u).encode().as_slice())
}

pub fn make_account_without_did<T: Trait>(
    name: &'static str,
    u: u32,
) -> (T::AccountId, RawOrigin<T::AccountId>) {
    let account: T::AccountId = account(name, u, SEED);
    let origin = RawOrigin::Signed(account.clone());
    let _ = balances::Module::<T>::make_free_balance_be(&account, 1_000_000.into());
    (account, origin)
}

pub fn make_account<T: Trait>(name: &'static str, u: u32) -> Account<T> {
    let (account, origin) = make_account_without_did::<T>(name, u);
    let uid = uid_from_name_and_idx(name, u);
    let _ = identity::Module::<T>::register_did(origin.clone().into(), uid, vec![]);
    let did = identity::Module::<T>::get_identity(&account).unwrap();

    Account {
        account,
        origin,
        did,
    }
}

pub fn make_cdd_account<T: Trait>(u: u32) -> Account<T> {
    let account = make_account::<T>("cdd", u);
    T::CddServiceProviders::add_member(account.did).unwrap();

    account
}

pub fn make_token<T: Trait>(owner: &Account<T>, name: Vec<u8>) -> Ticker {
    let token = SecurityToken {
        name: name.into(),
        owner_did: owner.did.clone(),
        total_supply: 1_000_000,
        divisible: true,
        asset_type: AssetType::default(),
        ..Default::default()
    };
    let ticker = Ticker::try_from(token.name.0.as_slice()).unwrap();

    T::Asset::create_asset(
        owner.origin.clone().into(),
        token.name.clone(),
        ticker.clone(),
        token.total_supply.into(),
        true,
        token.asset_type.clone(),
        vec![],
        None,
    )
    .expect("Cannot create an asset");

    ticker
}

fn make_issuer(id: u128) -> TrustedIssuer {
    TrustedIssuer {
        issuer: IdentityId::from(id),
        trusted_for: TrustedFor::Any,
    }
}

// TODO More complexity... TrustedFor: Any or Vec<ClaimType>
fn make_issuers(s: u32) -> Vec<TrustedIssuer> {
    (0u128..s as u128).map(make_issuer).collect::<Vec<_>>()
}

fn make_conditions(s: u32, issuers: &Vec<TrustedIssuer>) -> Vec<Condition> {
    (0..s)
        .map(|_i| Condition {
            condition_type: ConditionType::IsPresent(Claim::NoData),
            issuers: issuers.clone(),
        })
        .collect::<Vec<_>>()
}

struct ComplianceRequirementData<T: Trait> {
    pub owner: Account<T>,
    pub buyer: Account<T>,
    pub ticker: Ticker,
    pub issuers: Vec<TrustedIssuer>,
    pub sender_conditions: Vec<Condition>,
    pub receiver_conditions: Vec<Condition>,
}

impl<T: Trait> ComplianceRequirementData<T> {
    pub fn new(
        trusted_issuer_count: u32,
        sender_conditions_count: u32,
        receiver_conditions_count: u32,
    ) -> Self {
        // Create accounts and token.
        let owner = make_account::<T>("OWNER", SEED);
        let buyer = make_account::<T>("BUYER", SEED);
        let ticker = make_token::<T>(&owner, b"1".to_vec());

        // Create issuers (i) and conditions(s & r).
        let issuers = make_issuers(trusted_issuer_count);
        let sender_conditions = make_conditions(sender_conditions_count, &issuers);
        let receiver_conditions = make_conditions(receiver_conditions_count, &issuers);

        Self {
            owner,
            buyer,
            ticker,
            issuers,
            sender_conditions,
            receiver_conditions,
        }
    }
}

fn add_compliance_requirement_with_data<T: Trait>(d: &ComplianceRequirementData<T>) {
    Module::<T>::add_compliance_requirement(
        d.owner.origin.clone().into(),
        d.ticker.clone(),
        d.sender_conditions.clone(),
        d.receiver_conditions.clone(),
    )
    .expect("Compliance requirement cannot be added");
}

fn add_default_trusted_claim_issuer_with_data<T: Trait>(d: &ComplianceRequirementData<T>, i: u32) {
    make_issuers(i).into_iter().for_each(|issuer| {
        Module::<T>::add_default_trusted_claim_issuer(
            d.owner.origin.clone().into(),
            d.ticker.clone(),
            issuer,
        )
        .expect("Default trusted claim issuer cannot be added");
    });
}

benchmarks! {
    _ {}

    // TODO - Issuer of a claim should be limited!
    //
    add_compliance_requirement {
        let s in 1..5; // TODO Max sender conditions.
        let r in 1..5; // TODO Max receiver conditions.
        let i in 1..3; // TODO Max trusted issuers.

        let d = ComplianceRequirementData::<T>::new(i, s, r);

    }: _(d.owner.origin, d.ticker, d.sender_conditions, d.receiver_conditions)

    remove_compliance_requirement {
        let s in 1..5; // TODO Max sender conditions.
        let r in 1..5; // TODO Max receiver conditions.
        let i in 1..3; // TODO Max trusted issuers.

        // Add the compliance requirement.
        let d = ComplianceRequirementData::<T>::new(i, s, r);
        add_compliance_requirement_with_data(&d);

        // Remove the latest one.
        let id = Module::<T>::get_latest_requirement_id(d.ticker);
    }: _(d.owner.origin, d.ticker, id)
    verify {
        let is_removed = Module::<T>::asset_compliance(d.ticker)
            .requirements
            .into_iter()
            .find(|r| r.id == id)
            .is_none();

        ensure!( is_removed, "Compliance requirement was not removed");
    }

    pause_asset_compliance {
        // Create and add the compliance requirement.
        let d = ComplianceRequirementData::<T>::new(2, 1, 1);
        add_compliance_requirement_with_data(&d);
    }: _(d.owner.origin, d.ticker)

    resume_asset_compliance {
        // Create and add the compliance requirement.
        let d = ComplianceRequirementData::<T>::new(2, 1, 1);
        add_compliance_requirement_with_data(&d);

        Module::<T>::pause_asset_compliance(
            d.owner.origin.clone().into(),
            d.ticker.clone()).unwrap();
    }: _(d.owner.origin, d.ticker)

    /*
       replace_asset_compliance {
       } _(origin, ticker: Ticker, asset_compliance: Vec<ComplianceRequirement>)
       reset_asset_compliance {

       } _(origin, ticker: Ticker)

    add_default_trusted_claim_issuer {
        let i in 0..10; // TODO Max trusted issuers.

        // Create and add the compliance requirement.
        let d = ComplianceRequirementData::<T>::new(2, 1, 0);
        add_compliance_requirement_with_data(&d);

        // Generate some trusted issuer.
        add_default_trusted_claim_issuer_with_data( &d, i);

        // Add one more for benchmarking.
        // TODO Issuer ID is checked here but not when it is added in the CR.
        let new_issuer = TrustedIssuer::from( make_account::<T>("ISSUER", SEED).did);
    }: _(d.owner.origin, d.ticker, new_issuer)

    remove_default_trusted_claim_issuer {
        let i in 1..10; // TODO Max trusted issuers.

        // Create and add the compliance requirement.
        let d = ComplianceRequirementData::<T>::new(2, 1, i);
        add_compliance_requirement_with_data(&d);

        // Generate some trusted issuer.
        add_default_trusted_claim_issuer_with_data(&d, i);

        // Delete the latest trusted issuer.
        let issuer = make_issuer(i as u128);
    }: _(d.owner.origin, d.ticker, issuer)

       change_compliance_requirement {
       } _(origin, ticker: Ticker, new_req: ComplianceRequirement)
       */
}
