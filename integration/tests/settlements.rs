use anyhow::Result;
use std::collections::BTreeSet;

use integration::*;
use polymesh_api::types::polymesh_primitives::{
    asset::{AssetName, AssetType},
    identity_id::{PortfolioId, PortfolioKind},
    settlement::{Leg, LegId, ReceiptDetails, SettlementType, VenueDetails, VenueId, VenueType},
};

/// Asset Helper.
pub struct AssetHelper {
    pub api: Api,
    pub asset_id: AssetId,
    pub issuer: User,
    pub issuer_venue_id: VenueId,
    pub issuer_did: IdentityId,
}

impl AssetHelper {
    /// Create a new asset, mint some tokens, and pause compliance rules.
    pub async fn new(
        api: &Api,
        issuer: &mut User,
        name: &str,
        mint: u128,
        signers: Vec<AccountId>,
    ) -> Result<Self> {
        // Create a new venue.
        let mut venue_res = api
            .call()
            .settlement()
            .create_venue(
                VenueDetails(format!("Venue for {name}").into()),
                signers,
                VenueType::Other,
            )?
            .submit_and_watch(issuer)
            .await?;

        // Create a new asset.
        let mut asset_res = api
            .call()
            .asset()
            .create_asset(
                AssetName(name.into()),
                true, // Divisible token.
                AssetType::EquityCommon,
                vec![],
                None,
            )?
            .submit_and_watch(issuer)
            .await?;

        // Get the asset ID from the response.
        let asset_id = get_asset_id(&mut asset_res)
            .await?
            .expect("Asset ID not found");

        // Mint some tokens.
        let mut mint_res = api
            .call()
            .asset()
            .issue(asset_id, mint * ONE_POLYX, PortfolioKind::Default)?
            .submit_and_watch(issuer)
            .await?;

        // Pause compliance rules to allow transfers.
        let mut pause_res = api
            .call()
            .compliance_manager()
            .pause_asset_compliance(asset_id)?
            .submit_and_watch(issuer)
            .await?;

        // Wait for mint and pause to complete.
        mint_res.ok().await?;
        pause_res.ok().await?;

        // Get the venue ID from the response.
        let issuer_venue_id = get_venue_id(&mut venue_res)
            .await?
            .expect("Venue ID not found");

        Ok(Self {
            api: api.clone(),
            asset_id,
            issuer: issuer.clone(),
            issuer_venue_id,
            issuer_did: issuer.did.expect("Issuer DID"),
        })
    }

    /// Mint some more tokens.
    pub async fn mint(&mut self, amount: u128) -> Result<()> {
        // Mint some tokens.
        let mut mint_res = self
            .api
            .call()
            .asset()
            .issue(self.asset_id, amount * ONE_POLYX, PortfolioKind::Default)?
            .submit_and_watch(&mut self.issuer)
            .await?;

        // Wait for mint to complete.
        mint_res.ok().await?;

        Ok(())
    }

    /// Fund the investors portfolio with some tokens.
    pub async fn fund_investors(
        &mut self,
        investors: &mut [&mut User],
        amount: u128,
    ) -> Result<()> {
        let amount: u128 = amount * ONE_POLYX;

        // Make sure the asset issuer has enough tokens.
        let total = amount * investors.len() as u128;
        self.mint(total).await?;

        // Issuer portfolios.
        let issuer_portfolio = PortfolioId {
            did: self.issuer_did,
            kind: PortfolioKind::Default,
        };
        let issuer_portfolios = [issuer_portfolio].into_iter().collect::<BTreeSet<_>>();

        let mut pending_settlements = Vec::new();
        for batch in investors.chunks_mut(10) {
            let mut legs = Vec::new();
            for investor in batch.iter() {
                // Get the investor DID.
                let investor_did = investor.did.expect("Investor DID");

                // User portfolios.
                let investor_portfolio = PortfolioId {
                    did: investor_did,
                    kind: PortfolioKind::Default,
                };

                // Create a simple Settlement to transfer tokens from the issuer to the investor.
                legs.push(Leg::Fungible {
                    sender: issuer_portfolio,
                    receiver: investor_portfolio,
                    asset_id: self.asset_id,
                    amount,
                });
            }
            let leg_count = legs.len() as u32;

            // Create a simple Settlement to transfer tokens from the issuer to the investors.
            let mut settlement_res = self
                .api
                .call()
                .settlement()
                .add_and_affirm_instruction(
                    Some(self.issuer_venue_id),
                    SettlementType::SettleManual(0),
                    None,
                    None,
                    legs.clone(),
                    issuer_portfolios.clone(),
                    None,
                )?
                .submit_and_watch(&mut self.issuer)
                .await?;

            // Get the settlement ID from the response.
            let settlement_id = get_instruction_id(&mut settlement_res)
                .await?
                .expect("Settlement ID not found");

            // The investors need to affirm the settlement.
            let mut pending_affirms = Vec::new();
            for investor in batch.iter_mut() {
                let affirm_res = self
                    .api
                    .call()
                    .settlement()
                    .affirm_instruction(
                        settlement_id,
                        vec![PortfolioId {
                            did: investor.did.expect("Investor DID"),
                            kind: PortfolioKind::Default,
                        }]
                        .into_iter()
                        .collect(),
                    )?
                    .submit_and_watch(*investor)
                    .await?;
                pending_affirms.push(affirm_res);
            }

            // Wait for investors affirmations to complete.
            for mut affirm_res in pending_affirms {
                affirm_res.ok().await?;
            }

            // Execute the settlement
            let execute_res = self
                .api
                .call()
                .settlement()
                .execute_manual_instruction(settlement_id, None, leg_count, 0, 0, None)?
                .submit_and_watch(&mut self.issuer)
                .await?;
            pending_settlements.push(execute_res);
        }

        // Wait for the settlements to complete.
        for mut execute_res in pending_settlements {
            execute_res.ok().await?;
        }

        Ok(())
    }
}

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
        vec![],
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

    // Create a new venue.
    let mut venue_res = tester
        .api
        .call()
        .settlement()
        .create_venue(VenueDetails(b"Test1".to_vec()), vec![], VenueType::Other)?
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

    // Mint some tokens.
    let mut mint_res = tester
        .api
        .call()
        .asset()
        .issue(asset_id, 1_000_000 * ONE_POLYX, PortfolioKind::Default)?
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
                sender: issuer_portfolio,
                receiver: investor_portfolio,
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
        .affirm_instruction(settlement_id, vec![issuer_portfolio].into_iter().collect())?
        .submit_and_watch(&mut asset_issuer)
        .await?;

    // The investor needs to affirm the settlement.
    let mut affirm_res2 = tester
        .api
        .call()
        .settlement()
        .affirm_instruction(
            settlement_id,
            vec![investor_portfolio].into_iter().collect(),
        )?
        .submit_and_watch(&mut investor)
        .await?;

    // Wait for the affirmations to complete.
    affirm_res.ok().await?;
    affirm_res2.ok().await?;

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
        1_000_000,
        vec![signer1.account()],
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

    let ticker = Ticker(*b"OFFCHAIN0000");
    let amount = 10u128;

    // Create a Settlement to transfer offchain assets using a receipt signed by a venue signer.
    let mut settlement_res = tester
        .api
        .call()
        .settlement()
        .add_and_affirm_instruction(
            Some(venue_id),
            SettlementType::SettleManual(0),
            None,
            None,
            vec![
                Leg::Fungible {
                    sender: issuer_portfolio,
                    receiver: investor_portfolio,
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
            [issuer_portfolio].into(),
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

    // The investor needs to affirm the settlement with the offchain receipt.
    let mut affirm_res = tester
        .api
        .call()
        .settlement()
        .affirm_with_receipts(
            instruction_id,
            vec![ReceiptDetails {
                uid,
                instruction_id,
                leg_id,
                signer: signer1.account(),
                signature: sig,
                metadata: None,
            }],
            [investor_portfolio].into(),
        )?
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
