use frame_support::{assert_noop, assert_ok};
use sp_runtime::DispatchError;

use pallet_sto::{
    Fundraiser, FundraiserId, FundraiserName, FundraiserStatus, FundraiserTier, PriceTier,
    MAX_TIERS,
};
use polymesh_primitives::asset::AssetID;
use polymesh_primitives::checked_inc::CheckedInc;
use polymesh_primitives::settlement::{InstructionStatus, VenueDetails, VenueId, VenueType};
use polymesh_primitives::{IdentityId, PortfolioId, WeightMeter};
use sp_keyring::AccountKeyring;

use crate::asset_pallet::setup::create_and_issue_sample_asset;

use super::asset_test::{max_len_bytes, set_timestamp};
use super::storage::{make_account_with_portfolio, TestStorage, User};
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

struct RaiseContext {
    alice: User,
    alice_portfolio: PortfolioId,
    bob: User,
    bob_portfolio: PortfolioId,
    offering_asset: AssetID,
    raise_asset: Option<AssetID>,
}

fn init_raise_context() -> RaiseContext {
    let (alice, alice_portfolio) = make_account_with_portfolio(AccountKeyring::Alice);
    let (bob, bob_portfolio) = make_account_with_portfolio(AccountKeyring::Bob);

    // Register tokens
    let offering_asset = create_and_issue_sample_asset(&alice);

    let raise_asset = Some(create_and_issue_sample_asset(&alice));

    RaiseContext {
        alice,
        alice_portfolio,
        bob,
        bob_portfolio,
        offering_asset,
        raise_asset,
    }
}

fn raise_happy_path() {
    const RAISE_SUPPLY: u128 = 1_000_000;
    let RaiseContext {
        alice,
        alice_portfolio,
        bob,
        bob_portfolio,
        offering_asset,
        raise_asset,
    } = init_raise_context();
    let raise_asset = raise_asset.unwrap();

    let mut weight_meter = WeightMeter::max_limit_no_minimum();
    assert_ok!(Asset::unverified_transfer_asset(
        alice_portfolio,
        bob_portfolio,
        raise_asset,
        RAISE_SUPPLY,
        None,
        None,
        IdentityId::default(),
        &mut weight_meter
    ));

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
    let alice_init_offering = Asset::balance_of(&offering_asset, alice.did);
    let bob_init_offering = Asset::balance_of(&offering_asset, bob.did);
    let alice_init_raise = Asset::balance_of(&raise_asset, alice.did);
    let bob_init_raise = Asset::balance_of(&raise_asset, bob.did);

    // Alice starts a fundraiser
    let fundraiser_id = Sto::fundraiser_count(offering_asset);
    let fundraiser_name: FundraiserName = max_len_bytes(0);
    exec_ok!(Sto::create_fundraiser(
        alice.origin(),
        alice_portfolio,
        offering_asset,
        alice_portfolio,
        raise_asset,
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
            Sto::fundraisers(offering_asset, fundraiser_id),
            Some(Fundraiser {
                creator: alice.did,
                offering_portfolio: alice_portfolio,
                offering_asset,
                raising_portfolio: alice_portfolio,
                raising_asset: raise_asset,
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
        Asset::balance_of(&offering_asset, alice.did),
        alice_init_offering
    );
    assert_eq!(
        Asset::balance_of(&offering_asset, bob.did),
        bob_init_offering
    );
    assert_eq!(Asset::balance_of(&raise_asset, alice.did), alice_init_raise);
    assert_eq!(Asset::balance_of(&raise_asset, bob.did), bob_init_raise);
    assert_eq!(
        Sto::fundraiser_name(offering_asset, fundraiser_id),
        Some(fundraiser_name)
    );
    let sto_invest = |purchase_amount, max_price, err: Error| {
        exec_noop!(
            Sto::invest(
                bob.origin(),
                bob_portfolio,
                bob_portfolio,
                offering_asset,
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
        offering_asset,
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
        Asset::balance_of(&offering_asset, alice.did),
        alice_init_offering - amount
    );
    assert_eq!(
        Asset::balance_of(&offering_asset, bob.did),
        bob_init_offering + amount
    );
    assert_eq!(
        Asset::balance_of(&raise_asset, alice.did),
        alice_init_raise + amount
    );
    assert_eq!(
        Asset::balance_of(&raise_asset, bob.did),
        bob_init_raise - amount
    );
}

fn raise_unhappy_path() {
    let (alice, alice_portfolio) = make_account_with_portfolio(AccountKeyring::Alice);
    let (bob, bob_portfolio) = make_account_with_portfolio(AccountKeyring::Bob);

    // Offering asset not created
    assert_noop!(
        Sto::create_fundraiser(
            alice.origin(),
            alice_portfolio,
            [0; 16].into(),
            alice_portfolio,
            [1; 16].into(),
            Vec::new(),
            VenueId(0),
            None,
            None,
            0,
            b"Name".into(),
        ),
        EAError::UnauthorizedAgent
    );

    let offering_asset = create_and_issue_sample_asset(&alice);
    let raise_asset = create_and_issue_sample_asset(&alice);

    let fundraise = |tiers, venue, name| {
        Sto::create_fundraiser(
            alice.origin(),
            alice_portfolio,
            offering_asset,
            alice_portfolio,
            raise_asset,
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

    // Venue does not exist
    check_venue(VenueId(0));

    let bad_venue = create_venue(bob, VenueType::Other);

    // Venue not created by primary issuance agent
    check_venue(bad_venue);

    let bad_venue = create_venue(alice, VenueType::Other);

    // Venue type not Sto
    check_venue(bad_venue);

    let correct_venue = create_venue(alice, VenueType::Sto);

    let mut weight_meter = WeightMeter::max_limit_no_minimum();
    assert_ok!(Asset::unverified_transfer_asset(
        alice_portfolio,
        bob_portfolio,
        raise_asset,
        1_000_000,
        None,
        None,
        IdentityId::default(),
        &mut weight_meter
    ));

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
            offering_asset,
            alice_portfolio,
            raise_asset,
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
        offering_asset,
        raise_asset,
        ..
    } = init_raise_context();

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
            offering_asset,
            alice_portfolio,
            raise_asset.unwrap(),
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
    let context = init_raise_context();

    let venue_counter = Settlement::venue_counter();
    assert_ok!(Settlement::create_venue(
        context.alice.origin(),
        VenueDetails::default(),
        vec![AccountKeyring::Alice.to_account_id()],
        VenueType::Sto
    ));
    let fundraiser_id = Sto::fundraiser_count(context.offering_asset);
    assert_ok!(Sto::create_fundraiser(
        context.alice.origin(),
        context.alice_portfolio,
        context.offering_asset,
        context.alice_portfolio,
        context.raise_asset.unwrap(),
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
            offering_asset,
            bob,
            bob_portfolio,
            ..
        },
    ) = basic_fundraiser();

    assert_ok!(Sto::modify_fundraiser_window(
        alice.origin(),
        offering_asset,
        fundraiser_id,
        Timestamp::get(),
        Some(Timestamp::get() + 1)
    ));

    set_timestamp(Timestamp::get() + 2);

    assert_noop!(
        Sto::modify_fundraiser_window(
            alice.origin(),
            offering_asset,
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
            offering_asset,
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
            offering_asset,
            raise_asset,
            ..
        },
    ) = basic_fundraiser();

    // Wrong ticker
    assert_noop!(
        Sto::modify_fundraiser_window(
            alice.origin(),
            raise_asset.unwrap(),
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
            offering_asset,
            FundraiserId(u64::MAX),
            Timestamp::get(),
            None
        ),
        Error::FundraiserNotFound
    );

    let bad_modify_fundraiser_window = |start, end| {
        Sto::modify_fundraiser_window(alice.origin(), offering_asset, fundraiser_id, start, end)
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
            offering_asset,
            raise_asset,
            ..
        },
    ) = basic_fundraiser();

    // Wrong ticker
    assert_noop!(
        Sto::freeze_fundraiser(alice.origin(), raise_asset.unwrap(), fundraiser_id,),
        Error::FundraiserNotFound
    );

    // Bad fundraiser id
    assert_noop!(
        Sto::freeze_fundraiser(alice.origin(), offering_asset, FundraiserId(u64::MAX)),
        Error::FundraiserNotFound
    );

    assert_ok!(Sto::freeze_fundraiser(
        alice.origin(),
        offering_asset,
        fundraiser_id,
    ));

    assert_ok!(Sto::unfreeze_fundraiser(
        alice.origin(),
        offering_asset,
        fundraiser_id,
    ));
}

fn stop_fundraiser() {
    let (
        fundraiser_id,
        RaiseContext {
            alice,
            bob,
            offering_asset,
            raise_asset,
            ..
        },
    ) = basic_fundraiser();

    // Wrong ticker
    assert_noop!(
        Sto::stop(alice.origin(), raise_asset.unwrap(), fundraiser_id,),
        Error::FundraiserNotFound
    );

    // Bad fundraiser id
    assert_noop!(
        Sto::stop(alice.origin(), offering_asset, FundraiserId(u64::MAX)),
        Error::FundraiserNotFound
    );

    // Unauthorized
    assert_noop!(
        Sto::stop(bob.origin(), offering_asset, fundraiser_id),
        EAError::UnauthorizedAgent
    );

    assert_ok!(Sto::stop(alice.origin(), offering_asset, fundraiser_id,));

    assert_noop!(
        Sto::stop(alice.origin(), offering_asset, fundraiser_id,),
        Error::FundraiserClosed
    );
}
