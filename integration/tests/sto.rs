use anyhow::Result;

use integration::*;
use polymesh_api::types::pallet_sto::{FundingMethod, FundraiserName, PriceTier};
use polymesh_api::types::polymesh_primitives::{
    identity_id::{PortfolioId, PortfolioKind},
    settlement::{VenueDetails, VenueType},
};

/// Test a STO with onchain asset funding.
#[tokio::test]
async fn sto_onchain_funding() -> Result<()> {
    let mut tester = PolymeshTester::new().await?;
    let mut users = tester
        .users(&["VenueUser1", "Investor1"])
        .await?
        .into_iter();
    let mut venue = users.next().expect("Venue user");
    let mut investor1 = users.next().expect("Investor1");

    // Create a new venue.
    let mut v = venue.clone();
    let mut sto_venue_res = tester
        .api
        .call()
        .settlement()
        .create_venue(
            VenueDetails(format!("Venue for STO").into()),
            vec![],
            VenueType::Sto,
        )?
        .submit_and_watch(&mut v)
        .await?;

    // Create two assets one as the offering asset and one as the funding asset.
    let mut v = venue.clone();
    let api = tester.api.clone();
    let offering_asset = tokio::spawn(async move {
        AssetHelper::new(&api, &mut v, "TestOfferingAsset", 1_000_000, vec![]).await
    });
    let mut v = venue.clone();
    let api = tester.api.clone();
    let mut investor = investor1.clone();
    let funding_asset = tokio::spawn(async move {
        let mut funding_asset =
            AssetHelper::new(&api, &mut v, "TestFundingCoin", 1_000_000, vec![]).await?;

        // Give some funds to the investor.
        funding_asset
            .fund_investors(&mut [&mut investor], 100_000_000)
            .await?;

        Ok::<_, anyhow::Error>(funding_asset)
    });

    // Wait for the assets to be created.
    let offering_asset = offering_asset.await??;
    let funding_asset = funding_asset.await??;
    let venue_did = offering_asset.issuer_did;

    // Get the DIDs of the users.
    let investor1_did = investor1.did.expect("Investor 1 DID");

    // STO fundraiser portfolios.
    let fundraiser_portfolio = PortfolioId {
        did: venue_did,
        kind: PortfolioKind::Default,
    };
    let investor_portfolio = PortfolioId {
        did: investor1_did,
        kind: PortfolioKind::Default,
    };

    // Get the venue ID from the response.
    let sto_venue_id = get_venue_id(&mut sto_venue_res)
        .await?
        .expect("STO Venue ID not found");

    // Create the fundraiser using the STO pallet.
    let mut fundraiser_res = tester
        .api
        .call()
        .sto()
        .create_fundraiser(
            fundraiser_portfolio,
            offering_asset.asset_id,
            fundraiser_portfolio,
            funding_asset.asset_id,
            vec![
                PriceTier {
                    total: 100,
                    price: 500_000, // 1 offering token = 0.5 funding token
                },
                PriceTier {
                    total: 100,
                    price: 1_000_000, // 1 offering token = 1.0 funding token
                },
                PriceTier {
                    total: 100,
                    price: 2_000_000, // 1 offering token = 2.0 funding token
                },
            ],
            sto_venue_id,
            None,
            None,
            1u128,
            FundraiserName("TestFundraiser".into()),
        )?
        .submit_and_watch(&mut venue)
        .await?;

    // Get the fundraiser ID from the response.
    let (_, fundraiser_id) = get_fundraiser_id(&mut fundraiser_res)
        .await?
        .expect("Fundraiser ID not found");

    // Invest in the fundraiser using onchain asset funding.
    let mut invest_res = tester
        .api
        .call()
        .sto()
        .invest(
            offering_asset.asset_id,
            fundraiser_id,
            investor_portfolio,
            FundingMethod::OnChain(investor_portfolio),
            200,             // 200 tokens (avg price 0.75 funding tokens per offering token).
            Some(1_000_000), // pay a maximum of 1.0 funding tokens per offering token.
        )?
        .submit_and_watch(&mut investor1)
        .await?;

    // Wait for the investment to be processed.
    invest_res.ok().await?;

    Ok(())
}
