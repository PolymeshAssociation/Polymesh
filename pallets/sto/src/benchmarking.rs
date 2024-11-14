use frame_benchmarking::benchmarks;
use frame_support::dispatch::DispatchError;
use scale_info::prelude::format;

use pallet_asset::benchmarking::setup_asset_transfer;
use polymesh_common_utilities::benchs::{AccountIdOf, User, UserBuilder};
use polymesh_common_utilities::TestUtilsFn;
use polymesh_primitives::settlement::VenueDetails;
use polymesh_primitives::TrustedIssuer;

use crate::*;

pub type Asset<T> = pallet_asset::Module<T>;
pub type ComplianceManager<T> = pallet_compliance_manager::Module<T>;
pub type Identity<T> = pallet_identity::Module<T>;
pub type Timestamp<T> = pallet_timestamp::Pallet<T>;
pub type Settlement<T> = pallet_settlement::Module<T>;
pub type Sto<T> = crate::Module<T>;

struct SetupPortfolios {
    pub fundraiser_offering_portfolio: PortfolioId,
    pub investor_offering_portfolio: PortfolioId,
    pub fundraiser_raising_portfolio: PortfolioId,
    pub investor_raising_portfolio: PortfolioId,
    pub offering_asset_id: AssetId,
    pub raising_asset_id: AssetId,
}

fn create_assets_and_compliance<T>(fundraiser: &User<T>, investor: &User<T>) -> SetupPortfolios
where
    T: Config + TestUtilsFn<AccountIdOf<T>>,
{
    let (fundraiser_offering_portfolio, investor_offering_portfolio, _, offering_asset_id) =
        setup_asset_transfer(
            fundraiser,
            investor,
            Some(&format!("SdrPortfolio{:?}", fundraiser.did())),
            Some(&format!("RcvPortfolio{:?}", investor.did())),
            false,
            false,
            0,
        );
    let (investor_raising_portfolio, fundraiser_raising_portfolio, _, raising_asset_id) =
        setup_asset_transfer(
            investor,
            fundraiser,
            Some(&format!("SdrPortfolio{:?}", investor.did())),
            Some(&format!("RcvPortfolio{:?}", fundraiser.did())),
            false,
            false,
            0,
        );

    let trusted_user = UserBuilder::<T>::default()
        .generate_did()
        .build("TrustedUser");
    let trusted_issuer = TrustedIssuer::from(trusted_user.did());
    pallet_compliance_manager::Module::<T>::add_default_trusted_claim_issuer(
        fundraiser.origin().into(),
        offering_asset_id,
        trusted_issuer.clone(),
    )
    .unwrap();
    pallet_compliance_manager::Module::<T>::add_default_trusted_claim_issuer(
        investor.origin().into(),
        raising_asset_id,
        trusted_issuer,
    )
    .unwrap();

    SetupPortfolios {
        fundraiser_offering_portfolio,
        investor_offering_portfolio,
        fundraiser_raising_portfolio,
        investor_raising_portfolio,
        offering_asset_id,
        raising_asset_id,
    }
}

fn generate_tiers(n: u32) -> Vec<PriceTier> {
    (0..n)
        .map(|i| PriceTier {
            total: 100_000,
            price: i as u128 + 100_000,
        })
        .collect()
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

fn setup_fundraiser<T>(fundraiser: &User<T>, investor: &User<T>, tiers: u32) -> SetupPortfolios
where
    T: Config + TestUtilsFn<AccountIdOf<T>>,
{
    let setup_portfolios = create_assets_and_compliance::<T>(&fundraiser, &investor);
    let venue_id = create_venue(&fundraiser).unwrap();

    <Sto<T>>::create_fundraiser(
        fundraiser.origin().into(),
        setup_portfolios.fundraiser_offering_portfolio,
        setup_portfolios.offering_asset_id,
        setup_portfolios.fundraiser_raising_portfolio,
        setup_portfolios.raising_asset_id,
        generate_tiers(tiers),
        venue_id,
        None,
        Some(101u32.into()),
        2,
        vec![].into(),
    )
    .unwrap();

    setup_portfolios
}

benchmarks! {
    where_clause { where T: TestUtilsFn<AccountIdOf<T>> }

    create_fundraiser {
        // Number of tiers
        let i in 1 .. MAX_TIERS as u32;

        let alice = <UserBuilder<T>>::default().generate_did().build("Alice");
        let bob = <UserBuilder<T>>::default().generate_did().build("Bob");
        let setup_portfolios = create_assets_and_compliance::<T>(&alice, &bob);

        let venue_id = create_venue(&alice).unwrap();
        let tiers = generate_tiers(i);
    }: _(
            alice.origin(),
            setup_portfolios.fundraiser_offering_portfolio,
            setup_portfolios.offering_asset_id,
            setup_portfolios.fundraiser_raising_portfolio,
            setup_portfolios.raising_asset_id,
            tiers,
            venue_id,
            None,
            None,
            0u32.into(),
            vec![].into()
        )
    verify {
        assert!(FundraiserCount::get(setup_portfolios.offering_asset_id) > FundraiserId(0), "create_fundraiser");
    }

    invest {
        let alice = <UserBuilder<T>>::default().generate_did().build("Alice");
        let bob = <UserBuilder<T>>::default().generate_did().build("Bob");
        let setup_portfolios = setup_fundraiser::<T>(&alice, &bob, MAX_TIERS as u32);
    }: _(
            bob.origin(),
            setup_portfolios.investor_offering_portfolio,
            setup_portfolios.investor_raising_portfolio,
            setup_portfolios.offering_asset_id,
            FundraiserId(0),
            100,
            Some(1_000_000u128.into()),
            None
        )
    verify {
        assert!(<Asset<T>>::balance_of(&setup_portfolios.offering_asset_id, bob.did()) > 0u32.into(), "invest");
    }

    freeze_fundraiser {
        let id = FundraiserId(0);
        let alice = <UserBuilder<T>>::default().generate_did().build("Alice");
        let bob = <UserBuilder<T>>::default().generate_did().build("Bob");
        let setup_portfolios = setup_fundraiser::<T>(&alice, &bob, 1);
    }: _(alice.origin(), setup_portfolios.offering_asset_id, id)
    verify {
        assert_eq!(<Fundraisers<T>>::get(setup_portfolios.offering_asset_id, id).unwrap().status, FundraiserStatus::Frozen, "freeze_fundraiser");
    }

    unfreeze_fundraiser {
        let id = FundraiserId(0);
        let alice = <UserBuilder<T>>::default().generate_did().build("Alice");
        let bob = <UserBuilder<T>>::default().generate_did().build("Bob");
        let setup_portfolios = setup_fundraiser::<T>(&alice, &bob, 1);
        <Sto<T>>::freeze_fundraiser(alice.origin().into(), setup_portfolios.offering_asset_id, id).unwrap();
    }: _(alice.origin(), setup_portfolios.offering_asset_id, id)
    verify {
        assert_eq!(<Fundraisers<T>>::get(setup_portfolios.offering_asset_id, id).unwrap().status, FundraiserStatus::Live, "unfreeze_fundraiser");
    }

    modify_fundraiser_window {
        let id = FundraiserId(0);
        let alice = <UserBuilder<T>>::default().generate_did().build("Alice");
        let bob = <UserBuilder<T>>::default().generate_did().build("Bob");
        let setup_portfolios = setup_fundraiser::<T>(&alice, &bob, 1);
    }: _(alice.origin(), setup_portfolios.offering_asset_id, id, 100u32.into(), Some(101u32.into()))
    verify {
        assert_eq!(<Fundraisers<T>>::get(setup_portfolios.offering_asset_id, id).unwrap().end, Some(101u32.into()), "modify_fundraiser_window");
    }

    stop {
        let id = FundraiserId(0);
        let alice = <UserBuilder<T>>::default().generate_did().build("Alice");
        let bob = <UserBuilder<T>>::default().generate_did().build("Bob");
        let setup_portfolios = setup_fundraiser::<T>(&alice, &bob, 1);
    }: _(alice.origin(), setup_portfolios.offering_asset_id, id)
    verify {
        assert!(<Fundraisers<T>>::get(setup_portfolios.offering_asset_id, id).unwrap().is_closed(), "stop");
    }
}
