use super::{
    storage::{register_keyring_account, TestStorage},
    ExtBuilder,
};
use chrono::prelude::Utc;
use frame_support::{assert_err, assert_noop, assert_ok, traits::Currency};
use pallet_asset::{self as asset, AssetName, AssetType, Error as AssetError, SecurityToken};
use pallet_balances as balances;
use pallet_compliance_manager::{self as compliance_manager, AssetTransferRule, Error as CMError};
use pallet_group as group;
use pallet_identity::{self as identity, BatchAddClaimItem};
use polymesh_common_utilities::traits::compliance_manager::Trait as ComplianceManagerTrait;
use polymesh_common_utilities::{
    constants::{ERC1400_TRANSFER_FAILURE, ERC1400_TRANSFER_SUCCESS},
    Context,
};
use polymesh_primitives::{
    AuthorizationData, Claim, CountryCode, IdentityId, Rule, RuleType, Scope, Signatory,
    TargetIdentity, Ticker,
};
use sp_std::{convert::TryFrom, prelude::*};
use test_client::AccountKeyring;

type Identity = identity::Module<TestStorage>;
type Balances = balances::Module<TestStorage>;
type Timestamp = pallet_timestamp::Module<TestStorage>;
type Asset = asset::Module<TestStorage>;
type ComplianceManager = compliance_manager::Module<TestStorage>;
type CDDGroup = group::Module<TestStorage, group::Instance2>;
type Moment = u64;
type Origin = <TestStorage as frame_system::Trait>::Origin;

macro_rules! assert_invalid_transfer {
    ($ticker:expr, $from:expr, $to:expr, $amount:expr) => {
        assert_ne!(
            Asset::_is_valid_transfer(
                &$ticker,
                AccountKeyring::Alice.public(),
                Some($from),
                Some($to),
                $amount
            )
            .map(|(a, _)| a),
            Ok(ERC1400_TRANSFER_SUCCESS)
        );
    };
}

macro_rules! assert_valid_transfer {
    ($ticker:expr, $from:expr, $to:expr, $amount:expr) => {
        assert_eq!(
            Asset::_is_valid_transfer(
                &$ticker,
                AccountKeyring::Alice.public(),
                Some($from),
                Some($to),
                $amount
            )
            .map(|(a, _)| a),
            Ok(ERC1400_TRANSFER_SUCCESS)
        );
    };
}

fn make_ticker_env(owner: AccountKeyring, token_name: AssetName) -> (Ticker, IdentityId) {
    let owner_id = register_keyring_account(owner.clone()).unwrap();

    // 1. Create a token.
    let token = SecurityToken {
        name: token_name,
        owner_did: owner_id,
        total_supply: 1_000_000,
        divisible: true,
        ..Default::default()
    };

    let ticker = Ticker::try_from(token.name.0.as_slice()).unwrap();
    assert_ok!(Asset::create_asset(
        Origin::signed(owner.public()),
        token.name.clone(),
        ticker,
        token.total_supply,
        true,
        token.asset_type.clone(),
        vec![],
        None,
    ));

    (ticker, owner_id)
}

#[test]
fn should_add_and_verify_asset_rule() {
    ExtBuilder::default()
        .build()
        .execute_with(should_add_and_verify_asset_rule_we);
}

fn should_add_and_verify_asset_rule_we() {
    // 0. Create accounts
    let root = Origin::from(frame_system::RawOrigin::Root);
    let token_owner_acc = AccountKeyring::Alice.public();
    let token_owner_signed = Origin::signed(AccountKeyring::Alice.public());
    let token_owner_did = register_keyring_account(AccountKeyring::Alice).unwrap();
    let token_rec_did = register_keyring_account(AccountKeyring::Charlie).unwrap();
    let cdd_signed = Origin::signed(AccountKeyring::Eve.public());
    let cdd_id = register_keyring_account(AccountKeyring::Eve).unwrap();

    assert_ok!(CDDGroup::reset_members(root, vec![cdd_id]));

    // A token representing 1M shares
    let token = SecurityToken {
        name: vec![0x01].into(),
        owner_did: token_owner_did.clone(),
        total_supply: 1_000_000,
        divisible: true,
        asset_type: AssetType::default(),
        ..Default::default()
    };
    let ticker = Ticker::try_from(token.name.0.as_slice()).unwrap();
    Balances::make_free_balance_be(&token_owner_acc, 1_000_000);

    // Share issuance is successful
    assert_ok!(Asset::create_asset(
        token_owner_signed.clone(),
        token.name.clone(),
        ticker,
        token.total_supply,
        true,
        token.asset_type.clone(),
        vec![],
        None,
    ));
    let claim_issuer_acc = AccountKeyring::Bob.public();
    Balances::make_free_balance_be(&claim_issuer_acc, 1_000_000);
    let claim_issuer_signed = Origin::signed(AccountKeyring::Bob.public());
    let claim_issuer_did = register_keyring_account(AccountKeyring::Bob).unwrap();

    assert_ok!(Identity::add_claim(
        claim_issuer_signed.clone(),
        token_owner_did,
        Claim::NoData,
        None,
    ));

    assert_ok!(Identity::add_claim(
        claim_issuer_signed.clone(),
        token_rec_did,
        Claim::NoData,
        None,
    ));

    let now = Utc::now();
    Timestamp::set_timestamp(now.timestamp() as u64);

    let sender_rule = Rule {
        issuers: vec![claim_issuer_did],
        rule_type: RuleType::IsPresent(Claim::NoData),
    };

    let receiver_rule1 = Rule {
        issuers: vec![cdd_id],
        rule_type: RuleType::IsAbsent(Claim::make_cdd_wildcard()),
    };

    let receiver_rule2 = Rule {
        issuers: vec![claim_issuer_did],
        rule_type: RuleType::IsPresent(Claim::Accredited(token_owner_did)),
    };

    assert_ok!(ComplianceManager::add_active_rule(
        token_owner_signed.clone(),
        ticker,
        vec![sender_rule.clone()],
        vec![receiver_rule1.clone(), receiver_rule2.clone()]
    ));

    assert_ok!(Identity::add_claim(
        claim_issuer_signed.clone(),
        token_rec_did,
        Claim::Accredited(claim_issuer_did),
        None,
    ));

    //Transfer tokens to investor - fails wrong Accredited scope
    assert_invalid_transfer!(ticker, token_owner_did, token_rec_did, token.total_supply);
    let result = ComplianceManager::granular_verify_restriction(
        &ticker,
        Some(token_owner_did),
        Some(token_rec_did),
        None,
    );
    assert!(!result.final_result);
    assert!(!result.rules[0].transfer_rule_result);
    assert!(result.rules[0].sender_rules[0].result);
    assert!(result.rules[0].receiver_rules[0].result);
    assert!(!result.rules[0].receiver_rules[1].result);
    assert_eq!(result.rules[0].sender_rules[0].rule, sender_rule);
    assert_eq!(result.rules[0].receiver_rules[0].rule, receiver_rule1);
    assert_eq!(result.rules[0].receiver_rules[1].rule, receiver_rule2);

    assert_ok!(Identity::add_claim(
        claim_issuer_signed.clone(),
        token_rec_did,
        Claim::Accredited(token_owner_did),
        None,
    ));

    assert_valid_transfer!(ticker, token_owner_did, token_rec_did, 10);
    let result = ComplianceManager::granular_verify_restriction(
        &ticker,
        Some(token_owner_did),
        Some(token_rec_did),
        None,
    );
    assert!(result.final_result);
    assert!(result.rules[0].transfer_rule_result);
    assert!(result.rules[0].sender_rules[0].result);
    assert!(result.rules[0].receiver_rules[0].result);
    assert!(result.rules[0].receiver_rules[1].result);
    assert_eq!(result.rules[0].sender_rules[0].rule, sender_rule);
    assert_eq!(result.rules[0].receiver_rules[0].rule, receiver_rule1);
    assert_eq!(result.rules[0].receiver_rules[1].rule, receiver_rule2);

    assert_ok!(Identity::add_claim(
        cdd_signed.clone(),
        token_rec_did,
        Claim::make_cdd_wildcard(),
        None,
    ));

    assert_invalid_transfer!(ticker, token_owner_did, token_rec_did, 10);
    let result = ComplianceManager::granular_verify_restriction(
        &ticker,
        Some(token_owner_did),
        Some(token_rec_did),
        None,
    );
    assert!(!result.final_result);
    assert!(!result.rules[0].transfer_rule_result);
    assert!(result.rules[0].sender_rules[0].result);
    assert!(!result.rules[0].receiver_rules[0].result);
    assert!(result.rules[0].receiver_rules[1].result);
    assert_eq!(result.rules[0].sender_rules[0].rule, sender_rule);
    assert_eq!(result.rules[0].receiver_rules[0].rule, receiver_rule1);
    assert_eq!(result.rules[0].receiver_rules[1].rule, receiver_rule2);

    for _ in 0..2 {
        ComplianceManager::add_active_rule(
            token_owner_signed.clone(),
            ticker,
            vec![sender_rule.clone()],
            vec![receiver_rule1.clone(), receiver_rule2.clone()],
        );
    }
    assert_ok!(ComplianceManager::remove_active_rule(
        token_owner_signed.clone(),
        ticker,
        1
    )); // OK; latest == 3
    assert_err!(
        ComplianceManager::remove_active_rule(token_owner_signed.clone(), ticker, 1),
        CMError::<TestStorage>::InvalidRuleId
    ); // BAD OK; latest == 3, but 1 was just removed.
    assert_noop!(
        ComplianceManager::remove_active_rule(token_owner_signed.clone(), ticker, 1),
        CMError::<TestStorage>::InvalidRuleId
    );
}

#[test]
fn should_replace_asset_rules() {
    ExtBuilder::default()
        .build()
        .execute_with(should_replace_asset_rules_we);
}

fn should_replace_asset_rules_we() {
    let token_owner_acc = AccountKeyring::Alice.public();
    let token_owner_signed = Origin::signed(AccountKeyring::Alice.public());
    let token_owner_did = register_keyring_account(AccountKeyring::Alice).unwrap();

    // A token representing 1M shares
    let token = SecurityToken {
        name: vec![0x01].into(),
        owner_did: token_owner_did,
        total_supply: 1_000_000,
        divisible: true,
        asset_type: AssetType::default(),
        ..Default::default()
    };
    let ticker = Ticker::try_from(token.name.0.as_slice()).unwrap();
    Balances::make_free_balance_be(&token_owner_acc, 1_000_000);

    // Share issuance is successful
    assert_ok!(Asset::create_asset(
        token_owner_signed.clone(),
        token.name.clone(),
        ticker,
        token.total_supply,
        true,
        token.asset_type.clone(),
        vec![],
        None,
    ));

    assert_ok!(ComplianceManager::add_active_rule(
        token_owner_signed.clone(),
        ticker,
        vec![],
        vec![]
    ));

    let asset_rules = ComplianceManager::asset_rules(ticker);
    assert_eq!(asset_rules.rules.len(), 1);

    // Create three rules with different rule IDs.
    let new_asset_rules: Vec<AssetTransferRule> =
        std::iter::repeat(|rule_id: u32| AssetTransferRule {
            sender_rules: vec![],
            receiver_rules: vec![],
            rule_id,
        })
        .take(3)
        .enumerate()
        .map(|(n, f)| f(n as u32))
        .collect();

    assert_ok!(ComplianceManager::replace_asset_rules(
        token_owner_signed.clone(),
        ticker,
        new_asset_rules.clone(),
    ));

    let asset_rules = ComplianceManager::asset_rules(ticker);
    assert_eq!(asset_rules.rules, new_asset_rules);
}

#[test]
fn should_reset_asset_rules() {
    ExtBuilder::default()
        .build()
        .execute_with(should_reset_asset_rules_we);
}

fn should_reset_asset_rules_we() {
    let token_owner_acc = AccountKeyring::Alice.public();
    let token_owner_signed = Origin::signed(AccountKeyring::Alice.public());
    let token_owner_did = register_keyring_account(AccountKeyring::Alice).unwrap();

    // A token representing 1M shares
    let token = SecurityToken {
        name: vec![0x01].into(),
        owner_did: token_owner_did,
        total_supply: 1_000_000,
        divisible: true,
        asset_type: AssetType::default(),
        ..Default::default()
    };
    let ticker = Ticker::try_from(token.name.0.as_slice()).unwrap();
    Balances::make_free_balance_be(&token_owner_acc, 1_000_000);

    // Share issuance is successful
    assert_ok!(Asset::create_asset(
        token_owner_signed.clone(),
        token.name.clone(),
        ticker,
        token.total_supply,
        true,
        token.asset_type.clone(),
        vec![],
        None,
    ));

    assert_ok!(ComplianceManager::add_active_rule(
        token_owner_signed.clone(),
        ticker,
        vec![],
        vec![]
    ));

    let asset_rules = ComplianceManager::asset_rules(ticker);
    assert_eq!(asset_rules.rules.len(), 1);

    assert_ok!(ComplianceManager::reset_active_rules(
        token_owner_signed.clone(),
        ticker
    ));

    let asset_rules_new = ComplianceManager::asset_rules(ticker);
    assert_eq!(asset_rules_new.rules.len(), 0);
}

#[test]
fn pause_resume_asset_rules() {
    ExtBuilder::default()
        .build()
        .execute_with(pause_resume_asset_rules_we);
}

fn pause_resume_asset_rules_we() {
    // 0. Create accounts
    let token_owner_acc = AccountKeyring::Alice.public();
    let token_owner_signed = Origin::signed(AccountKeyring::Alice.public());
    let token_owner_did = register_keyring_account(AccountKeyring::Alice).unwrap();
    let receiver_signed = Origin::signed(AccountKeyring::Charlie.public());
    let receiver_did = register_keyring_account(AccountKeyring::Charlie).unwrap();

    // 1. A token representing 1M shares
    let token = SecurityToken {
        name: vec![0x01].into(),
        owner_did: token_owner_did.clone(),
        total_supply: 1_000_000,
        divisible: true,
        asset_type: AssetType::default(),
        ..Default::default()
    };
    let ticker = Ticker::try_from(token.name.0.as_slice()).unwrap();
    Balances::make_free_balance_be(&token_owner_acc, 1_000_000);

    // 2. Share issuance is successful
    assert_ok!(Asset::create_asset(
        token_owner_signed.clone(),
        token.name.clone(),
        ticker,
        token.total_supply,
        true,
        token.asset_type.clone(),
        vec![],
        None,
    ));

    assert_ok!(Identity::add_claim(
        receiver_signed.clone(),
        receiver_did.clone(),
        Claim::NoData,
        Some(99999999999999999u64),
    ));

    let now = Utc::now();
    Timestamp::set_timestamp(now.timestamp() as u64);

    // 4. Define rules
    let receiver_rules = vec![Rule {
        issuers: vec![receiver_did],
        rule_type: RuleType::IsAbsent(Claim::NoData),
    }];

    assert_ok!(ComplianceManager::add_active_rule(
        token_owner_signed.clone(),
        ticker,
        vec![],
        receiver_rules
    ));

    // 5. Verify pause/resume mechanism.
    // 5.1. Transfer should be cancelled.
    assert_invalid_transfer!(ticker, token_owner_did, receiver_did, 10);

    Context::set_current_identity::<Identity>(Some(token_owner_did));
    // 5.2. Pause asset rules, and run the transaction.
    assert_ok!(ComplianceManager::pause_asset_rules(
        token_owner_signed.clone(),
        ticker
    ));
    Context::set_current_identity::<Identity>(None);
    assert_valid_transfer!(ticker, token_owner_did, receiver_did, 10);

    Context::set_current_identity::<Identity>(Some(token_owner_did));
    // 5.3. Resume asset rules, and new transfer should fail again.
    assert_ok!(ComplianceManager::resume_asset_rules(
        token_owner_signed.clone(),
        ticker
    ));
    Context::set_current_identity::<Identity>(None);
    assert_invalid_transfer!(ticker, token_owner_did, receiver_did, 10);
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
    let token_owner_signed = Origin::signed(AccountKeyring::Alice.public());
    let token_owner_did = register_keyring_account(AccountKeyring::Alice).unwrap();
    let trusted_issuer_signed = Origin::signed(AccountKeyring::Charlie.public());
    let trusted_issuer_did = register_keyring_account(AccountKeyring::Charlie).unwrap();
    let receiver_did = register_keyring_account(AccountKeyring::Dave).unwrap();

    assert_ok!(CDDGroup::reset_members(root, vec![trusted_issuer_did]));

    // 1. A token representing 1M shares
    let token = SecurityToken {
        name: vec![0x01].into(),
        owner_did: token_owner_did,
        total_supply: 1_000_000,
        divisible: true,
        asset_type: AssetType::default(),
        ..Default::default()
    };
    let ticker = Ticker::try_from(token.name.0.as_slice()).unwrap();

    // 2. Share issuance is successful
    assert_ok!(Asset::create_asset(
        token_owner_signed.clone(),
        token.name.clone(),
        ticker,
        token.total_supply,
        true,
        token.asset_type.clone(),
        vec![],
        None,
    ));

    assert_ok!(Asset::remove_primary_issuance_agent(
        token_owner_signed.clone(),
        ticker
    ));

    // Failed because trusted issuer identity not exist
    assert_err!(
        ComplianceManager::add_default_trusted_claim_issuer(
            token_owner_signed.clone(),
            ticker,
            IdentityId::from(1)
        ),
        CMError::<TestStorage>::DidNotExist
    );

    assert_ok!(ComplianceManager::add_default_trusted_claim_issuer(
        token_owner_signed.clone(),
        ticker,
        trusted_issuer_did
    ));

    assert_eq!(ComplianceManager::trusted_claim_issuer(ticker).len(), 1);
    assert_eq!(
        ComplianceManager::trusted_claim_issuer(ticker),
        vec![trusted_issuer_did]
    );

    assert_ok!(Identity::add_claim(
        trusted_issuer_signed.clone(),
        receiver_did.clone(),
        Claim::make_cdd_wildcard(),
        Some(99999999999999999u64),
    ));

    let now = Utc::now();
    Timestamp::set_timestamp(now.timestamp() as u64);

    let sender_rule = Rule {
        issuers: vec![],
        rule_type: RuleType::IsPresent(Claim::make_cdd_wildcard()),
    };

    let receiver_rule = Rule {
        issuers: vec![],
        rule_type: RuleType::IsPresent(Claim::make_cdd_wildcard()),
    };

    assert_ok!(ComplianceManager::add_active_rule(
        token_owner_signed.clone(),
        ticker,
        vec![sender_rule],
        vec![receiver_rule]
    ));

    // fail when token owner doesn't has the valid claim
    assert_invalid_transfer!(ticker, token_owner_did, receiver_did, 100);

    assert_ok!(Identity::add_claim(
        trusted_issuer_signed.clone(),
        token_owner_did.clone(),
        Claim::make_cdd_wildcard(),
        Some(99999999999999999u64),
    ));

    assert_valid_transfer!(ticker, token_owner_did, receiver_did, 100);
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
    let token_owner_signed = Origin::signed(AccountKeyring::Alice.public());
    let token_owner_did = register_keyring_account(AccountKeyring::Alice).unwrap();
    let trusted_issuer_signed_1 = Origin::signed(AccountKeyring::Charlie.public());
    let trusted_issuer_did_1 = register_keyring_account(AccountKeyring::Charlie).unwrap();
    let trusted_issuer_signed_2 = Origin::signed(AccountKeyring::Ferdie.public());
    let trusted_issuer_did_2 = register_keyring_account(AccountKeyring::Ferdie).unwrap();
    let receiver_signed = Origin::signed(AccountKeyring::Dave.public());
    let receiver_did = register_keyring_account(AccountKeyring::Dave).unwrap();

    // Providing a random DID to root but in real world Root should posses a DID
    assert_ok!(CDDGroup::reset_members(
        root,
        vec![trusted_issuer_did_1, trusted_issuer_did_2]
    ));

    // 1. A token representing 1M shares
    let token = SecurityToken {
        name: vec![0x01].into(),
        owner_did: token_owner_did.clone(),
        total_supply: 1_000_000,
        divisible: true,
        asset_type: AssetType::default(),
        ..Default::default()
    };
    let ticker = Ticker::try_from(token.name.0.as_slice()).unwrap();

    // 2. Share issuance is successful
    assert_ok!(Asset::create_asset(
        token_owner_signed.clone(),
        token.name.clone(),
        ticker,
        token.total_supply,
        true,
        token.asset_type.clone(),
        vec![],
        None,
    ));

    // Failed because caller is not the owner of the ticker
    assert_err!(
        ComplianceManager::batch_add_default_trusted_claim_issuer(
            receiver_signed.clone(),
            vec![trusted_issuer_did_1, trusted_issuer_did_2],
            ticker,
        ),
        CMError::<TestStorage>::Unauthorized
    );

    // Failed because trusted issuer identity not exist
    assert_err!(
        ComplianceManager::batch_add_default_trusted_claim_issuer(
            token_owner_signed.clone(),
            vec![IdentityId::from(1), IdentityId::from(2)],
            ticker,
        ),
        CMError::<TestStorage>::DidNotExist
    );

    // Failed because trusted issuers length < 0
    assert_err!(
        ComplianceManager::batch_add_default_trusted_claim_issuer(
            token_owner_signed.clone(),
            vec![],
            ticker,
        ),
        CMError::<TestStorage>::InvalidLength
    );

    assert_ok!(ComplianceManager::batch_add_default_trusted_claim_issuer(
        token_owner_signed.clone(),
        vec![trusted_issuer_did_1, trusted_issuer_did_2],
        ticker,
    ));

    assert_eq!(ComplianceManager::trusted_claim_issuer(ticker).len(), 2);
    assert_eq!(
        ComplianceManager::trusted_claim_issuer(ticker),
        vec![trusted_issuer_did_1, trusted_issuer_did_2]
    );

    // adding claim by trusted issuer 1
    assert_ok!(Identity::add_claim(
        trusted_issuer_signed_1.clone(),
        receiver_did.clone(),
        Claim::make_cdd_wildcard(),
        None,
    ));

    // adding claim by trusted issuer 1
    assert_ok!(Identity::add_claim(
        trusted_issuer_signed_1.clone(),
        receiver_did.clone(),
        Claim::NoData,
        None,
    ));

    // adding claim by trusted issuer 2
    assert_ok!(Identity::add_claim(
        trusted_issuer_signed_2.clone(),
        token_owner_did.clone(),
        Claim::make_cdd_wildcard(),
        None,
    ));

    let now = Utc::now();
    Timestamp::set_timestamp(now.timestamp() as u64);

    let sender_rule = Rule {
        issuers: vec![],
        rule_type: RuleType::IsPresent(Claim::make_cdd_wildcard()),
    };

    let receiver_rule_1 = Rule {
        issuers: vec![],
        rule_type: RuleType::IsPresent(Claim::make_cdd_wildcard()),
    };

    let receiver_rule_2 = Rule {
        issuers: vec![],
        rule_type: RuleType::IsPresent(Claim::NoData),
    };

    let x = vec![sender_rule.clone()];
    let y = vec![receiver_rule_1, receiver_rule_2];

    assert_ok!(ComplianceManager::add_active_rule(
        token_owner_signed.clone(),
        ticker,
        x,
        y
    ));

    assert_valid_transfer!(ticker, token_owner_did, receiver_did, 10);

    // Remove the trusted issuer 1 from the list
    assert_ok!(
        ComplianceManager::batch_remove_default_trusted_claim_issuer(
            token_owner_signed.clone(),
            vec![trusted_issuer_did_1],
            ticker,
        )
    );

    assert_eq!(ComplianceManager::trusted_claim_issuer(ticker).len(), 1);
    assert_eq!(
        ComplianceManager::trusted_claim_issuer(ticker),
        vec![trusted_issuer_did_2]
    );

    // Transfer should fail as issuer doesn't exist anymore but the rule data still exist
    assert_invalid_transfer!(ticker, token_owner_did, receiver_did, 500);

    // Change the asset rule to all the transfer happen again

    let receiver_rule_1 = Rule {
        issuers: vec![trusted_issuer_did_1],
        rule_type: RuleType::IsPresent(Claim::make_cdd_wildcard()),
    };

    let receiver_rule_2 = Rule {
        issuers: vec![trusted_issuer_did_1],
        rule_type: RuleType::IsPresent(Claim::NoData),
    };

    let x = vec![sender_rule];
    let y = vec![receiver_rule_1, receiver_rule_2];

    let asset_rule = AssetTransferRule {
        sender_rules: x.clone(),
        receiver_rules: y.clone(),
        rule_id: 1,
    };

    // Failed because sender is not the owner of the ticker
    assert_err!(
        ComplianceManager::change_asset_rule(receiver_signed.clone(), ticker, asset_rule.clone()),
        CMError::<TestStorage>::Unauthorized
    );

    let asset_rule_failure = AssetTransferRule {
        sender_rules: x,
        receiver_rules: y,
        rule_id: 5,
    };

    // Failed because passed rule id is not valid
    assert_err!(
        ComplianceManager::change_asset_rule(
            token_owner_signed.clone(),
            ticker,
            asset_rule_failure.clone()
        ),
        CMError::<TestStorage>::InvalidRuleId
    );

    // Should successfully change the asset rule
    assert_ok!(ComplianceManager::change_asset_rule(
        token_owner_signed.clone(),
        ticker,
        asset_rule
    ));

    // Now the transfer should pass
    assert_valid_transfer!(ticker, token_owner_did, receiver_did, 500);
}

#[test]
fn jurisdiction_asset_rules() {
    ExtBuilder::default()
        .build()
        .execute_with(jurisdiction_asset_rules_we);
}
fn jurisdiction_asset_rules_we() {
    // 0. Create accounts
    let token_owner_signed = Origin::signed(AccountKeyring::Alice.public());
    let token_owner_id = register_keyring_account(AccountKeyring::Alice).unwrap();
    let cdd_signed = Origin::signed(AccountKeyring::Bob.public());
    let cdd_id = register_keyring_account(AccountKeyring::Bob).unwrap();
    let user_id = register_keyring_account(AccountKeyring::Charlie).unwrap();
    // 1. Create a token.
    let token = SecurityToken {
        name: vec![0x01].into(),
        owner_did: token_owner_id.clone(),
        total_supply: 1_000_000,
        divisible: true,
        ..Default::default()
    };
    let ticker = Ticker::try_from(token.name.0.as_slice()).unwrap();
    assert_ok!(Asset::create_asset(
        token_owner_signed.clone(),
        token.name.clone(),
        ticker,
        token.total_supply,
        true,
        token.asset_type.clone(),
        vec![],
        None,
    ));
    // 2. Set up rules for Asset transfer.
    let scope = Scope::from(0);
    let receiver_rules = vec![
        Rule {
            rule_type: RuleType::IsAnyOf(vec![
                Claim::Jurisdiction(CountryCode::CA, scope),
                Claim::Jurisdiction(CountryCode::ES, scope),
            ]),
            issuers: vec![cdd_id],
        },
        Rule {
            rule_type: RuleType::IsAbsent(Claim::Blocked(scope)),
            issuers: vec![token_owner_id],
        },
    ];
    assert_ok!(ComplianceManager::add_active_rule(
        token_owner_signed.clone(),
        ticker,
        vec![],
        receiver_rules
    ));
    // 3. Validate behaviour.
    // 3.1. Invalid transfer because missing jurisdiction.
    assert_invalid_transfer!(ticker, token_owner_id, user_id, 10);
    // 3.2. Add jurisdiction and transfer will be OK.
    assert_ok!(Identity::add_claim(
        cdd_signed.clone(),
        user_id,
        Claim::Jurisdiction(CountryCode::CA, scope),
        None
    ));
    assert_valid_transfer!(ticker, token_owner_id, user_id, 10);
    // 3.3. Add user to Blocked
    assert_ok!(Identity::add_claim(
        token_owner_signed.clone(),
        user_id,
        Claim::Blocked(scope),
        None,
    ));
    assert_invalid_transfer!(ticker, token_owner_id, user_id, 10);
}

#[test]
fn scope_asset_rules() {
    ExtBuilder::default()
        .build()
        .execute_with(scope_asset_rules_we);
}
fn scope_asset_rules_we() {
    // 0. Create accounts
    let owner = AccountKeyring::Alice;
    let owner_signed = Origin::signed(owner.public());
    let cdd_signed = Origin::signed(AccountKeyring::Bob.public());
    let cdd_id = register_keyring_account(AccountKeyring::Bob).unwrap();
    let user_id = register_keyring_account(AccountKeyring::Charlie).unwrap();
    // 1. Create a token.
    let (ticker, owner_did) = make_ticker_env(owner, vec![0x01].into());

    // 2. Set up rules for Asset transfer.
    let scope = Identity::get_token_did(&ticker).unwrap();
    let receiver_rules = vec![Rule {
        rule_type: RuleType::IsPresent(Claim::Affiliate(scope)),
        issuers: vec![cdd_id],
    }];
    assert_ok!(ComplianceManager::add_active_rule(
        owner_signed.clone(),
        ticker,
        vec![],
        receiver_rules
    ));
    // 3. Validate behaviour.
    // 3.1. Invalid transfer because missing jurisdiction.
    assert_invalid_transfer!(ticker, owner_did, user_id, 10);
    // 3.2. Add jurisdiction and transfer will be OK.
    assert_ok!(Identity::add_claim(
        cdd_signed.clone(),
        user_id,
        Claim::Affiliate(scope),
        None
    ));
    assert_valid_transfer!(ticker, owner_did, user_id, 10);
}

#[test]
fn cm_test_case_9() {
    ExtBuilder::default()
        .build()
        .execute_with(cm_test_case_9_we);
}
/// Is any of: KYC’d, Affiliate, Accredited, Exempted
fn cm_test_case_9_we() {
    // 0. Create accounts
    let owner = Origin::signed(AccountKeyring::Alice.public());
    let issuer = Origin::signed(AccountKeyring::Bob.public());
    let issuer_id = register_keyring_account(AccountKeyring::Bob).unwrap();

    // 1. Create a token.
    let (ticker, owner_did) = make_ticker_env(AccountKeyring::Alice, vec![0x01].into());
    // 2. Set up rules for Asset transfer.
    let scope = Identity::get_token_did(&ticker).unwrap();
    let receiver_rules = vec![Rule {
        rule_type: RuleType::IsAnyOf(vec![
            Claim::KnowYourCustomer(scope),
            Claim::Affiliate(scope),
            Claim::Accredited(scope),
            Claim::Exempted(scope),
        ]),
        issuers: vec![issuer_id],
    }];
    assert_ok!(ComplianceManager::add_active_rule(
        owner.clone(),
        ticker,
        vec![],
        receiver_rules
    ));

    // 3. Validate behaviour.
    let charlie = register_keyring_account(AccountKeyring::Charlie).unwrap();
    let dave = register_keyring_account(AccountKeyring::Dave).unwrap();
    let eve = register_keyring_account(AccountKeyring::Eve).unwrap();
    let ferdie = register_keyring_account(AccountKeyring::Ferdie).unwrap();

    // 3.1. Charlie has a 'KnowYourCustomer' Claim.
    assert_ok!(Identity::add_claim(
        issuer.clone(),
        charlie,
        Claim::KnowYourCustomer(scope),
        None
    ));
    assert_valid_transfer!(ticker, owner_did, charlie, 100);
    let result = ComplianceManager::granular_verify_restriction(&ticker, None, Some(charlie), None);
    assert!(result.final_result);
    assert!(result.rules[0].transfer_rule_result);
    assert!(result.rules[0].receiver_rules[0].result);

    // 3.2. Dave has a 'Affiliate' Claim
    assert_ok!(Identity::add_claim(
        issuer.clone(),
        dave,
        Claim::Affiliate(scope),
        None
    ));
    assert_valid_transfer!(ticker, owner_did, dave, 100);
    let result = ComplianceManager::granular_verify_restriction(&ticker, None, Some(dave), None);
    assert!(result.final_result);
    assert!(result.rules[0].transfer_rule_result);
    assert!(result.rules[0].receiver_rules[0].result);

    // 3.3. Eve has a 'Exempted' Claim
    assert_ok!(Identity::add_claim(
        issuer.clone(),
        eve,
        Claim::Exempted(scope),
        None
    ));
    let result = ComplianceManager::granular_verify_restriction(&ticker, None, Some(eve), None);
    assert!(result.rules[0].transfer_rule_result);
    assert!(result.rules[0].receiver_rules[0].result);

    // 3.4 Ferdie has none of the required claims
    assert_invalid_transfer!(ticker, owner_did, ferdie, 100);
    let result = ComplianceManager::granular_verify_restriction(&ticker, None, Some(ferdie), None);
    assert!(!result.final_result);
    assert!(!result.rules[0].transfer_rule_result);
    assert!(!result.rules[0].receiver_rules[0].result);
}

#[test]
fn cm_test_case_11() {
    ExtBuilder::default()
        .build()
        .execute_with(cm_test_case_11_we);
}

// Is any of: KYC’d, Affiliate, Accredited, Exempted, is none of: Jurisdiction=x, y, z,
fn cm_test_case_11_we() {
    // 0. Create accounts
    let owner = Origin::signed(AccountKeyring::Alice.public());
    let issuer = Origin::signed(AccountKeyring::Bob.public());
    let issuer_id = register_keyring_account(AccountKeyring::Bob).unwrap();

    // 1. Create a token.
    let (ticker, owner_did) = make_ticker_env(AccountKeyring::Alice, vec![0x01].into());
    // 2. Set up rules for Asset transfer.
    let scope = Identity::get_token_did(&ticker).unwrap();
    let receiver_rules = vec![
        Rule {
            rule_type: RuleType::IsAnyOf(vec![
                Claim::KnowYourCustomer(scope),
                Claim::Affiliate(scope),
                Claim::Accredited(scope),
                Claim::Exempted(scope),
            ]),
            issuers: vec![issuer_id],
        },
        Rule {
            rule_type: RuleType::IsNoneOf(vec![
                Claim::Jurisdiction(CountryCode::US, scope),
                Claim::Jurisdiction(CountryCode::KP, scope),
            ]),
            issuers: vec![issuer_id],
        },
    ];
    assert_ok!(ComplianceManager::add_active_rule(
        owner.clone(),
        ticker,
        vec![],
        receiver_rules
    ));

    // 3. Validate behaviour.
    let charlie = register_keyring_account(AccountKeyring::Charlie).unwrap();
    let dave = register_keyring_account(AccountKeyring::Dave).unwrap();
    let eve = register_keyring_account(AccountKeyring::Eve).unwrap();

    // 3.1. Charlie has a 'KnowYourCustomer' Claim.
    assert_ok!(Identity::add_claim(
        issuer.clone(),
        charlie,
        Claim::KnowYourCustomer(scope),
        None
    ));
    assert_valid_transfer!(ticker, owner_did, charlie, 100);
    let result = ComplianceManager::granular_verify_restriction(&ticker, None, Some(charlie), None);
    assert!(result.final_result);
    assert!(result.rules[0].transfer_rule_result);
    assert!(result.rules[0].receiver_rules[0].result);
    assert!(result.rules[0].receiver_rules[1].result);

    // 3.2. Dave has a 'Affiliate' Claim but he is from USA
    assert_ok!(Identity::add_claim(
        issuer.clone(),
        dave,
        Claim::Affiliate(scope),
        None
    ));
    assert_ok!(Identity::add_claim(
        issuer.clone(),
        dave,
        Claim::Jurisdiction(CountryCode::US, scope),
        None
    ));

    assert_invalid_transfer!(ticker, owner_did, dave, 100);
    let result = ComplianceManager::granular_verify_restriction(&ticker, None, Some(dave), None);
    assert!(!result.final_result);
    assert!(!result.rules[0].transfer_rule_result);
    assert!(result.rules[0].receiver_rules[0].result);
    assert!(!result.rules[0].receiver_rules[1].result);

    // 3.3. Eve has a 'Exempted' Claim
    assert_ok!(Identity::add_claim(
        issuer.clone(),
        eve,
        Claim::Exempted(scope),
        None
    ));
    assert_ok!(Identity::add_claim(
        issuer.clone(),
        eve,
        Claim::Jurisdiction(CountryCode::GB, scope),
        None
    ));

    assert_valid_transfer!(ticker, owner_did, eve, 100);
    let result = ComplianceManager::granular_verify_restriction(&ticker, None, Some(eve), None);
    assert!(result.final_result);
    assert!(result.rules[0].transfer_rule_result);
    assert!(result.rules[0].receiver_rules[0].result);
    assert!(result.rules[0].receiver_rules[1].result);
}

#[test]
fn cm_test_case_13() {
    ExtBuilder::default()
        .build()
        .execute_with(cm_test_case_13_we);
}

// Must be KYC’d, is any of: Affiliate, Exempted, Accredited, is none of: Jurisdiction=x, y, z, etc.
fn cm_test_case_13_we() {
    // 0. Create accounts
    let owner = Origin::signed(AccountKeyring::Alice.public());
    let issuer = Origin::signed(AccountKeyring::Bob.public());
    let issuer_id = register_keyring_account(AccountKeyring::Bob).unwrap();

    // 1. Create a token.
    let (ticker, owner_did) = make_ticker_env(AccountKeyring::Alice, vec![0x01].into());
    // 2. Set up rules for Asset transfer.
    let scope = Identity::get_token_did(&ticker).unwrap();
    let receiver_rules = vec![
        Rule {
            rule_type: RuleType::IsPresent(Claim::KnowYourCustomer(scope)),
            issuers: vec![issuer_id],
        },
        Rule {
            rule_type: RuleType::IsAnyOf(vec![
                Claim::Affiliate(scope),
                Claim::Accredited(scope),
                Claim::Exempted(scope),
            ]),
            issuers: vec![issuer_id],
        },
        Rule {
            rule_type: RuleType::IsNoneOf(vec![
                Claim::Jurisdiction(CountryCode::US, scope),
                Claim::Jurisdiction(CountryCode::KP, scope),
            ]),
            issuers: vec![issuer_id],
        },
    ];
    assert_ok!(ComplianceManager::add_active_rule(
        owner.clone(),
        ticker,
        vec![],
        receiver_rules
    ));

    // 3. Validate behaviour.
    let charlie = register_keyring_account(AccountKeyring::Charlie).unwrap();
    let dave = register_keyring_account(AccountKeyring::Dave).unwrap();
    let eve = register_keyring_account(AccountKeyring::Eve).unwrap();

    // 3.1. Charlie has a 'KnowYourCustomer' Claim BUT he does not have any of { 'Affiliate',
    //   'Accredited', 'Exempted'}.
    assert_ok!(Identity::add_claim(
        issuer.clone(),
        charlie,
        Claim::KnowYourCustomer(scope),
        None
    ));

    assert_invalid_transfer!(ticker, owner_did, charlie, 100);
    let result = ComplianceManager::granular_verify_restriction(&ticker, None, Some(charlie), None);
    assert!(!result.final_result);
    assert!(!result.rules[0].transfer_rule_result);
    assert!(result.rules[0].receiver_rules[0].result);
    assert!(!result.rules[0].receiver_rules[1].result);
    assert!(result.rules[0].receiver_rules[2].result);

    // 3.2. Dave has a 'Affiliate' Claim but he is from USA
    let dave_claims = vec![
        BatchAddClaimItem::<Moment> {
            target: dave,
            claim: Claim::Exempted(scope),
            expiry: None,
        },
        BatchAddClaimItem::<Moment> {
            target: dave,
            claim: Claim::KnowYourCustomer(scope),
            expiry: None,
        },
        BatchAddClaimItem::<Moment> {
            target: dave,
            claim: Claim::Jurisdiction(CountryCode::US, scope),
            expiry: None,
        },
    ];

    assert_ok!(Identity::batch_add_claim(issuer.clone(), dave_claims));

    assert_invalid_transfer!(ticker, owner_did, dave, 100);
    let result = ComplianceManager::granular_verify_restriction(&ticker, None, Some(dave), None);
    assert!(!result.final_result);
    assert!(!result.rules[0].transfer_rule_result);
    assert!(result.rules[0].receiver_rules[0].result);
    assert!(result.rules[0].receiver_rules[1].result);
    assert!(!result.rules[0].receiver_rules[2].result);

    // 3.3. Eve has a 'Exempted' Claim
    let eve_claims = vec![
        BatchAddClaimItem::<Moment> {
            target: eve,
            claim: Claim::Exempted(scope),
            expiry: None,
        },
        BatchAddClaimItem::<Moment> {
            target: eve,
            claim: Claim::KnowYourCustomer(scope),
            expiry: None,
        },
        BatchAddClaimItem::<Moment> {
            target: eve,
            claim: Claim::Jurisdiction(CountryCode::GB, scope),
            expiry: None,
        },
    ];

    assert_ok!(Identity::batch_add_claim(issuer.clone(), eve_claims));
    assert_valid_transfer!(ticker, owner_did, eve, 100);
    let result = ComplianceManager::granular_verify_restriction(&ticker, None, Some(eve), None);
    assert!(result.final_result);
    assert!(result.rules[0].transfer_rule_result);
    assert!(result.rules[0].receiver_rules[0].result);
    assert!(result.rules[0].receiver_rules[1].result);
    assert!(result.rules[0].receiver_rules[2].result);
}

#[test]
fn can_verify_restriction_with_primary_issuance_agent() {
    ExtBuilder::default()
        .build()
        .execute_with(can_verify_restriction_with_primary_issuance_agent_we);
}

fn can_verify_restriction_with_primary_issuance_agent_we() {
    let owner = AccountKeyring::Alice.public();
    let owner_origin = Origin::signed(owner);
    let owner_id = register_keyring_account(AccountKeyring::Alice).unwrap();
    let issuer = AccountKeyring::Bob.public();
    let issuer_id = register_keyring_account(AccountKeyring::Bob).unwrap();
    let random_guy_id = register_keyring_account(AccountKeyring::Charlie).unwrap();
    let token_name: AssetName = vec![0x01].into();
    let ticker = Ticker::try_from(token_name.0.as_slice()).unwrap();
    assert_ok!(Asset::create_asset(
        owner_origin.clone(),
        token_name,
        ticker,
        1_000_000,
        true,
        Default::default(),
        vec![],
        None,
    ));
    let auth_id = Identity::add_auth(
        owner_id,
        Signatory::from(issuer_id),
        AuthorizationData::TransferPrimaryIssuanceAgent(ticker),
        None,
    );
    assert_ok!(Asset::accept_primary_issuance_agent_transfer(
        Origin::signed(issuer),
        auth_id
    ));
    let amount = 1_000;

    // No rule is present, compliance should fail
    assert_ok!(
        ComplianceManager::verify_restriction(
            &ticker,
            None,
            Some(issuer_id),
            amount,
            Some(issuer_id)
        )
        .map(|(a, _)| a),
        ERC1400_TRANSFER_FAILURE
    );

    // Add rule that requires sender to be primary issuance agent (dynamic) and receiver to be a specific random_guy_id
    assert_ok!(ComplianceManager::add_active_rule(
        owner_origin,
        ticker,
        vec![Rule {
            rule_type: RuleType::IsIdentity(TargetIdentity::PrimaryIssuanceAgent),
            issuers: vec![],
        }],
        vec![Rule {
            rule_type: RuleType::IsIdentity(TargetIdentity::Specific(random_guy_id)),
            issuers: vec![],
        }]
    ));

    // From primary issuance agent to the random guy should succeed
    assert_ok!(
        ComplianceManager::verify_restriction(
            &ticker,
            Some(issuer_id),
            Some(random_guy_id),
            amount,
            Some(issuer_id)
        )
        .map(|(a, _)| a),
        ERC1400_TRANSFER_SUCCESS
    );

    // From primary issuance agent to owner should fail
    assert_ok!(
        ComplianceManager::verify_restriction(
            &ticker,
            Some(issuer_id),
            Some(owner_id),
            amount,
            Some(issuer_id)
        )
        .map(|(a, _)| a),
        ERC1400_TRANSFER_FAILURE
    );

    // From random guy to primary issuance agent should fail
    assert_ok!(
        ComplianceManager::verify_restriction(
            &ticker,
            Some(random_guy_id),
            Some(issuer_id),
            amount,
            Some(issuer_id)
        )
        .map(|(a, _)| a),
        ERC1400_TRANSFER_FAILURE
    );
}

#[test]
fn should_limit_rules_complexity() {
    ExtBuilder::default()
        .build()
        .execute_with(should_reset_asset_rules_we);
}

fn should_limit_rules_complexity_we() {
    let token_owner_acc = AccountKeyring::Alice.public();
    let token_owner_signed = Origin::signed(token_owner_acc.clone());
    let token_owner_did = register_keyring_account(AccountKeyring::Alice).unwrap();

    // A token representing 1M shares
    let token = SecurityToken {
        name: vec![0x01].into(),
        owner_did: token_owner_did,
        total_supply: 1_000_000,
        divisible: true,
        asset_type: AssetType::default(),
        ..Default::default()
    };
    let ticker = Ticker::try_from(token.name.0.as_slice()).unwrap();
    let scope = Identity::get_token_did(&ticker).unwrap();
    Balances::make_free_balance_be(&token_owner_acc, 1_000_000);

    // Share issuance is successful
    assert_ok!(Asset::create_asset(
        token_owner_signed.clone(),
        token.name.clone(),
        ticker,
        token.total_supply,
        true,
        token.asset_type.clone(),
        vec![],
        None,
    ));

    let rules_with_issuer = vec![
        Rule {
            rule_type: RuleType::IsPresent(Claim::KnowYourCustomer(scope)),
            issuers: vec![token_owner_did],
        };
        30
    ];

    let rules_without_issuers = vec![
        Rule {
            rule_type: RuleType::IsPresent(Claim::KnowYourCustomer(scope)),
            issuers: vec![],
        };
        15
    ];

    // Complexity = 30*1 + 30*1 = 60
    assert_noop!(
        ComplianceManager::add_active_rule(
            token_owner_signed.clone(),
            ticker,
            rules_with_issuer.clone(),
            rules_with_issuer.clone()
        ),
        CMError::<TestStorage>::RuleTooComplex
    );

    // Complexity = 30*1 + 15*0 = 30
    assert_ok!(ComplianceManager::add_active_rule(
        token_owner_signed.clone(),
        ticker,
        rules_with_issuer.clone(),
        rules_without_issuers,
    ));

    // Complexity = 30*1 + 15*1 = 45
    assert_ok!(ComplianceManager::add_default_trusted_claim_issuer(
        token_owner_signed.clone(),
        ticker,
        token_owner_did
    ));

    // Complexity = 30*1 + 15*2 = 60
    assert_noop!(
        ComplianceManager::add_default_trusted_claim_issuer(
            token_owner_signed.clone(),
            ticker,
            token_owner_did
        ),
        CMError::<TestStorage>::RuleTooComplex
    );

    let asset_rules = ComplianceManager::asset_rules(ticker);
    assert_eq!(asset_rules.rules.len(), 1);
}
