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

use frame_benchmarking::benchmarks;
use scale_info::prelude::format;

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
                None => ConditionType::IsPresent(Claim::Blocked(Scope::Custom(vec![0]))),
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

/// Creates a new [`SecurityToken`] and issues 1_000_000 tokens for that token.
pub fn create_and_issue_sample_asset<T: Config>(
    asset_owner: &User<T>,
    asset_name: Vec<u8>,
) -> AssetID {
    let asset_id = T::Asset::generate_asset_id(asset_owner.account());
    T::Asset::create_asset(
        asset_owner.origin.clone().into(),
        asset_name.into(),
        true,
        AssetType::default(),
        vec![],
        None,
    )
    .unwrap();

    T::Asset::issue(
        asset_owner.origin.clone().into(),
        asset_id,
        1_000_000 as u128,
        PortfolioKind::Default,
    )
    .unwrap();

    asset_id
}

/// This struct helps to simplify the parameter copy/pass during the benchmarks.
struct ComplianceRequirementInfo<T: Config> {
    pub owner: User<T>,
    pub asset_id: AssetID,
    pub sender_conditions: Vec<Condition>,
    pub receiver_conditions: Vec<Condition>,
}

impl<T: Config + TestUtilsFn<AccountIdOf<T>>> ComplianceRequirementInfo<T> {
    pub fn add_default_trusted_claim_issuer(self: &Self, i: u32) {
        make_issuers::<T>(i, None).into_iter().for_each(|issuer| {
            Module::<T>::add_default_trusted_claim_issuer(
                self.owner.origin.clone().into(),
                self.asset_id,
                issuer,
            )
            .unwrap();
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
        let asset_id = create_and_issue_sample_asset::<T>(&owner, b"1".to_vec());

        // Create issuers (i) and conditions(s & r).
        let issuers = make_issuers::<T>(trusted_issuer_count, None);
        let sender_conditions = make_conditions(sender_count, None, &issuers);
        let receiver_conditions = make_conditions(receiver_count, None, &issuers);

        let info = ComplianceRequirementInfo {
            owner,
            asset_id,
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
            self.info.asset_id.clone(),
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
    asset_id: AssetID,
    ticker_owner: IdentityId,
    external_agent_id: IdentityId,
    external_agent_origin: T::RuntimeOrigin,
) where
    T: Config,
{
    let auth_id = pallet_identity::Module::<T>::add_auth(
        ticker_owner,
        external_agent_id.into(),
        AuthorizationData::BecomeAgent(asset_id, AgentGroup::Full),
        None,
    )
    .unwrap();
    pallet_external_agents::Module::<T>::accept_become_agent(external_agent_origin, auth_id)
        .unwrap();
}

fn setup_is_condition_satisfied<T>(
    sender: &User<T>,
    asset_id: AssetID,
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
        // Adds all trusted issuers as the default for the asset_id
        trusted_issuers.into_iter().for_each(|trusted_issuer| {
            Module::<T>::base_add_default_trusted_claim_issuer(
                sender.did(),
                asset_id,
                trusted_issuer,
            )
            .unwrap();
        });
        let condition = Condition::new(ConditionType::IsNoneOf(claims), Vec::new());
        return condition;
    }

    Condition::new(ConditionType::IsNoneOf(claims), trusted_issuers)
}

/// Adds `n` requirements for `asset_id` and pauses compliance if `pause_compliance` is true.
pub fn setup_asset_compliance<T: Config>(
    caller_did: IdentityId,
    asset_id: AssetID,
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
            asset_id,
            sender_conditions,
            Vec::new(),
        )
        .unwrap();
    });

    if pause_compliance {
        AssetCompliances::mutate(&asset_id, |compliance| compliance.paused = true);
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

    }: _(d.owner.origin, d.asset_id, d.sender_conditions.clone(), d.receiver_conditions.clone())
    verify {
        let req = Module::<T>::asset_compliance(d.asset_id).requirements.pop().unwrap();
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
        let id = Module::<T>::get_latest_requirement_id(d.asset_id);
    }: _(d.owner.origin, d.asset_id, id)
    verify {
        let is_removed = Module::<T>::asset_compliance(d.asset_id)
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
    }: _(d.owner.origin, d.asset_id)
    verify {
        assert!( Module::<T>::asset_compliance(d.asset_id).paused, "Asset compliance is not paused");
    }

    resume_asset_compliance {
        let d = ComplianceRequirementBuilder::<T>::new(2, 2)
            .add_compliance_requirement().build();

        Module::<T>::pause_asset_compliance(
            d.owner.origin.clone().into(),
            d.asset_id.clone()).unwrap();
    }: _(d.owner.origin, d.asset_id)
    verify {
        assert!( !Module::<T>::asset_compliance(d.asset_id).paused, "Asset compliance is paused");
    }

    add_default_trusted_claim_issuer {
        // Create and add the compliance requirement.
        let d = ComplianceRequirementBuilder::<T>::new(1, 1)
            .add_compliance_requirement()
            .build();
        d.add_default_trusted_claim_issuer(MAX_DEFAULT_TRUSTED_CLAIM_ISSUERS -1);

        // Add one more for benchmarking.
        let new_issuer = make_issuer::<T>(MAX_DEFAULT_TRUSTED_CLAIM_ISSUERS, None);
    }: _(d.owner.origin, d.asset_id, new_issuer.clone())
    verify {
        let trusted_issuers = Module::<T>::trusted_claim_issuer(d.asset_id);
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
        let issuer = Module::<T>::trusted_claim_issuer(d.asset_id).pop().unwrap();
    }: _(d.owner.origin, d.asset_id, issuer.issuer.clone())
    verify {
        let trusted_issuers = Module::<T>::trusted_claim_issuer(d.asset_id);
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
        let id = Module::<T>::get_latest_requirement_id(d.asset_id);

        // Build a new set of compliance requirements.
        let (sender_count, receiver_count) = split_conditions(c);
        let issuers = make_issuers::<T>(MAX_TRUSTED_ISSUER_PER_CONDITION, None);
        let new_req = ComplianceRequirement {
            id,
            sender_conditions: make_conditions(sender_count, None, &issuers),
            receiver_conditions: make_conditions(receiver_count, None, &issuers),
        };
    }: _(d.owner.origin, d.asset_id, new_req.clone())
    verify {
        let req = Module::<T>::asset_compliance(d.asset_id)
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
                d.asset_id.clone(),
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
    }: _(d.owner.origin, d.asset_id, asset_compliance.clone())
    verify {
        let reqs = Module::<T>::asset_compliance(d.asset_id).requirements;
        assert_eq!( reqs, asset_compliance, "Asset compliance was not replaced");
    }

    reset_asset_compliance {
        // Add the compliance requirement.
        let d = ComplianceRequirementBuilder::<T>::new(
            MAX_TRUSTED_ISSUER_PER_CONDITION,
            MAX_CONDITIONS_PER_COMPLIANCE)
            .add_compliance_requirement().build();
    }: _(d.owner.origin, d.asset_id)
    verify {
        assert!(
            Module::<T>::asset_compliance(d.asset_id).requirements.is_empty(),
            "Compliance Requeriment was not reset");
    }

    is_condition_satisfied {
        // Number of claims * issuers
        let c in 1..400;
        // If `TrustedClaimIssuer` should be read
        let t in 0..1;

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");

        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        let asset_id = create_and_issue_sample_asset::<T>(&alice, b"MyAsset".to_vec());
        let condition = setup_is_condition_satisfied::<T>(&alice, asset_id, 1, c, t == 1);
    }: {
        assert!(
            Module::<T>::is_condition_satisfied(
                &asset_id,
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

        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        let asset_id = create_and_issue_sample_asset::<T>(&alice, b"MyAsset".to_vec());
        let condition = if e == 1 {
            add_external_agent::<T>(asset_id, alice.did(), bob.did(), bob.origin().into());
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
                &asset_id,
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
        let mut weight_meter = WeightMeter::max_limit_no_minimum();

        let asset_id = create_and_issue_sample_asset::<T>(&alice, b"MyAsset".to_vec());
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
                &asset_id,
                &requirements,
                alice.did(),
                bob.did(),
                &mut weight_meter
            )
            .unwrap()
        );
    }
}
