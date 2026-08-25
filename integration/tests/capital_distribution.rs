//! Capital distribution: initiate a benefit CA, distribute, claim.
#[cfg(feature = "current_release")]
mod capital_distribution_tests {
    use std::collections::BTreeSet;

    use anyhow::Result;

    use integration::*;
    use polymesh_api::types::pallet_corporate_actions::{CADetails, CAKind, RecordDateSpec};
    use polymesh_api::types::polymesh_primitives::asset::AssetHolderKind;

    async fn create_asset(
        tester: &mut PolymeshTester,
        owner: &mut User,
        ticker: &str,
        amount: u128,
    ) -> Result<AssetId> {
        let helper = AssetHelper::new_full(
            &tester.api,
            owner,
            ticker,
            amount,
            BTreeSet::new(),
            false,
            Some(AssetHolderKind::Account),
        )
        .await?;
        Ok(helper.asset_id)
    }

    /// Full cycle: checkpoint → benefit CA → distribute payment asset → holder claims.
    #[tokio::test]
    #[test_log::test]
    async fn distribute_and_claim() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["CdOwner", "CdHolder"])
            .await?
            .into_iter();
        let mut owner = users.next().unwrap();
        let mut holder = users.next().unwrap();

        // Equity held by owner + holder.
        let equity = create_asset(&mut tester, &mut owner, "CDEQTY", 1_000_000).await?;
        tester
            .api
            .call()
            .asset()
            .transfer_asset(equity.clone(), holder.account(), 100_000, None)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // Snapshot holders.
        let mut cp_res = tester
            .api
            .call()
            .checkpoint()
            .create_checkpoint(equity.clone())?
            .submit_and_watch(&mut owner)
            .await?;
        cp_res.ok().await?;
        let cp_id = get_checkpoint_id(&mut cp_res)
            .await?
            .expect("checkpoint id");

        let now = tester.api.query().timestamp().now().await?;
        let mut ca_res = tester
            .api
            .call()
            .corporate_action()
            .initiate_corporate_action(
                equity.clone(),
                CAKind::UnpredictableBenefit,
                now,
                Some(RecordDateSpec::Existing(cp_id)),
                CADetails(b"dividend".to_vec()),
                None,
                None,
                None,
            )?
            .submit_and_watch(&mut owner)
            .await?;
        ca_res.ok().await?;
        let ca_id = get_ca_id(&mut ca_res).await?.expect("ca id");

        // Payment currency (another asset in the issuer's default portfolio).
        // Payment tokens must sit in the issuer's default portfolio.
        let pay_helper = AssetHelper::new_full(
            &tester.api,
            &mut owner,
            "CDPAY",
            1_000_000,
            BTreeSet::new(),
            false,
            Some(AssetHolderKind::DefaultPortfolio),
        )
        .await?;
        let pay = pay_helper.asset_id;

        // 1 payment unit per equity unit (per_share is 1e6-scaled).
        tester
            .api
            .call()
            .capital_distribution()
            .distribute(
                ca_id.clone(),
                None,
                pay.clone(),
                1_000_000,
                1_000_000,
                now,
                None,
            )?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        tester
            .api
            .call()
            .capital_distribution()
            .claim(ca_id.clone())?
            .submit_and_watch(&mut holder)
            .await?
            .ok()
            .await?;

        Ok(())
    }

    /// push_benefit pays a non-claiming holder; reclaim returns leftovers after expiry.
    #[cfg(feature = "timed")]
    #[tokio::test]
    #[test_log::test]
    async fn push_benefit_and_reclaim() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["CdOwner2", "CdHolder2"])
            .await?
            .into_iter();
        let mut owner = users.next().unwrap();
        let holder = users.next().unwrap();

        let equity = create_asset(&mut tester, &mut owner, "CDEQT2", 1_000_000).await?;
        tester
            .api
            .call()
            .asset()
            .transfer_asset(equity.clone(), holder.account(), 50_000, None)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        let mut cp_res = tester
            .api
            .call()
            .checkpoint()
            .create_checkpoint(equity.clone())?
            .submit_and_watch(&mut owner)
            .await?;
        cp_res.ok().await?;
        let cp_id = get_checkpoint_id(&mut cp_res).await?.expect("checkpoint");

        let now = tester.api.query().timestamp().now().await?;
        let mut ca_res = tester
            .api
            .call()
            .corporate_action()
            .initiate_corporate_action(
                equity.clone(),
                CAKind::UnpredictableBenefit,
                now,
                Some(RecordDateSpec::Existing(cp_id)),
                CADetails(b"push".to_vec()),
                None,
                None,
                None,
            )?
            .submit_and_watch(&mut owner)
            .await?;
        ca_res.ok().await?;
        let ca_id = get_ca_id(&mut ca_res).await?.expect("ca id");

        let pay = AssetHelper::new_full(
            &tester.api,
            &mut owner,
            "CDPAY2",
            1_000_000,
            BTreeSet::new(),
            false,
            Some(AssetHolderKind::DefaultPortfolio),
        )
        .await?
        .asset_id;
        let now = tester.api.query().timestamp().now().await?;
        tester
            .api
            .call()
            .capital_distribution()
            .distribute(
                ca_id.clone(),
                None,
                pay,
                1_000_000,
                1_000_000,
                now,
                Some(now + 12_000),
            )?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        tester
            .api
            .call()
            .capital_distribution()
            .push_benefit(ca_id.clone(), holder.did.expect("holder did"))?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        tokio::time::sleep(std::time::Duration::from_secs(13)).await;

        tester
            .api
            .call()
            .capital_distribution()
            .reclaim(ca_id)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        Ok(())
    }
}