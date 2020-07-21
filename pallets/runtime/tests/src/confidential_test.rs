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
