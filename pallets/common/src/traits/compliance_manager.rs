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

use core::result::Result;
use frame_support::{dispatch::DispatchError, weights::Weight};
use polymesh_primitives::{
    compliance_manager::{AssetComplianceResult, ComplianceRequirement},
    condition::{conditions_total_counts, Condition},
    Balance, IdentityId, Ticker,
};

pub trait Config {
    fn verify_restriction(
        ticker: &Ticker,
        from_id: Option<IdentityId>,
        to_id: Option<IdentityId>,
        _value: Balance,
    ) -> Result<u8, DispatchError>;

    fn verify_restriction_granular(
        ticker: &Ticker,
        from_did_opt: Option<IdentityId>,
        to_did_opt: Option<IdentityId>,
    ) -> AssetComplianceResult;
}

pub trait WeightInfo {
    fn add_compliance_requirement(s: u32, r: u32) -> Weight;
    fn remove_compliance_requirement() -> Weight;
    fn pause_asset_compliance() -> Weight;
    fn resume_asset_compliance() -> Weight;
    fn add_default_trusted_claim_issuer() -> Weight;
    fn remove_default_trusted_claim_issuer() -> Weight;
    fn change_compliance_requirement(s: u32, r: u32) -> Weight;
    fn replace_asset_compliance(c: u32) -> Weight;
    fn reset_asset_compliance() -> Weight;

    fn condition_costs(conditions: u32, claims: u32, issuers: u32, claim_types: u32) -> Weight;

    fn add_compliance_requirement_full(sender: &[Condition], receiver: &[Condition]) -> Weight {
        let (_, claims, issuers, claim_types) =
            conditions_total_counts(sender.iter().chain(receiver.iter()));
        Self::add_compliance_requirement(sender.len() as u32, receiver.len() as u32)
            .saturating_add(Self::condition_costs(0, claims, issuers, claim_types))
    }

    fn change_compliance_requirement_full(req: &ComplianceRequirement) -> Weight {
        let (_, claims, issuers, claim_types) = req.counts();
        Self::change_compliance_requirement(
            req.sender_conditions.len() as u32,
            req.receiver_conditions.len() as u32,
        )
        .saturating_add(Self::condition_costs(0, claims, issuers, claim_types))
    }

    fn replace_asset_compliance_full(reqs: &[ComplianceRequirement]) -> Weight {
        let (conditions, claims, issuers, claim_types) =
            conditions_total_counts(reqs.iter().flat_map(|req| req.conditions()));
        Self::replace_asset_compliance(reqs.len() as u32).saturating_add(Self::condition_costs(
            conditions,
            claims,
            issuers,
            claim_types,
        ))
    }
}
