use crate::{
    asset::{self as asset, AssetType, Error as AssetError, SecurityToken},
    general_tm::{self as general_tm, AssetRule, RuleData, RuleType},
    test::{
        storage::{make_account, TestStorage},
        ExtBuilder,
    },
};

use polymesh_primitives::{IdentityClaimData, Ticker};
use polymesh_runtime_balances as balances;
use polymesh_runtime_group::{self as group};
use polymesh_runtime_identity::{self as identity};

use chrono::prelude::Utc;
use frame_support::{assert_err, assert_ok, traits::Currency};
use test_client::AccountKeyring;

use sp_std::prelude::*;

type Identity = identity::Module<TestStorage>;
type Balances = balances::Module<TestStorage>;
type Timestamp = pallet_timestamp::Module<TestStorage>;
type Asset = asset::Module<TestStorage>;
type GeneralTM = general_tm::Module<TestStorage>;
type CDDGroup = group::Module<TestStorage, group::Instance2>;

type Origin = <TestStorage as frame_system::Trait>::Origin;

#[test]
fn should_add_and_verify_assetrule() {
    ExtBuilder::default()
        .build()
        .execute_with(should_add_and_verify_assetrule_we);
}

fn should_add_and_verify_assetrule_we() {
    // 0. Create accounts
    let root = Origin::system(frame_system::RawOrigin::Root);
    let token_owner_acc = AccountKeyring::Alice.public();
    let (token_owner_signed, token_owner_did) = make_account(token_owner_acc).unwrap();
    let cdd_provider = AccountKeyring::Eve.public();
    let (cdd_signed, cdd_id) = make_account(cdd_provider).unwrap();

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
    let ticker = Ticker::from(token.name.0.as_slice());
    Balances::make_free_balance_be(&token_owner_acc, 1_000_000);

    // Share issuance is successful
    assert_ok!(Asset::create_token(
        token_owner_signed.clone(),
        token.name.clone(),
        ticker,
        token.total_supply,
        true,
        token.asset_type.clone(),
        vec![],
        None
    ));
    let claim_issuer_acc = AccountKeyring::Bob.public();
    Balances::make_free_balance_be(&claim_issuer_acc, 1_000_000);
    let (claim_issuer_signed, claim_issuer_did) = make_account(claim_issuer_acc).unwrap();

    assert_ok!(Identity::add_claim(
        claim_issuer_signed.clone(),
        token_owner_did,
        IdentityClaimData::NoData,
        None,
    ));

    let now = Utc::now();
    Timestamp::set_timestamp(now.timestamp() as u64);

    let sender_rule = RuleData {
        claim: IdentityClaimData::NoData,
        trusted_issuers: vec![claim_issuer_did],
        rule_type: RuleType::ClaimIsPresent,
    };

    let receiver_rule1 = RuleData {
        claim: IdentityClaimData::CustomerDueDiligence,
        trusted_issuers: vec![cdd_id],
        rule_type: RuleType::ClaimIsAbsent,
    };

    let receiver_rule2 = RuleData {
        claim: IdentityClaimData::Accredited(token_owner_did),
        trusted_issuers: vec![claim_issuer_did],
        rule_type: RuleType::ClaimIsPresent,
    };

    let x = vec![sender_rule];
    let y = vec![receiver_rule1, receiver_rule2];

    let asset_rule = AssetRule {
        sender_rules: x,
        receiver_rules: y,
    };

    assert_ok!(GeneralTM::add_active_rule(
        token_owner_signed.clone(),
        ticker,
        asset_rule
    ));

    assert_ok!(Identity::add_claim(
        claim_issuer_signed.clone(),
        token_owner_did,
        IdentityClaimData::Accredited(claim_issuer_did),
        None,
    ));

    //Transfer tokens to investor
    assert_err!(
        Asset::transfer(
            token_owner_signed.clone(),
            ticker,
            token_owner_did.clone(),
            token.total_supply
        ),
        AssetError::<TestStorage>::InvalidTransfer
    );

    assert_ok!(Identity::add_claim(
        claim_issuer_signed.clone(),
        token_owner_did,
        IdentityClaimData::Accredited(token_owner_did),
        None,
    ));

    assert_ok!(Asset::transfer(
        token_owner_signed.clone(),
        ticker,
        token_owner_did.clone(),
        token.total_supply
    ));

    assert_ok!(Identity::add_claim(
        cdd_signed.clone(),
        token_owner_did,
        IdentityClaimData::CustomerDueDiligence,
        None,
    ));

    assert_err!(
        Asset::transfer(
            token_owner_signed.clone(),
            ticker,
            token_owner_did.clone(),
            token.total_supply
        ),
        AssetError::<TestStorage>::InvalidTransfer
    );
}

#[test]
fn should_reset_assetrules() {
    ExtBuilder::default()
        .build()
        .execute_with(should_reset_assetrules_we);
}

fn should_reset_assetrules_we() {
    let token_owner_acc = AccountKeyring::Alice.public();
    let (token_owner_signed, token_owner_did) = make_account(token_owner_acc).unwrap();

    // A token representing 1M shares
    let token = SecurityToken {
        name: vec![0x01].into(),
        owner_did: token_owner_did,
        total_supply: 1_000_000,
        divisible: true,
        asset_type: AssetType::default(),
        ..Default::default()
    };
    let ticker = Ticker::from(token.name.0.as_slice());
    Balances::make_free_balance_be(&token_owner_acc, 1_000_000);

    // Share issuance is successful
    assert_ok!(Asset::create_token(
        token_owner_signed.clone(),
        token.name.clone(),
        ticker,
        token.total_supply,
        true,
        token.asset_type.clone(),
        vec![],
        None
    ));

    let asset_rule = AssetRule {
        sender_rules: vec![],
        receiver_rules: vec![],
    };

    assert_ok!(GeneralTM::add_active_rule(
        token_owner_signed.clone(),
        ticker,
        asset_rule
    ));

    let asset_rules = GeneralTM::asset_rules(ticker);
    assert_eq!(asset_rules.rules.len(), 1);

    assert_ok!(GeneralTM::reset_active_rules(
        token_owner_signed.clone(),
        ticker
    ));

    let asset_rules_new = GeneralTM::asset_rules(ticker);
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
    let (token_owner_signed, token_owner_did) = make_account(token_owner_acc).unwrap();
    let receiver_acc = AccountKeyring::Charlie.public();
    let (receiver_signed, receiver_did) = make_account(receiver_acc).unwrap();

    // 1. A token representing 1M shares
    let token = SecurityToken {
        name: vec![0x01].into(),
        owner_did: token_owner_did.clone(),
        total_supply: 1_000_000,
        divisible: true,
        asset_type: AssetType::default(),
        ..Default::default()
    };
    let ticker = Ticker::from(token.name.0.as_slice());
    Balances::make_free_balance_be(&token_owner_acc, 1_000_000);

    // 2. Share issuance is successful
    assert_ok!(Asset::create_token(
        token_owner_signed.clone(),
        token.name.clone(),
        ticker,
        token.total_supply,
        true,
        token.asset_type.clone(),
        vec![],
        None
    ));

    assert_ok!(Identity::add_claim(
        receiver_signed.clone(),
        receiver_did.clone(),
        IdentityClaimData::NoData,
        Some(99999999999999999u64),
    ));

    let now = Utc::now();
    Timestamp::set_timestamp(now.timestamp() as u64);

    // 4. Define rules
    let receiver_rules = vec![RuleData {
        claim: IdentityClaimData::NoData,
        trusted_issuers: vec![receiver_did],
        rule_type: RuleType::ClaimIsAbsent,
    }];

    let asset_rule = AssetRule {
        sender_rules: vec![],
        receiver_rules,
    };

    assert_ok!(GeneralTM::add_active_rule(
        token_owner_signed.clone(),
        ticker,
        asset_rule
    ));

    // 5. Verify pause/resume mechanism.
    // 5.1. Transfer should be cancelled.
    assert_err!(
        Asset::transfer(token_owner_signed.clone(), ticker, receiver_did, 10),
        AssetError::<TestStorage>::InvalidTransfer
    );

    // 5.2. Pause asset rules, and run the transaction.
    assert_ok!(GeneralTM::pause_asset_rules(
        token_owner_signed.clone(),
        ticker
    ));
    assert_ok!(Asset::transfer(
        token_owner_signed.clone(),
        ticker,
        receiver_did,
        10
    ));

    // 5.3. Resume asset rules, and new transfer should fail again.
    assert_ok!(GeneralTM::resume_asset_rules(
        token_owner_signed.clone(),
        ticker
    ));
    assert_err!(
        Asset::transfer(token_owner_signed.clone(), ticker, receiver_did, 10),
        AssetError::<TestStorage>::InvalidTransfer
    );
}
