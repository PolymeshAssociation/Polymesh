//! Asset lifecycle controls: freeze, redeem, divisibility, rename, type, docs, identifiers.
#[cfg(feature = "current_release")]
mod asset_controls_tests {
    use std::collections::BTreeSet;

    use anyhow::Result;

    use integration::*;
    use polymesh_api::types::polymesh_primitives::{
        asset::{AssetHolder, AssetHolderKind, AssetName, AssetType},
        asset_identifier::AssetIdentifier,
        document::{Document, DocumentId, DocumentName, DocumentUri},
        identity_id::PortfolioId,
    };

    async fn create_asset(
        tester: &mut PolymeshTester,
        owner: &mut User,
        ticker: &str,
        amount: u128,
        kind: Option<AssetHolderKind>,
    ) -> Result<AssetId> {
        let helper = AssetHelper::new_full(
            &tester.api,
            owner,
            ticker,
            amount,
            BTreeSet::new(),
            false,
            kind,
        )
        .await?;
        Ok(helper.asset_id)
    }

    async fn account_balance(
        tester: &PolymeshTester,
        who: &AccountId,
        asset_id: &AssetId,
    ) -> Result<u128> {
        Ok(tester
            .api
            .query()
            .asset()
            .asset_balance(who.clone(), asset_id.clone())
            .await?)
    }

    /// Freezing an asset blocks transfers until unfrozen.
    #[tokio::test]
    #[test_log::test]
    async fn freeze_blocks_transfers() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["AFzOwner", "AFzInv1"]).await?.into_iter();
        let mut owner = users.next().unwrap();
        let inv1 = users.next().unwrap();

        let asset_id = create_asset(
            &mut tester,
            &mut owner,
            "AFREEZE",
            1_000_000,
            Some(AssetHolderKind::Account),
        )
        .await?;

        // Sanity transfer works.
        tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id.clone(), inv1.account(), 100, None)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // Freeze -> transfer fails.
        tester
            .api
            .call()
            .asset()
            .freeze(asset_id.clone())?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;
        let mut res = tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id.clone(), inv1.account(), 100, None)?
            .submit_and_watch(&mut owner)
            .await?;
        assert!(res.ok().await.is_err(), "frozen asset must not transfer");

        // Unfreeze -> transfers resume.
        tester
            .api
            .call()
            .asset()
            .unfreeze(asset_id.clone())?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;
        tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id.clone(), inv1.account(), 100, None)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        Ok(())
    }

    /// Redeem burns tokens from the caller and reduces total supply.
    #[tokio::test]
    #[test_log::test]
    async fn redeem_reduces_supply() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["ARdOwner", "ARdInv1"]).await?.into_iter();
        let mut owner = users.next().unwrap();
        let _inv1 = users.next().unwrap();

        let asset_id = create_asset(
            &mut tester,
            &mut owner,
            "AREDEEM",
            1_000_000,
            Some(AssetHolderKind::Account),
        )
        .await?;

        let before = account_balance(&tester, &owner.account(), &asset_id).await?;

        // Redeem 500 from the owner's account balance.
        tester
            .api
            .call()
            .asset()
            .redeem(asset_id.clone(), 500, AssetHolderKind::Account)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        let after = account_balance(&tester, &owner.account(), &asset_id).await?;
        assert_eq!(after + 500, before, "redeem should burn exactly 500");

        Ok(())
    }

    /// make_divisible flips a whole (indivisible) asset to divisible.
    #[tokio::test]
    #[test_log::test]
    async fn make_divisible_updates_details() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["ADvOwner", "ADvInv1"]).await?.into_iter();
        let mut owner = users.next().unwrap();
        let inv1 = users.next().unwrap();

        // AssetHelper mints divisible tokens; create an indivisible one ourselves.
        let mut res = tester
            .api
            .call()
            .asset()
            .create_asset(
                AssetName(b"ADIVIS".to_vec()),
                false,
                AssetType::EquityCommon,
                vec![],
                None,
            )?
            .submit_and_watch(&mut owner)
            .await?;
        res.ok().await?;
        let asset_id = get_asset_id(&mut res).await?.expect("asset id");
        tester
            .api
            .call()
            .asset()
            .issue(asset_id.clone(), 10_000_000_000, AssetHolderKind::Account)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        let details_before = tester
            .api
            .query()
            .asset()
            .assets(asset_id.clone())
            .await?
            .expect("asset exists");
        assert!(
            !details_before.divisible,
            "fresh asset should be indivisible"
        );

        // Whole-coin fractional transfer is rejected while indivisible.
        let mut res = tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id.clone(), inv1.account(), 1_500_000, None)? // 1.5 units
            .submit_and_watch(&mut owner)
            .await?;
        assert!(
            res.ok().await.is_err(),
            "fractional transfer on indivisible asset should fail"
        );

        tester
            .api
            .call()
            .asset()
            .make_divisible(asset_id.clone())?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        let details_after = tester
            .api
            .query()
            .asset()
            .assets(asset_id.clone())
            .await?
            .expect("asset exists");
        assert!(details_after.divisible, "asset should now be divisible");

        // Fractional transfer works after divisibility change.
        tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id.clone(), inv1.account(), 1500000, None)? // 1.5 coins
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        Ok(())
    }

    /// Rename + update type reflect in asset details.
    #[tokio::test]
    #[test_log::test]
    async fn rename_and_update_type() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["ARnOwner", "ARnInv1"]).await?.into_iter();
        let mut owner = users.next().unwrap();
        let _inv1 = users.next().unwrap();

        let asset_id = create_asset(
            &mut tester,
            &mut owner,
            "ARENAMX",
            1_000_000,
            Some(AssetHolderKind::Account),
        )
        .await?;

        tester
            .api
            .call()
            .asset()
            .rename_asset(asset_id.clone(), AssetName(b"Renamed Asset".to_vec()))?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        tester
            .api
            .call()
            .asset()
            .update_asset_type(asset_id.clone(), AssetType::FixedIncome)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        let details = tester
            .api
            .query()
            .asset()
            .assets(asset_id.clone())
            .await?
            .expect("asset exists");
        assert_eq!(details.asset_type, AssetType::FixedIncome);
        let name = tester
            .api
            .query()
            .asset()
            .asset_names(asset_id.clone())
            .await?
            .expect("asset name");
        assert_eq!(name, AssetName(b"Renamed Asset".to_vec()));

        Ok(())
    }

    /// Documents can be added and removed by name.
    #[tokio::test]
    #[test_log::test]
    async fn add_remove_documents() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["ADcOwner", "ADcInv1"]).await?.into_iter();
        let mut owner = users.next().unwrap();
        let _inv1 = users.next().unwrap();

        let asset_id = create_asset(
            &mut tester,
            &mut owner,
            "ADOCSXX",
            1_000_000,
            Some(AssetHolderKind::Account),
        )
        .await?;

        let doc = Document {
            uri: DocumentUri(b"ipfs://QmTest".to_vec()),
            content_hash:
                polymesh_api::types::polymesh_primitives::document_hash::DocumentHash::None,
            name: DocumentName(b"Prospectus".to_vec()),
            doc_type: None,
            filing_date: None,
        };

        tester
            .api
            .call()
            .asset()
            .add_documents(vec![doc.clone()], asset_id.clone())?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        let stored = tester
            .api
            .query()
            .asset()
            .asset_documents(asset_id.clone(), DocumentId(0))
            .await?
            .expect("document 1 should exist");
        assert_eq!(stored.name, doc.name);

        tester
            .api
            .call()
            .asset()
            .remove_documents(vec![DocumentId(0)], asset_id.clone())?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        let gone = tester
            .api
            .query()
            .asset()
            .asset_documents(asset_id.clone(), DocumentId(0))
            .await?;
        assert!(gone.is_none(), "document removed");

        Ok(())
    }

    /// Funding round name settable; identifiers update per-type storage.
    #[tokio::test]
    #[test_log::test]
    async fn funding_round_and_identifiers() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["AIdOwner", "AIdInv1"]).await?.into_iter();
        let mut owner = users.next().unwrap();
        let _inv1 = users.next().unwrap();

        let asset_id = create_asset(
            &mut tester,
            &mut owner,
            "AIDENTS",
            1_000_000,
            Some(AssetHolderKind::Account),
        )
        .await?;

        // Funding round.
        tester
            .api
            .call()
            .asset()
            .set_funding_round(
                asset_id.clone(),
                polymesh_api::types::polymesh_primitives::asset::FundingRoundName(
                    b"Series A".to_vec(),
                ),
            )?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;
        let round = tester
            .api
            .query()
            .asset()
            .funding_round(asset_id.clone())
            .await?;
        assert_eq!(round.0, b"Series A".to_vec());

        // Identifiers (ISIN + CUSIP).
        tester
            .api
            .call()
            .asset()
            .update_identifiers(
                asset_id.clone(),
                vec![
                    AssetIdentifier::ISIN(*b"US0378331005"),
                    AssetIdentifier::CUSIP(*b"037833100"),
                ],
            )?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        Ok(())
    }

    /// controller_transfer force-moves tokens between holders.
    #[tokio::test]
    #[test_log::test]
    async fn controller_transfer_moves_funds() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["ACtOwner", "ACtInv1", "ACtInv2"])
            .await?
            .into_iter();
        let mut owner = users.next().unwrap();
        let inv1 = users.next().unwrap();
        let inv2 = users.next().unwrap();

        let asset_id = create_asset(
            &mut tester,
            &mut owner,
            "ACTRLTX",
            1_000_000,
            Some(AssetHolderKind::Account),
        )
        .await?;

        tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id.clone(), inv1.account(), 1000, None)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // Owner forces 400 from inv1's account to their own default portfolio.
        let issuer_did = owner.did.unwrap();
        tester
            .api
            .call()
            .asset()
            .controller_transfer(
                asset_id.clone(),
                400,
                AssetHolder::Account(inv1.account()),
                AssetHolderKind::DefaultPortfolio,
            )?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        let inv1_bal = account_balance(&tester, &inv1.account(), &asset_id).await?;
        assert_eq!(inv1_bal, 600);

        let owner_pf = PortfolioId {
            did: issuer_did,
            kind: polymesh_api::types::polymesh_primitives::identity_id::PortfolioKind::Default,
        };
        let pf_bal = tester
            .api
            .query()
            .portfolio()
            .portfolio_asset_balances(owner_pf, asset_id.clone())
            .await?;
        assert_eq!(pf_bal, 400);

        // A second controller transfer pulls another 100 into the caller's account.
        tester
            .api
            .call()
            .asset()
            .controller_transfer(
                asset_id.clone(),
                100,
                AssetHolder::Account(inv1.account()),
                AssetHolderKind::Account,
            )?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;
        let inv1_bal = account_balance(&tester, &inv1.account(), &asset_id).await?;
        assert_eq!(inv1_bal, 500);
        let _ = inv2;

        Ok(())
    }
}
