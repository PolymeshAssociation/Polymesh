use anyhow::Result;
use codec::{Decode, Encode};

use integration::*;
use polymesh_api::types::pallet_sto::{FundingMethod, FundraiserName, PriceTier};
use polymesh_api::types::polymesh_primitives::{
    identity_id::{PortfolioId, PortfolioKind},
    settlement::{VenueDetails, VenueType},
    sto::{FundraiserId, FundraiserReceiptDetails},
};

/// An offchain fundraiser receipt.
#[derive(Encode, Decode, Clone, Debug)]
pub struct FundraiserReceipt {
    uid: u64,
    fundraiser_id: FundraiserId,
    sender_identity: IdentityId,
    receiver_identity: IdentityId,
    ticker: Ticker,
    amount: u128,
}

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

/// Test STO with offchain asset funding.
#[tokio::test]
async fn sto_offchain_funding() -> Result<()> {
    let mut tester = PolymeshTester::new().await?;
    let mut users = tester
        .users(&["VenueUser1", "VenueSigner1", "Investor1"])
        .await?
        .into_iter();
    let mut venue = users.next().expect("Venue user");
    let signer1 = users.next().expect("Venue signer 1");
    let mut investor1 = users.next().expect("Investor1");

    // Create a new venue.
    let mut v = venue.clone();
    let mut sto_venue_res = tester
        .api
        .call()
        .settlement()
        .create_venue(
            VenueDetails(format!("Venue for STO").into()),
            vec![signer1.account()],
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
            .fund_investors(&mut [&mut investor], 200_000_000)
            .await?;

        Ok::<_, anyhow::Error>(funding_asset)
    });

    // Wait for the assets to be created.
    let offering_asset = offering_asset.await??;
    let funding_asset = funding_asset.await??;
    let venue_did = offering_asset.issuer_did;

    // A ticker for the offchain asset.
    let ticker = Ticker(*b"OFFCHAIN0000");

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

    // Enable offchain asset funding for the venue.
    let mut enable_offchain_funding_res = tester
        .api
        .call()
        .sto()
        .enable_offchain_funding(offering_asset.asset_id, fundraiser_id.clone(), ticker)?
        .submit_and_watch(&mut venue)
        .await?;

    // Create a receipt for the offchain asset funding.
    let uid = 0u64;
    let amount = 150u128;
    let receipt = FundraiserReceipt {
        uid,
        fundraiser_id: fundraiser_id.clone(),
        sender_identity: investor1_did,
        receiver_identity: venue_did,
        ticker,
        amount,
    };
    eprintln!("Receipt: {:?}", receipt);
    let sig = sign_with_key(&signer1, &receipt, false).await?;
    let receipt_details = FundraiserReceiptDetails {
        uid,
        signer: signer1.account(),
        signature: sig,
        metadata: None,
    };

    // Ensure the venue has the offchain asset funding enabled.
    enable_offchain_funding_res.ok().await?;

    // Invest in the fundraiser using offchain asset funding.
    let mut offchain_invest_res = tester
        .api
        .call()
        .sto()
        .invest(
            offering_asset.asset_id,
            fundraiser_id.clone(),
            investor_portfolio,
            FundingMethod::OffChain(receipt_details),
            200,             // 200 tokens (avg price 0.75 funding tokens per offering token).
            Some(1_000_000), // pay a maximum of 1.0 funding tokens per offering token.
        )?
        .submit_and_watch(&mut investor1)
        .await?;

    // Also invest in the fundraiser using onchain asset funding.
    let mut invest_res = tester
        .api
        .call()
        .sto()
        .invest(
            offering_asset.asset_id,
            fundraiser_id,
            investor_portfolio,
            FundingMethod::OnChain(investor_portfolio),
            100,             // 100 tokens (avg price 2.00 funding tokens per offering token).
            Some(2_000_000), // pay a maximum of 2.0 funding tokens per offering token.
        )?
        .submit_and_watch(&mut investor1)
        .await?;

    // Wait for the investments to be processed.
    offchain_invest_res.ok().await?;
    invest_res.ok().await?;

    Ok(())
}
