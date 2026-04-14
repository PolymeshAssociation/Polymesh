use frame_benchmarking::benchmarks;
use frame_support::pallet_prelude::DispatchError;
use scale_info::prelude::format;
use sp_runtime::MultiSignature;

use pallet_asset::benchmarking::setup_asset_transfer;
use pallet_asset::BalanceOf;
use pallet_identity::benchmarking::{User, UserBuilder};
use pallet_settlement::VenueCounter;
use polymesh_primitives::crypto::{ChainScopedMessage, STO_FUNDRAISER_RECEIPT_LABEL};
use polymesh_primitives::settlement::{AffirmationRequirement, VenueDetails};
use polymesh_primitives::traits::AffirmationFnTrait;
use polymesh_primitives::{Ticker, TrustedIssuer};

use crate::*;

pub type Asset<T> = pallet_asset::Pallet<T>;
pub type ComplianceManager<T> = pallet_compliance_manager::Pallet<T>;
pub type Identity<T> = pallet_identity::Pallet<T>;
pub type Timestamp<T> = pallet_timestamp::Pallet<T>;
pub type Settlement<T> = pallet_settlement::Pallet<T>;
pub type Sto<T> = crate::pallet::Pallet<T>;

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
    T: Config,
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
            true,
            false,
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
            true,
            false,
        );

    let trusted_user = UserBuilder::<T>::default()
        .generate_did()
        .build("TrustedUser");
    let trusted_issuer = TrustedIssuer::from(trusted_user.did());
    pallet_compliance_manager::Pallet::<T>::add_default_trusted_claim_issuer(
        fundraiser.origin().into(),
        offering_asset_id,
        trusted_issuer.clone(),
    )
    .unwrap();
    pallet_compliance_manager::Pallet::<T>::add_default_trusted_claim_issuer(
        investor.origin().into(),
        raising_asset_id,
        trusted_issuer,
    )
    .unwrap();

    SetupPortfolios {
        fundraiser_offering_portfolio: fundraiser_offering_portfolio.try_into().unwrap(),
        investor_offering_portfolio: investor_offering_portfolio.try_into().unwrap(),
        fundraiser_raising_portfolio: fundraiser_raising_portfolio.try_into().unwrap(),
        investor_raising_portfolio: investor_raising_portfolio.try_into().unwrap(),
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
    let venue_id = VenueCounter::<T>::get();
    <Settlement<T>>::create_venue(
        user.origin().into(),
        VenueDetails::default(),
        BTreeSet::from([user.account()]),
        VenueType::Sto,
    )
    .unwrap();
    Ok(venue_id)
}

fn setup_fundraiser<T>(fundraiser: &User<T>, investor: &User<T>, tiers: u32) -> SetupPortfolios
where
    T: Config,
{
    let setup_portfolios = create_assets_and_compliance::<T>(&fundraiser, &investor);
    let venue_id = create_venue(&fundraiser).unwrap();

    <Sto<T>>::create_fundraiser(
        fundraiser.origin().into(),
        setup_portfolios.fundraiser_offering_portfolio.clone(),
        setup_portfolios.offering_asset_id,
        setup_portfolios.fundraiser_raising_portfolio.clone(),
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

fn sign_receipt<T: Config>(
    signer: &User<T>,
    uid: u64,
    fundraiser_id: FundraiserId,
    fundraiser_did: IdentityId,
    investor_did: IdentityId,
    ticker: Ticker,
    amount: u128,
) -> FundraiserReceiptDetails<T::AccountId, T::OffChainSignature, T::Moment> {
    let expires_at = pallet_timestamp::Pallet::<T>::get() + 1000u32.into();
    let receipt = ChainScopedMessage::<T, _>::new_unchecked(
        uid,
        STO_FUNDRAISER_RECEIPT_LABEL,
        expires_at,
        FundraiserReceipt::new(fundraiser_id, investor_did, fundraiser_did, ticker, amount),
    );
    let signature = receipt
        .native_sign(
            &signer
                .secret
                .as_ref()
                .expect("User without secret key")
                .to_bytes(),
        )
        .expect("Failed to sign receipt");
    let encoded_signature = MultiSignature::from(signature).encode();
    let signature = T::OffChainSignature::decode(&mut &encoded_signature[..]).unwrap();
    FundraiserReceiptDetails {
        uid,
        signer: signer.account(),
        signature: signature.into(),
        expires_at,
        metadata: None,
    }
}

benchmarks! {
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
        assert!(FundraiserCount::<T>::get(setup_portfolios.offering_asset_id) > FundraiserId(0), "create_fundraiser");
    }

    invest_onchain {
        let alice = <UserBuilder<T>>::default().generate_did().build("Alice");
        let bob = <UserBuilder<T>>::default().generate_did().build("Bob");
        let setup_portfolios = setup_fundraiser::<T>(&alice, &bob, MAX_TIERS as u32);
        // Opt-in receivers to mandatory affirmation so benchmarks measure worst-case weights.
        T::AffirmationFn::set_mandatory_receiver_affirmation(alice.did(), AffirmationRequirement::Required);
        T::AffirmationFn::set_mandatory_receiver_affirmation(bob.did(), AffirmationRequirement::Required);
    }: invest(
            bob.origin(),
            setup_portfolios.offering_asset_id,
            FundraiserId(0),
            setup_portfolios.investor_offering_portfolio,
            FundingMethod::OnChain(setup_portfolios.investor_raising_portfolio),
            100,
            Some(1_000_000u128.into())
        )
    verify {
        assert!(BalanceOf::<T>::get(&setup_portfolios.offering_asset_id, bob.did()) > 0u32.into(), "invest");
    }

    invest_offchain {
        let id = FundraiserId(0);
        let alice = <UserBuilder<T>>::default().generate_did().build("Alice");
        let bob = <UserBuilder<T>>::default().generate_did().build("Bob");
        let ticker = Ticker::from_slice_truncated(b"TEST");

        let setup_portfolios = setup_fundraiser::<T>(&alice, &bob, MAX_TIERS as u32);
        // Opt-in receiver to mandatory affirmation so benchmarks measure worst-case weights.
        T::AffirmationFn::set_mandatory_receiver_affirmation(bob.did(), AffirmationRequirement::Required);
        Sto::<T>::enable_offchain_funding(
            alice.origin().into(),
            setup_portfolios.offering_asset_id,
            id,
            ticker
        ).unwrap();

        let receipt = sign_receipt(
            &alice,
            0,
            id,
            alice.did(),
            bob.did(),
            ticker,
            10
        );
    }: invest(
            bob.origin(),
            setup_portfolios.offering_asset_id,
            FundraiserId(0),
            setup_portfolios.investor_offering_portfolio,
            FundingMethod::OffChain(receipt),
            100,
            Some(1_000_000u128.into())
        )
    verify {
        assert!(BalanceOf::<T>::get(&setup_portfolios.offering_asset_id, bob.did()) > 0u32.into(), "invest");
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

    enable_offchain_funding {
        let id = FundraiserId(0);
        let alice = <UserBuilder<T>>::default().generate_did().build("Alice");
        let bob = <UserBuilder<T>>::default().generate_did().build("Bob");
        let ticker = Ticker::from_slice_truncated(b"TEST");
        let setup_portfolios = setup_fundraiser::<T>(&alice, &bob, 1);
    }: _(alice.origin(), setup_portfolios.offering_asset_id, id, ticker)
}
