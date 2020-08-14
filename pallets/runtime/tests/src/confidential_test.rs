use super::{
    storage::{make_account_with_balance, register_keyring_account, TestStorage},
    ExtBuilder,
};

use cryptography::{
    asset_proofs::{CommitmentWitness, ElgamalSecretKey},
    claim_proofs::{compute_cdd_id, compute_scope_id},
    mercat::account::{convert_asset_ids, AccountCreator},
    mercat::asset::AssetIssuer,
    mercat::{
        Account, AccountCreatorInitializer, AssetTransactionIssuer, EncryptionKeys, SecAccount,
    },
    AssetId,
};
use curve25519_dalek::scalar::Scalar;
use pallet_asset::{self as asset, AssetType, IdentifierType, SecurityToken};
use pallet_compliance_manager as compliance_manager;
use pallet_confidential as confidential;
use pallet_identity::{self as identity, Error};
use polymesh_primitives::{
    Claim, IdentityId, InvestorUid, InvestorZKProofData, Rule, RuleType, Ticker,
};

use core::convert::TryFrom;
use frame_support::{assert_err, assert_ok};
use rand::rngs::StdRng;
use rand::SeedableRng;
use test_client::AccountKeyring;

type Identity = identity::Module<TestStorage>;
type Asset = asset::Module<TestStorage>;
type AssetError = asset::Error<TestStorage>;
type Confidential = confidential::Module<TestStorage>;
type ComplianceManager = compliance_manager::Module<TestStorage>;
type Origin = <TestStorage as frame_system::Trait>::Origin;

#[test]
fn range_proof() {
    ExtBuilder::default().build().execute_with(range_proof_we);
}

fn range_proof_we() {
    let alice = AccountKeyring::Alice.public();
    let prover = AccountKeyring::Bob.public();
    let verifier = AccountKeyring::Charlie.public();
    let alice_id = register_keyring_account(AccountKeyring::Alice).unwrap();
    let prover_id = register_keyring_account(AccountKeyring::Bob).unwrap();
    let verifier_id = register_keyring_account(AccountKeyring::Charlie).unwrap();

    // 1. Alice creates her security token.
    let token = SecurityToken {
        name: "ALI_ST".as_bytes().to_owned().into(),
        owner_did: alice_id,
        total_supply: 1_000_000,
        divisible: true,
        asset_type: AssetType::default(),
        ..Default::default()
    };
    let identifiers = vec![(IdentifierType::Isin, b"0123".into())];
    let ticker = Ticker::try_from(token.name.as_slice()).unwrap();

    assert_ok!(Asset::create_asset(
        Origin::signed(alice),
        token.name.clone(),
        ticker,
        token.total_supply,
        true,
        token.asset_type.clone(),
        identifiers.clone(),
        None,
    ));

    // 2. X add a range proof
    let secret_value = 42;
    assert_ok!(Confidential::add_range_proof(
        Origin::signed(prover),
        alice_id,
        ticker.clone(),
        secret_value,
    ));

    assert_ok!(Confidential::add_verify_range_proof(
        Origin::signed(verifier),
        alice_id,
        prover_id,
        ticker.clone()
    ));

    assert_eq!(
        Confidential::range_proof_verification((alice_id, ticker), verifier_id),
        true
    );
}

#[test]
fn scope_claims() {
    ExtBuilder::default()
        .monied(true)
        .cdd_providers(vec![AccountKeyring::Eve.public()])
        .build()
        .execute_with(scope_claims_we);
}

fn scope_claims_we() {
    let alice = AccountKeyring::Alice.public();
    let alice_id = register_keyring_account(AccountKeyring::Alice).unwrap();
    let investor = InvestorUid::from("inv_1");
    let inv_acc_1 = AccountKeyring::Bob.public();
    let (_, inv_did_1) = make_account_with_balance(inv_acc_1, investor, 1_000_000).unwrap();
    let inv_acc_2 = AccountKeyring::Charlie.public();
    let (_, inv_did_2) = make_account_with_balance(inv_acc_2, investor, 2_000_000).unwrap();
    let other_investor = InvestorUid::from("inv_2");
    let inv_acc_3 = AccountKeyring::Dave.public();
    let (_, inv_did_3) = make_account_with_balance(inv_acc_3, other_investor, 3_000_000).unwrap();

    // 1. Alice creates her ST and set up its rules.
    let st = SecurityToken {
        name: "ALI_ST".as_bytes().to_owned().into(),
        owner_did: alice_id,
        total_supply: 1_000_000,
        divisible: true,
        asset_type: AssetType::default(),
        ..Default::default()
    };
    let identifiers = vec![(IdentifierType::Isin, b"0123".into())];
    let st_id = Ticker::try_from(st.name.as_slice()).unwrap();

    assert_ok!(Asset::create_asset(
        Origin::signed(alice),
        st.name.clone(),
        st_id,
        st.total_supply,
        true,
        st.asset_type.clone(),
        identifiers.clone(),
        None,
    ));

    // 2. Alice defines the asset complain rules.
    let st_scope = IdentityId::try_from(st_id.as_slice()).unwrap();
    let sender_rules = vec![];
    let receiver_rules = vec![Rule::from(RuleType::HasValidProofOfInvestor(st_id))];
    assert_ok!(ComplianceManager::add_active_rule(
        Origin::signed(alice),
        st_id,
        sender_rules,
        receiver_rules
    ));

    // 2. Investor adds its Confidential Scope claims.
    let scope_claim = InvestorZKProofData::make_scope_claim(&st_id, &investor);
    let scope_id = compute_scope_id(&scope_claim).compress().to_bytes().into();

    let inv_1_proof = InvestorZKProofData::new(&inv_did_1, &investor, &st_id);
    let cdd_claim_1 = InvestorZKProofData::make_cdd_claim(&inv_did_1, &investor);
    let cdd_id_1 = compute_cdd_id(&cdd_claim_1).compress().to_bytes().into();

    let conf_scope_claim_1 =
        Claim::InvestorZKProof(st_scope, scope_id, cdd_id_1, inv_1_proof.clone());

    assert_ok!(Identity::add_claim(
        Origin::signed(inv_acc_1),
        inv_did_1,
        conf_scope_claim_1.clone(),
        None
    ));

    let inv_2_proof = InvestorZKProofData::new(&inv_did_2, &investor, &st_id);
    let cdd_claim_2 = InvestorZKProofData::make_cdd_claim(&inv_did_2, &investor);
    let cdd_id_2 = compute_cdd_id(&cdd_claim_2).compress().to_bytes().into();

    let conf_scope_claim_2 = Claim::InvestorZKProof(st_scope, scope_id, cdd_id_2, inv_2_proof);
    assert_ok!(Identity::add_claim(
        Origin::signed(inv_acc_2),
        inv_did_2,
        conf_scope_claim_2,
        None
    ));

    // 3. Transfer some tokens to Inv. 1 and 2.
    assert_eq!(Asset::balance_of(st_id, inv_did_1), 0);
    assert_ok!(Asset::transfer(Origin::signed(alice), st_id, inv_did_1, 10));
    assert_eq!(Asset::balance_of(st_id, inv_did_1), 10);

    assert_eq!(Asset::balance_of(st_id, inv_did_2), 0);
    assert_ok!(Asset::transfer(Origin::signed(alice), st_id, inv_did_2, 20));
    assert_eq!(Asset::balance_of(st_id, inv_did_2), 20);

    // 4. ERROR: Investor 2 cannot add a claim of the real investor.
    assert_err!(
        Identity::add_claim(
            Origin::signed(inv_acc_3),
            inv_did_3,
            conf_scope_claim_1,
            None
        ),
        Error::<TestStorage>::ConfidentialScopeClaimNotAllowed
    );

    // 5. ERROR: Replace the scope
    let st_2 = SecurityToken {
        name: "ALI2_ST".as_bytes().to_owned().into(),
        owner_did: alice_id,
        total_supply: 1_000_000,
        divisible: true,
        asset_type: AssetType::default(),
        ..Default::default()
    };
    let identifiers = vec![(IdentifierType::Isin, b"X123".into())];
    let st2_id = Ticker::try_from(st_2.name.as_slice()).unwrap();

    assert_ok!(Asset::create_asset(
        Origin::signed(alice),
        st_2.name.clone(),
        st2_id,
        st_2.total_supply,
        true,
        st_2.asset_type.clone(),
        identifiers.clone(),
        None,
    ));

    let st_scope = IdentityId::try_from(st2_id.as_slice()).unwrap();
    let corrupted_scope_claim = InvestorZKProofData::make_scope_claim(&st2_id, &investor);
    let corrupted_scope_id = compute_scope_id(&corrupted_scope_claim)
        .compress()
        .to_bytes()
        .into();

    let conf_scope_claim_3 =
        Claim::InvestorZKProof(st_scope, corrupted_scope_id, cdd_id_1, inv_1_proof);
    assert_ok!(Identity::add_claim(
        Origin::signed(inv_acc_1),
        inv_did_1,
        conf_scope_claim_3.clone(),
        None
    ));
    assert_err!(
        Asset::transfer(Origin::signed(alice), st2_id, inv_did_1, 10),
        AssetError::InvalidTransfer
    );
}

#[test]
fn asset_issuance() {
    ExtBuilder::default()
        .build()
        .execute_with(asset_issuance_we);
}

fn asset_issuance_we() {
    let mut rng = StdRng::from_seed([10u8; 32]);
    let alice = AccountKeyring::Alice.public();
    // let mediator = AccountKeyring::Bob.public();
    let validator = AccountKeyring::Charlie.public();
    let alice_id = register_keyring_account(AccountKeyring::Alice).unwrap();
    let mediator_id = register_keyring_account(AccountKeyring::Bob).unwrap();

    let mediator_elg_secret_key = ElgamalSecretKey::new(Scalar::random(&mut rng));
    let mediator_enc_pub_key = mediator_elg_secret_key.get_public_key();

    // 1. Alice creates her security token.
    // ToDo: This has to change to a confidential security token type.
    let token = SecurityToken {
        name: "ALI_ST".as_bytes().to_owned().into(),
        owner_did: alice_id,
        total_supply: 1_000_000,
        divisible: true,
        asset_type: AssetType::default(),
        ..Default::default()
    };
    let identifiers = vec![(IdentifierType::Isin, b"0123".into())];
    let ticker = Ticker::try_from(token.name.as_slice()).unwrap();

    assert_ok!(Asset::create_asset(
        Origin::signed(alice),
        token.name.clone(),
        ticker,
        token.total_supply,
        true,
        token.asset_type.clone(),
        identifiers.clone(),
        None,
    ));

    // 2. Alice issues some assets to herself.
    let issuer = AssetIssuer {};
    let issuer_account = make_confidential_account(
        cryptography::asset_id_from_ticker(std::str::from_utf8(&token.name).unwrap()).unwrap(),
    );
    let tx_id = 1;
    let conf_tx_id = confidential::TransactionId(tx_id);
    let balance = 100;
    let tx = issuer
        .initialize_asset_transaction(
            tx_id,
            &issuer_account,
            &mediator_enc_pub_key,
            &[],
            balance,
            &mut rng,
        )
        .unwrap();
    assert_ok!(Confidential::add_asset_issuance_transaction(
        Origin::signed(alice),
        ticker.clone(),
        conf_tx_id.clone(),
        tx
    ));

    // 3. Mediator approves it.
    // let mediator = AssetMediator {};

    // 4. Validator verifies the transaction and updates the issuer's account.
    assert_ok!(Confidential::validate_asset_issuance_transaction(
        Origin::signed(validator),
        alice_id,
        ticker.clone(),
        conf_tx_id.clone(),
        mediator_id,
    ));

    assert_eq!(
        Confidential::asset_transaction_validation((alice_id, ticker), conf_tx_id.clone()),
        true
    );
}

fn make_confidential_account(asset_id: AssetId) -> Account {
    let mut rng = StdRng::from_seed([10u8; 32]);
    let issuer_elg_secret_key = ElgamalSecretKey::new(Scalar::random(&mut rng));
    let issuer_enc_key = EncryptionKeys {
        pblc: issuer_elg_secret_key.get_public_key().into(),
        scrt: issuer_elg_secret_key.into(),
    };

    let issuer_secret_account = SecAccount {
        enc_keys: issuer_enc_key.clone(),
        asset_id_witness: CommitmentWitness::from((asset_id.clone().into(), &mut rng)),
    };

    let account_id = 1234u32;
    let mut valid_asset_ids: Vec<AssetId> = vec![1, 2, 3]
        .iter()
        .map(|id| AssetId::from(id.clone()))
        .collect();
    // Add the given asset id to the list. This could be prettier.
    valid_asset_ids.push(asset_id);
    let valid_asset_ids = convert_asset_ids(valid_asset_ids);

    let account_creator = AccountCreator {};
    let tx_id = 0;
    let issuer_account_tx = account_creator
        .create(
            tx_id,
            &issuer_secret_account,
            &valid_asset_ids,
            account_id,
            &mut rng,
        )
        .unwrap();

    Account {
        pblc: issuer_account_tx.pub_account.clone(),
        scrt: issuer_secret_account,
    }
}
