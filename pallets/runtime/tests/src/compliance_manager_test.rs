use chrono::prelude::Utc;
use frame_support::traits::Currency;
use frame_support::{assert_noop, assert_ok};
use sp_std::prelude::*;

use pallet_compliance_manager::Error as CMError;
use polymesh_common_utilities::compliance_manager::ComplianceFnConfig;
use polymesh_common_utilities::constants::ERC1400_TRANSFER_SUCCESS;
use polymesh_primitives::agent::AgentGroup;
use polymesh_primitives::compliance_manager::{
    AssetComplianceResult, ComplianceRequirement, ComplianceRequirementResult,
};
use polymesh_primitives::{
    AuthorizationData, Claim, ClaimType, Condition, ConditionType, CountryCode, IdentityId,
    PortfolioId, Scope, Signatory, TargetIdentity, Ticker, TrustedFor, WeightMeter,
};
use test_client::AccountKeyring;

use super::asset_test::{allow_all_transfers, create_token, set_timestamp};
use super::storage::{set_curr_did, TestStorage, User};
use super::ExtBuilder;

type Identity = pallet_identity::Module<TestStorage>;
type IdError = pallet_identity::Error<TestStorage>;
type Balances = pallet_balances::Module<TestStorage>;
type Timestamp = pallet_timestamp::Pallet<TestStorage>;
type Asset = pallet_asset::Module<TestStorage>;
type ComplianceManager = pallet_compliance_manager::Module<TestStorage>;
type CDDGroup = pallet_group::Module<TestStorage, pallet_group::Instance2>;
type Moment = u64;
type Origin = <TestStorage as frame_system::Config>::RuntimeOrigin;
type ExternalAgents = pallet_external_agents::Module<TestStorage>;
type EAError = pallet_external_agents::Error<TestStorage>;

macro_rules! assert_invalid_transfer {
    ($ticker:expr, $from:expr, $to:expr, $amount:expr) => {
        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        assert_ne!(
            Asset::_is_valid_transfer(
                &$ticker,
                PortfolioId::default_portfolio($from),
                PortfolioId::default_portfolio($to),
                $amount,
                &mut weight_meter
            ),
            Ok(ERC1400_TRANSFER_SUCCESS)
        );
    };
}

macro_rules! assert_valid_transfer {
    ($ticker:expr, $from:expr, $to:expr, $amount:expr) => {
        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        assert_eq!(
            Asset::_is_valid_transfer(
                &$ticker,
                PortfolioId::default_portfolio($from),
                PortfolioId::default_portfolio($to),
                $amount,
                &mut weight_meter
            ),
            Ok(ERC1400_TRANSFER_SUCCESS)
        );
    };
}

macro_rules! assert_add_claim {
    ($signer:expr, $target:expr, $claim:expr, $expiry:expr) => {
        assert_ok!(Identity::add_claim($signer, $target, $claim, $expiry));
    };
}

fn get_latest_requirement_id(ticker: Ticker) -> u32 {
    ComplianceManager::asset_compliance(ticker)
        .requirements
        .last()
        .map(|r| r.id)
        .unwrap_or(0)
}

#[test]
fn should_add_and_verify_compliance_requirement() {
    ExtBuilder::default()
        .build()
        .execute_with(should_add_and_verify_compliance_requirement_we);
}

fn should_add_and_verify_compliance_requirement_we() {
    // 0. Create accounts
    let root = Origin::from(frame_system::RawOrigin::Root);
    let owner = User::new(AccountKeyring::Alice);
    let token_rec = User::new(AccountKeyring::Charlie);
    let cdd = User::new(AccountKeyring::Eve);

    assert_ok!(CDDGroup::reset_members(root, vec![cdd.did]));
    // Create & mint token
    let (ticker, token) = create_token(owner);

    Balances::make_free_balance_be(&owner.acc(), 1_000_000);

    let claim_issuer = User::new(AccountKeyring::Bob);
    Balances::make_free_balance_be(&claim_issuer.acc(), 1_000_000);
    let ferdie = User::new(AccountKeyring::Ferdie);

    let claim = Claim::Blocked(ticker.into());
    assert_ok!(Identity::add_claim(
        claim_issuer.origin(),
        owner.did,
        claim.clone(),
        None,
    ));

    assert_ok!(Identity::add_claim(
        claim_issuer.origin(),
        token_rec.did,
        claim.clone(),
        None,
    ));

    let now = Utc::now();
    set_timestamp(now.timestamp() as u64);

    let sender_condition =
        Condition::from_dids(ConditionType::IsPresent(claim), &[claim_issuer.did]);

    let receiver_condition1 = Condition::from_dids(
        ConditionType::IsAbsent(Claim::KnowYourCustomer(owner.scope())),
        &[cdd.did],
    );

    let receiver_condition2 = Condition {
        condition_type: ConditionType::IsPresent(Claim::Accredited(owner.scope())),
        issuers: vec![
            claim_issuer.trusted_issuer_for(TrustedFor::Specific(vec![ClaimType::Accredited])),
            ferdie.trusted_issuer_for(TrustedFor::Specific(vec![
                ClaimType::Affiliate,
                ClaimType::BuyLockup,
            ])),
        ],
    };

    assert_ok!(ComplianceManager::add_compliance_requirement(
        owner.origin(),
        ticker,
        vec![sender_condition.clone()],
        vec![receiver_condition1.clone(), receiver_condition2.clone()]
    ));

    assert_ok!(Identity::add_claim(
        claim_issuer.origin(),
        token_rec.did,
        Claim::Accredited(claim_issuer.scope()),
        None,
    ));

    //Transfer tokens to investor - fails wrong Accredited scope
    assert_invalid_transfer!(ticker, owner.did, token_rec.did, token.total_supply);
    let get_result = || {
        ComplianceManager::verify_restriction_granular(
            &ticker,
            Some(owner.did),
            Some(token_rec.did),
            &mut WeightMeter::max_limit_no_minimum(),
        )
        .unwrap()
    };
    let second_unpassed = |result: AssetComplianceResult| {
        assert!(!result.result);
        assert!(!result.requirements[0].result);
        assert!(result.requirements[0].sender_conditions[0].result);
        assert!(result.requirements[0].receiver_conditions[0].result);
        assert!(!result.requirements[0].receiver_conditions[1].result);
        assert_eq!(
            result.requirements[0].sender_conditions[0].condition,
            sender_condition
        );
        assert_eq!(
            result.requirements[0].receiver_conditions[0].condition,
            receiver_condition1
        );
        assert_eq!(
            result.requirements[0].receiver_conditions[1].condition,
            receiver_condition2
        );
    };
    second_unpassed(get_result());

    // Ferdie isn't trusted for `Accredited`, so the claim isn't enough.
    assert_ok!(Identity::add_claim(
        ferdie.origin(),
        token_rec.did,
        Claim::Accredited(owner.scope()),
        None,
    ));
    assert_invalid_transfer!(ticker, owner.did, token_rec.did, 10);
    second_unpassed(get_result());

    // Now we add a claim from a trusted issuer, so the transfer will be valid.
    assert_ok!(Identity::add_claim(
        claim_issuer.origin(),
        token_rec.did,
        Claim::Accredited(owner.scope()),
        None,
    ));
    assert_valid_transfer!(ticker, owner.did, token_rec.did, 10);
    let result = get_result();
    assert!(result.result);
    assert!(result.requirements[0].result);
    assert!(result.requirements[0].sender_conditions[0].result);
    assert!(result.requirements[0].receiver_conditions[0].result);
    assert!(result.requirements[0].receiver_conditions[1].result);
    assert_eq!(
        result.requirements[0].sender_conditions[0].condition,
        sender_condition
    );
    assert_eq!(
        result.requirements[0].receiver_conditions[0].condition,
        receiver_condition1
    );
    assert_eq!(
        result.requirements[0].receiver_conditions[1].condition,
        receiver_condition2
    );

    assert_ok!(Identity::add_claim(
        cdd.origin(),
        token_rec.did,
        Claim::KnowYourCustomer(owner.scope()),
        None,
    ));

    assert_invalid_transfer!(ticker, owner.did, token_rec.did, 10);
    let result = get_result();
    assert!(!result.result);
    assert!(!result.requirements[0].result);
    assert!(result.requirements[0].sender_conditions[0].result);
    assert!(!result.requirements[0].receiver_conditions[0].result);
    assert!(result.requirements[0].receiver_conditions[1].result);
    assert_eq!(
        result.requirements[0].sender_conditions[0].condition,
        sender_condition
    );
    assert_eq!(
        result.requirements[0].receiver_conditions[0].condition,
        receiver_condition1
    );
    assert_eq!(
        result.requirements[0].receiver_conditions[1].condition,
        receiver_condition2
    );

    let comp_req = ComplianceRequirement {
        sender_conditions: vec![sender_condition.clone()],
        receiver_conditions: vec![receiver_condition1.clone(), receiver_condition2.clone()],
        id: 1,
    };

    // Add two more compliance requirements.
    for _ in 0..2 {
        assert_ok!(ComplianceManager::add_compliance_requirement(
            owner.origin(),
            ticker,
            comp_req.sender_conditions.clone(),
            comp_req.receiver_conditions.clone(),
        ));
    }
    assert_eq!(get_latest_requirement_id(ticker), 3);
    assert_ok!(ComplianceManager::remove_compliance_requirement(
        owner.origin(),
        ticker,
        1
    )); // OK; latest == 3
    assert_eq!(get_latest_requirement_id(ticker), 3);

    // Try changing the removed compliance requirement.
    assert_noop!(
        ComplianceManager::change_compliance_requirement(owner.origin(), ticker, comp_req),
        CMError::<TestStorage>::InvalidComplianceRequirementId
    ); // BAD OK; latest == 3, but 1 was just removed.

    assert_noop!(
        ComplianceManager::remove_compliance_requirement(owner.origin(), ticker, 1),
        CMError::<TestStorage>::InvalidComplianceRequirementId
    ); // BAD OK; latest == 3, but 1 was just removed.
    assert_noop!(
        ComplianceManager::remove_compliance_requirement(owner.origin(), ticker, 1),
        CMError::<TestStorage>::InvalidComplianceRequirementId
    );
    assert_eq!(get_latest_requirement_id(ticker), 3);
}

#[test]
fn should_replace_asset_compliance() {
    ExtBuilder::default()
        .build()
        .execute_with(should_replace_asset_compliance_we);
}

fn should_replace_asset_compliance_we() {
    let owner = User::new(AccountKeyring::Alice);

    // Create & mint token
    let (ticker, _) = create_token(owner);

    Balances::make_free_balance_be(&owner.acc(), 1_000_000);

    allow_all_transfers(ticker, owner);

    let asset_compliance = ComplianceManager::asset_compliance(ticker);
    assert_eq!(asset_compliance.requirements.len(), 1);

    // Create three requirements with different requirement IDs.
    let new_asset_compliance: Vec<ComplianceRequirement> =
        std::iter::repeat(|id: u32| ComplianceRequirement {
            sender_conditions: vec![],
            receiver_conditions: vec![],
            id,
        })
        .take(3)
        .enumerate()
        .map(|(n, f)| f(n as u32))
        .collect();

    assert_ok!(ComplianceManager::replace_asset_compliance(
        owner.origin(),
        ticker,
        new_asset_compliance.clone(),
    ));

    let asset_compliance = ComplianceManager::asset_compliance(ticker);
    assert_eq!(asset_compliance.requirements, new_asset_compliance);
}

#[test]
fn test_dedup_replace_asset_compliance() {
    ExtBuilder::default()
        .build()
        .execute_with(test_dedup_replace_asset_compliance_we);
}

fn test_dedup_replace_asset_compliance_we() {
    let owner = User::new(AccountKeyring::Alice);

    // Create & mint token
    let (ticker, _) = create_token(owner);

    Balances::make_free_balance_be(&owner.acc(), 1_000_000);

    allow_all_transfers(ticker, owner);

    let asset_compliance = ComplianceManager::asset_compliance(ticker);
    assert_eq!(asset_compliance.requirements.len(), 1);

    let make_req = |id: u32| ComplianceRequirement {
        sender_conditions: vec![],
        receiver_conditions: vec![],
        id,
    };

    // Replace should throw an error if there are duplicate requirement ids.
    assert_noop!(
        ComplianceManager::replace_asset_compliance(
            owner.origin(),
            ticker,
            vec![make_req(1), make_req(2), make_req(2),],
        ),
        CMError::<TestStorage>::DuplicateComplianceRequirements
    );

    // Test with mixed duplicate ids.  To test sorting.
    assert_noop!(
        ComplianceManager::replace_asset_compliance(
            owner.origin(),
            ticker,
            vec![make_req(2), make_req(1), make_req(2),],
        ),
        CMError::<TestStorage>::DuplicateComplianceRequirements
    );
}

#[test]
fn should_reset_asset_compliance() {
    ExtBuilder::default()
        .build()
        .execute_with(should_reset_asset_compliance_we);
}

fn should_reset_asset_compliance_we() {
    let owner = User::new(AccountKeyring::Alice);

    // Create & mint token
    let (ticker, _) = create_token(owner);

    Balances::make_free_balance_be(&owner.acc(), 1_000_000);

    allow_all_transfers(ticker, owner);

    let asset_compliance = ComplianceManager::asset_compliance(ticker);
    assert_eq!(asset_compliance.requirements.len(), 1);

    assert_ok!(ComplianceManager::reset_asset_compliance(
        owner.origin(),
        ticker
    ));

    let asset_compliance_new = ComplianceManager::asset_compliance(ticker);
    assert_eq!(asset_compliance_new.requirements.len(), 0);
}

#[test]
fn pause_resume_asset_compliance() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Eve.to_account_id()])
        .build()
        .execute_with(pause_resume_asset_compliance_we);
}

fn pause_resume_asset_compliance_we() {
    // 0. Create accounts
    let owner = User::new(AccountKeyring::Alice);
    let receiver = User::new(AccountKeyring::Charlie);

    // 1. Create & mint token
    let (ticker, _) = create_token(owner);

    Balances::make_free_balance_be(&owner.acc(), 1_000_000);

    let claim = Claim::Blocked(ticker.into());
    assert_ok!(Identity::add_claim(
        receiver.origin(),
        receiver.did.clone(),
        claim.clone(),
        Some(99999999999999999u64),
    ));

    let now = Utc::now();
    set_timestamp(now.timestamp() as u64);

    // 4. Define conditions
    let receiver_conditions = vec![Condition::from_dids(
        ConditionType::IsAbsent(claim.clone()),
        &[receiver.did],
    )];

    assert_ok!(ComplianceManager::add_compliance_requirement(
        owner.origin(),
        ticker,
        vec![],
        receiver_conditions
    ));

    // 5. Verify pause/resume mechanism.
    // 5.1. Transfer should be cancelled.
    assert_invalid_transfer!(ticker, owner.did, receiver.did, 10);

    set_curr_did(Some(owner.did));
    // 5.2. Pause asset compliance, and run the transaction.
    assert_ok!(ComplianceManager::pause_asset_compliance(
        owner.origin(),
        ticker
    ));
    set_curr_did(None);
    assert_valid_transfer!(ticker, owner.did, receiver.did, 10);

    set_curr_did(Some(owner.did));
    // 5.3. Resume asset compliance, and new transfer should fail again.
    assert_ok!(ComplianceManager::resume_asset_compliance(
        owner.origin(),
        ticker
    ));
    set_curr_did(None);
    assert_invalid_transfer!(ticker, owner.did, receiver.did, 10);
}

#[test]
fn should_successfully_add_and_use_default_issuers() {
    ExtBuilder::default()
        .build()
        .execute_with(should_successfully_add_and_use_default_issuers_we);
}

fn should_successfully_add_and_use_default_issuers_we() {
    // 0. Create accounts
    let root = Origin::from(frame_system::RawOrigin::Root);
    let owner = User::new(AccountKeyring::Alice);
    let trusted_issuer = User::new(AccountKeyring::Charlie);
    let receiver = User::new(AccountKeyring::Dave);
    let eve = User::new(AccountKeyring::Eve);
    let ferdie = User::new(AccountKeyring::Ferdie);

    assert_ok!(CDDGroup::reset_members(root, vec![trusted_issuer.did]));

    // 1. Create & mint token
    let (ticker, _) = create_token(owner);

    /*
    assert_ok!(Asset::remove_primary_issuance_agent(
        owner.origin(),
        ticker
    ));
    */

    // Failed because trusted issuer identity not exist
    assert_noop!(
        ComplianceManager::add_default_trusted_claim_issuer(
            owner.origin(),
            ticker,
            IdentityId::from(1).into()
        ),
        CMError::<TestStorage>::DidNotExist
    );

    let add_issuer = |ti| {
        assert_ok!(ComplianceManager::add_default_trusted_claim_issuer(
            owner.origin(),
            ticker,
            ti
        ));
    };

    let trusted_issuers = vec![
        trusted_issuer.issuer(),
        eve.trusted_issuer_for(TrustedFor::Specific(vec![ClaimType::Affiliate])),
        ferdie.trusted_issuer_for(TrustedFor::Specific(vec![ClaimType::Accredited])),
    ];
    for ti in trusted_issuers.clone() {
        add_issuer(ti);
    }

    assert_eq!(ComplianceManager::trusted_claim_issuer(ticker).len(), 3);
    assert_eq!(
        ComplianceManager::trusted_claim_issuer(ticker),
        trusted_issuers
    );
    assert_ok!(Identity::add_claim(
        trusted_issuer.origin(),
        receiver.did.clone(),
        Claim::CustomerDueDiligence(Default::default()),
        Some(u64::MAX),
    ));

    let now = Utc::now();
    set_timestamp(now.timestamp() as u64);

    let claim_need_to_posses_1 = Claim::Affiliate(owner.scope());
    let claim_need_to_posses_2 = Claim::Accredited(owner.scope());
    let sender_condition: Condition =
        ConditionType::IsPresent(claim_need_to_posses_1.clone()).into();
    let receiver_condition1 = sender_condition.clone();
    let receiver_condition2 = ConditionType::IsPresent(claim_need_to_posses_2.clone()).into();
    assert_ok!(ComplianceManager::add_compliance_requirement(
        owner.origin(),
        ticker,
        vec![sender_condition],
        vec![receiver_condition1, receiver_condition2]
    ));

    let provide_affiliated_claim = |claim_for| {
        assert_ok!(Identity::add_claim(
            eve.origin(),
            claim_for,
            claim_need_to_posses_1.clone(),
            Some(u64::MAX),
        ));
    };

    let provide_accredited_claim = |claim_by| {
        assert_ok!(Identity::add_claim(
            claim_by,
            receiver.did,
            claim_need_to_posses_2.clone(),
            Some(u64::MAX),
        ));
    };

    provide_affiliated_claim(receiver.did);
    provide_affiliated_claim(owner.did);

    // fail when token owner doesn't has the valid claim
    assert_invalid_transfer!(ticker, owner.did, receiver.did, 100);

    // Right claim, but Eve not trusted for this asset.
    provide_accredited_claim(eve.origin());

    assert_invalid_transfer!(ticker, owner.did, receiver.did, 100);

    // Right claim, and Ferdie is trusted for this asset.
    provide_accredited_claim(ferdie.origin());

    assert_valid_transfer!(ticker, owner.did, receiver.did, 100);
}

#[test]
fn should_modify_vector_of_trusted_issuer() {
    ExtBuilder::default()
        .build()
        .execute_with(should_modify_vector_of_trusted_issuer_we);
}

fn should_modify_vector_of_trusted_issuer_we() {
    // 0. Create accounts
    let root = Origin::from(frame_system::RawOrigin::Root);
    let owner = User::new(AccountKeyring::Alice);
    let trusted_issuer_1 = User::new(AccountKeyring::Charlie);
    let trusted_issuer_2 = User::new(AccountKeyring::Ferdie);
    let receiver = User::new(AccountKeyring::Dave);

    // Providing a random DID to root but in real world Root should posses a DID
    assert_ok!(CDDGroup::reset_members(
        root,
        vec![trusted_issuer_1.did, trusted_issuer_2.did]
    ));

    // 1. Create & mint token
    let (ticker, _) = create_token(owner);

    assert_ok!(ComplianceManager::add_default_trusted_claim_issuer(
        owner.origin(),
        ticker,
        trusted_issuer_1.issuer()
    ));

    assert_ok!(ComplianceManager::add_default_trusted_claim_issuer(
        owner.origin(),
        ticker,
        trusted_issuer_2.issuer()
    ));

    assert_eq!(ComplianceManager::trusted_claim_issuer(ticker).len(), 2);
    assert_eq!(
        ComplianceManager::trusted_claim_issuer(ticker),
        vec![trusted_issuer_1.issuer(), trusted_issuer_2.issuer()]
    );

    let accredited_claim = Claim::Accredited(Scope::Custom(vec![b't']));

    let provide_claim = |claim_for, claim_by, claim| {
        assert_ok!(Identity::add_claim(claim_for, claim_by, claim, None,));
    };

    // adding claim by trusted issuer 1
    provide_claim(
        trusted_issuer_1.origin(),
        receiver.did,
        accredited_claim.clone(),
    );

    // adding claim by trusted issuer 1
    let claim = Claim::Blocked(ticker.into());
    provide_claim(trusted_issuer_1.origin(), receiver.did, claim.clone());

    // adding claim by trusted issuer 2
    provide_claim(
        trusted_issuer_2.origin(),
        owner.did,
        accredited_claim.clone(),
    );

    let now = Utc::now();
    set_timestamp(now.timestamp() as u64);

    let create_condition = |claim| -> Condition {
        Condition {
            issuers: vec![],
            condition_type: ConditionType::IsPresent(claim),
        }
    };

    let sender_condition = create_condition(accredited_claim.clone());

    let receiver_condition_1 = sender_condition.clone();

    let receiver_condition_2 = create_condition(claim.clone());

    let x = vec![sender_condition.clone()];
    let y = vec![receiver_condition_1, receiver_condition_2];

    assert_ok!(ComplianceManager::add_compliance_requirement(
        owner.origin(),
        ticker,
        x,
        y
    ));

    assert_valid_transfer!(ticker, owner.did, receiver.did, 10);

    // Remove the trusted issuer 1 from the list
    assert_ok!(ComplianceManager::remove_default_trusted_claim_issuer(
        owner.origin(),
        ticker,
        trusted_issuer_1.did
    ));

    assert_eq!(ComplianceManager::trusted_claim_issuer(ticker).len(), 1);
    assert_eq!(
        ComplianceManager::trusted_claim_issuer(ticker),
        vec![trusted_issuer_2.issuer()]
    );

    // Transfer should fail as issuer doesn't exist anymore but the compliance data still exist
    assert_invalid_transfer!(ticker, owner.did, receiver.did, 500);

    // Change the compliance requirement to all the transfer happen again

    let receiver_condition_1 = Condition::from_dids(
        ConditionType::IsPresent(accredited_claim.clone()),
        &[trusted_issuer_1.did],
    );

    let receiver_condition_2 = Condition::from_dids(
        ConditionType::IsPresent(claim.clone()),
        &[trusted_issuer_1.did],
    );

    let x = vec![sender_condition];
    let y = vec![receiver_condition_1, receiver_condition_2];

    let compliance_requirement = ComplianceRequirement {
        sender_conditions: x.clone(),
        receiver_conditions: y.clone(),
        id: 1,
    };

    // Failed because sender is not an agent of the ticker
    assert_noop!(
        ComplianceManager::change_compliance_requirement(
            receiver.origin(),
            ticker,
            compliance_requirement.clone()
        ),
        EAError::UnauthorizedAgent
    );

    let compliance_requirement_failure = ComplianceRequirement {
        sender_conditions: x,
        receiver_conditions: y,
        id: 5,
    };

    // Failed because passed id is not valid
    assert_noop!(
        ComplianceManager::change_compliance_requirement(
            owner.origin(),
            ticker,
            compliance_requirement_failure.clone()
        ),
        CMError::<TestStorage>::InvalidComplianceRequirementId
    );

    // Should successfully change the compliance requirement
    assert_ok!(ComplianceManager::change_compliance_requirement(
        owner.origin(),
        ticker,
        compliance_requirement
    ));

    // Now the transfer should pass
    assert_valid_transfer!(ticker, owner.did, receiver.did, 500);
}

#[test]
fn jurisdiction_asset_compliance() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Eve.to_account_id()])
        .build()
        .execute_with(jurisdiction_asset_compliance_we);
}
fn jurisdiction_asset_compliance_we() {
    // 0. Create accounts
    let owner = User::new(AccountKeyring::Alice);
    let cdd = User::new(AccountKeyring::Bob);
    let user = User::new(AccountKeyring::Charlie);

    // 1. Create & mint token
    let (ticker, _) = create_token(owner);

    // 2. Set up compliance requirements for Asset transfer.
    let scope = Scope::from(IdentityId::from(0));
    let receiver_conditions = vec![
        Condition::from_dids(
            ConditionType::IsAnyOf(vec![
                Claim::Jurisdiction(CountryCode::CA, scope.clone()),
                Claim::Jurisdiction(CountryCode::ES, scope.clone()),
            ]),
            &[cdd.did],
        ),
        Condition::from_dids(
            ConditionType::IsAbsent(Claim::Blocked(scope.clone())),
            &[owner.did],
        ),
    ];
    assert_ok!(ComplianceManager::add_compliance_requirement(
        owner.origin(),
        ticker,
        vec![],
        receiver_conditions
    ));
    // 3. Validate behaviour.
    // 3.1. Invalid transfer because missing jurisdiction.
    assert_invalid_transfer!(ticker, owner.did, user.did, 10);
    // 3.2. Add jurisdiction and transfer will be OK.
    assert_ok!(Identity::add_claim(
        cdd.origin(),
        user.did,
        Claim::Jurisdiction(CountryCode::CA, scope.clone()),
        None
    ));
    assert_valid_transfer!(ticker, owner.did, user.did, 10);
    // 3.3. Add user to Blocked
    assert_ok!(Identity::add_claim(
        owner.origin(),
        user.did,
        Claim::Blocked(scope.clone()),
        None,
    ));
    assert_invalid_transfer!(ticker, owner.did, user.did, 10);
}

#[test]
fn scope_asset_compliance() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Eve.to_account_id()])
        .build()
        .execute_with(scope_asset_compliance_we);
}
fn scope_asset_compliance_we() {
    // 0. Create accounts
    let owner = User::new(AccountKeyring::Alice);
    let cdd = User::new(AccountKeyring::Bob);
    let user = User::new(AccountKeyring::Charlie);

    // 1. Create a token.
    let (ticker, _) = create_token(owner);

    // 2. Set up compliance requirements for Asset transfer.
    let scope = Scope::Ticker(ticker);
    let receiver_conditions = vec![Condition::from_dids(
        ConditionType::IsPresent(Claim::Affiliate(scope.clone())),
        &[cdd.did],
    )];
    assert_ok!(ComplianceManager::add_compliance_requirement(
        owner.origin(),
        ticker,
        vec![],
        receiver_conditions
    ));
    // 3. Validate behaviour.
    // 3.1. Invalid transfer because missing jurisdiction.
    assert_invalid_transfer!(ticker, owner.did, user.did, 10);
    // 3.2. Add jurisdiction and transfer will be OK.
    assert_ok!(Identity::add_claim(
        cdd.origin(),
        user.did,
        Claim::Affiliate(scope.clone()),
        None
    ));
    assert_valid_transfer!(ticker, owner.did, user.did, 10);
}

#[test]
fn ensure_custom_scopes_limited() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = User::new(AccountKeyring::Alice);
        let (ticker, _) = create_token(owner);

        let fill = Condition::from_dids(
            ConditionType::IsIdentity(TargetIdentity::ExternalAgent),
            &[],
        );
        let data = |n| [b'1'].repeat(n);
        let cond = |data| {
            let scope = Scope::Custom(data);
            let ct = ConditionType::IsPresent(Claim::Affiliate(scope));
            Condition::from_dids(ct, &[])
        };
        let conds = |n| vec![fill.clone(), fill.clone(), cond(data(n)), fill.clone()];
        let req_s = |n| ComplianceRequirement {
            sender_conditions: conds(n),
            receiver_conditions: vec![],
            id: 0,
        };
        let req_r = |n| ComplianceRequirement {
            sender_conditions: vec![],
            receiver_conditions: conds(n),
            id: 0,
        };

        let add =
            |s, r| ComplianceManager::add_compliance_requirement(owner.origin(), ticker, s, r);

        let replace =
            |cr| ComplianceManager::replace_asset_compliance(owner.origin(), ticker, vec![cr]);

        let change =
            |cr| ComplianceManager::change_compliance_requirement(owner.origin(), ticker, cr);

        // Adding:
        // Check <= 32.
        assert_ok!(add(conds(31), vec![]));
        assert_ok!(add(vec![], conds(31)));
        assert_ok!(add(conds(32), vec![]));
        assert_ok!(add(vec![], conds(32)));
        // Check 33.
        assert_noop!(add(conds(33), vec![]), IdError::CustomScopeTooLong);
        assert_noop!(add(vec![], conds(33)), IdError::CustomScopeTooLong);

        // Replacing:
        // Check <= 32.
        assert_ok!(replace(req_s(31)));
        assert_ok!(replace(req_r(31)));
        assert_ok!(replace(req_s(32)));
        assert_ok!(replace(req_r(32)));
        // Check 33.
        assert_noop!(replace(req_s(33)), IdError::CustomScopeTooLong);
        assert_noop!(replace(req_r(33)), IdError::CustomScopeTooLong);

        // Changing:
        // Check <= 32.
        assert_ok!(change(req_s(31)));
        assert_ok!(change(req_r(31)));
        assert_ok!(change(req_s(32)));
        assert_ok!(change(req_r(32)));
        // Check 33.
        assert_noop!(change(req_s(33)), IdError::CustomScopeTooLong);
        assert_noop!(change(req_r(33)), IdError::CustomScopeTooLong);
    });
}

#[test]
fn cm_test_case_9() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::One.to_account_id()])
        .build()
        .execute_with(cm_test_case_9_we);
}
/// Is any of: KYC’d, Affiliate, Accredited, Exempted
fn cm_test_case_9_we() {
    // 0. Create accounts
    let owner = User::new(AccountKeyring::Alice);
    let issuer = User::new(AccountKeyring::Bob);

    // 1. Create a token.
    let (ticker, _) = create_token(owner);
    // 2. Set up compliance requirements for Asset transfer.
    let scope = Scope::Ticker(ticker);
    let receiver_conditions = vec![Condition::from_dids(
        ConditionType::IsAnyOf(vec![
            Claim::KnowYourCustomer(scope.clone()),
            Claim::Affiliate(scope.clone()),
            Claim::Accredited(scope.clone()),
            Claim::Exempted(scope.clone()),
        ]),
        &[issuer.did],
    )];
    assert_ok!(ComplianceManager::add_compliance_requirement(
        owner.origin(),
        ticker,
        vec![],
        receiver_conditions
    ));

    // 3. Validate behaviour.
    let charlie = User::new(AccountKeyring::Charlie);
    let dave = User::new(AccountKeyring::Dave);
    let eve = User::new(AccountKeyring::Eve);
    let ferdie = User::new(AccountKeyring::Ferdie);

    let verify_restriction_granular = |user: User, claim| {
        assert_ok!(Identity::add_claim(issuer.origin(), user.did, claim, None));
        assert_valid_transfer!(ticker, owner.did, user.did, 100);
        ComplianceManager::verify_restriction_granular(
            &ticker,
            Some(owner.did),
            Some(user.did),
            &mut WeightMeter::max_limit_no_minimum(),
        )
        .unwrap()
    };

    // 3.1. Charlie has a 'KnowYourCustomer' Claim.
    let result = verify_restriction_granular(charlie, Claim::KnowYourCustomer(scope.clone()));
    assert!(result.result);
    assert!(result.requirements[0].result);
    assert!(result.requirements[0].receiver_conditions[0].result);

    // 3.2. Dave has a 'Affiliate' Claim
    let result = verify_restriction_granular(dave, Claim::Affiliate(scope.clone()));
    assert!(result.result);
    assert!(result.requirements[0].result);
    assert!(result.requirements[0].receiver_conditions[0].result);

    // 3.3. Eve has a 'Exempted' Claim
    let result = verify_restriction_granular(eve, Claim::Exempted(scope.clone()));
    assert!(result.requirements[0].result);
    assert!(result.requirements[0].receiver_conditions[0].result);

    // 3.4 Ferdie has none of the required claims
    assert_invalid_transfer!(ticker, owner.did, ferdie.did, 100);
    let result = ComplianceManager::verify_restriction_granular(
        &ticker,
        Some(owner.did),
        Some(ferdie.did),
        &mut WeightMeter::max_limit_no_minimum(),
    )
    .unwrap();
    assert!(!result.result);
    assert!(!result.requirements[0].result);
    assert!(!result.requirements[0].receiver_conditions[0].result);
}

#[test]
fn cm_test_case_11() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Ferdie.to_account_id()])
        .build()
        .execute_with(cm_test_case_11_we);
}

// Is any of: KYC’d, Affiliate, Accredited, Exempted, is none of: Jurisdiction=x, y, z,
fn cm_test_case_11_we() {
    // 0. Create accounts
    let owner = User::new(AccountKeyring::Alice);
    let issuer = User::new(AccountKeyring::Bob);

    // 1. Create a token.
    let (ticker, _) = create_token(owner);
    // 2. Set up compliance requirements for Asset transfer.
    let scope = Scope::Ticker(ticker);
    let receiver_conditions = vec![
        Condition::from_dids(
            ConditionType::IsAnyOf(vec![
                Claim::KnowYourCustomer(scope.clone()),
                Claim::Affiliate(scope.clone()),
                Claim::Accredited(scope.clone()),
                Claim::Exempted(scope.clone()),
            ]),
            &[issuer.did],
        ),
        Condition::from_dids(
            ConditionType::IsNoneOf(vec![
                Claim::Jurisdiction(CountryCode::US, scope.clone()),
                Claim::Jurisdiction(CountryCode::KP, scope.clone()),
            ]),
            &[issuer.did],
        ),
    ];
    assert_ok!(ComplianceManager::add_compliance_requirement(
        owner.origin(),
        ticker,
        vec![],
        receiver_conditions
    ));

    // 3. Validate behaviour.
    let charlie = User::new(AccountKeyring::Charlie);
    let dave = User::new(AccountKeyring::Dave);
    let eve = User::new(AccountKeyring::Eve);

    // 3.1. Charlie has a 'KnowYourCustomer' Claim.
    assert_ok!(Identity::add_claim(
        issuer.origin(),
        charlie.did,
        Claim::KnowYourCustomer(scope.clone()),
        None
    ));
    assert_valid_transfer!(ticker, owner.did, charlie.did, 100);
    let result = ComplianceManager::verify_restriction_granular(
        &ticker,
        Some(owner.did),
        Some(charlie.did),
        &mut WeightMeter::max_limit_no_minimum(),
    )
    .unwrap();
    assert!(result.result);
    assert!(result.requirements[0].result);
    assert!(result.requirements[0].receiver_conditions[0].result);
    assert!(result.requirements[0].receiver_conditions[1].result);

    // 3.2. Dave has a 'Affiliate' Claim but he is from USA
    assert_ok!(Identity::add_claim(
        issuer.origin(),
        dave.did,
        Claim::Affiliate(scope.clone()),
        None
    ));
    assert_ok!(Identity::add_claim(
        issuer.origin(),
        dave.did,
        Claim::Jurisdiction(CountryCode::US, scope.clone()),
        None
    ));

    assert_invalid_transfer!(ticker, owner.did, dave.did, 100);
    let result = ComplianceManager::verify_restriction_granular(
        &ticker,
        Some(owner.did),
        Some(dave.did),
        &mut WeightMeter::max_limit_no_minimum(),
    )
    .unwrap();
    assert!(!result.result);
    assert!(!result.requirements[0].result);
    assert!(result.requirements[0].receiver_conditions[0].result);
    assert!(!result.requirements[0].receiver_conditions[1].result);

    // 3.3. Eve has a 'Exempted' Claim
    assert_ok!(Identity::add_claim(
        issuer.origin(),
        eve.did,
        Claim::Exempted(scope.clone()),
        None
    ));
    assert_ok!(Identity::add_claim(
        issuer.origin(),
        eve.did,
        Claim::Jurisdiction(CountryCode::GB, scope.clone()),
        None
    ));

    assert_valid_transfer!(ticker, owner.did, eve.did, 100);
    let result = ComplianceManager::verify_restriction_granular(
        &ticker,
        Some(owner.did),
        Some(eve.did),
        &mut WeightMeter::max_limit_no_minimum(),
    )
    .unwrap();
    assert!(result.result);
    assert!(result.requirements[0].result);
    assert!(result.requirements[0].receiver_conditions[0].result);
    assert!(result.requirements[0].receiver_conditions[1].result);
}

#[test]
fn cm_test_case_13() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Ferdie.to_account_id()])
        .build()
        .execute_with(cm_test_case_13_we);
}

// Must be KYC’d, is any of: Affiliate, Exempted, Accredited, is none of: Jurisdiction=x, y, z, etc.
fn cm_test_case_13_we() {
    // 0. Create accounts
    let owner = User::new(AccountKeyring::Alice);
    let issuer = User::new(AccountKeyring::Bob);

    // 1. Create a token.
    let (ticker, _) = create_token(owner);
    // 2. Set up compliance requirements for Asset transfer.
    let scope = Scope::Ticker(ticker);
    let receiver_conditions = vec![
        Condition::from_dids(
            ConditionType::IsPresent(Claim::KnowYourCustomer(scope.clone())),
            &[issuer.did],
        ),
        Condition::from_dids(
            ConditionType::IsAnyOf(vec![
                Claim::Affiliate(scope.clone()),
                Claim::Accredited(scope.clone()),
                Claim::Exempted(scope.clone()),
            ]),
            &[issuer.did],
        ),
        Condition::from_dids(
            ConditionType::IsNoneOf(vec![
                Claim::Jurisdiction(CountryCode::US, scope.clone()),
                Claim::Jurisdiction(CountryCode::KP, scope.clone()),
            ]),
            &[issuer.did],
        ),
    ];
    assert_ok!(ComplianceManager::add_compliance_requirement(
        owner.origin(),
        ticker,
        vec![],
        receiver_conditions
    ));

    // 3. Validate behaviour.
    let charlie = User::new(AccountKeyring::Charlie);
    let dave = User::new(AccountKeyring::Dave);
    let eve = User::new(AccountKeyring::Eve);

    // 3.1. Charlie has a 'KnowYourCustomer' Claim BUT he does not have any of { 'Affiliate',
    //   'Accredited', 'Exempted'}.
    assert_ok!(Identity::add_claim(
        issuer.origin(),
        charlie.did,
        Claim::KnowYourCustomer(scope.clone()),
        None
    ));

    assert_invalid_transfer!(ticker, owner.did, charlie.did, 100);
    let result = ComplianceManager::verify_restriction_granular(
        &ticker,
        None,
        Some(charlie.did),
        &mut WeightMeter::max_limit_no_minimum(),
    )
    .unwrap();
    assert!(!result.result);
    assert!(!result.requirements[0].result);
    assert!(result.requirements[0].receiver_conditions[0].result);
    assert!(!result.requirements[0].receiver_conditions[1].result);
    assert!(result.requirements[0].receiver_conditions[2].result);

    // 3.2. Dave has a 'Affiliate' Claim but he is from USA

    assert_add_claim!(
        issuer.origin(),
        dave.did,
        Claim::Exempted(scope.clone()),
        None
    );
    assert_add_claim!(
        issuer.origin(),
        dave.did,
        Claim::KnowYourCustomer(scope.clone()),
        None
    );
    assert_add_claim!(
        issuer.origin(),
        dave.did,
        Claim::Jurisdiction(CountryCode::US, scope.clone()),
        None
    );

    assert_invalid_transfer!(ticker, owner.did, dave.did, 100);
    let result = ComplianceManager::verify_restriction_granular(
        &ticker,
        None,
        Some(dave.did),
        &mut WeightMeter::max_limit_no_minimum(),
    )
    .unwrap();
    assert!(!result.result);
    assert!(!result.requirements[0].result);
    assert!(result.requirements[0].receiver_conditions[0].result);
    assert!(result.requirements[0].receiver_conditions[1].result);
    assert!(!result.requirements[0].receiver_conditions[2].result);

    // 3.3. Eve has a 'Exempted' Claim
    assert_add_claim!(
        issuer.origin(),
        eve.did,
        Claim::Exempted(scope.clone()),
        None
    );
    assert_add_claim!(
        issuer.origin(),
        eve.did,
        Claim::KnowYourCustomer(scope.clone()),
        None
    );
    assert_add_claim!(
        issuer.origin(),
        eve.did,
        Claim::Jurisdiction(CountryCode::GB, scope.clone()),
        None
    );

    assert_valid_transfer!(ticker, owner.did, eve.did, 100);
    let result = ComplianceManager::verify_restriction_granular(
        &ticker,
        Some(owner.did),
        Some(eve.did),
        &mut WeightMeter::max_limit_no_minimum(),
    )
    .unwrap();
    assert!(result.result);
    assert!(result.requirements[0].result);
    assert!(result.requirements[0].receiver_conditions[0].result);
    assert!(result.requirements[0].receiver_conditions[1].result);
    assert!(result.requirements[0].receiver_conditions[2].result);
}

#[test]
fn can_verify_restriction_with_primary_issuance_agent() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Eve.to_account_id()])
        .build()
        .execute_with(can_verify_restriction_with_primary_issuance_agent_we);
}

fn can_verify_restriction_with_primary_issuance_agent_we() {
    let owner = User::new(AccountKeyring::Alice);
    let issuer = User::new(AccountKeyring::Bob);
    let other = User::new(AccountKeyring::Charlie);

    // 1. Create a token.
    let (ticker, _) = create_token(owner);

    let auth_id = Identity::add_auth(
        owner.did,
        Signatory::from(issuer.did),
        AuthorizationData::BecomeAgent(ticker, AgentGroup::Full),
        None,
    );
    assert_ok!(ExternalAgents::accept_become_agent(
        issuer.origin(),
        auth_id
    ));

    // No compliance requirement is present, compliance should fail
    let mut weight_meter = WeightMeter::max_limit_no_minimum();
    assert_ok!(
        ComplianceManager::is_compliant(&ticker, issuer.did, other.did, &mut weight_meter),
        false
    );

    let conditions = |ident: TargetIdentity| {
        vec![Condition {
            condition_type: ConditionType::IsIdentity(ident),
            issuers: vec![],
        }]
    };

    // Add compliance requirement that requires sender to be primary issuance agent (dynamic)
    // and receiver to be a specific other id.
    assert_ok!(ComplianceManager::add_compliance_requirement(
        owner.origin(),
        ticker,
        conditions(TargetIdentity::ExternalAgent),
        conditions(TargetIdentity::Specific(other.did)),
    ));

    let verify = |from: User, to: User, weight_meter: &mut WeightMeter| {
        ComplianceManager::is_compliant(&ticker, from.did, to.did, weight_meter)
    };

    // From primary issuance agent to the random guy should succeed
    assert_ok!(verify(issuer, other, &mut weight_meter), true);

    // From primary issuance agent to owner should fail
    assert_ok!(verify(issuer, owner, &mut weight_meter), false);

    // From random guy to primary issuance agent should fail
    assert_ok!(verify(other, issuer, &mut weight_meter), false);
}

#[test]
fn should_limit_compliance_requirement_complexity() {
    ExtBuilder::default()
        .build()
        .execute_with(should_limit_compliance_requirements_complexity_we);
}

fn should_limit_compliance_requirements_complexity_we() {
    let owner = User::new(AccountKeyring::Alice);

    // 1. Create & mint token
    let (ticker, _) = create_token(owner);

    let scope = Scope::Ticker(ticker);
    Balances::make_free_balance_be(&owner.acc(), 1_000_000);

    let ty = ConditionType::IsPresent(Claim::KnowYourCustomer(scope.clone()));
    let conditions_with_issuer = vec![Condition::from_dids(ty.clone(), &[owner.did]); 30];

    let conditions_without_issuers = vec![Condition::from_dids(ty, &[]); 15];

    // Complexity = 30*1 + 30*1 = 60
    assert_noop!(
        ComplianceManager::add_compliance_requirement(
            owner.origin(),
            ticker,
            conditions_with_issuer.clone(),
            conditions_with_issuer.clone()
        ),
        CMError::<TestStorage>::ComplianceRequirementTooComplex
    );

    // Complexity = 30*1 + 15*0 = 30
    assert_ok!(ComplianceManager::add_compliance_requirement(
        owner.origin(),
        ticker,
        conditions_with_issuer.clone(),
        conditions_without_issuers,
    ));

    // Complexity = 30*1 + 15*1 = 45
    assert_ok!(ComplianceManager::add_default_trusted_claim_issuer(
        owner.origin(),
        ticker,
        owner.issuer(),
    ));

    // Complexity = 30*1 + 15*2 = 60
    let other = User::new(AccountKeyring::Bob);
    assert_noop!(
        ComplianceManager::add_default_trusted_claim_issuer(owner.origin(), ticker, other.issuer(),),
        CMError::<TestStorage>::ComplianceRequirementTooComplex
    );

    let asset_compliance = ComplianceManager::asset_compliance(ticker);
    assert_eq!(asset_compliance.requirements.len(), 1);
}

#[test]
fn check_new_return_type_of_rpc() {
    ExtBuilder::default().build().execute_with(|| {
        // 0. Create accounts
        let owner = User::new(AccountKeyring::Alice);
        let receiver = User::new(AccountKeyring::Charlie);

        // 1. Create & mint token
        let (ticker, _) = create_token(owner);

        Balances::make_free_balance_be(&owner.acc(), 1_000_000);

        // Add empty rules
        allow_all_transfers(ticker, owner);

        let result = ComplianceManager::verify_restriction_granular(
            &ticker,
            Some(owner.did),
            Some(receiver.did),
            &mut WeightMeter::max_limit_no_minimum(),
        )
        .unwrap();

        let compliance_requirement = ComplianceRequirementResult {
            sender_conditions: vec![],
            receiver_conditions: vec![],
            id: 1,
            result: true,
        };

        assert!(result.requirements.len() == 1);
        assert_eq!(result.requirements[0], compliance_requirement);
        assert_eq!(result.result, true);

        // Transfer should be valid as there are no restrictions.
        assert_valid_transfer!(ticker, owner.did, receiver.did, 100);
    });
}
