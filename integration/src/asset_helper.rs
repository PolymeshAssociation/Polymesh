use anyhow::Result;
use std::collections::BTreeSet;

#[cfg(feature = "current_release")]
use polymesh_api::types::polymesh_primitives::{
    asset::{AssetHolder, AssetHolderKind},
    settlement::InstructionId,
};

use polymesh_api::types::polymesh_primitives::{
    asset::{AssetName, AssetType},
    identity_id::{PortfolioId, PortfolioKind},
    settlement::{Leg, SettlementType, VenueDetails, VenueId, VenueType},
};

use crate::*;

/// Get Pending Transfer ID from the transaction results.
#[cfg(feature = "current_release")]
pub async fn get_pending_transfer_id(
    res: &mut TransactionResults,
) -> Result<Option<InstructionId>> {
    if let Some(events) = res.events().await? {
        for rec in &events.0 {
            match &rec.event {
                RuntimeEvent::Asset(AssetEvent::CreatedAssetTransfer {
                    pending_transfer_id,
                    ..
                }) => {
                    return Ok(pending_transfer_id.clone());
                }
                _ => (),
            }
        }
    }
    Ok(None)
}

/// Asset Helper.
pub struct AssetHelper {
    pub api: Api,
    pub asset_id: AssetId,
    pub issuer: User,
    pub issuer_venue_id: Option<VenueId>,
    pub issuer_did: IdentityId,
}

impl AssetHelper {
    /// Create a new asset and mint some tokens.
    pub async fn new(
        api: &Api,
        issuer: &mut User,
        name: &str,
        mint: u128,
        signers: BTreeSet<AccountId>,
    ) -> Result<Self> {
        Self::new_full(api, issuer, name, mint, signers, true, None).await
    }

    /// Create a new asset and mint some tokens.
    pub async fn new_full(
        api: &Api,
        issuer: &mut User,
        name: &str,
        mint: u128,
        signers: BTreeSet<AccountId>,
        need_venue: bool,
        #[cfg(feature = "current_release")] kind: Option<AssetHolderKind>,
        #[cfg(feature = "previous_release")] kind: Option<PortfolioKind>,
    ) -> Result<Self> {
        // Create a new venue.
        let venue_res = if need_venue {
            Some(
                api.call()
                    .settlement()
                    .create_venue(
                        VenueDetails(format!("Venue for {name}").into()),
                        signers,
                        VenueType::Other,
                    )?
                    .submit_and_watch(issuer)
                    .await?,
            )
        } else {
            None
        };

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
        #[cfg(feature = "current_release")]
        let kind = kind.unwrap_or(AssetHolderKind::DefaultPortfolio);
        #[cfg(feature = "previous_release")]
        let kind = kind.unwrap_or(PortfolioKind::Default);

        let mut mint_res = api
            .call()
            .asset()
            .issue(asset_id, mint, kind)?
            .submit_and_watch(issuer)
            .await?;

        // Wait for mint to complete.
        mint_res.ok().await?;

        // Get the venue ID from the response.
        let issuer_venue_id = if let Some(mut venue_res) = venue_res {
            Some(
                get_venue_id(&mut venue_res)
                    .await?
                    .expect("Venue ID not found"),
            )
        } else {
            None
        };

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
        #[cfg(feature = "current_release")]
        let kind = AssetHolderKind::DefaultPortfolio;
        #[cfg(feature = "previous_release")]
        let kind = PortfolioKind::Default;

        // Mint some tokens.
        let mut mint_res = self
            .api
            .call()
            .asset()
            .issue(self.asset_id, amount, kind)?
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
        // Make sure the asset issuer has enough tokens.
        let total = amount * investors.len() as u128;
        self.mint(total).await?;

        let issuer_portfolio = PortfolioId {
            did: self.issuer_did,
            kind: PortfolioKind::Default,
        };

        // Issuer.
        #[cfg(feature = "current_release")]
        let issuer = AssetHolder::Portfolio(issuer_portfolio);
        #[cfg(feature = "previous_release")]
        let issuer = issuer_portfolio;

        let issuer_holdings = [issuer.clone()].into_iter().collect::<BTreeSet<_>>();

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

                #[cfg(feature = "current_release")]
                let investor = AssetHolder::Portfolio(investor_portfolio);
                #[cfg(feature = "previous_release")]
                let investor = investor_portfolio;

                // Create a simple Settlement to transfer tokens from the issuer to the investor.
                legs.push(Leg::Fungible {
                    sender: issuer.clone(),
                    receiver: investor,
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
                    self.issuer_venue_id,
                    SettlementType::SettleManual(0),
                    None,
                    None,
                    legs.clone(),
                    issuer_holdings.clone(),
                    None,
                )?
                .submit_and_watch(&mut self.issuer)
                .await?;

            // Get the settlement ID from the response.
            let settlement_id = get_instruction_id(&mut settlement_res)
                .await?
                .expect("Settlement ID not found");

            // On the previous release, investors need to affirm the settlement.
            #[cfg(feature = "previous_release")]
            {
                let mut pending_affirms = Vec::new();
                for investor in batch.iter_mut() {
                    let inv_holding = PortfolioId {
                        did: investor.did.expect("Investor DID"),
                        kind: PortfolioKind::Default,
                    };

                    let affirm_res = self
                        .api
                        .call()
                        .settlement()
                        .affirm_instruction(settlement_id, vec![inv_holding].into_iter().collect())?
                        .submit_and_watch(*investor)
                        .await?;
                    pending_affirms.push(affirm_res);
                }

                // Wait for investors affirmations to complete.
                for mut affirm_res in pending_affirms {
                    affirm_res.ok().await?;
                }
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
