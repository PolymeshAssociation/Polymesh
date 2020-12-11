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

use super::*;
use crate::CAKind;
use crate::benchmarking::{setup_ca, currency, set_ca_targets, user, SEED};
use polymesh_common_utilities::benchs::User;
use frame_benchmarking::benchmarks;
use frame_support::assert_ok;
use pallet_timestamp::Module as Timestamp;
use pallet_compliance_manager::Module as ComplianceManager;
use pallet_portfolio::MovePortfolioItem;

const MAX_TARGETS: u32 = 100;

fn portfolio<T: Trait>(owner: &User<T>, pnum: PortfolioNumber, ticker: Ticker, amount: T::Balance) {
    let did = owner.did();
    let origin: T::Origin = owner.origin().into();
    assert_ok!(<Portfolio<T>>::create_portfolio(origin.clone(), "portfolio".into()));
    assert_ok!(<Portfolio<T>>::move_portfolio_funds(
        origin,
        PortfolioId::default_portfolio(did),
        PortfolioId::user_portfolio(did, pnum),
        vec![MovePortfolioItem { ticker, amount }],
    ));
}

fn dist<T: Trait>(k: u32) -> (User<T>, CAId, Ticker) {
    let (owner, ca_id) = setup_ca::<T>(CAKind::UnpredictableBenefit);

    let currency = currency::<T>(&owner);
    let amount = 1000.into();
    let pnum = 1.into();
    portfolio::<T>(&owner, pnum, currency, amount);

    assert_ok!(<Module<T>>::distribute(
        owner.origin().into(),
        ca_id,
        Some(pnum),
        currency,
        amount,
        3000,
        Some(4000),
    ));

    set_ca_targets::<T>(ca_id, k);

    (owner, ca_id, currency)
}

fn add_investor_uniqueness_claim<T: Trait>(user: &User<T>, scope: Ticker) {
    use frame_system::Origin;
    use polymesh_primitives::{Claim, Scope, InvestorZKProofData};
    use cryptography::claim_proofs::{compute_cdd_id, compute_scope_id};

    let claim_to = user.did();
    let investor_uid = user.uid();
    let proof: InvestorZKProofData = InvestorZKProofData::new(&claim_to, &investor_uid, &scope);
    let cdd_claim = InvestorZKProofData::make_cdd_claim(&claim_to, &investor_uid);
    let cdd_id = compute_cdd_id(&cdd_claim).compress().to_bytes().into();
    let scope_claim = InvestorZKProofData::make_scope_claim(&scope.as_slice(), &investor_uid);
    let scope_id = compute_scope_id(&scope_claim).compress().to_bytes().into();

    let signed_claim_to = <Origin<T>>::Signed(<Identity<T>>::did_records(claim_to).primary_key);

    assert_ok!(<Identity<T>>::add_investor_uniqueness_claim(
        signed_claim_to.into(),
        claim_to,
        Claim::InvestorUniqueness(Scope::Ticker(scope), scope_id, cdd_id),
        proof,
        None
    ));
}

fn prepare_transfer<T: Trait + pallet_compliance_manager::Trait>(k: u32) -> (User<T>, User<T>, CAId) {
    let (owner, ca_id, currency) = dist::<T>(k);
    <Timestamp<T>>::set_timestamp(3000.into());

    let holder = user::<T>("holder", SEED);
    add_investor_uniqueness_claim(&owner, currency);
    add_investor_uniqueness_claim(&holder, currency);
    assert_ok!(<ComplianceManager<T>>::add_compliance_requirement(
        owner.origin().into(),
        currency,
        vec![],
        vec![]
    ));

    (owner, holder, ca_id)
}

benchmarks! {
    where_clause { where T: pallet_compliance_manager::Trait }

    _ {}

    distribute {
        let (owner, ca_id) = setup_ca::<T>(CAKind::UnpredictableBenefit);
        let currency = currency::<T>(&owner);
        let amount = 1000.into();
        let pnum = 1.into();
        portfolio::<T>(&owner, pnum, currency, amount);
    }: _(owner.origin(), ca_id, Some(pnum), currency, amount, 3000, Some(4000))
    verify {
        ensure!(<Distributions<T>>::get(ca_id).is_some(), "distribution not created");
    }

    claim {
        let k in 0..MAX_TARGETS;

        let (_, holder, ca_id) = prepare_transfer::<T>(k);
    }: _(holder.origin(), ca_id)
    verify {
        ensure!(HolderPaid::get((ca_id, holder.did())), "not paid");
    }

    push_benefit {
        let k in 0..MAX_TARGETS;

        let (owner, holder, ca_id) = prepare_transfer::<T>(k);
    }: _(owner.origin(), ca_id, holder.did())
    verify {
        ensure!(HolderPaid::get((ca_id, holder.did())), "not paid");
    }

    reclaim {
        let (owner, ca_id, currency) = dist::<T>(0);
        <Timestamp<T>>::set_timestamp(5000.into());
    }: _(owner.origin(), ca_id)
    verify {
        ensure!(<Distributions<T>>::get(ca_id).unwrap().reclaimed, "not reclaimed");
    }

    remove_distribution {
        let (owner, ca_id, currency) = dist::<T>(0);
    }: _(owner.origin(), ca_id)
    verify {
        ensure!(<Distributions<T>>::get(ca_id).is_none(), "not removed");
    }
}
