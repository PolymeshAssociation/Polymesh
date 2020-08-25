use super::{
    storage::{register_keyring_account, TestStorage},
    ExtBuilder,
};
use pallet_asset::{self as asset, AssetType};
use pallet_basic_sto::{self as sto, Fundraiser};
use pallet_compliance_manager as compliance_manager;
use pallet_settlement::{self as settlement, VenueDetails};
use polymesh_primitives::Ticker;

use frame_support::assert_ok;
use sp_std::convert::TryFrom;
use test_client::AccountKeyring;

type Origin = <TestStorage as frame_system::Trait>::Origin;
type Asset = asset::Module<TestStorage>;
type STO = sto::Module<TestStorage>;
type ComplianceManager = compliance_manager::Module<TestStorage>;
type Settlement = settlement::Module<TestStorage>;

#[test]
fn basic_raise() {
    ExtBuilder::default()
        .build()
        .execute_with(|| basic_raise_with_ext);
}

fn basic_raise_with_ext() {
    let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
    let alice_signed = Origin::signed(AccountKeyring::Alice.public());
    let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
    let bob_signed = Origin::signed(AccountKeyring::Bob.public());

    // Register tokens
    let ticker = Ticker::try_from(&[0x01][..]).unwrap();
    let raise_ticker = Ticker::try_from(&[0x02][..]).unwrap();
    assert_ok!(Asset::create_asset(
        alice_signed.clone(),
        vec![0x01].into(),
        ticker,
        1_000_000, // Total supply over the limit
        true,
        AssetType::default(),
        vec![],
        None,
    ));

    assert_ok!(Asset::create_asset(
        bob_signed.clone(),
        vec![0x01].into(),
        raise_ticker,
        1_000_000, // Total supply over the limit
        true,
        AssetType::default(),
        vec![],
        None,
    ));

    // Add empty compliance requirements
    assert_ok!(ComplianceManager::add_compliance_requirement(
        alice_signed.clone(),
        ticker,
        vec![],
        vec![]
    ));
    assert_ok!(ComplianceManager::add_compliance_requirement(
        bob_signed.clone(),
        raise_ticker,
        vec![],
        vec![]
    ));

    // Register a venue
    let venue_counter = Settlement::venue_counter();
    assert_ok!(Settlement::create_venue(
        alice_signed.clone(),
        VenueDetails::default(),
        vec![]
    ));

    let amount = 100u128;
    let alice_init_balance = Asset::balance_of(&ticker, alice_did);
    let bob_init_balance = Asset::balance_of(&ticker, bob_did);
    let alice_init_balance2 = Asset::balance_of(&raise_ticker, alice_did);
    let bob_init_balance2 = Asset::balance_of(&raise_ticker, bob_did);

    // Alice starts a fundraiser
    assert_ok!(STO::create_fundraiser(
        alice_signed.clone(),
        ticker,
        raise_ticker,
        amount,
        1_000_000u128,
        venue_counter
    ));
    assert_eq!(
        STO::fundraisers(ticker, 1),
        Fundraiser {
            raise_token: raise_ticker,
            remaining_amount: amount,
            price_per_token: 1_000_000u128,
            venue_id: venue_counter
        }
    );

    assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
    assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);
    assert_eq!(
        Asset::balance_of(&raise_ticker, alice_did),
        alice_init_balance2
    );
    assert_eq!(Asset::balance_of(&raise_ticker, bob_did), bob_init_balance2);

    // Bob invests in Alice's fundraiser
    assert_ok!(STO::invest(bob_signed.clone(), ticker, 1, amount));
    assert_eq!(
        STO::fundraisers(ticker, 1),
        Fundraiser {
            raise_token: raise_ticker,
            remaining_amount: 0u128,
            price_per_token: 1_000_000u128,
            venue_id: venue_counter
        }
    );
    assert_eq!(
        Asset::balance_of(&ticker, alice_did),
        alice_init_balance - amount
    );
    assert_eq!(
        Asset::balance_of(&ticker, bob_did),
        bob_init_balance + amount
    );
    assert_eq!(
        Asset::balance_of(&raise_ticker, alice_did),
        alice_init_balance2 + amount
    );
    assert_eq!(
        Asset::balance_of(&raise_ticker, bob_did),
        bob_init_balance2 - amount
    );
}
