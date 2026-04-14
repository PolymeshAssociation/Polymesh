use std::collections::BTreeSet;

use anyhow::Result;

use integration::*;
use polymesh_api::types::polymesh_primitives::{
    asset::{AssetName, AssetType},
    identity_id::{PortfolioId, PortfolioKind},
    settlement::{Leg, LegId, ReceiptDetails, SettlementType, VenueDetails, VenueType},
};

#[cfg(feature = "current_release")]
use polymesh_api::types::polymesh_primitives::asset::{AssetHolder, AssetHolderKind};

/// Test the asset helper.
#[tokio::test]
async fn asset_helper() -> Result<()> {
    let mut tester = PolymeshTester::new().await?;
    let mut users = tester
        .users(&["AssetIssuer1", "Investor1"])
        .await?
        .into_iter();
    let mut asset_issuer = users.next().expect("Asset issuer");
    let mut investor = users.next().expect("Investor");

    // Create a new asset and mint some tokens.
    let mut asset_helper = AssetHelper::new(
        &tester.api,
        &mut asset_issuer,
        "TestAsset",
        1_000_000,
        BTreeSet::new(),
    )
    .await?;

    // Fund the investors portfolio with some tokens.
    asset_helper
        .fund_investors(&mut [&mut investor], 10)
        .await?;

    Ok(())
}

/// Test for a simple settlement.
#[tokio::test]
#[allow(unused_mut)]
async fn simple_settlement() -> Result<()> {
    let mut tester = PolymeshTester::new().await?;
    let mut users = tester
        .users(&["VenueUser1", "AssetIssuer1", "Investor1"])
        .await?
        .into_iter();
    let mut venue = users.next().expect("Venue user");
    let mut asset_issuer = users.next().expect("Asset issuer");
    let mut investor = users.next().expect("Investor");

    // Get the DIDs of the users.
    let issuer_did = asset_issuer.did.expect("Asset issuer DID");
    let investor_did = investor.did.expect("Investor DID");

    // User portfolios.
    let issuer_portfolio = PortfolioId {
        did: issuer_did,
        kind: PortfolioKind::Default,
    };
    let investor_portfolio = PortfolioId {
        did: investor_did,
        kind: PortfolioKind::Default,
    };

    #[cfg(feature = "current_release")]
    let issuer = AssetHolder::Portfolio(issuer_portfolio);
    #[cfg(feature = "previous_release")]
    let issuer = issuer_portfolio;

    #[cfg(feature = "current_release")]
    let investor_holding = AssetHolder::Portfolio(investor_portfolio);
    #[cfg(feature = "previous_release")]
    let investor_holding = investor_portfolio;

    // Create a new venue.
    let mut venue_res = tester
        .api
        .call()
        .settlement()
        .create_venue(VenueDetails(b"Test1".to_vec()), BTreeSet::new(), VenueType::Other)?
        .submit_and_watch(&mut venue)
        .await?;

    // Create a new asset.
    let mut asset_res = tester
        .api
        .call()
        .asset()
        .create_asset(
            AssetName(b"TestAsset".to_vec()),
            true, // Divisible token.
            AssetType::EquityCommon,
            vec![],
            None,
        )?
        .submit_and_watch(&mut asset_issuer)
        .await?;

    // Get the venue ID from the response.
    let venue_id = get_venue_id(&mut venue_res)
        .await?
        .expect("Venue ID not found");

    // Get the asset ID from the response.
    let asset_id = get_asset_id(&mut asset_res)
        .await?
        .expect("Asset ID not found");

    #[cfg(feature = "current_release")]
    let kind = AssetHolderKind::DefaultPortfolio;
    #[cfg(feature = "previous_release")]
    let kind = PortfolioKind::Default;

    // Mint some tokens.
    let mut mint_res = tester
        .api
        .call()
        .asset()
        .issue(asset_id, 1_000_000 * ONE_POLYX, kind)?
        .submit_and_watch(&mut asset_issuer)
        .await?;

    // Pause compliance rules to allow transfers.
    let mut pause_res = tester
        .api
        .call()
        .compliance_manager()
        .pause_asset_compliance(asset_id)?
        .submit_and_watch(&mut asset_issuer)
        .await?;

    // Create a simple Settlement to transfer tokens from the issuer to the investor.
    let mut settlement_res = tester
        .api
        .call()
        .settlement()
        .add_instruction(
            Some(venue_id),
            SettlementType::SettleManual(0),
            None,
            None,
            vec![Leg::Fungible {
                sender: issuer.clone(),
                receiver: investor_holding.clone(),
                asset_id,
                amount: 10 * ONE_POLYX,
            }],
            None,
        )?
        .submit_and_watch(&mut venue)
        .await?;

    // Get the settlement ID from the response.
    let settlement_id = get_instruction_id(&mut settlement_res)
        .await?
        .expect("Settlement ID not found");

    // Wait for mint and pause to complete.
    mint_res.ok().await?;
    pause_res.ok().await?;

    // The issuer needs to affirm the settlement.
    let mut affirm_res = tester
        .api
        .call()
        .settlement()
        .affirm_instruction(settlement_id, vec![issuer].into_iter().collect())?
        .submit_and_watch(&mut asset_issuer)
        .await?;

    // Wait for the issuer affirmation to complete.
    affirm_res.ok().await?;

    // On the previous release, the investor needs to affirm the settlement.
    #[cfg(feature = "previous_release")]
    {
        let mut affirm_res2 = tester
            .api
            .call()
            .settlement()
            .affirm_instruction(settlement_id, vec![investor_holding].into_iter().collect())?
            .submit_and_watch(&mut investor)
            .await?;
        affirm_res2.ok().await?;
    }

    // Execute the settlement
    let mut execute_res = tester
        .api
        .call()
        .settlement()
        .execute_manual_instruction(settlement_id, None, 1, 0, 0, None)?
        .submit_and_watch(&mut venue)
        .await?;

    // Wait for the settlement to complete.
    execute_res.ok().await?;

    Ok(())
}

/// Test a settlement with offchain leg and onchain fungible leg.
#[tokio::test]
async fn offchain_settlement() -> Result<()> {
    let mut tester = PolymeshTester::new().await?;
    let mut users = tester
        .users(&["VenueUser1", "VenueSigner1", "Investor1"])
        .await?
        .into_iter();
    let mut venue = users.next().expect("Venue user");
    let signer1 = users.next().expect("Venue signer 1");
    let mut investor1 = users.next().expect("Investor1");

    // Create a new asset and mint some tokens.
    let asset_helper = AssetHelper::new(
        &tester.api,
        &mut venue,
        "TestAsset",
        1_000_000_000,
        BTreeSet::from([signer1.account()]),
    )
    .await?;
    let venue_id = asset_helper.issuer_venue_id;
    let venue_did = asset_helper.issuer_did;

    // Get the DIDs of the users.
    let sender_identity = investor1.did.expect("Investor 1 DID");
    let receiver_identity = venue_did;

    // User portfolios.
    let issuer_portfolio = PortfolioId {
        did: venue_did,
        kind: PortfolioKind::Default,
    };
    let investor_portfolio = PortfolioId {
        did: sender_identity,
        kind: PortfolioKind::Default,
    };

    #[cfg(feature = "current_release")]
    let issuer_holding = AssetHolder::Portfolio(issuer_portfolio);
    #[cfg(feature = "previous_release")]
    let issuer_holding = issuer_portfolio;

    #[cfg(feature = "current_release")]
    let investor_holding = AssetHolder::Portfolio(investor_portfolio);
    #[cfg(feature = "previous_release")]
    let investor_holding = investor_portfolio;

    let ticker = Ticker(*b"OFFCHAIN0000");
    let amount = 10u128;

    // Create a Settlement to transfer offchain assets using a receipt signed by a venue signer.
    let mut settlement_res = tester
        .api
        .call()
        .settlement()
        .add_and_affirm_instruction(
            venue_id,
            SettlementType::SettleManual(0),
            None,
            None,
            vec![
                Leg::Fungible {
                    sender: issuer_holding.clone(),
                    receiver: investor_holding.clone(),
                    asset_id: asset_helper.asset_id,
                    amount: 10 * ONE_POLYX,
                },
                Leg::OffChain {
                    sender_identity,
                    receiver_identity,
                    ticker,
                    amount,
                },
            ],
            [issuer_holding].into(),
            None,
        )?
        .submit_and_watch(&mut venue)
        .await?;

    // Get the settlement ID from the response.
    let instruction_id = get_instruction_id(&mut settlement_res)
        .await?
        .expect("Settlement ID not found");

    // Create a receipt for the offchain asset and sign it with the venue signer.
    let uid = 0u64;
    let leg_id = LegId(1u64);
    #[cfg(feature = "previous_release")]
    let receipt_details = {
        let receipt = Receipt {
            uid,
            instruction_id,
            leg_id,
            sender_identity,
            receiver_identity,
            ticker,
            amount,
        };
        let sig = sign_with_key(&signer1, &receipt).await?;

        ReceiptDetails {
            uid,
            instruction_id,
            leg_id,
            signer: signer1.account(),
            signature: sig,
            metadata: None,
        }
    };
    #[cfg(feature = "current_release")]
    let receipt_details = {
        let receipt = Receipt {
            instruction_id,
            leg_id,
            sender_identity,
            receiver_identity,
            ticker,
            amount,
        };
        let message =
            ChainScopedMessage::new(&tester.api, uid, SETTLEMENT_RECEIPT_LABEL, None, &receipt)
                .await?;
        let sig = sign_with_key(&signer1, &message).await?;

        ReceiptDetails {
            uid,
            instruction_id,
            leg_id,
            signer: signer1.account(),
            signature: sig,
            metadata: None,
            expires_at: message.expires_at,
        }
    };

    // The investor needs to affirm the settlement with the offchain receipt.
    let mut affirm_res = tester
        .api
        .call()
        .settlement()
        .affirm_with_receipts(instruction_id, vec![receipt_details], {
            #[cfg(feature = "previous_release")]
            {
                [investor_holding].into()
            }
            #[cfg(feature = "current_release")]
            {
                [].into()
            }
        })?
        .submit_and_watch(&mut investor1)
        .await?;

    // Wait for the affirmations to complete.
    affirm_res.ok().await?;

    // Execute the settlement
    let mut execute_res = tester
        .api
        .call()
        .settlement()
        .execute_manual_instruction(instruction_id, None, 1, 0, 1, None)?
        .submit_and_watch(&mut venue)
        .await?;

    // Wait for the settlement to complete.
    execute_res.ok().await?;

    Ok(())
}
