use crate::{
    asset::{self as asset, AssetType, Error as AssetError, SecurityToken},
    general_tm::{self as general_tm, AssetTransferRule, Error as GTMError},
    test::{
        storage::{make_account, TestStorage},
        ExtBuilder,
    },
};

use polymesh_primitives::{Claim, IdentityId, Rule, RuleType, Scope, Ticker};
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
fn should_add_and_verify_asset_rule() {
    ExtBuilder::default()
        .build()
        .execute_with(should_add_and_verify_asset_rule_we);
}

fn should_add_and_verify_asset_rule_we() {
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
        rule_type: RuleType::IsAbsent(Claim::CustomerDueDiligence),
    };

    let receiver_rule2 = Rule {
        issuers: vec![claim_issuer_did],
        rule_type: RuleType::IsPresent(Claim::Accredited(token_owner_did)),
    };

    assert_ok!(GeneralTM::add_active_rule(
        token_owner_signed.clone(),
        ticker,
        vec![sender_rule],
        vec![receiver_rule1, receiver_rule2]
    ));

    assert_ok!(Identity::add_claim(
        claim_issuer_signed.clone(),
        token_owner_did,
        Claim::Accredited(claim_issuer_did),
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
        Claim::Accredited(token_owner_did),
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
        Claim::CustomerDueDiligence,
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
fn should_reset_asset_rules() {
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

    assert_ok!(GeneralTM::add_active_rule(
        token_owner_signed.clone(),
        ticker,
        vec![],
        vec![]
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

    assert_ok!(GeneralTM::add_active_rule(
        token_owner_signed.clone(),
        ticker,
        vec![],
        receiver_rules
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

#[test]
fn should_successfully_add_and_use_default_issuers() {
    ExtBuilder::default()
        .build()
        .execute_with(should_successfully_add_and_use_default_issuers_we);
}

fn should_successfully_add_and_use_default_issuers_we() {
    // 0. Create accounts
    let root = Origin::system(frame_system::RawOrigin::Root);
    let token_owner_acc = AccountKeyring::Alice.public();
    let (token_owner_signed, token_owner_did) = make_account(token_owner_acc).unwrap();
    let trusted_issuer_acc = AccountKeyring::Charlie.public();
    let (trusted_issuer_signed, trusted_issuer_did) = make_account(trusted_issuer_acc).unwrap();
    let receiver_acc = AccountKeyring::Dave.public();
    let (_, receiver_did) = make_account(receiver_acc).unwrap();

    assert_ok!(CDDGroup::reset_members(root, vec![trusted_issuer_did]));

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

    // Failed because trusted issuer identity not exist
    assert_err!(
        GeneralTM::add_default_trusted_claim_issuer(
            token_owner_signed.clone(),
            ticker,
            IdentityId::from(1)
        ),
        GTMError::<TestStorage>::DidNotExist
    );

    assert_ok!(GeneralTM::add_default_trusted_claim_issuer(
        token_owner_signed.clone(),
        ticker,
        trusted_issuer_did
    ));

    assert_eq!(GeneralTM::trusted_claim_issuer(ticker).len(), 1);
    assert_eq!(
        GeneralTM::trusted_claim_issuer(ticker),
        vec![trusted_issuer_did]
    );

    assert_ok!(Identity::add_claim(
        trusted_issuer_signed.clone(),
        receiver_did.clone(),
        Claim::CustomerDueDiligence,
        Some(99999999999999999u64),
    ));

    let now = Utc::now();
    Timestamp::set_timestamp(now.timestamp() as u64);

    let sender_rule = Rule {
        issuers: vec![],
        rule_type: RuleType::IsPresent(Claim::CustomerDueDiligence),
    };

    let receiver_rule = Rule {
        issuers: vec![],
        rule_type: RuleType::IsPresent(Claim::CustomerDueDiligence),
    };

    assert_ok!(GeneralTM::add_active_rule(
        token_owner_signed.clone(),
        ticker,
        vec![sender_rule],
        vec![receiver_rule]
    ));

    // fail when token owner doesn't has the valid claim
    assert_err!(
        Asset::transfer(
            token_owner_signed.clone(),
            ticker,
            receiver_did.clone(),
            100
        ),
        AssetError::<TestStorage>::InvalidTransfer
    );

    assert_ok!(Identity::add_claim(
        trusted_issuer_signed.clone(),
        token_owner_did.clone(),
        Claim::CustomerDueDiligence,
        Some(99999999999999999u64),
    ));
    assert_ok!(Asset::transfer(
        token_owner_signed.clone(),
        ticker,
        receiver_did.clone(),
        100
    ));
}

#[test]
fn should_modify_vector_of_trusted_issuer() {
    ExtBuilder::default()
        .build()
        .execute_with(should_modify_vector_of_trusted_issuer_we);
}

fn should_modify_vector_of_trusted_issuer_we() {
    // 0. Create accounts
    let root = Origin::system(frame_system::RawOrigin::Root);
    let token_owner_acc = AccountKeyring::Alice.public();
    let (token_owner_signed, token_owner_did) = make_account(token_owner_acc).unwrap();
    let trusted_issuer_acc_1 = AccountKeyring::Charlie.public();
    let (trusted_issuer_signed_1, trusted_issuer_did_1) =
        make_account(trusted_issuer_acc_1).unwrap();
    let trusted_issuer_acc_2 = AccountKeyring::Ferdie.public();
    let (trusted_issuer_signed_2, trusted_issuer_did_2) =
        make_account(trusted_issuer_acc_2).unwrap();
    let receiver_acc = AccountKeyring::Dave.public();
    let (receiver_signed, receiver_did) = make_account(receiver_acc).unwrap();

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
    let ticker = Ticker::from(token.name.0.as_slice());

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

    // Failed because caller is not the owner of the ticker
    assert_err!(
        GeneralTM::add_default_trusted_claim_issuers_batch(
            receiver_signed.clone(),
            ticker,
            vec![trusted_issuer_did_1, trusted_issuer_did_2]
        ),
        GTMError::<TestStorage>::Unauthorized
    );

    // Failed because trusted issuer identity not exist
    assert_err!(
        GeneralTM::add_default_trusted_claim_issuers_batch(
            token_owner_signed.clone(),
            ticker,
            vec![IdentityId::from(1), IdentityId::from(2)]
        ),
        GTMError::<TestStorage>::DidNotExist
    );

    // Failed because trusted issuers length < 0
    assert_err!(
        GeneralTM::add_default_trusted_claim_issuers_batch(
            token_owner_signed.clone(),
            ticker,
            vec![]
        ),
        GTMError::<TestStorage>::InvalidLength
    );

    assert_ok!(GeneralTM::add_default_trusted_claim_issuers_batch(
        token_owner_signed.clone(),
        ticker,
        vec![trusted_issuer_did_1, trusted_issuer_did_2]
    ));

    assert_eq!(GeneralTM::trusted_claim_issuer(ticker).len(), 2);
    assert_eq!(
        GeneralTM::trusted_claim_issuer(ticker),
        vec![trusted_issuer_did_1, trusted_issuer_did_2]
    );

    // adding claim by trusted issuer 1
    assert_ok!(Identity::add_claim(
        trusted_issuer_signed_1.clone(),
        receiver_did.clone(),
        Claim::CustomerDueDiligence,
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
        Claim::CustomerDueDiligence,
        None,
    ));

    let now = Utc::now();
    Timestamp::set_timestamp(now.timestamp() as u64);

    let sender_rule = Rule {
        issuers: vec![],
        rule_type: RuleType::IsPresent(Claim::CustomerDueDiligence),
    };

    let receiver_rule_1 = Rule {
        issuers: vec![],
        rule_type: RuleType::IsPresent(Claim::CustomerDueDiligence),
    };

    let receiver_rule_2 = Rule {
        issuers: vec![],
        rule_type: RuleType::IsPresent(Claim::NoData),
    };

    let x = vec![sender_rule.clone()];
    let y = vec![receiver_rule_1, receiver_rule_2];

    assert_ok!(GeneralTM::add_active_rule(
        token_owner_signed.clone(),
        ticker,
        x,
        y
    ));

    assert_ok!(Asset::transfer(
        token_owner_signed.clone(),
        ticker,
        receiver_did.clone(),
        100
    ));

    // Remove the trusted issuer 1 from the list
    assert_ok!(GeneralTM::remove_default_trusted_claim_issuers_batch(
        token_owner_signed.clone(),
        ticker,
        vec![trusted_issuer_did_1]
    ));

    assert_eq!(GeneralTM::trusted_claim_issuer(ticker).len(), 1);
    assert_eq!(
        GeneralTM::trusted_claim_issuer(ticker),
        vec![trusted_issuer_did_2]
    );

    // Transfer should fail as issuer doesn't exist anymore but the rule data still exist
    assert_err!(
        Asset::transfer(
            token_owner_signed.clone(),
            ticker,
            receiver_did.clone(),
            500
        ),
        AssetError::<TestStorage>::InvalidTransfer
    );

    // Change the asset rule to all the transfer happen again

    let receiver_rule_1 = Rule {
        issuers: vec![trusted_issuer_did_1],
        rule_type: RuleType::IsPresent(Claim::CustomerDueDiligence),
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
        GeneralTM::change_asset_rule(receiver_signed.clone(), ticker, asset_rule.clone()),
        GTMError::<TestStorage>::Unauthorized
    );

    let asset_rule_failure = AssetTransferRule {
        sender_rules: x,
        receiver_rules: y,
        rule_id: 5,
    };

    // Failed because passed rule id is not valid
    assert_err!(
        GeneralTM::change_asset_rule(
            token_owner_signed.clone(),
            ticker,
            asset_rule_failure.clone()
        ),
        GTMError::<TestStorage>::InvalidRuleId
    );

    // Should successfully change the asset rule
    assert_ok!(GeneralTM::change_asset_rule(
        token_owner_signed.clone(),
        ticker,
        asset_rule
    ));

    // Now the transfer should pass
    assert_ok!(Asset::transfer(
        token_owner_signed.clone(),
        ticker,
        receiver_did.clone(),
        500
    ));
}

#[test]
fn jurisdiction_asset_rules() {
    ExtBuilder::default()
        .build()
        .execute_with(jurisdiction_asset_rules_we);
}
fn jurisdiction_asset_rules_we() {
    // 0. Create accounts
    let token_owner_acc = AccountKeyring::Alice.public();
    let (token_owner_signed, token_owner_id) = make_account(token_owner_acc).unwrap();
    let cdd_acc = AccountKeyring::Bob.public();
    let (cdd_signed, cdd_id) = make_account(cdd_acc).unwrap();
    let user_acc = AccountKeyring::Charlie.public();
    let (_, user_id) = make_account(user_acc).unwrap();
    // 1. Create a token.
    let token = SecurityToken {
        name: vec![0x01].into(),
        owner_did: token_owner_id.clone(),
        total_supply: 1_000_000,
        divisible: true,
        ..Default::default()
    };
    let ticker = Ticker::from(token.name.0.as_slice());
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
    // 2. Set up rules for Asset transfer.
    let scope = Scope::from(0);
    let receiver_rules = vec![
        Rule {
            rule_type: RuleType::IsAnyOf(vec![
                Claim::Jurisdiction(b"Canada".into(), scope),
                Claim::Jurisdiction(b"Spain".into(), scope),
            ]),
            issuers: vec![cdd_id],
        },
        Rule {
            rule_type: RuleType::IsAbsent(Claim::BlackListed(scope)),
            issuers: vec![token_owner_id],
        },
    ];
    assert_ok!(GeneralTM::add_active_rule(
        token_owner_signed.clone(),
        ticker,
        vec![],
        receiver_rules
    ));
    // 3. Validate behaviour.
    // 3.1. Invalid transfer because missing jurisdiction.
    assert_err!(
        Asset::transfer(token_owner_signed.clone(), ticker, user_id, 100),
        AssetError::<TestStorage>::InvalidTransfer
    );
    // 3.2. Add jurisdiction and transfer will be OK.
    assert_ok!(Identity::add_claim(
        cdd_signed.clone(),
        user_id,
        Claim::Jurisdiction(b"Canada".into(), scope),
        None
    ));
    assert_ok!(Asset::transfer(
        token_owner_signed.clone(),
        ticker,
        user_id,
        100
    ));
    // 3.3. Add user to blacklist
    assert_ok!(Identity::add_claim(
        token_owner_signed.clone(),
        user_id,
        Claim::BlackListed(scope),
        None,
    ));
    assert_err!(
        Asset::transfer(token_owner_signed.clone(), ticker, user_id, 100),
        AssetError::<TestStorage>::InvalidTransfer
    );
}

#[test]
fn scope_asset_rules() {
    ExtBuilder::default()
        .build()
        .execute_with(scope_asset_rules_we);
}
fn scope_asset_rules_we() {
    // 0. Create accounts
    let token_owner_acc = AccountKeyring::Alice.public();
    let (token_owner_signed, token_owner_id) = make_account(token_owner_acc).unwrap();
    let cdd_acc = AccountKeyring::Bob.public();
    let (cdd_signed, cdd_id) = make_account(cdd_acc).unwrap();
    let user_acc = AccountKeyring::Charlie.public();
    let (_, user_id) = make_account(user_acc).unwrap();
    // 1. Create a token.
    let token = SecurityToken {
        name: vec![0x01].into(),
        owner_did: token_owner_id.clone(),
        total_supply: 1_000_000,
        divisible: true,
        ..Default::default()
    };
    let ticker = Ticker::from(token.name.0.as_slice());
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
    // 2. Set up rules for Asset transfer.
    let scope = Identity::get_token_did(&ticker).unwrap();
    let receiver_rules = vec![Rule {
        rule_type: RuleType::IsPresent(Claim::Affiliate(scope)),
        issuers: vec![cdd_id],
    }];
    assert_ok!(GeneralTM::add_active_rule(
        token_owner_signed.clone(),
        ticker,
        vec![],
        receiver_rules
    ));
    // 3. Validate behaviour.
    // 3.1. Invalid transfer because missing jurisdiction.
    assert_err!(
        Asset::transfer(token_owner_signed.clone(), ticker, user_id, 100),
        AssetError::<TestStorage>::InvalidTransfer
    );
    // 3.2. Add jurisdiction and transfer will be OK.
    assert_ok!(Identity::add_claim(
        cdd_signed.clone(),
        user_id,
        Claim::Affiliate(scope),
        None
    ));
    assert_ok!(Asset::transfer(
        token_owner_signed.clone(),
        ticker,
        user_id,
        100
    ));
}
