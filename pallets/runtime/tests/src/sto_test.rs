use super::{
    storage::{register_keyring_account, TestStorage},
    ExtBuilder,
};
use pallet_asset::{self as asset, AssetType};
use pallet_compliance_manager as compliance_manager;
use pallet_settlement::{self as settlement, Receipt, ReceiptDetails, VenueDetails, VenueType};
use pallet_sto::{self as sto, Fundraiser, FundraiserTier, PriceTier};
use polymesh_primitives::{PortfolioId, Ticker};

use frame_support::assert_ok;
use sp_std::convert::TryFrom;
use test_client::AccountKeyring;

type Origin = <TestStorage as frame_system::Trait>::Origin;
type Asset = asset::Module<TestStorage>;
type STO = sto::Module<TestStorage>;
type ComplianceManager = compliance_manager::Module<TestStorage>;
type Settlement = settlement::Module<TestStorage>;
type Timestamp = pallet_timestamp::Module<TestStorage>;

#[test]
fn basic_raise() {
    ExtBuilder::default()
        .build()
        .execute_with(|| raise_happy_path);
}

fn raise_happy_path() {
    let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
    let alice_signed = Origin::signed(AccountKeyring::Alice.public());
    let alice = AccountKeyring::Alice.public();
    let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
    let bob_signed = Origin::signed(AccountKeyring::Bob.public());
    let bob = AccountKeyring::Bob.public();

    // Register tokens
    let offering_ticker = Ticker::try_from(&[0x01][..]).unwrap();
    let raise_ticker = Ticker::try_from(&[0x02][..]).unwrap();
    assert_ok!(Asset::create_asset(
        alice_signed.clone(),
        vec![0x01].into(),
        offering_ticker,
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
        offering_ticker,
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
        vec![alice],
        VenueType::Other
    ));

    let amount = 100u128;
    let alice_init_balance = Asset::balance_of(&offering_ticker, alice_did);
    let bob_init_balance = Asset::balance_of(&offering_ticker, bob_did);
    let alice_init_balance2 = Asset::balance_of(&raise_ticker, alice_did);
    let bob_init_balance2 = Asset::balance_of(&raise_ticker, bob_did);

    // Alice starts a fundraiser
    let fundraiser_id = STO::fundraiser_count(offering_ticker);
    assert_ok!(STO::create_fundraiser(
        alice_signed.clone(),
        PortfolioId::default_portfolio(alice_did),
        offering_ticker,
        PortfolioId::default_portfolio(alice_did),
        raise_ticker,
        vec![PriceTier {
            total: 1_000_000u128,
            price: 1u128
        }],
        venue_counter,
        None,
        None,
    ));
    assert_eq!(
        STO::fundraisers(offering_ticker, 1),
        Some(Fundraiser {
            offering_portfolio: PortfolioId::default_portfolio(alice_did),
            offering_asset: offering_ticker,
            raising_portfolio: PortfolioId::default_portfolio(alice_did),
            raising_asset: raise_ticker,
            tiers: vec![FundraiserTier {
                total: 1_000_000u128,
                remaining: 1_000_000u128,
                price: 1u128
            }],
            venue_id: venue_counter,
            start: Timestamp::get(),
            end: None,
            frozen: false
        })
    );

    assert_eq!(
        Asset::balance_of(&offering_ticker, alice_did),
        alice_init_balance
    );
    assert_eq!(
        Asset::balance_of(&offering_ticker, bob_did),
        bob_init_balance
    );
    assert_eq!(
        Asset::balance_of(&raise_ticker, alice_did),
        alice_init_balance2
    );
    assert_eq!(Asset::balance_of(&raise_ticker, bob_did), bob_init_balance2);

    // Bob invests in Alice's fundraiser
    assert_ok!(STO::invest(
        bob_signed.clone(),
        PortfolioId::default_portfolio(bob_did),
        PortfolioId::default_portfolio(bob_did),
        offering_ticker,
        fundraiser_id,
        amount.into(),
        2u128,
        None
    ));
    assert_eq!(
        STO::fundraisers(offering_ticker, 1),
        Some(Fundraiser {
            offering_portfolio: PortfolioId::default_portfolio(alice_did),
            offering_asset: offering_ticker,
            raising_portfolio: PortfolioId::default_portfolio(alice_did),
            raising_asset: raise_ticker,
            tiers: vec![FundraiserTier {
                total: 1_000_000u128,
                remaining: (1_000_000 - amount).into(),
                price: 1u128
            }],
            venue_id: venue_counter,
            start: Timestamp::get(),
            end: None,
            frozen: false
        })
    );
    assert_eq!(
        Asset::balance_of(&offering_ticker, alice_did),
        alice_init_balance - amount
    );
    assert_eq!(
        Asset::balance_of(&offering_ticker, bob_did),
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
