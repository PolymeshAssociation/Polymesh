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

use frame_benchmarking::benchmarks;
use scale_info::prelude::format;
use sp_std::convert::TryFrom;

use pallet_asset::SecurityToken;
use polymesh_common_utilities::asset::AssetFnTrait;
use polymesh_common_utilities::benchs::{AccountIdOf, User, UserBuilder};
use polymesh_common_utilities::{identity::Config as IdentityConfig, TestUtilsFn};
use polymesh_primitives::agent::AgentGroup;
use polymesh_primitives::{
    asset::AssetType, AuthorizationData, ClaimType, CountryCode, PortfolioKind, Scope,
    TargetIdentity, TrustedFor, TrustedIssuer, WeightMeter,
};

use crate::*;

const MAX_DEFAULT_TRUSTED_CLAIM_ISSUERS: u32 = 3;
const MAX_TRUSTED_ISSUER_PER_CONDITION: u32 = 3;
const MAX_SENDER_CONDITIONS_PER_COMPLIANCE: u32 = 3;
const MAX_RECEIVER_CONDITIONS_PER_COMPLIANCE: u32 = 3;
const MAX_CONDITIONS_PER_COMPLIANCE: u32 =
    MAX_SENDER_CONDITIONS_PER_COMPLIANCE + MAX_RECEIVER_CONDITIONS_PER_COMPLIANCE;
const MAX_COMPLIANCE_REQUIREMENTS: u32 = 2;

const MAX_CONDITIONS: u32 = 10;
const MAX_CONDITION_TYPE_CLAIMS: u32 = 10;
const MAX_CONDITION_ISSUERS: u32 = 10;
const MAX_CONDITION_ISSUER_CLAIM_TYPES: u32 = 10;

const CLAIM_TYPES: &[ClaimType] = &[
    ClaimType::Accredited,
    ClaimType::Affiliate,
    ClaimType::BuyLockup,
    ClaimType::SellLockup,
    ClaimType::CustomerDueDiligence,
    ClaimType::KnowYourCustomer,
    ClaimType::Jurisdiction,
    ClaimType::Exempted,
    ClaimType::Blocked,
    ClaimType::InvestorUniqueness,
    ClaimType::NoType,
    ClaimType::InvestorUniquenessV2,
];

/// Create a token issuer trusted for `Any`.
pub fn make_issuer<T: IdentityConfig + TestUtilsFn<AccountIdOf<T>>>(
    id: u32,
    claim_type_len: Option<usize>,
) -> TrustedIssuer {
    let u = UserBuilder::<T>::default()
        .generate_did()
        .seed(id)
        .build("ISSUER");
    TrustedIssuer {
        issuer: IdentityId::from(u.did.unwrap()),
        trusted_for: match claim_type_len {
            None => TrustedFor::Any,
            Some(len) => TrustedFor::Specific(
                (0..len)
                    .into_iter()
                    .map(|idx| CLAIM_TYPES[idx % CLAIM_TYPES.len()])
                    .collect(),
            ),
        },
    }
}

/// Helper function to create `s` token issuers with `fn make_issuer`.
/// # TODO
///   - It could have more complexity if `TrustedIssuer::trusted_for` is a vector but not on
///   benchmarking of add/remove. That could be useful for benchmarking executions/evaluation of
///   complience requiriments.
pub fn make_issuers<T: IdentityConfig + TestUtilsFn<AccountIdOf<T>>>(
    s: u32,
    claim_type_len: Option<usize>,
) -> Vec<TrustedIssuer> {
    (0..s)
        .map(|i| make_issuer::<T>(i, claim_type_len))
        .collect()
}

/// Create simple conditions with a variable number of `issuers`.
pub fn make_conditions(s: u32, claims: Option<usize>, issuers: &[TrustedIssuer]) -> Vec<Condition> {
    (0..s)
        .map(|_| Condition {
            condition_type: match claims {
                None => ConditionType::IsPresent(Claim::NoData),
                Some(len) => ConditionType::IsAnyOf(
                    (0..len)
                        .into_iter()
                        .map(|_| Claim::Blocked(Scope::Custom(vec![0])))
                        .collect(),
                ),
            },
            issuers: issuers.to_vec(),
        })
        .collect()
}

fn split_conditions(count: u32) -> (u32, u32) {
    // split the number of conditions between the sender and receiver.
    let s = (count / 2).min(MAX_SENDER_CONDITIONS_PER_COMPLIANCE);
    let r = count
        .saturating_sub(s)
        .min(MAX_RECEIVER_CONDITIONS_PER_COMPLIANCE);
    (s, r)
}

/// Create a new token with name `name` on behalf of `owner`.
/// The new token is a _divisible_ one with 1_000_000 units.
pub fn make_token<T: Config>(owner: &User<T>, name: Vec<u8>) -> Ticker {
    let token = SecurityToken {
        owner_did: owner.did.clone().unwrap(),
        total_supply: 1_000_000,
        divisible: true,
        asset_type: AssetType::default(),
        ..Default::default()
    };
    let ticker = Ticker::from_slice_truncated(&*name);

    T::Asset::create_asset(
        owner.origin.clone().into(),
        name.into(),
        ticker,
        true,
        token.asset_type.clone(),
        vec![],
        None,
        true,
    )
    .expect("Cannot create an asset");

    T::Asset::issue(
        owner.origin.clone().into(),
        ticker,
        u128::try_from(token.total_supply).unwrap().into(),
        PortfolioKind::Default,
    )
    .expect("Cannot mint for asset");

    ticker
}

/// This struct helps to simplify the parameter copy/pass during the benchmarks.
struct ComplianceRequirementInfo<T: Config> {
    pub owner: User<T>,
    pub ticker: Ticker,
    pub sender_conditions: Vec<Condition>,
    pub receiver_conditions: Vec<Condition>,
}

impl<T: Config + TestUtilsFn<AccountIdOf<T>>> ComplianceRequirementInfo<T> {
    pub fn add_default_trusted_claim_issuer(self: &Self, i: u32) {
        make_issuers::<T>(i, None).into_iter().for_each(|issuer| {
            Module::<T>::add_default_trusted_claim_issuer(
                self.owner.origin.clone().into(),
                self.ticker.clone(),
                issuer,
            )
            .expect("Default trusted claim issuer cannot be added");
        });
    }
}

struct ComplianceRequirementBuilder<T: Config> {
    info: ComplianceRequirementInfo<T>,
    has_been_added: bool,
}

impl<T: Config + TestUtilsFn<AccountIdOf<T>>> ComplianceRequirementBuilder<T> {
    pub fn new(trusted_issuer_count: u32, conditions_count: u32) -> Self {
        // split the number of conditions between the sender and receiver.
        let (sender_count, receiver_count) = split_conditions(conditions_count);
        // Create accounts and token.
        let owner = UserBuilder::<T>::default().generate_did().build("OWNER");
        let ticker = make_token::<T>(&owner, b"1".to_vec());

        // Create issuers (i) and conditions(s & r).
        let issuers = make_issuers::<T>(trusted_issuer_count, None);
        let sender_conditions = make_conditions(sender_count, None, &issuers);
        let receiver_conditions = make_conditions(receiver_count, None, &issuers);

        let info = ComplianceRequirementInfo {
            owner,
            ticker,
            sender_conditions,
            receiver_conditions,
        };

        Self {
            info,
            has_been_added: false,
        }
    }
}

impl<T: Config> ComplianceRequirementBuilder<T> {
    /// Register the compliance requirement in the module.
    pub fn add_compliance_requirement(mut self: Self) -> Self {
        assert!(!self.has_been_added, "Compliance has been added before");
        Module::<T>::add_compliance_requirement(
            self.info.owner.origin.clone().into(),
            self.info.ticker.clone(),
            self.info.sender_conditions.clone(),
            self.info.receiver_conditions.clone(),
        )
        .expect("Compliance requirement cannot be added");
        self.has_been_added = true;
        self
    }

    pub fn build(self: Self) -> ComplianceRequirementInfo<T> {
        self.info
    }
}

fn setup_conditions_bench<T: Config + TestUtilsFn<AccountIdOf<T>>>(
    conditions: u32,
    claims: u32,
    issuers: u32,
    claim_types: u32,
) -> Vec<Condition> {
    let issuers = make_issuers::<T>(issuers, Some(claim_types as usize));
    let conditions = make_conditions(conditions, Some(claims as usize), &issuers);
    conditions
}

fn conditions_bench(conditions: Vec<Condition>) {
    let encoded = conditions.encode();
    let decoded = Vec::<Condition>::decode(&mut encoded.as_slice())
        .expect("This shouldn't fail since we just encoded a `Vec<Condition>` value.");
    if !conditions.eq(&decoded) {
        panic!("This shouldn't fail.");
    }
}

/// Adds `claim` issued by `trusted_issuer_id` to `id`.
fn add_identity_claim<T: Config>(id: IdentityId, claim: Claim, trusted_issuer_id: IdentityId) {
    pallet_identity::Module::<T>::unverified_add_claim_with_scope(
        id,
        claim.clone(),
        claim.as_scope().cloned(),
        trusted_issuer_id,
        None,
    );
}

/// Adds `external_agent_id` as an enternal agent for `ticker`.
fn add_external_agent<T>(
    ticker: Ticker,
    ticker_owner: IdentityId,
    external_agent_id: IdentityId,
    external_agent_origin: T::RuntimeOrigin,
) where
    T: Config,
{
    let auth_id = pallet_identity::Module::<T>::add_auth(
        ticker_owner,
        external_agent_id.into(),
        AuthorizationData::BecomeAgent(ticker, AgentGroup::Full),
        None,
    );
    pallet_external_agents::Module::<T>::accept_become_agent(external_agent_origin, auth_id)
        .unwrap();
}

fn setup_is_condition_satisfied<T>(
    sender: &User<T>,
    ticker: Ticker,
    n_claims: u32,
    n_issuers: u32,
    read_trusted_issuers_storage: bool,
) -> Condition
where
    T: Config + TestUtilsFn<AccountIdOf<T>>,
{
    let claims: Vec<Claim> = (0..n_claims)
        .map(|i| Claim::Jurisdiction(CountryCode::BR, Scope::Custom(vec![i as u8])))
        .collect();

    let trusted_issuers: Vec<TrustedIssuer> = (0..n_issuers)
        .map(|i| {
            let trusted_user = UserBuilder::<T>::default()
                .generate_did()
                .build(&format!("TrustedIssuer{}", i));
            TrustedIssuer::from(trusted_user.did())
        })
        .collect();

    // Adds claims for the identity
    (0..n_issuers).for_each(|i| {
        let claim: Claim = Claim::Jurisdiction(CountryCode::US, Scope::Custom(vec![i as u8]));
        add_identity_claim::<T>(sender.did(), claim, trusted_issuers[i as usize].issuer);
    });

    if read_trusted_issuers_storage {
        // Adds all trusted issuers as the default for the ticker
        trusted_issuers.into_iter().for_each(|trusted_issuer| {
            Module::<T>::base_add_default_trusted_claim_issuer(
                sender.did(),
                ticker,
                trusted_issuer,
            )
            .unwrap();
        });
        let condition = Condition::new(ConditionType::IsNoneOf(claims), Vec::new());
        return condition;
    }

    Condition::new(ConditionType::IsNoneOf(claims), trusted_issuers)
}

/// Adds `n` requirements for `ticker` and pauses compliance if `pause_compliance` is true.
pub fn setup_ticker_compliance<T: Config>(
    caller_did: IdentityId,
    ticker: Ticker,
    n: u32,
    pause_compliance: bool,
) {
    (0..n).for_each(|i| {
        let trusted_issuers = vec![TrustedIssuer::from(IdentityId::from(i as u128))];
        let claims = vec![Claim::Jurisdiction(
            CountryCode::BR,
            Scope::Custom(vec![i as u8]),
        )];
        let sender_conditions = vec![Condition::new(
            ConditionType::IsNoneOf(claims),
            trusted_issuers,
        )];
        Module::<T>::base_add_compliance_requirement(
            caller_did,
            ticker,
            sender_conditions,
            Vec::new(),
        )
        .unwrap();
    });

    if pause_compliance {
        AssetCompliances::mutate(&ticker, |compliance| compliance.paused = true);
    }
}

benchmarks! {
    where_clause { where T: TestUtilsFn<AccountIdOf<T>> }

    condition_costs {
        let a in 1..MAX_CONDITIONS;
        let b in 1..MAX_CONDITION_TYPE_CLAIMS;
        let c in 1..MAX_CONDITION_ISSUERS;
        let d in 1..MAX_CONDITION_ISSUER_CLAIM_TYPES;

        let conditions = setup_conditions_bench::<T>(a, b, c, d);
    }: {
        conditions_bench(conditions);
    }

    add_compliance_requirement {
        // INTERNAL: This benchmark only evaluate the adding operation. Its execution should be measured in another module.
        let c in 1..MAX_CONDITIONS_PER_COMPLIANCE;

        let d = ComplianceRequirementBuilder::<T>::new(MAX_TRUSTED_ISSUER_PER_CONDITION, c).build();

    }: _(d.owner.origin, d.ticker, d.sender_conditions.clone(), d.receiver_conditions.clone())
    verify {
        let req = Module::<T>::asset_compliance(d.ticker).requirements.pop().unwrap();
        assert_eq!( req.sender_conditions, d.sender_conditions, "Sender conditions not expected");
        assert_eq!( req.receiver_conditions, d.receiver_conditions, "Sender conditions not expected");
    }

    remove_compliance_requirement {
        // Add the compliance requirement.
        let d = ComplianceRequirementBuilder::<T>::new(
            MAX_TRUSTED_ISSUER_PER_CONDITION,
            MAX_CONDITIONS_PER_COMPLIANCE)
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

        assert!( is_removed, "Compliance requirement was not removed");
    }

    pause_asset_compliance {
        let d = ComplianceRequirementBuilder::<T>::new(
            MAX_TRUSTED_ISSUER_PER_CONDITION,
            MAX_CONDITIONS_PER_COMPLIANCE)
            .add_compliance_requirement().build();
    }: _(d.owner.origin, d.ticker)
    verify {
        assert!( Module::<T>::asset_compliance(d.ticker).paused, "Asset compliance is not paused");
    }

    resume_asset_compliance {
        let d = ComplianceRequirementBuilder::<T>::new(2, 2)
            .add_compliance_requirement().build();

        Module::<T>::pause_asset_compliance(
            d.owner.origin.clone().into(),
            d.ticker.clone()).unwrap();
    }: _(d.owner.origin, d.ticker)
    verify {
        assert!( !Module::<T>::asset_compliance(d.ticker).paused, "Asset compliance is paused");
    }

    add_default_trusted_claim_issuer {
        // Create and add the compliance requirement.
        let d = ComplianceRequirementBuilder::<T>::new(1, 1)
            .add_compliance_requirement()
            .build();
        d.add_default_trusted_claim_issuer(MAX_DEFAULT_TRUSTED_CLAIM_ISSUERS -1);

        // Add one more for benchmarking.
        let new_issuer = make_issuer::<T>(MAX_DEFAULT_TRUSTED_CLAIM_ISSUERS, None);
    }: _(d.owner.origin, d.ticker, new_issuer.clone())
    verify {
        let trusted_issuers = Module::<T>::trusted_claim_issuer(d.ticker);
        assert!(
            trusted_issuers.contains(&new_issuer),
            "Default trusted claim issuer was not added");
    }

    remove_default_trusted_claim_issuer {
        // Create and add the compliance requirement.
        let d = ComplianceRequirementBuilder::<T>::new(2, 1)
            .add_compliance_requirement().build();

        // Generate some trusted issuer.
        d.add_default_trusted_claim_issuer(MAX_DEFAULT_TRUSTED_CLAIM_ISSUERS);

        // Delete the latest trusted issuer.
        let issuer = Module::<T>::trusted_claim_issuer(d.ticker).pop().unwrap();
    }: _(d.owner.origin, d.ticker, issuer.issuer.clone())
    verify {
        let trusted_issuers = Module::<T>::trusted_claim_issuer(d.ticker);
        assert!(
            !trusted_issuers.contains(&issuer),
            "Default trusted claim issuer was not removed"
        );
    }

    change_compliance_requirement {
        let c in 1..MAX_CONDITIONS_PER_COMPLIANCE;

        // Add maximum size compliance requirements.
        let d = ComplianceRequirementBuilder::<T>::new(
            MAX_TRUSTED_ISSUER_PER_CONDITION,
            MAX_CONDITIONS_PER_COMPLIANCE)
            .add_compliance_requirement().build();

        // Change the latest one.
        let id = Module::<T>::get_latest_requirement_id(d.ticker);

        // Build a new set of compliance requirements.
        let (sender_count, receiver_count) = split_conditions(c);
        let issuers = make_issuers::<T>(MAX_TRUSTED_ISSUER_PER_CONDITION, None);
        let new_req = ComplianceRequirement {
            id,
            sender_conditions: make_conditions(sender_count, None, &issuers),
            receiver_conditions: make_conditions(receiver_count, None, &issuers),
        };
    }: _(d.owner.origin, d.ticker, new_req.clone())
    verify {
        let req = Module::<T>::asset_compliance(d.ticker)
            .requirements
            .into_iter()
            .find(|req| req.id == new_req.id)
            .unwrap();
        assert_eq!( req, new_req,
            "Compliance requirement was not updated");
    }

    replace_asset_compliance {
        let c in 0..MAX_COMPLIANCE_REQUIREMENTS;

        // Always add at least one compliance requirement.
        let d = ComplianceRequirementBuilder::<T>::new(
            MAX_TRUSTED_ISSUER_PER_CONDITION,
            MAX_CONDITIONS_PER_COMPLIANCE)
            .add_compliance_requirement().build();

        let issuers = make_issuers::<T>(MAX_TRUSTED_ISSUER_PER_CONDITION, None);
        let sender_conditions = make_conditions(MAX_SENDER_CONDITIONS_PER_COMPLIANCE, None, &issuers);
        let receiver_conditions = make_conditions(MAX_RECEIVER_CONDITIONS_PER_COMPLIANCE, None, &issuers);

        // Add more requirements to the asset, if `c > 1`.
        (1..c).for_each( |_i| {
            let _ = Module::<T>::add_compliance_requirement(
                d.owner.origin.clone().into(),
                d.ticker.clone(),
                sender_conditions.clone(),
                receiver_conditions.clone()).unwrap();
        });

        // Create the replacement asset compiance.
        let asset_compliance = (0..c).map(|id| {
            ComplianceRequirement {
                sender_conditions: sender_conditions.clone(),
                receiver_conditions: receiver_conditions.clone(),
                id,
            }}).collect::<Vec<_>>();
    }: _(d.owner.origin, d.ticker, asset_compliance.clone())
    verify {
        let reqs = Module::<T>::asset_compliance(d.ticker).requirements;
        assert_eq!( reqs, asset_compliance, "Asset compliance was not replaced");
    }

    reset_asset_compliance {
        // Add the compliance requirement.
        let d = ComplianceRequirementBuilder::<T>::new(
            MAX_TRUSTED_ISSUER_PER_CONDITION,
            MAX_CONDITIONS_PER_COMPLIANCE)
            .add_compliance_requirement().build();
    }: _(d.owner.origin, d.ticker)
    verify {
        assert!(
            Module::<T>::asset_compliance(d.ticker).requirements.is_empty(),
            "Compliance Requeriment was not reset");
    }
    is_condition_satisfied {
        // Number of claims * issuers
        let c in 1..400;
        // If `TrustedClaimIssuer` should be read
        let t in 0..1;

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let mut weight_meter = WeightMeter::max_limit_no_minimum();

        make_token::<T>(&alice, ticker.as_ref().to_vec());
        let condition = setup_is_condition_satisfied::<T>(&alice, ticker, 1, c, t == 1);
    }: {
        assert!(
            Module::<T>::is_condition_satisfied(
                &ticker,
                alice.did(),
                &condition,
                &mut None,
                &mut weight_meter
            )
            .unwrap()
        );
    }

    is_identity_condition {
        // If `ExternalAgents::<T>::agents` should be read
        let e in 0..1;

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let mut weight_meter = WeightMeter::max_limit_no_minimum();

        make_token::<T>(&alice, ticker.as_ref().to_vec());
        let condition = if e == 1 {
            add_external_agent::<T>(ticker, alice.did(), bob.did(), bob.origin().into());
            Condition::new(
                ConditionType::IsIdentity(TargetIdentity::ExternalAgent),
                Vec::new(),
            )
        } else {
            Condition::new(
                ConditionType::IsIdentity(TargetIdentity::Specific(alice.did())),
                Vec::new(),
            )
        };
    }: {
        assert!(
            Module::<T>::is_condition_satisfied(
                &ticker,
                alice.did(),
                &condition,
                &mut None,
                &mut weight_meter
            )
            .unwrap()
        );
    }

    is_any_requirement_compliant {
        let i in 0..10_000;

        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let mut weight_meter = WeightMeter::max_limit_no_minimum();

        let requirements: Vec<ComplianceRequirement> = (0..i)
            .map(|i| ComplianceRequirement {
                sender_conditions: vec![Condition {
                    condition_type: ConditionType::IsIdentity(TargetIdentity::Specific(bob.did())),
                    issuers: Vec::new(),
                }],
                receiver_conditions: Vec::new(),
                id: i as u32,
            })
            .collect();
    }: {
        // We want this to return false to make sure it loops through all requirements
        assert!(
            !Module::<T>::is_any_requirement_compliant(
                &ticker,
                &requirements,
                alice.did(),
                bob.did(),
                &mut weight_meter
            )
            .unwrap()
        );
    }
}
