// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymesh Association

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
use frame_support::decl_event;
use frame_support::dispatch::DispatchError;
use frame_support::traits::Get;
use frame_support::weights::Weight;
use sp_std::prelude::*;

use polymesh_primitives::compliance_manager::{AssetComplianceResult, ComplianceRequirement};
use polymesh_primitives::condition::{conditions_total_counts, Condition};
use polymesh_primitives::{IdentityId, Ticker, TrustedIssuer, WeightMeter};

use crate::asset::AssetFnTrait;
use crate::balances::Config as BalancesConfig;
use crate::identity::Config as IdentityConfig;
use crate::traits::external_agents::Config as EAConfig;

/// The module's configuration trait.
pub trait Config:
    pallet_timestamp::Config + frame_system::Config + BalancesConfig + IdentityConfig + EAConfig
{
    /// The overarching event type.
    type RuntimeEvent: From<Event> + Into<<Self as frame_system::Config>::RuntimeEvent>;

    /// Asset module
    type Asset: AssetFnTrait<Self::AccountId, Self::RuntimeOrigin>;

    /// Weight details of all extrinsic
    type WeightInfo: WeightInfo;

    /// The maximum claim reads that are allowed to happen in worst case of a condition resolution
    type MaxConditionComplexity: Get<u32>;
}

decl_event!(
    pub enum Event {
        /// Emitted when new compliance requirement is created.
        /// (caller DID, Ticker, ComplianceRequirement).
        ComplianceRequirementCreated(IdentityId, Ticker, ComplianceRequirement),
        /// Emitted when a compliance requirement is removed.
        /// (caller DID, Ticker, requirement_id).
        ComplianceRequirementRemoved(IdentityId, Ticker, u32),
        /// Emitted when an asset compliance is replaced.
        /// Parameters: caller DID, ticker, new asset compliance.
        AssetComplianceReplaced(IdentityId, Ticker, Vec<ComplianceRequirement>),
        /// Emitted when an asset compliance of a ticker is reset.
        /// (caller DID, Ticker).
        AssetComplianceReset(IdentityId, Ticker),
        /// Emitted when an asset compliance for a given ticker gets resume.
        /// (caller DID, Ticker).
        AssetComplianceResumed(IdentityId, Ticker),
        /// Emitted when an asset compliance for a given ticker gets paused.
        /// (caller DID, Ticker).
        AssetCompliancePaused(IdentityId, Ticker),
        /// Emitted when compliance requirement get modified/change.
        /// (caller DID, Ticker, ComplianceRequirement).
        ComplianceRequirementChanged(IdentityId, Ticker, ComplianceRequirement),
        /// Emitted when default claim issuer list for a given ticker gets added.
        /// (caller DID, Ticker, Added TrustedIssuer).
        TrustedDefaultClaimIssuerAdded(IdentityId, Ticker, TrustedIssuer),
        /// Emitted when default claim issuer list for a given ticker get removed.
        /// (caller DID, Ticker, Removed TrustedIssuer).
        TrustedDefaultClaimIssuerRemoved(IdentityId, Ticker, IdentityId),
    }
);

pub trait ComplianceFnConfig {
    /// Returns `false` if there are no requirements for the asset or if all of the
    /// asset's requirement don't hold, otherwise returns`true`.
    fn is_compliant(
        ticker: &Ticker,
        sender_did: IdentityId,
        receiver_did: IdentityId,
        weight_meter: &mut WeightMeter,
    ) -> Result<bool, DispatchError>;

    fn verify_restriction_granular(
        ticker: &Ticker,
        from_did_opt: Option<IdentityId>,
        to_did_opt: Option<IdentityId>,
        weight_meter: &mut WeightMeter,
    ) -> Result<AssetComplianceResult, DispatchError>;

    #[cfg(feature = "runtime-benchmarks")]
    fn setup_ticker_compliance(
        caler_did: IdentityId,
        ticker: Ticker,
        n: u32,
        pause_compliance: bool,
    );
}

pub trait WeightInfo {
    fn add_compliance_requirement(c: u32) -> Weight;
    fn remove_compliance_requirement() -> Weight;
    fn pause_asset_compliance() -> Weight;
    fn resume_asset_compliance() -> Weight;
    fn add_default_trusted_claim_issuer() -> Weight;
    fn remove_default_trusted_claim_issuer() -> Weight;
    fn change_compliance_requirement(c: u32) -> Weight;
    fn replace_asset_compliance(c: u32) -> Weight;
    fn reset_asset_compliance() -> Weight;
    fn is_condition_satisfied(c: u32, t: u32) -> Weight;
    fn is_identity_condition(e: u32) -> Weight;
    fn is_any_requirement_compliant(i: u32) -> Weight;

    fn condition_costs(conditions: u32, claims: u32, issuers: u32, claim_types: u32) -> Weight;

    fn add_compliance_requirement_full(sender: &[Condition], receiver: &[Condition]) -> Weight {
        let (condtions, claims, issuers, claim_types) =
            conditions_total_counts(sender.iter().chain(receiver.iter()));
        Self::add_compliance_requirement(condtions).saturating_add(Self::condition_costs(
            0,
            claims,
            issuers,
            claim_types,
        ))
    }

    fn change_compliance_requirement_full(req: &ComplianceRequirement) -> Weight {
        let (conditions, claims, issuers, claim_types) = req.counts();
        Self::change_compliance_requirement(conditions).saturating_add(Self::condition_costs(
            0,
            claims,
            issuers,
            claim_types,
        ))
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

    fn is_any_requirement_compliant_loop(i: u32) -> Weight {
        Self::is_any_requirement_compliant(i)
            .saturating_sub(Self::is_identity_condition(0).saturating_mul(i.into()))
    }
}
