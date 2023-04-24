use frame_benchmarking::benchmarks;
use frame_support::dispatch::DispatchError;

use pallet_asset::benchmarking::setup_asset_transfer;
use polymesh_common_utilities::benchs::{AccountIdOf, User, UserBuilder};
use polymesh_common_utilities::TestUtilsFn;
use polymesh_primitives::settlement::VenueDetails;
use polymesh_primitives::TrustedIssuer;

use crate::*;

const OFFERING_TICKER: Ticker = Ticker::repeating(b'A');
const RAISE_TICKER: Ticker = Ticker::repeating(b'B');

pub type Asset<T> = pallet_asset::Module<T>;
pub type ComplianceManager<T> = pallet_compliance_manager::Module<T>;
pub type Identity<T> = pallet_identity::Module<T>;
pub type Timestamp<T> = pallet_timestamp::Pallet<T>;
pub type Settlement<T> = pallet_settlement::Module<T>;
pub type Sto<T> = crate::Module<T>;

fn create_assets_and_compliance<T>(
    from: &User<T>,
    to: &User<T>,
    offering_ticker: Ticker,
    raise_ticker: Ticker,
) -> DispatchResult
where
    T: Config + TestUtilsFn<AccountIdOf<T>>,
{
    setup_asset_transfer(from, to, offering_ticker, None, None, false, false);
    setup_asset_transfer(to, from, raise_ticker, None, None, false, false);

    let trusted_user = UserBuilder::<T>::default()
        .generate_did()
        .build("TrustedUser");
    let trusted_issuer = TrustedIssuer::from(trusted_user.did());
    pallet_compliance_manager::Module::<T>::add_default_trusted_claim_issuer(
        from.origin().into(),
        offering_ticker,
        trusted_issuer.clone(),
    )
    .unwrap();
    pallet_compliance_manager::Module::<T>::add_default_trusted_claim_issuer(
        to.origin().into(),
        raise_ticker,
        trusted_issuer,
    )
    .unwrap();

    Ok(())
}

fn generate_tiers<T: Config>(n: u32) -> Vec<PriceTier> {
    let n = n as usize;
    let mut tiers = Vec::with_capacity(n);
    for i in 0..n {
        tiers.push(PriceTier {
            total: 100_000u128.into(),
            price: (i as u128 + 100_000).into(),
        })
    }
    tiers
}

fn create_venue<T: Config>(user: &User<T>) -> Result<VenueId, DispatchError> {
    let venue_id = <Settlement<T>>::venue_counter();
    <Settlement<T>>::create_venue(
        user.origin().into(),
        VenueDetails::default(),
        vec![user.account()],
        VenueType::Sto,
    )
    .unwrap();
    Ok(venue_id)
}

struct UserWithPortfolio<T: Config> {
    user: User<T>,
    portfolio: PortfolioId,
}

fn setup_fundraiser<T>(
    tiers: u32,
) -> Result<(UserWithPortfolio<T>, UserWithPortfolio<T>), DispatchError>
where
    T: Config + TestUtilsFn<AccountIdOf<T>>,
{
    let alice = user::<T>("alice");
    let bob = user::<T>("bob");

    create_assets_and_compliance::<T>(&alice.user, &bob.user, OFFERING_TICKER, RAISE_TICKER)
        .unwrap();

    let venue_id = create_venue(&alice.user).unwrap();

    <Sto<T>>::create_fundraiser(
        alice.user.origin().into(),
        alice.portfolio,
        OFFERING_TICKER,
        alice.portfolio,
        RAISE_TICKER,
        generate_tiers::<T>(tiers),
        venue_id,
        None,
        Some(101u32.into()),
        2,
        vec![].into(),
    )
    .unwrap();

    Ok((alice, bob))
}

fn user<T: Config + TestUtilsFn<AccountIdOf<T>>>(name: &'static str) -> UserWithPortfolio<T> {
    let user = <UserBuilder<T>>::default().generate_did().build(name);
    let portfolio = PortfolioId::default_portfolio(user.did());
    UserWithPortfolio { user, portfolio }
}

benchmarks! {
    where_clause { where T: TestUtilsFn<AccountIdOf<T>> }

    create_fundraiser {
        // Number of tiers
        let i in 1 .. MAX_TIERS as u32;

        let alice = user::<T>("alice");
        create_assets_and_compliance::<T>(&alice.user, &alice.user, OFFERING_TICKER, RAISE_TICKER).unwrap();

        let venue_id = create_venue(&alice.user).unwrap();
        let tiers = generate_tiers::<T>(i);
    }: _(
            alice.user.origin(),
            alice.portfolio,
            OFFERING_TICKER,
            alice.portfolio,
            RAISE_TICKER,
            tiers,
            venue_id,
            None,
            None,
            0u32.into(),
            vec![].into()
        )
    verify {
        assert!(FundraiserCount::get(OFFERING_TICKER) > FundraiserId(0), "create_fundraiser");
    }

    invest {
        let (alice, bob) = setup_fundraiser::<T>(MAX_TIERS as u32).unwrap();
        let amount = 100u128;
    }: _(
            bob.user.origin(),
            bob.portfolio,
            bob.portfolio,
            OFFERING_TICKER,
            FundraiserId(0),
            amount.into(),
            Some(1_000_000u128.into()),
            None
        )
    verify {
        assert!(<Asset<T>>::balance_of(&OFFERING_TICKER, bob.user.did()) > 0u32.into(), "invest");
    }

    freeze_fundraiser {
        let id = FundraiserId(0);
        let (alice, _) = setup_fundraiser::<T>(1).unwrap();
    }: _(alice.user.origin(), OFFERING_TICKER, id)
    verify {
        assert_eq!(<Fundraisers<T>>::get(OFFERING_TICKER, id).unwrap().status, FundraiserStatus::Frozen, "freeze_fundraiser");
    }

    unfreeze_fundraiser {
        let id = FundraiserId(0);
        let (alice, _) = setup_fundraiser::<T>(1).unwrap();
        <Sto<T>>::freeze_fundraiser(alice.user.origin().into(), OFFERING_TICKER, id).unwrap();
    }: _(alice.user.origin(), OFFERING_TICKER, id)
    verify {
        assert_eq!(<Fundraisers<T>>::get(OFFERING_TICKER, id).unwrap().status, FundraiserStatus::Live, "unfreeze_fundraiser");
    }

    modify_fundraiser_window {
        let id = FundraiserId(0);
        let (alice, _) = setup_fundraiser::<T>(1).unwrap();
    }: _(alice.user.origin(), OFFERING_TICKER, id, 100u32.into(), Some(101u32.into()))
    verify {
        assert_eq!(<Fundraisers<T>>::get(OFFERING_TICKER, id).unwrap().end, Some(101u32.into()), "modify_fundraiser_window");
    }

    stop {
        let id = FundraiserId(0);
        let (alice, _) = setup_fundraiser::<T>(1).unwrap();
    }: _(alice.user.origin(), OFFERING_TICKER, id)
    verify {
        assert!(<Fundraisers<T>>::get(OFFERING_TICKER, id).unwrap().is_closed(), "stop");
    }
}
