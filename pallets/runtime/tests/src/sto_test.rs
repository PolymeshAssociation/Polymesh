use super::{
    storage::{register_keyring_account, TestStorage},
    ExtBuilder,
};
use pallet_asset::{self as asset, AssetType};
use pallet_compliance_manager as compliance_manager;
use pallet_settlement::{self as settlement, VenueDetails, VenueType};
use pallet_sto::{self as sto, Fundraiser, FundraiserTier, PriceTier};
use polymesh_primitives::{PortfolioId, Ticker};

use crate::storage::provide_scope_claim_to_multiple_parties;
use frame_support::{assert_noop, assert_ok};
use sp_runtime::DispatchError;
use sp_std::convert::TryFrom;
use test_client::AccountKeyring;

type Origin = <TestStorage as frame_system::Trait>::Origin;
type Asset = asset::Module<TestStorage>;
type STO = sto::Module<TestStorage>;
type Error = sto::Error<TestStorage>;
type PortfolioError = pallet_portfolio::Error<TestStorage>;
type ComplianceManager = compliance_manager::Module<TestStorage>;
type Settlement = settlement::Module<TestStorage>;
type Timestamp = pallet_timestamp::Module<TestStorage>;

#[test]
fn raise_happy_path_ext() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Eve.public()])
        .set_max_legs_allowed(2)
        .build()
        .execute_with(raise_happy_path);
}
#[test]
fn raise_unhappy_path_ext() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Eve.public()])
        .set_max_legs_allowed(2)
        .build()
        .execute_with(raise_unhappy_path);
}

fn create_asset(origin: Origin, ticker: Ticker, supply: u128) {
    assert_ok!(Asset::create_asset(
        origin,
        vec![0x01].into(),
        ticker,
        supply,
        true,
        AssetType::default(),
        vec![],
        None,
    ));
}

fn empty_compliance(origin: Origin, ticker: Ticker) {
    assert_ok!(ComplianceManager::add_compliance_requirement(
        origin,
        ticker,
        vec![],
        vec![]
    ));
}

fn raise_happy_path() {
    let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
    let alice_signed = Origin::signed(AccountKeyring::Alice.public());
    let alice = AccountKeyring::Alice.public();
    let alice_portfolio = PortfolioId::default_portfolio(alice_did);
    let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
    let bob_signed = Origin::signed(AccountKeyring::Bob.public());
    let _bob = AccountKeyring::Bob.public();
    let bob_portfolio = PortfolioId::default_portfolio(bob_did);

    // Register tokens
    let offering_ticker = Ticker::try_from(&[0x01][..]).unwrap();
    let raise_ticker = Ticker::try_from(&[0x02][..]).unwrap();
    create_asset(alice_signed.clone(), offering_ticker, 1_000_000);
    create_asset(alice_signed.clone(), raise_ticker, 1_000_000);

    // Provide scope claim to both the parties of the transaction.
    let eve = AccountKeyring::Eve.public();
    provide_scope_claim_to_multiple_parties(&[alice_did, bob_did], offering_ticker, eve);
    provide_scope_claim_to_multiple_parties(&[alice_did, bob_did], raise_ticker, eve);

    assert_ok!(Asset::unsafe_transfer(
        alice_portfolio,
        bob_portfolio,
        &raise_ticker,
        1_000_000
    ));

    empty_compliance(alice_signed.clone(), offering_ticker);
    empty_compliance(alice_signed.clone(), raise_ticker);

    // Register a venue
    let venue_counter = Settlement::venue_counter();
    assert_ok!(Settlement::create_venue(
        alice_signed.clone(),
        VenueDetails::default(),
        vec![alice],
        VenueType::Sto
    ));

    let amount = 100u128;
    let alice_init_offering = Asset::balance_of(&offering_ticker, alice_did);
    let bob_init_offering = Asset::balance_of(&offering_ticker, bob_did);
    let alice_init_raise = Asset::balance_of(&raise_ticker, alice_did);
    let bob_init_raise = Asset::balance_of(&raise_ticker, bob_did);

    // Alice starts a fundraiser
    let fundraiser_id = STO::fundraiser_count(offering_ticker);
    assert_ok!(STO::create_fundraiser(
        alice_signed.clone(),
        alice_portfolio,
        offering_ticker,
        alice_portfolio,
        raise_ticker,
        vec![PriceTier {
            total: 1_000_000u128,
            price: 1u128
        }],
        venue_counter,
        None,
        None,
    ));

    let check_fundraiser = |remaining| {
        assert_eq!(
            STO::fundraisers(offering_ticker, fundraiser_id),
            Some(Fundraiser {
                creator: alice_did,
                offering_portfolio: alice_portfolio,
                offering_asset: offering_ticker,
                raising_portfolio: alice_portfolio,
                raising_asset: raise_ticker,
                tiers: vec![FundraiserTier {
                    total: 1_000_000u128,
                    remaining,
                    price: 1u128
                }],
                venue_id: venue_counter,
                start: Timestamp::get(),
                end: None,
                frozen: false
            })
        );
    };

    check_fundraiser(1_000_000u128);

    assert_eq!(
        Asset::balance_of(&offering_ticker, alice_did),
        alice_init_offering
    );
    assert_eq!(
        Asset::balance_of(&offering_ticker, bob_did),
        bob_init_offering
    );
    assert_eq!(
        Asset::balance_of(&raise_ticker, alice_did),
        alice_init_raise
    );
    assert_eq!(Asset::balance_of(&raise_ticker, bob_did), bob_init_raise);

    // Bob invests in Alice's fundraiser
    assert_ok!(STO::invest(
        bob_signed.clone(),
        bob_portfolio,
        bob_portfolio,
        offering_ticker,
        fundraiser_id,
        amount.into(),
        Some(2u128),
        None
    ));

    check_fundraiser(1_000_000u128 - amount);

    assert_eq!(
        Asset::balance_of(&offering_ticker, alice_did),
        alice_init_offering - amount
    );
    assert_eq!(
        Asset::balance_of(&offering_ticker, bob_did),
        bob_init_offering + amount
    );
    assert_eq!(
        Asset::balance_of(&raise_ticker, alice_did),
        alice_init_raise + amount
    );
    assert_eq!(
        Asset::balance_of(&raise_ticker, bob_did),
        bob_init_raise - amount
    );
}

fn raise_unhappy_path() {
    let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
    let alice_signed = Origin::signed(AccountKeyring::Alice.public());
    let alice = AccountKeyring::Alice.public();
    let alice_portfolio = PortfolioId::default_portfolio(alice_did);
    let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
    let bob_signed = Origin::signed(AccountKeyring::Bob.public());
    let _bob = AccountKeyring::Bob.public();
    let bob_portfolio = PortfolioId::default_portfolio(bob_did);

    let offering_ticker = Ticker::try_from(&[0x03][..]).unwrap();
    let raise_ticker = Ticker::try_from(&[0x04][..]).unwrap();

    // Provide scope claim to both the parties of the transaction.
    let eve = AccountKeyring::Eve.public();
    provide_scope_claim_to_multiple_parties(&[alice_did, bob_did], offering_ticker, eve);
    provide_scope_claim_to_multiple_parties(&[alice_did, bob_did], raise_ticker, eve);

    let check_fundraiser = |tiers, venue, error: DispatchError| {
        assert_noop!(
            STO::create_fundraiser(
                alice_signed.clone(),
                alice_portfolio,
                offering_ticker,
                alice_portfolio,
                raise_ticker,
                tiers,
                venue,
                None,
                None,
            ),
            error
        );
    };

    let create_venue = |origin, type_| {
        let bad_venue = Settlement::venue_counter();
        assert_ok!(Settlement::create_venue(
            origin,
            VenueDetails::default(),
            vec![alice],
            type_
        ));
        bad_venue
    };

    let default_tiers = vec![PriceTier {
        total: 1_000_000u128,
        price: 1u128,
    }];

    let check_venue = |id| {
        check_fundraiser(default_tiers.clone(), id, Error::InvalidVenue.into());
    };

    // Offering asset not created
    check_fundraiser(default_tiers.clone(), 0, Error::Unauthorized.into());

    create_asset(alice_signed.clone(), offering_ticker, 1_000_000);

    // Venue does not exist
    check_venue(0);

    let bad_venue = create_venue(bob_signed.clone(), VenueType::Other);

    // Venue not created by primary issuance agent
    check_venue(bad_venue);

    let bad_venue = create_venue(alice_signed.clone(), VenueType::Other);

    // Venue type not Sto
    check_venue(bad_venue);

    let correct_venue = create_venue(alice_signed.clone(), VenueType::Sto);

    // Raise asset not created
    check_fundraiser(
        default_tiers.clone(),
        correct_venue,
        Error::InvalidPortfolio.into(),
    );

    create_asset(alice_signed.clone(), raise_ticker, 1_000_000);

    assert_ok!(Asset::unsafe_transfer(
        alice_portfolio,
        bob_portfolio,
        &raise_ticker,
        1_000_000
    ));

    empty_compliance(alice_signed.clone(), offering_ticker);
    empty_compliance(alice_signed.clone(), raise_ticker);

    // No prices
    check_fundraiser(vec![], correct_venue, Error::InvalidPriceTiers.into());

    // Zero total
    check_fundraiser(
        vec![PriceTier {
            total: 0u128,
            price: 1u128,
        }],
        correct_venue,
        Error::InvalidPriceTiers.into(),
    );

    check_fundraiser(
        vec![PriceTier {
            total: u128::MAX,
            price: 1u128,
        }],
        correct_venue,
        PortfolioError::InsufficientPortfolioBalance.into(),
    );

    // Invalid time window
    assert_noop!(
        STO::create_fundraiser(
            alice_signed.clone(),
            alice_portfolio,
            offering_ticker,
            alice_portfolio,
            raise_ticker,
            vec![PriceTier {
                total: 100u128,
                price: 1u128
            }],
            correct_venue,
            Some(1),
            Some(0),
        ),
        Error::InvalidOfferingWindow
    );
}
