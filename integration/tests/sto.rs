use std::collections::BTreeSet;

use anyhow::Result;

use integration::*;
use polymesh_api::types::pallet_sto::{FundingMethod, FundraiserName, PriceTier};
use polymesh_api::types::polymesh_primitives::{
    identity_id::{PortfolioId, PortfolioKind},
    settlement::{VenueDetails, VenueType},
};

/// Test a STO with onchain asset funding.
#[tokio::test]
#[test_log::test]
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
            Default::default(),
            VenueType::Sto,
        )?
        .submit_and_watch(&mut v)
        .await?;

    // Create two assets one as the offering asset and one as the funding asset.
    let mut v = venue.clone();
    let api = tester.api.clone();
    let offering_asset = tokio::spawn(async move {
        // Mint 10,000.0 tokens.
        AssetHelper::new(
            &api,
            &mut v,
            "TestOfferingAsset",
            10_000_000_000,
            BTreeSet::new(),
        )
        .await
    });
    let mut v = venue.clone();
    let api = tester.api.clone();
    let mut investor = investor1.clone();
    let funding_asset = tokio::spawn(async move {
        let mut funding_asset =
            AssetHelper::new(&api, &mut v, "TestFundingCoin", 1_000_000, BTreeSet::new()).await?;

        // Give 1,000.0 funds to the investor.
        funding_asset
            .fund_investors(&mut [&mut investor], 1_000_000_000)
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
                    total: 1_000_000_000, // 1,000.0 tokens.
                    price: 800_000,       // 1 offering token = 0.8 funding token
                },
                PriceTier {
                    total: 1_000_000_000, // 1,000.0 tokens.
                    price: 1_600_000,     // 1 offering token = 1.6 funding token
                },
                PriceTier {
                    total: 1_000_000_000, // 1,000.0 tokens.
                    price: 2_400_000,     // 1 offering token = 2.4 funding token
                },
            ],
            sto_venue_id,
            None,
            None,
            1_000_000u128,
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
            1_050_000_000, // 1,050.0 tokens (avg price 0.838095 funding tokens per offering token).
            Some(900_000), // pay a maximum of 0.9 funding tokens per offering token.
        )?
        .submit_and_watch(&mut investor1)
        .await?;

    // Wait for the investment to be processed.
    invest_res.ok().await?;

    Ok(())
}

// <=v7.4
#[cfg(feature = "previous_release")]
mod sto_v7_tests {
    use super::*;
    use codec::{Decode, Encode};

    use polymesh_api::types::polymesh_primitives::sto::{FundraiserId, FundraiserReceiptDetails};

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

    /// Test STO with offchain asset funding.
    #[tokio::test]
    #[test_log::test]
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
                [signer1.account()].into(),
                VenueType::Sto,
            )?
            .submit_and_watch(&mut v)
            .await?;

        // Create two assets one as the offering asset and one as the funding asset.
        let mut v = venue.clone();
        let api = tester.api.clone();
        let offering_asset = tokio::spawn(async move {
            // Mint 10,000.0 tokens.
            AssetHelper::new(
                &api,
                &mut v,
                "TestOfferingAsset",
                10_000_000_000,
                BTreeSet::new(),
            )
            .await
        });
        let mut v = venue.clone();
        let api = tester.api.clone();
        let mut investor = investor1.clone();
        let funding_asset = tokio::spawn(async move {
            let mut funding_asset =
                AssetHelper::new(&api, &mut v, "TestFundingCoin", 1_000_000, BTreeSet::new())
                    .await?;

            // Give 3,000.0 funds to the investor.
            funding_asset
                .fund_investors(&mut [&mut investor], 3_000_000_000)
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
                        total: 1_000_000_000, // 1,000.0 tokens.
                        price: 800_000,       // 1 offering token = 0.8 funding token
                    },
                    PriceTier {
                        total: 1_000_000_000, // 1,000.0 tokens.
                        price: 1_600_000,     // 1 offering token = 1.6 funding token
                    },
                    PriceTier {
                        total: 1_000_000_000, // 1,000.0 tokens.
                        price: 2_400_000,     // 1 offering token = 2.4 funding token
                    },
                ],
                sto_venue_id,
                None,
                None,
                1_000_000u128,
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

        let offchain_purchase_amount = 2_000_000_000u128; // 2,000 tokens (avg price 1.2 funding tokens per offering token).
        let offchain_funding_amount = 2_400_000_000u128;
        let onchain_purchase_amount = 1_000_000_000u128; // 1,000 tokens (avg price 2.4 funding tokens per offering token).

        // Create a receipt for the offchain asset funding.
        let uid = 0u64;
        let receipt = FundraiserReceipt {
            uid,
            fundraiser_id: fundraiser_id.clone(),
            sender_identity: investor1_did,
            receiver_identity: venue_did,
            ticker,
            amount: offchain_funding_amount,
        };
        let sig = sign_with_key(&signer1, &receipt).await?;

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
                offchain_purchase_amount,
                Some(1_300_000), // pay a maximum of 1.3 funding tokens per offering token.
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
                onchain_purchase_amount,
                Some(2_400_000), // pay a maximum of 2.4 funding tokens per offering token.
            )?
            .submit_and_watch(&mut investor1)
            .await?;

        // Wait for the investments to be processed.
        offchain_invest_res.ok().await?;
        invest_res.ok().await?;

        Ok(())
    }
}
// >=v8.0
#[cfg(feature = "current_release")]
mod sto_v8_tests {
    use super::*;
    use codec::{Decode, Encode};

    use polymesh_api::types::polymesh_primitives::sto::{FundraiserId, FundraiserReceiptDetails};

    /// An offchain fundraiser receipt.
    #[derive(Encode, Decode, Clone, Debug)]
    pub struct FundraiserReceipt {
        fundraiser_id: FundraiserId,
        sender_identity: IdentityId,
        receiver_identity: IdentityId,
        ticker: Ticker,
        amount: u128,
    }

    /// Test STO with offchain asset funding.
    #[tokio::test]
    #[test_log::test]
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
                BTreeSet::from([signer1.account()]),
                VenueType::Sto,
            )?
            .submit_and_watch(&mut v)
            .await?;

        // Create two assets one as the offering asset and one as the funding asset.
        let mut v = venue.clone();
        let api = tester.api.clone();
        let offering_asset = tokio::spawn(async move {
            // Mint 10,000.0 tokens.
            AssetHelper::new(
                &api,
                &mut v,
                "TestOfferingAsset",
                10_000_000_000,
                BTreeSet::new(),
            )
            .await
        });
        let mut v = venue.clone();
        let api = tester.api.clone();
        let mut investor = investor1.clone();
        let funding_asset = tokio::spawn(async move {
            let mut funding_asset =
                AssetHelper::new(&api, &mut v, "TestFundingCoin", 1_000_000, BTreeSet::new())
                    .await?;

            // Give 3,000.0 funds to the investor.
            funding_asset
                .fund_investors(&mut [&mut investor], 3_000_000_000)
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
                        total: 1_000_000_000, // 1,000.0 tokens.
                        price: 800_000,       // 1 offering token = 0.8 funding token
                    },
                    PriceTier {
                        total: 1_000_000_000, // 1,000.0 tokens.
                        price: 1_600_000,     // 1 offering token = 1.6 funding token
                    },
                    PriceTier {
                        total: 1_000_000_000, // 1,000.0 tokens.
                        price: 2_400_000,     // 1 offering token = 2.4 funding token
                    },
                ],
                sto_venue_id,
                None,
                None,
                1_000_000u128,
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

        let offchain_purchase_amount = 2_000_000_000u128; // 2,000 tokens (avg price 1.2 funding tokens per offering token).
        let offchain_funding_amount = 2_400_000_000u128;
        let onchain_purchase_amount = 1_000_000_000u128; // 1,000 tokens (avg price 2.4 funding tokens per offering token).

        // Create a receipt for the offchain asset funding.
        let uid = 0u64;
        let receipt = FundraiserReceipt {
            fundraiser_id: fundraiser_id.clone(),
            sender_identity: investor1_did,
            receiver_identity: venue_did,
            ticker,
            amount: offchain_funding_amount,
        };
        let message = ChainScopedMessage::new(
            &tester.api,
            uid,
            STO_FUNDRAISER_RECEIPT_LABEL,
            None,
            &receipt,
        )
        .await?;
        let sig = sign_with_key(&signer1, &message).await?;

        let receipt_details = FundraiserReceiptDetails {
            uid,
            signer: signer1.account(),
            signature: sig,
            metadata: None,
            expires_at: message.expires_at,
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
                offchain_purchase_amount,
                Some(1_300_000), // pay a maximum of 1.3 funding tokens per offering token.
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
                onchain_purchase_amount,
                Some(2_400_000), // pay a maximum of 2.4 funding tokens per offering token.
            )?
            .submit_and_watch(&mut investor1)
            .await?;

        // Wait for the investments to be processed.
        offchain_invest_res.ok().await?;
        invest_res.ok().await?;

        Ok(())
    }
}

// >=v8.0 - Additional STO tests for freeze/unfreeze/modify/stop
#[cfg(feature = "current_release")]
mod sto_extended_tests {
    use super::*;

    struct StoSetup {
        offering: AssetId,
        fundraiser_id: polymesh_api::types::polymesh_primitives::sto::FundraiserId,
        investor_portfolio: PortfolioId,
    }

    async fn setup_sto(
        tester: &mut PolymeshTester,
        venue_user: &mut User,
        investor: &mut User,
        offering_name: &str,
        funding_name: &str,
        venue_name: &str,
    ) -> Result<StoSetup> {
        let mut venue_res = tester
            .api
            .call()
            .settlement()
            .create_venue(
                VenueDetails(venue_name.as_bytes().to_vec()),
                Default::default(),
                VenueType::Sto,
            )?
            .submit_and_watch(venue_user)
            .await?;
        venue_res.ok().await?;
        let venue_id = get_venue_id(&mut venue_res).await?.expect("venue");

        let mut v = venue_user.clone();
        let api = tester.api.clone();
        let offering_name = offering_name.to_string();
        let offering = tokio::spawn(async move {
            AssetHelper::new(
                &api,
                &mut v,
                &offering_name,
                20_000_000_000,
                BTreeSet::new(),
            )
            .await
        });
        let mut v = venue_user.clone();
        let api = tester.api.clone();
        let mut inv = investor.clone();
        let funding_name = funding_name.to_string();
        let funding = tokio::spawn(async move {
            let mut a =
                AssetHelper::new(&api, &mut v, &funding_name, 1_000_000, BTreeSet::new()).await?;
            a.fund_investors(&mut [&mut inv], 1_000_000_000).await?;
            Ok::<_, anyhow::Error>(a)
        });
        let offering = offering.await??;
        let funding = funding.await??;
        let venue_did = offering.issuer_did;
        let investor_did = investor.did.expect("investor did");
        let fundraiser_portfolio = PortfolioId {
            did: venue_did,
            kind: PortfolioKind::Default,
        };
        let investor_portfolio = PortfolioId {
            did: investor_did,
            kind: PortfolioKind::Default,
        };

        let mut fr = tester
            .api
            .call()
            .sto()
            .create_fundraiser(
                fundraiser_portfolio,
                offering.asset_id.clone(),
                fundraiser_portfolio,
                funding.asset_id,
                vec![PriceTier {
                    total: 3_000_000_000,
                    price: 800_000,
                }],
                venue_id,
                None,
                None,
                1_000_000u128,
                FundraiserName(b"ExtFundraiser".to_vec()),
            )?
            .submit_and_watch(venue_user)
            .await?;
        fr.ok().await?;
        let (_, fundraiser_id) = get_fundraiser_id(&mut fr).await?.expect("fundraiser");
        Ok(StoSetup {
            offering: offering.asset_id,
            fundraiser_id,
            investor_portfolio,
        })
    }

    #[tokio::test]
    #[test_log::test]
    async fn sto_freeze_unfreeze() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["StoFrzVenue", "StoFrzInv"])
            .await?
            .into_iter();
        let mut venue = users.next().unwrap();
        let mut investor = users.next().unwrap();
        let setup = setup_sto(
            &mut tester,
            &mut venue,
            &mut investor,
            "StoFrzOff",
            "StoFrzFund",
            "StoFrzVenue",
        )
        .await?;

        tester
            .api
            .call()
            .sto()
            .freeze_fundraiser(setup.offering.clone(), setup.fundraiser_id.clone())?
            .submit_and_watch(&mut venue)
            .await?
            .ok()
            .await?;

        let mut invest = tester
            .api
            .call()
            .sto()
            .invest(
                setup.offering.clone(),
                setup.fundraiser_id.clone(),
                setup.investor_portfolio,
                FundingMethod::OnChain(setup.investor_portfolio),
                1_050_000_000,
                Some(900_000),
            )?
            .submit_and_watch(&mut investor)
            .await?;
        assert!(invest.ok().await.is_err(), "invest while frozen must fail");

        tester
            .api
            .call()
            .sto()
            .unfreeze_fundraiser(setup.offering.clone(), setup.fundraiser_id.clone())?
            .submit_and_watch(&mut venue)
            .await?
            .ok()
            .await?;

        tester
            .api
            .call()
            .sto()
            .invest(
                setup.offering,
                setup.fundraiser_id,
                setup.investor_portfolio,
                FundingMethod::OnChain(setup.investor_portfolio),
                1_050_000_000,
                Some(900_000),
            )?
            .submit_and_watch(&mut investor)
            .await?
            .ok()
            .await?;

        Ok(())
    }

    #[tokio::test]
    #[test_log::test]
    async fn sto_modify_window() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["StoWinVenue", "StoWinInv"])
            .await?
            .into_iter();
        let mut venue = users.next().unwrap();
        let mut investor = users.next().unwrap();
        let setup = setup_sto(
            &mut tester,
            &mut venue,
            &mut investor,
            "StoWinOff",
            "StoWinFund",
            "StoWinVenue",
        )
        .await?;

        let now = tester.api.query().timestamp().now().await?;
        tester
            .api
            .call()
            .sto()
            .modify_fundraiser_window(
                setup.offering.clone(),
                setup.fundraiser_id.clone(),
                now + 60_000,
                Some(now + 120_000),
            )?
            .submit_and_watch(&mut venue)
            .await?
            .ok()
            .await?;

        let mut invest = tester
            .api
            .call()
            .sto()
            .invest(
                setup.offering.clone(),
                setup.fundraiser_id.clone(),
                setup.investor_portfolio,
                FundingMethod::OnChain(setup.investor_portfolio),
                1_050_000_000,
                Some(900_000),
            )?
            .submit_and_watch(&mut investor)
            .await?;
        assert!(
            invest.ok().await.is_err(),
            "invest before the new window must fail"
        );

        tester
            .api
            .call()
            .sto()
            .modify_fundraiser_window(
                setup.offering.clone(),
                setup.fundraiser_id.clone(),
                now.saturating_sub(1_000),
                None,
            )?
            .submit_and_watch(&mut venue)
            .await?
            .ok()
            .await?;

        tester
            .api
            .call()
            .sto()
            .invest(
                setup.offering,
                setup.fundraiser_id,
                setup.investor_portfolio,
                FundingMethod::OnChain(setup.investor_portfolio),
                1_050_000_000,
                Some(900_000),
            )?
            .submit_and_watch(&mut investor)
            .await?
            .ok()
            .await?;

        Ok(())
    }

    #[tokio::test]
    #[test_log::test]
    async fn sto_stop() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["StoStpVenue", "StoStpInv"])
            .await?
            .into_iter();
        let mut venue = users.next().unwrap();
        let mut investor = users.next().unwrap();
        let setup = setup_sto(
            &mut tester,
            &mut venue,
            &mut investor,
            "StoStpOff",
            "StoStpFund",
            "StoStpVenue",
        )
        .await?;

        tester
            .api
            .call()
            .sto()
            .stop(setup.offering.clone(), setup.fundraiser_id.clone())?
            .submit_and_watch(&mut venue)
            .await?
            .ok()
            .await?;

        let mut invest = tester
            .api
            .call()
            .sto()
            .invest(
                setup.offering,
                setup.fundraiser_id,
                setup.investor_portfolio,
                FundingMethod::OnChain(setup.investor_portfolio),
                1_050_000_000,
                Some(900_000),
            )?
            .submit_and_watch(&mut investor)
            .await?;
        assert!(invest.ok().await.is_err(), "invest after stop must fail");

        Ok(())
    }
}
