#![cfg(feature = "runtime-benchmarks")]
use crate::*;
use core::convert::TryFrom;
use frame_benchmarking::benchmarks;
use frame_support::traits::Get;
use frame_system::RawOrigin;
use pallet_settlement::{benchmarking::compliance_setup, VenueDetails};
use polymesh_common_utilities::benchs::User;
use polymesh_common_utilities::{asset::AssetType, benchs::UserBuilder};
use polymesh_primitives::TrustedIssuer;

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

fn add_empty_compliance<T: Trait>(
    origin: RawOrigin<T::AccountId>,
    ticker: Ticker,
) -> DispatchResult {
    <ComplianceManager<T>>::add_compliance_requirement(origin.into(), ticker, vec![], vec![])
}

fn create_asset_and_compliance<T: Trait>(
    origin: RawOrigin<T::AccountId>,
    tickers: &[Ticker],
    supply: u128,
) -> DispatchResult {
    for ticker in tickers {
        create_asset::<T>(origin.clone(), ticker.clone(), supply.clone())?;
        add_empty_compliance::<T>(origin.clone(), ticker.clone())?;
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

fn setup_fundraiser<T: Trait>() -> Result<(User<T>, Ticker), DispatchError> {
    let (alice, alice_portfolio) = user::<T>("alice");

    let offering_ticker = Ticker::try_from(&[b'A'][..]).unwrap();
    let raise_ticker = Ticker::try_from(&[b'B'][..]).unwrap();
    create_asset_and_compliance::<T>(alice.origin(), &[offering_ticker, raise_ticker], 1_000_000)?;

    let venue_id = create_venue(&alice)?;

    <Sto<T>>::create_fundraiser(
        alice.origin().into(),
        alice_portfolio,
        offering_ticker,
        alice_portfolio,
        raise_ticker,
        generate_tiers::<T>(1),
        venue_id,
        None,
        Some(101.into()),
    )?;

    Ok((alice, offering_ticker))
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

        let offering_ticker = Ticker::try_from(&[b'A'][..]).unwrap();
        let raise_ticker = Ticker::try_from(&[b'B'][..]).unwrap();
        create_asset_and_compliance::<T>(alice.origin(), &[offering_ticker, raise_ticker], 1_000_000)?;

        let venue_id = create_venue(&alice)?;
        let tiers = generate_tiers::<T>(i);
    }: _(
            alice.origin(),
            alice_portfolio,
            offering_ticker,
            alice_portfolio,
            raise_ticker,
            tiers,
            venue_id,
            None,
            None
        )
    verify {
        ensure!(FundraiserCount::get(offering_ticker) > 0, "create_fundraiser");
    }

    invest {
        // Rule complexity
        let c in 1 .. T::MaxConditionComplexity::get() as u32;

        let (alice, alice_portfolio) = user::<T>("alice");
        let (bob, bob_portfolio) = user::<T>("bob");

        let offering_ticker = Ticker::try_from(&[b'A'][..]).unwrap();
        let raise_ticker = Ticker::try_from(&[b'B'][..]).unwrap();
        create_asset::<T>(alice.origin(), offering_ticker, 1_000_000)?;
        create_asset::<T>(alice.origin(), raise_ticker, 1_000_000)?;

        let t_issuer = UserBuilder::<T>::default().generate_did().build("TrustedClaimIssuer");
        let trusted_issuer = TrustedIssuer::from(t_issuer.did());

        compliance_setup::<T>(c, offering_ticker, alice.origin(), alice.did(), bob.did(), trusted_issuer.clone());
        compliance_setup::<T>(c, raise_ticker, alice.origin(), bob.did(), alice.did(), trusted_issuer);

        let venue_id = create_venue(&alice)?;

        <Asset<T>>::unsafe_transfer(
            alice_portfolio,
            bob_portfolio,
            &raise_ticker,
            1_000_000.into()
        )?;

        <Sto<T>>::create_fundraiser(
            alice.origin().into(),
            alice_portfolio,
            offering_ticker,
            alice_portfolio,
            raise_ticker,
            generate_tiers::<T>(MAX_TIERS as u32),
            venue_id,
            None,
            None
        )?;
    }: _(
            bob.origin(),
            bob_portfolio,
            bob_portfolio,
            offering_ticker,
            0,
            (MAX_TIERS as u128).into(),
            Some(100.into()),
            None
        )
    verify {
        ensure!(<Asset<T>>::balance_of(&offering_ticker, bob.did()) > 0.into(), "invest");
    }

    freeze_fundraiser {
        let (alice, offering_ticker) = setup_fundraiser::<T>()?;
    }: _(alice.origin(), offering_ticker, 0)
    verify {
        ensure!(FundraiserCount::get(offering_ticker) > 0, "freeze_fundraiser");
    }

    unfreeze_fundraiser {
        let (alice, offering_ticker) = setup_fundraiser::<T>()?;
    }: _(alice.origin(), offering_ticker, 0)
    verify {
        ensure!(FundraiserCount::get(offering_ticker) > 0, "unfreeze_fundraiser");
    }

    modify_fundraiser_window {
        let (alice, offering_ticker) = setup_fundraiser::<T>()?;
    }: _(alice.origin(), offering_ticker, 0, 100.into(), Some(101.into()))
    verify {
        ensure!(FundraiserCount::get(offering_ticker) > 0, "modify_fundraiser_window");
    }

    stop {
        let (alice, offering_ticker) = setup_fundraiser::<T>()?;
    }: _(alice.origin(), offering_ticker, 0)
    verify {
        ensure!(FundraiserCount::get(offering_ticker) > 0, "modify_fundraiser_window");
    }
}
