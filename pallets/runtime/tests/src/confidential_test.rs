use super::{
    storage::{register_keyring_account, TestStorage},
    ExtBuilder,
};

use pallet_asset::{self as asset, AssetType, IdentifierType, SecurityToken};
use pallet_confidential as confidential;
use polymesh_primitives::Ticker;

use core::convert::TryFrom;
use frame_support::assert_ok;
use test_client::AccountKeyring;

type Asset = asset::Module<TestStorage>;
type Confidential = confidential::Module<TestStorage>;
type Origin = <TestStorage as frame_system::Trait>::Origin;
use cryptography::{
    asset_proofs::{CommitmentWitness, ElgamalSecretKey},
    mercat::{
        account::{convert_asset_ids, AccountCreator},
        AccountCreatorInitializer, EncryptionKeys, PubAccountTx, SecAccount,
    },
    AssetId,
};
use curve25519_dalek::scalar::Scalar;
use rand::{rngs::StdRng, SeedableRng};
use schnorrkel::{ExpansionMode, MiniSecretKey};

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
        None
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
fn account_create_tx() {
    ExtBuilder::default()
        .build()
        .execute_with(account_create_tx_we);
}

fn account_create_tx_we() {
    let alice = AccountKeyring::Alice.public();
    let alice_id = register_keyring_account(AccountKeyring::Alice).unwrap();

    let valid_asset_ids: Vec<AssetId> = vec![1, 2, 3]
        .iter()
        .map(|id| AssetId::from(id.clone()))
        .collect();
    Confidential::store_valid_asset_ids(Origin::signed(alice), valid_asset_ids.clone());

    let mut rng = StdRng::from_seed([10u8; 32]);
    let elg_secret = ElgamalSecretKey::new(Scalar::random(&mut rng));
    let elg_pub = elg_secret.get_public_key();
    let enc_keys = EncryptionKeys {
        pblc: elg_pub.into(),
        scrt: elg_secret.into(),
    };

    let sign_keys = MiniSecretKey::from_bytes(&[11u8; 32])
        .expect("Invalid seed")
        .expand_to_keypair(ExpansionMode::Ed25519);

    let asset_id = AssetId::from(1);
    let asset_id_witness = CommitmentWitness::from((asset_id.clone().into(), &mut rng));
    let scrt_account = SecAccount {
        enc_keys,
        sign_keys,
        asset_id_witness,
    };

    let account_creator = AccountCreator {};
    let valid_asset_ids = convert_asset_ids(valid_asset_ids);
    let account_id = 0;
    let mercat_tx_id = 0;
    let a = account_creator
        .create(
            mercat_tx_id,
            &scrt_account,
            &valid_asset_ids,
            account_id,
            &mut rng,
        )
        .unwrap();

    Confidential::store_account_tx(Origin::signed(alice), a.clone());

    assert_eq!(Confidential::mercat_accounts(alice_id), vec![a.pub_account]);
}
