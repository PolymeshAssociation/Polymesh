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
use polymesh_common_utilities::asset::AssetType;
use polymesh_primitives::{IdentityId, InvestorUid, TrustedFor, TrustedIssuer};

use frame_benchmarking::{account, benchmarks};
use frame_support::traits::Currency;
use frame_system::RawOrigin;

const SEED: u32 = 1;

/// Helper class to create accounts and its DID to simplify benchmarks and UT.
pub struct User<T: Trait> {
    pub account: T::AccountId,
    pub origin: RawOrigin<T::AccountId>,
    pub did: Option<IdentityId>,
}

impl<T: Trait> User<T> {
    /// It creates an account based on `name` and `u` with 1_000_000 as free balance.
    /// It also register the DID for that account.
    pub fn new(name: &'static str, u: u32) -> Self {
        let mut user = Self::without_did(name, u);
        let uid = InvestorUid::from((name, u).encode().as_slice());
        let _ = identity::Module::<T>::register_did(user.origin.clone().into(), uid, vec![]);
        user.did = identity::Module::<T>::get_identity(&user.account);

        user
    }

    /// It creates an account based on `name` and `u` with 1_000_000 as free balance.
    pub fn without_did( name: &'static str, u: u32,) -> Self { 
        let account: T::AccountId = account(name, u, SEED);
        let origin = RawOrigin::Signed(account.clone());
        let _ = balances::Module::<T>::make_free_balance_be(&account, 1_000_000.into());

        Self { account, origin, did: None }
    }
}

/// It creates a new token with name `name` on behalf of `owner`.
/// It is a divisible token with 1_000_000 units.
pub fn make_token<T: Trait>(owner: &User<T>, name: Vec<u8>) -> Ticker {
    let token = SecurityToken {
        name: name.into(),
        owner_did: owner.did.clone().unwrap(),
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

/// It creates a token issuer trusted for `Any`.
fn make_issuer<T: Trait>(id: u32) -> TrustedIssuer {
    let u = User::<T>::new("ISSUER", id);
    TrustedIssuer {
        issuer: IdentityId::from(u.did.unwrap()),
        trusted_for: TrustedFor::Any,
    }
}

/// Helper function to create `s` token issuers with `fn make_issuer`.
/// # TODO 
///   - It could have more complexity if `TrustedIssuer::trusted_for` is a vector but not on
///   benchmarking of add/remove. That could be useful for benchmarking executions/evaluation of
///   complience requiriments.
fn make_issuers<T: Trait>(s: u32) -> Vec<TrustedIssuer> {
    (0..s).map(|i| make_issuer::<T>(i)).collect::<Vec<_>>()
}

/// It creates simple conditions with a variable number of `issuers`. 
fn make_conditions(s: u32, issuers: &Vec<TrustedIssuer>) -> Vec<Condition> {
    (0..s)
        .map(|_i| Condition {
            condition_type: ConditionType::IsPresent(Claim::NoData),
            issuers: issuers.clone(),
        })
        .collect::<Vec<_>>()
}

/// This struct helps to simplify the parameter copy/pass during the benchmarks. 
struct ComplianceRequirementInfo<T: Trait> {
    pub owner: User<T>,
    pub buyer: User<T>,
    pub ticker: Ticker,
    pub issuers: Vec<TrustedIssuer>,
    pub sender_conditions: Vec<Condition>,
    pub receiver_conditions: Vec<Condition>,
}

impl<T: Trait> ComplianceRequirementInfo<T> {
    pub fn add_default_trusted_claim_issuer(self: &Self, i: u32) {
        make_issuers::<T>(i).into_iter().for_each(|issuer| {
            Module::<T>::add_default_trusted_claim_issuer(
                self.owner.origin.clone().into(),
                self.ticker.clone(),
                issuer)
                .expect("Default trusted claim issuer cannot be added");
        });
    }
}

struct ComplianceRequirementBuilder<T: Trait> { 
    info: ComplianceRequirementInfo<T>,
    has_been_added: bool,
}

impl<T: Trait> ComplianceRequirementBuilder<T> {
    pub fn new(
        trusted_issuer_count: u32,
        sender_conditions_count: u32,
        receiver_conditions_count: u32,
    ) -> Self {
        // Create accounts and token.
        let owner = User::<T>::new("OWNER", SEED);
        let buyer = User::<T>::new("BUYER", SEED);
        let ticker = make_token::<T>(&owner, b"1".to_vec());

        // Create issuers (i) and conditions(s & r).
        let issuers = make_issuers::<T>(trusted_issuer_count);
        let sender_conditions = make_conditions(sender_conditions_count, &issuers);
        let receiver_conditions = make_conditions(receiver_conditions_count, &issuers);

        let info = ComplianceRequirementInfo {
            owner,
            buyer,
            ticker,
            issuers,
            sender_conditions,
            receiver_conditions,
        };
        
        Self { info, has_been_added: false }
    }

    /// It registers the compliance requirement in the module.
    pub fn add_compliance_requirement(mut self: Self) -> Self {
        assert!( self.has_been_added == false, "Compliance has been added before");
        Module::<T>::add_compliance_requirement(
            self.info.owner.origin.clone().into(),
            self.info.ticker.clone(),
            self.info.sender_conditions.clone(),
            self.info.receiver_conditions.clone())
            .expect("Compliance requirement cannot be added");
        self.has_been_added = true;
        self
    }

    pub fn build(self: Self) -> ComplianceRequirementInfo<T> {
        self.info
    }
}


benchmarks! {
    _ {}

    add_compliance_requirement {
        // INTERNAL: This benchmark only evaluate the adding operation. Its execution should be measured in another module.
        let s in 1..T::MaxReceiverConditionsPerCompliance::get() as u32;
        let r in 1..T::MaxSenderConditionsPerCompliance::get() as u32;
        let i in 1..T::MaxTrustedIssuerPerCondition::get() as u32;

        let d = ComplianceRequirementBuilder::<T>::new(i, s, r).build();

    }: _(d.owner.origin, d.ticker, d.sender_conditions.clone(), d.receiver_conditions.clone())
    verify {
        let req = Module::<T>::asset_compliance(d.ticker).requirements.pop().unwrap();
        ensure!( req.sender_conditions == d.sender_conditions, "Sender conditions not expected");
        ensure!( req.receiver_conditions == d.receiver_conditions, "Sender conditions not expected");
    }

    remove_compliance_requirement {
        let s in 1..T::MaxReceiverConditionsPerCompliance::get() as u32;
        let r in 1..T::MaxSenderConditionsPerCompliance::get() as u32;
        let i in 1..T::MaxTrustedIssuerPerCondition::get() as u32;

        // Add the compliance requirement.
        let d = ComplianceRequirementBuilder::<T>::new(i, s, r)
            .add_compliance_requirement().build();

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
        let d = ComplianceRequirementBuilder::<T>::new(2, 1, 1)
            .add_compliance_requirement().build();
    }: _(d.owner.origin, d.ticker)
    verify {
        ensure!( Module::<T>::asset_compliance(d.ticker).paused == true, "Asset compliance is not paused");
    }

    resume_asset_compliance {
        let d = ComplianceRequirementBuilder::<T>::new(2, 1, 1)
            .add_compliance_requirement().build();

        Module::<T>::pause_asset_compliance(
            d.owner.origin.clone().into(),
            d.ticker.clone()).unwrap();
    }: _(d.owner.origin, d.ticker)
    verify {
        ensure!( Module::<T>::asset_compliance(d.ticker).paused == false, "Asset compliance is paused");
    }

    add_default_trusted_claim_issuer {
        let i in 0..(T::MaxDefaultTrustedClaimIssuers::get() as u32 -1);

        // Create and add the compliance requirement.
        let d = ComplianceRequirementBuilder::<T>::new(1, 1, 0)
            .add_compliance_requirement()
            .build();
        d.add_default_trusted_claim_issuer(i);

        // Add one more for benchmarking.
        let new_issuer = make_issuer::<T>(i+1);
    }: _(d.owner.origin, d.ticker, new_issuer.clone())
    verify {
        let trusted_issuers = Module::<T>::trusted_claim_issuer(d.ticker);
        ensure!(
            trusted_issuers.contains(&new_issuer),
            "Default trusted claim issuer was not added");
    }

    remove_default_trusted_claim_issuer {
        let i in 1..T::MaxDefaultTrustedClaimIssuers::get() as u32;

        // Create and add the compliance requirement.
        let d = ComplianceRequirementBuilder::<T>::new(2, 1, i)
            .add_compliance_requirement().build();

        // Generate some trusted issuer.
        d.add_default_trusted_claim_issuer(i);

        // Delete the latest trusted issuer.
        let issuer = Module::<T>::trusted_claim_issuer(d.ticker).pop().unwrap();
    }: _(d.owner.origin, d.ticker, issuer.clone())
    verify {
        let trusted_issuers = Module::<T>::trusted_claim_issuer(d.ticker);
        ensure!(
            trusted_issuers.contains(&issuer) == false,
            "Default trusted claim issuer was not removed");
    }

    change_compliance_requirement {
        let s in 1..T::MaxReceiverConditionsPerCompliance::get() as u32;
        let r in 1..T::MaxSenderConditionsPerCompliance::get() as u32;
        let i in 1..T::MaxTrustedIssuerPerCondition::get() as u32;

        // Add the compliance requirement.
        let d = ComplianceRequirementBuilder::<T>::new(i, s, r)
            .add_compliance_requirement().build();

        // Remove the latest one.
        let id = Module::<T>::get_latest_requirement_id(d.ticker);
        let new_req = ComplianceRequirement {
            id,
            ..Default::default()
        };
    }: _(d.owner.origin, d.ticker, new_req.clone())
    verify {
        let req = Module::<T>::asset_compliance( d.ticker)
            .requirements
            .into_iter()
            .find(|req| req.id == new_req.id)
            .unwrap();
        ensure!( req == new_req,
            "Compliance requirement was not updated");
    }

    replace_asset_compliance {
        let s in 1..T::MaxReceiverConditionsPerCompliance::get() as u32;
        let r in 1..T::MaxSenderConditionsPerCompliance::get() as u32;
        let i in 1..T::MaxTrustedIssuerPerCondition::get() as u32;

        // Add the compliance requirement.
        let d = ComplianceRequirementBuilder::<T>::new(i, s, r)
            .add_compliance_requirement().build();

        // Create new asset compiance
        let asset_compliance = vec![
            ComplianceRequirement {
                sender_conditions: make_conditions(s, &vec![]),
                receiver_conditions: make_conditions(r, &vec![]),
                id: Module::<T>::get_latest_requirement_id(d.ticker) + 1,
            }
        ];

    }: _(d.owner.origin, d.ticker, asset_compliance.clone())
    verify {
        let reqs = Module::<T>::asset_compliance(d.ticker).requirements;
        ensure!( reqs == asset_compliance, "Asset compliance was not replaced");
    }

    reset_asset_compliance {
        let s in 1..T::MaxReceiverConditionsPerCompliance::get() as u32;
        let r in 1..T::MaxSenderConditionsPerCompliance::get() as u32;
        let i in 1..T::MaxTrustedIssuerPerCondition::get() as u32;

        // Add the compliance requirement.
        let d = ComplianceRequirementBuilder::<T>::new(i, s, r)
            .add_compliance_requirement().build();
    }: _(d.owner.origin, d.ticker)
    verify {
        ensure!(
            Module::<T>::asset_compliance(d.ticker).requirements.is_empty(),
            "Compliance Requeriment was not reset");
    }
}
