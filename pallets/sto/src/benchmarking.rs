#![cfg(feature = "runtime-benchmarks")]
use crate::*;
use frame_benchmarking::benchmarks;
use frame_support::traits::Get;
use frame_system::RawOrigin;
use pallet_settlement::{benchmarking::compliance_setup, VenueDetails};
use polymesh_common_utilities::benchs::User;
use polymesh_common_utilities::{asset::AssetType, benchs::UserBuilder};
use polymesh_primitives::TrustedIssuer;

const OFFERING_TICKER: Ticker = Ticker::repeating(b'A');
const RAISE_TICKER: Ticker = Ticker::repeating(b'B');

pub type Asset<T> = pallet_asset::Module<T>;
pub type ComplianceManager<T> = pallet_compliance_manager::Module<T>;
pub type Identity<T> = identity::Module<T>;
pub type Timestamp<T> = pallet_timestamp::Module<T>;
pub type Settlement<T> = pallet_settlement::Module<T>;
pub type Sto<T> = crate::Module<T>;

fn create_asset<T: Trait>(
    origin: RawOrigin<T::AccountId>,
    ticker: Ticker,
    supply: u128,
) -> DispatchResult {
    <Asset<T>>::create_asset(
        origin.into(),
        vec![b'A'].into(),
        ticker,
        supply.into(),
        true,
        AssetType::default(),
        vec![],
        None,
    )
}

fn create_assets_and_compliance<T: Trait>(
    user: &User<T>,
    tickers: &[Ticker],
    supply: u128,
    complexity: u32,
) -> DispatchResult {
    let t_issuer = UserBuilder::<T>::default()
        .generate_did()
        .build("TrustedClaimIssuer");
    let trusted_issuer = TrustedIssuer::from(t_issuer.did());
    for ticker in tickers {
        create_asset::<T>(user.origin(), ticker.clone(), supply.clone())?;
        compliance_setup::<T>(
            complexity,
            ticker.clone(),
            user.origin(),
            user.did(),
            user.did(),
            trusted_issuer.clone(),
        );
    }
    Ok(())
}

fn generate_tiers<T: Trait>(n: u32) -> Vec<PriceTier<T::Balance>> {
    let n = n as usize;
    let mut tiers = Vec::with_capacity(n);
    for i in 0..n {
        tiers.push(PriceTier {
            total: 1.into(),
            price: (i as u128 + 1).into(),
        })
    }
    tiers
}

fn create_venue<T: Trait>(user: &User<T>) -> Result<u64, DispatchError> {
    let venue_id = <Settlement<T>>::venue_counter();
    <Settlement<T>>::create_venue(
        user.origin().into(),
        VenueDetails::default(),
        vec![user.account()],
        VenueType::Sto,
    )?;
    Ok(venue_id)
}

fn setup_fundraiser<T: Trait>(complexity: u32, tiers: u32) -> Result<User<T>, DispatchError> {
    let (alice, alice_portfolio) = user::<T>("alice");

    create_assets_and_compliance::<T>(
        &alice,
        &[OFFERING_TICKER, RAISE_TICKER],
        1_000_000,
        complexity,
    )?;

    let venue_id = create_venue(&alice)?;

    <Sto<T>>::create_fundraiser(
        alice.origin().into(),
        alice_portfolio,
        OFFERING_TICKER,
        alice_portfolio,
        RAISE_TICKER,
        generate_tiers::<T>(tiers),
        venue_id,
        None,
        Some(101.into()),
        0.into(),
        vec![].into(),
    )?;

    Ok(alice)
}

fn user<T: Trait>(name: &'static str) -> (User<T>, PortfolioId) {
    let user = <UserBuilder<T>>::default().generate_did().build(name);
    let portfolio = PortfolioId::default_portfolio(user.did());
    (user, portfolio)
}

benchmarks! {
    _ {}

    create_fundraiser {
        // Number of tiers
        let i in 1 .. MAX_TIERS as u32;

        let (alice, alice_portfolio) = user::<T>("alice");

        create_assets_and_compliance::<T>(&alice, &[OFFERING_TICKER, RAISE_TICKER], 1_000_000, 0)?;

        let venue_id = create_venue(&alice)?;
        let tiers = generate_tiers::<T>(i);
    }: _(
            alice.origin(),
            alice_portfolio,
            OFFERING_TICKER,
            alice_portfolio,
            RAISE_TICKER,
            tiers,
            venue_id,
            None,
            None,
            0.into(),
            vec![].into()
        )
    verify {
        ensure!(FundraiserCount::get(OFFERING_TICKER) > 0, "create_fundraiser");
    }

    invest {
        // Rule complexity
        let c in 1 .. T::MaxConditionComplexity::get() as u32;

        let alice = setup_fundraiser::<T>(c, MAX_TIERS as u32)?;
        let alice_portfolio = PortfolioId::default_portfolio(alice.did());
        let (bob, bob_portfolio) = user::<T>("bob");

        <Asset<T>>::unsafe_transfer(
            alice_portfolio,
            bob_portfolio,
            &RAISE_TICKER,
            1_000_000.into()
        )?;
    }: _(
            bob.origin(),
            bob_portfolio,
            bob_portfolio,
            OFFERING_TICKER,
            0,
            (MAX_TIERS as u128).into(),
            Some(100.into()),
            None
        )
    verify {
        ensure!(<Asset<T>>::balance_of(&OFFERING_TICKER, bob.did()) > 0.into(), "invest");
    }

    freeze_fundraiser {
        let alice = setup_fundraiser::<T>(0, 1)?;
    }: _(alice.origin(), OFFERING_TICKER, 0)
    verify {
        ensure!(FundraiserCount::get(OFFERING_TICKER) > 0, "freeze_fundraiser");
    }

    unfreeze_fundraiser {
        let alice = setup_fundraiser::<T>(0, 1)?;
    }: _(alice.origin(), OFFERING_TICKER, 0)
    verify {
        ensure!(FundraiserCount::get(OFFERING_TICKER) > 0, "unfreeze_fundraiser");
    }

    modify_fundraiser_window {
        let alice = setup_fundraiser::<T>(0, 1)?;
    }: _(alice.origin(), OFFERING_TICKER, 0, 100.into(), Some(101.into()))
    verify {
        ensure!(FundraiserCount::get(OFFERING_TICKER) > 0, "modify_fundraiser_window");
    }

    stop {
        let alice = setup_fundraiser::<T>(0, 1)?;
    }: _(alice.origin(), OFFERING_TICKER, 0)
    verify {
        ensure!(FundraiserCount::get(OFFERING_TICKER) > 0, "modify_fundraiser_window");
    }
}
