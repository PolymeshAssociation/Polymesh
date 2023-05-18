use frame_support::{assert_noop, assert_ok};
use sp_runtime::DispatchError;

use pallet_sto::{
    Fundraiser, FundraiserId, FundraiserName, FundraiserStatus, FundraiserTier, PriceTier,
    MAX_TIERS,
};
use polymesh_primitives::settlement::{InstructionStatus, VenueDetails, VenueId, VenueType};
use polymesh_primitives::{
    asset::AssetType, checked_inc::CheckedInc, PortfolioId, Ticker, WeightMeter,
};
use test_client::AccountKeyring;

use super::asset_test::{allow_all_transfers, max_len_bytes, set_timestamp};
use super::storage::{
    make_account_with_portfolio, provide_scope_claim_to_multiple_parties, TestStorage, User,
};
use super::{exec_noop, exec_ok, ExtBuilder};

type Origin = <TestStorage as frame_system::Config>::RuntimeOrigin;
type Asset = pallet_asset::Module<TestStorage>;
type Sto = pallet_sto::Module<TestStorage>;
type Error = pallet_sto::Error<TestStorage>;
type EAError = pallet_external_agents::Error<TestStorage>;
type PortfolioError = pallet_portfolio::Error<TestStorage>;
type ComplianceManager = pallet_compliance_manager::Module<TestStorage>;
type Settlement = pallet_settlement::Module<TestStorage>;
type Timestamp = pallet_timestamp::Pallet<TestStorage>;
type System = frame_system::Pallet<TestStorage>;

#[track_caller]
fn test(logic: impl FnOnce()) {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Eve.to_account_id()])
        .build()
        .execute_with(logic);
}

#[test]
fn raise_happy_path_ext() {
    test(raise_happy_path);
}
#[test]
fn raise_unhappy_path_ext() {
    test(raise_unhappy_path);
}

#[test]
fn invalid_fundraiser_ext() {
    test(fundraiser_expired);
}

#[test]
fn fundraiser_expired_ext() {
    test(fundraiser_expired);
}

#[test]
fn modifying_fundraiser_window_ext() {
    test(modifying_fundraiser_window);
}

#[test]
fn freeze_unfreeze_fundraiser_ext() {
    test(freeze_unfreeze_fundraiser);
}

#[test]
fn stop_fundraiser_ext() {
    test(stop_fundraiser);
}

pub fn create_asset(origin: Origin, ticker: Ticker, supply: u128) {
    assert_ok!(Asset::create_asset(
        origin.clone(),
        vec![b'A'].into(),
        ticker,
        true,
        AssetType::default(),
        vec![],
        None,
        true,
    ));
    assert_ok!(Asset::issue(origin, ticker, supply));
}

struct RaiseContext {
    alice: User,
    alice_portfolio: PortfolioId,
    bob: User,
    bob_portfolio: PortfolioId,
    offering_ticker: Ticker,
    raise_ticker: Option<Ticker>,
}

fn init_raise_context(offering_supply: u128, raise_supply_opt: Option<u128>) -> RaiseContext {
    let (alice, alice_portfolio) = make_account_with_portfolio(AccountKeyring::Alice);
    let (bob, bob_portfolio) = make_account_with_portfolio(AccountKeyring::Bob);
    let eve = AccountKeyring::Eve.to_account_id();

    // Register tokens
    let offering_ticker = Ticker::from_slice_truncated(&[b'A'][..]);
    create_asset(alice.origin(), offering_ticker, offering_supply);
    provide_scope_claim_to_multiple_parties(&[alice.did, bob.did], offering_ticker, eve.clone());

    let raise_ticker = raise_supply_opt.map(|raise_supply| {
        let raise_ticker = Ticker::from_slice_truncated(&[b'B'][..]);
        create_asset(alice.origin(), raise_ticker, raise_supply);
        provide_scope_claim_to_multiple_parties(&[alice.did, bob.did], raise_ticker, eve);
        raise_ticker
    });

    RaiseContext {
        alice,
        alice_portfolio,
        bob,
        bob_portfolio,
        offering_ticker,
        raise_ticker,
    }
}

fn raise_happy_path() {
    const RAISE_SUPPLY: u128 = 1_000_000;
    let RaiseContext {
        alice,
        alice_portfolio,
        bob,
        bob_portfolio,
        offering_ticker,
        raise_ticker,
    } = init_raise_context(1_000_000, Some(RAISE_SUPPLY));
    let raise_ticker = raise_ticker.unwrap();

    let mut weight_meter = WeightMeter::max_limit_no_minimum();
    assert_ok!(Asset::unsafe_transfer(
        alice_portfolio,
        bob_portfolio,
        &raise_ticker,
        RAISE_SUPPLY,
        &mut weight_meter
    ));

    allow_all_transfers(offering_ticker, alice);
    allow_all_transfers(raise_ticker, alice);

    // Register a venue
    let venue_counter = Settlement::venue_counter();
    let instruction_id = Settlement::instruction_counter();
    exec_ok!(Settlement::create_venue(
        alice.origin(),
        VenueDetails::default(),
        vec![AccountKeyring::Alice.to_account_id()],
        VenueType::Sto
    ));

    let amount = 100u128;
    let alice_init_offering = Asset::balance_of(&offering_ticker, alice.did);
    let bob_init_offering = Asset::balance_of(&offering_ticker, bob.did);
    let alice_init_raise = Asset::balance_of(&raise_ticker, alice.did);
    let bob_init_raise = Asset::balance_of(&raise_ticker, bob.did);

    // Alice starts a fundraiser
    let fundraiser_id = Sto::fundraiser_count(offering_ticker);
    let fundraiser_name: FundraiserName = max_len_bytes(0);
    exec_ok!(Sto::create_fundraiser(
        alice.origin(),
        alice_portfolio,
        offering_ticker,
        alice_portfolio,
        raise_ticker,
        vec![PriceTier {
            total: 1_000_000u128,
            price: 1_000_000u128,
        }],
        venue_counter,
        None,
        None,
        2,
        fundraiser_name.clone(),
    ));

    let check_fundraiser = |remaining| {
        assert_eq!(
            Sto::fundraisers(offering_ticker, fundraiser_id),
            Some(Fundraiser {
                creator: alice.did,
                offering_portfolio: alice_portfolio,
                offering_asset: offering_ticker,
                raising_portfolio: alice_portfolio,
                raising_asset: raise_ticker,
                tiers: vec![FundraiserTier {
                    total: 1_000_000u128,
                    remaining,
                    price: 1_000_000u128
                }],
                venue_id: venue_counter,
                start: Timestamp::get(),
                end: None,
                status: FundraiserStatus::Live,
                minimum_investment: 2
            })
        );
    };

    check_fundraiser(1_000_000u128);

    assert_eq!(
        Asset::balance_of(&offering_ticker, alice.did),
        alice_init_offering
    );
    assert_eq!(
        Asset::balance_of(&offering_ticker, bob.did),
        bob_init_offering
    );
    assert_eq!(
        Asset::balance_of(&raise_ticker, alice.did),
        alice_init_raise
    );
    assert_eq!(Asset::balance_of(&raise_ticker, bob.did), bob_init_raise);
    assert_eq!(
        Sto::fundraiser_name(offering_ticker, fundraiser_id),
        fundraiser_name
    );
    let sto_invest = |purchase_amount, max_price, err: Error| {
        exec_noop!(
            Sto::invest(
                bob.origin(),
                bob_portfolio,
                bob_portfolio,
                offering_ticker,
                fundraiser_id,
                purchase_amount,
                max_price,
                None,
            ),
            err
        );
    };
    // Investment fails if the minimum investment amount is not met
    sto_invest(1, Some(1_000_000u128), Error::InvestmentAmountTooLow);
    // Investment fails if the order is not filled
    sto_invest(
        1_000_001u128,
        Some(1_000_000u128),
        Error::InsufficientTokensRemaining,
    );
    // Investment fails if the maximum price is breached
    sto_invest(amount.into(), Some(999_999u128), Error::MaxPriceExceeded);
    // Bob invests in Alice's fundraiser
    exec_ok!(Sto::invest(
        bob.origin(),
        bob_portfolio,
        bob_portfolio,
        offering_ticker,
        fundraiser_id,
        amount.into(),
        Some(1_000_000u128),
        None,
    ));
    check_fundraiser(1_000_000u128 - amount);
    assert_eq!(
        Some(Settlement::instruction_counter()),
        instruction_id.checked_inc()
    );
    assert_eq!(
        Settlement::instruction_status(instruction_id),
        InstructionStatus::Success(System::block_number())
    );

    assert_eq!(
        Asset::balance_of(&offering_ticker, alice.did),
        alice_init_offering - amount
    );
    assert_eq!(
        Asset::balance_of(&offering_ticker, bob.did),
        bob_init_offering + amount
    );
    assert_eq!(
        Asset::balance_of(&raise_ticker, alice.did),
        alice_init_raise + amount
    );
    assert_eq!(
        Asset::balance_of(&raise_ticker, bob.did),
        bob_init_raise - amount
    );
}

fn raise_unhappy_path() {
    let (alice, alice_portfolio) = make_account_with_portfolio(AccountKeyring::Alice);
    let (bob, bob_portfolio) = make_account_with_portfolio(AccountKeyring::Bob);

    let offering_ticker = Ticker::from_slice_truncated(&[b'C'][..]);
    let raise_ticker = Ticker::from_slice_truncated(&[b'D'][..]);

    // Provide scope claim to both the parties of the transaction.
    let eve = AccountKeyring::Eve.to_account_id();
    provide_scope_claim_to_multiple_parties(&[alice.did, bob.did], offering_ticker, eve.clone());
    provide_scope_claim_to_multiple_parties(&[alice.did, bob.did], raise_ticker, eve);

    let fundraise = |tiers, venue, name| {
        Sto::create_fundraiser(
            alice.origin(),
            alice_portfolio,
            offering_ticker,
            alice_portfolio,
            raise_ticker,
            tiers,
            venue,
            None,
            None,
            0,
            name,
        )
    };

    let check_fundraiser = |tiers, venue, error: DispatchError| {
        assert_noop!(fundraise(tiers, venue, <_>::default()), error);
    };

    let create_venue = |user: User, type_| {
        let bad_venue = Settlement::venue_counter();
        assert_ok!(Settlement::create_venue(
            user.origin(),
            VenueDetails::default(),
            vec![alice.acc()],
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

    // Name too long.
    assert_too_long!(fundraise(
        default_tiers.clone(),
        VenueId(0),
        max_len_bytes(1)
    ));

    // Offering asset not created
    check_fundraiser(
        default_tiers.clone(),
        VenueId(0),
        EAError::UnauthorizedAgent.into(),
    );

    create_asset(alice.origin(), offering_ticker, 1_000_000);

    // Venue does not exist
    check_venue(VenueId(0));

    let bad_venue = create_venue(bob, VenueType::Other);

    // Venue not created by primary issuance agent
    check_venue(bad_venue);

    let bad_venue = create_venue(alice, VenueType::Other);

    // Venue type not Sto
    check_venue(bad_venue);

    let correct_venue = create_venue(alice, VenueType::Sto);

    create_asset(alice.origin(), raise_ticker, 1_000_000);

    let mut weight_meter = WeightMeter::max_limit_no_minimum();
    assert_ok!(Asset::unsafe_transfer(
        alice_portfolio,
        bob_portfolio,
        &raise_ticker,
        1_000_000,
        &mut weight_meter
    ));

    allow_all_transfers(offering_ticker, alice);
    allow_all_transfers(raise_ticker, alice);

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
        Sto::create_fundraiser(
            alice.origin(),
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
            0,
            FundraiserName::default()
        ),
        Error::InvalidOfferingWindow
    );
}

fn invalid_fundraiser() {
    let RaiseContext {
        alice,
        alice_portfolio,
        offering_ticker,
        raise_ticker,
        ..
    } = init_raise_context(1_000_000, Some(1_000_000));

    let venue_counter = Settlement::venue_counter();
    assert_ok!(Settlement::create_venue(
        alice.origin(),
        VenueDetails::default(),
        vec![AccountKeyring::Alice.to_account_id()],
        VenueType::Sto
    ));

    let create_fundraiser_fn = |tiers| {
        Sto::create_fundraiser(
            alice.origin(),
            alice_portfolio,
            offering_ticker,
            alice_portfolio,
            raise_ticker.unwrap(),
            tiers,
            venue_counter,
            None,
            None,
            0,
            FundraiserName::default(),
        )
    };

    // No tiers
    let zero_tiers = vec![];
    assert_noop!(create_fundraiser_fn(zero_tiers), Error::InvalidPriceTiers);

    // Max tiers
    let max_tiers_pass = (0..MAX_TIERS + 1)
        .map(|_i| PriceTier::default())
        .collect::<Vec<_>>();
    assert_noop!(
        create_fundraiser_fn(max_tiers_pass),
        Error::InvalidPriceTiers
    );

    // price_total = 0
    let total_0_tiers = (0..MAX_TIERS)
        .map(|_i| PriceTier::default())
        .collect::<Vec<_>>();
    assert_noop!(
        create_fundraiser_fn(total_0_tiers),
        Error::InvalidPriceTiers
    );

    // Total overflow
    let total_overflow_tiers = vec![
        PriceTier {
            total: u128::MAX,
            price: 1,
        },
        PriceTier { total: 1, price: 2 },
    ];
    assert_noop!(
        create_fundraiser_fn(total_overflow_tiers),
        Error::InvalidPriceTiers
    );
}

fn basic_fundraiser() -> (FundraiserId, RaiseContext) {
    let context = init_raise_context(1_000_000, Some(1_000_000));

    let venue_counter = Settlement::venue_counter();
    assert_ok!(Settlement::create_venue(
        context.alice.origin(),
        VenueDetails::default(),
        vec![AccountKeyring::Alice.to_account_id()],
        VenueType::Sto
    ));
    let fundraiser_id = Sto::fundraiser_count(context.offering_ticker);
    assert_ok!(Sto::create_fundraiser(
        context.alice.origin(),
        context.alice_portfolio,
        context.offering_ticker,
        context.alice_portfolio,
        context.raise_ticker.unwrap(),
        vec![PriceTier { total: 1, price: 2 }],
        venue_counter,
        None,
        None,
        0,
        FundraiserName::default(),
    ));
    (fundraiser_id, context)
}

fn fundraiser_expired() {
    let (
        fundraiser_id,
        RaiseContext {
            alice,
            offering_ticker,
            bob,
            bob_portfolio,
            ..
        },
    ) = basic_fundraiser();

    assert_ok!(Sto::modify_fundraiser_window(
        alice.origin(),
        offering_ticker,
        fundraiser_id,
        Timestamp::get(),
        Some(Timestamp::get() + 1)
    ));

    set_timestamp(Timestamp::get() + 2);

    assert_noop!(
        Sto::modify_fundraiser_window(
            alice.origin(),
            offering_ticker,
            fundraiser_id,
            Timestamp::get(),
            None
        ),
        Error::FundraiserExpired
    );

    assert_noop!(
        Sto::invest(
            bob.origin(),
            bob_portfolio,
            bob_portfolio,
            offering_ticker,
            fundraiser_id,
            1000,
            None,
            None
        ),
        Error::FundraiserExpired
    );
}

fn modifying_fundraiser_window() {
    let (
        fundraiser_id,
        RaiseContext {
            alice,
            offering_ticker,
            raise_ticker,
            ..
        },
    ) = basic_fundraiser();

    // Wrong ticker
    assert_noop!(
        Sto::modify_fundraiser_window(
            alice.origin(),
            raise_ticker.unwrap(),
            fundraiser_id,
            Timestamp::get(),
            None
        ),
        Error::FundraiserNotFound
    );

    // Bad fundraiser id
    assert_noop!(
        Sto::modify_fundraiser_window(
            alice.origin(),
            offering_ticker,
            FundraiserId(u64::MAX),
            Timestamp::get(),
            None
        ),
        Error::FundraiserNotFound
    );

    let bad_modify_fundraiser_window = |start, end| {
        Sto::modify_fundraiser_window(alice.origin(), offering_ticker, fundraiser_id, start, end)
    };

    assert_ok!(bad_modify_fundraiser_window(0, None));
    assert_ok!(bad_modify_fundraiser_window(Timestamp::get(), None));
    assert_noop!(
        bad_modify_fundraiser_window(Timestamp::get() + 1, Some(Timestamp::get())),
        Error::InvalidOfferingWindow
    );
    assert_noop!(
        bad_modify_fundraiser_window(Timestamp::get() + 1, Some(Timestamp::get() + 1)),
        Error::InvalidOfferingWindow
    );
    assert_ok!(bad_modify_fundraiser_window(
        Timestamp::get() + 1,
        Some(Timestamp::get() + 2)
    ),);
}

fn freeze_unfreeze_fundraiser() {
    let (
        fundraiser_id,
        RaiseContext {
            alice,
            offering_ticker,
            raise_ticker,
            ..
        },
    ) = basic_fundraiser();

    // Wrong ticker
    assert_noop!(
        Sto::freeze_fundraiser(alice.origin(), raise_ticker.unwrap(), fundraiser_id,),
        Error::FundraiserNotFound
    );

    // Bad fundraiser id
    assert_noop!(
        Sto::freeze_fundraiser(alice.origin(), offering_ticker, FundraiserId(u64::MAX)),
        Error::FundraiserNotFound
    );

    assert_ok!(Sto::freeze_fundraiser(
        alice.origin(),
        offering_ticker,
        fundraiser_id,
    ));

    assert_ok!(Sto::unfreeze_fundraiser(
        alice.origin(),
        offering_ticker,
        fundraiser_id,
    ));
}

fn stop_fundraiser() {
    let (
        fundraiser_id,
        RaiseContext {
            alice,
            bob,
            offering_ticker,
            raise_ticker,
            ..
        },
    ) = basic_fundraiser();

    // Wrong ticker
    assert_noop!(
        Sto::stop(alice.origin(), raise_ticker.unwrap(), fundraiser_id,),
        Error::FundraiserNotFound
    );

    // Bad fundraiser id
    assert_noop!(
        Sto::stop(alice.origin(), offering_ticker, FundraiserId(u64::MAX)),
        Error::FundraiserNotFound
    );

    // Unauthorized
    assert_noop!(
        Sto::stop(bob.origin(), offering_ticker, fundraiser_id),
        EAError::UnauthorizedAgent
    );

    assert_ok!(Sto::stop(alice.origin(), offering_ticker, fundraiser_id,));

    assert_noop!(
        Sto::stop(alice.origin(), offering_ticker, fundraiser_id,),
        Error::FundraiserClosed
    );
}
