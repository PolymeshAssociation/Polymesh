// >=v7.4
#[cfg(feature = "current_release")]
mod asset_transfer_tests {
    use std::collections::BTreeSet;

    use anyhow::Result;

    use integration::*;

    #[cfg(feature = "previous_release")]
    use polymesh_api::types::polymesh_primitives::identity_id::PortfolioKind;

    #[cfg(feature = "current_release")]
    use polymesh_api::types::polymesh_primitives::asset::AssetHolderKind;

    #[cfg(feature = "current_release")]
    use polymesh_api::types::polymesh_primitives::settlement::AffirmationRequirement;

    /// Test for an asset transfer requiring receiver affirmation.
    #[tokio::test]
    async fn asset_transfer_with_receiver_affirm() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["AssetIssuer1", "Investor1"])
            .await?
            .into_iter();
        let mut asset_issuer = users.next().expect("Asset issuer");
        let mut investor = users.next().expect("Investor");

        #[cfg(feature = "current_release")]
        let kind = AssetHolderKind::Account;
        #[cfg(feature = "previous_release")]
        let kind = PortfolioKind::AccountId(asset_issuer.account());

        // Create a new asset and mint some tokens.
        let asset_helper = AssetHelper::new_full(
            &tester.api,
            &mut asset_issuer,
            "TestAsset",
            1_000_000,
            BTreeSet::new(),
            false,
            Some(kind),
        )
        .await?;
        let asset_id = asset_helper.asset_id;

        // Investor opts in to mandatory receiver affirmation.
        tester
            .api
            .call()
            .settlement()
            .set_mandatory_receiver_affirmation(AffirmationRequirement::Required)?
            .execute(&mut investor)
            .await?;

        // Create an asset transfer from the asset issuer to the investor.
        let mut transfer_res = tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id, investor.account(), 10, None)?
            .submit_and_watch(&mut asset_issuer)
            .await?;

        // Get the pending transfer ID from the response.
        let pending_transfer_id = get_pending_transfer_id(&mut transfer_res)
            .await?
            .expect("Pending Transfer ID not found");

        // The receiver needs to affirm the transfer
        let mut affirm_res = tester
            .api
            .call()
            .asset()
            .receiver_affirm_asset_transfer(pending_transfer_id)?
            .submit_and_watch(&mut investor)
            .await?;

        // Wait for the receiver affirmation to complete.
        affirm_res.ok().await?;

        Ok(())
    }

    /// Test the receiver rejecting an asset transfer.
    #[tokio::test]
    async fn asset_transfer_rejected_by_receiver() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["AssetIssuer1", "Investor1"])
            .await?
            .into_iter();
        let mut asset_issuer = users.next().expect("Asset issuer");
        let mut investor = users.next().expect("Investor");

        #[cfg(feature = "current_release")]
        let kind = AssetHolderKind::Account;
        #[cfg(feature = "previous_release")]
        let kind = PortfolioKind::AccountId(asset_issuer.account());

        // Create a new asset and mint some tokens.
        let asset_helper = AssetHelper::new_full(
            &tester.api,
            &mut asset_issuer,
            "TestAsset",
            1_000_000,
            BTreeSet::new(),
            false,
            Some(kind),
        )
        .await?;
        let asset_id = asset_helper.asset_id;

        // Investor opts in to mandatory receiver affirmation.
        tester
            .api
            .call()
            .settlement()
            .set_mandatory_receiver_affirmation(AffirmationRequirement::Required)?
            .execute(&mut investor)
            .await?;

        // Create an asset transfer from the asset issuer to the investor.
        let mut transfer_res = tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id, investor.account(), 10, None)?
            .submit_and_watch(&mut asset_issuer)
            .await?;

        // Get the pending transfer ID from the response.
        let pending_transfer_id = get_pending_transfer_id(&mut transfer_res)
            .await?
            .expect("Pending Transfer ID not found");

        // The receiver rejects the transfer
        let mut reject_res = tester
            .api
            .call()
            .asset()
            .reject_asset_transfer(pending_transfer_id)?
            .submit_and_watch(&mut investor)
            .await?;

        // Wait for the receiver rejection to complete.
        reject_res.ok().await?;

        Ok(())
    }

    /// The sender rejects the asset transfer before the receiver affirms it.
    #[tokio::test]
    async fn asset_transfer_rejected_by_sender() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["AssetIssuer1", "Investor1"])
            .await?
            .into_iter();
        let mut asset_issuer = users.next().expect("Asset issuer");
        let mut investor = users.next().expect("Investor");

        #[cfg(feature = "current_release")]
        let kind = AssetHolderKind::Account;
        #[cfg(feature = "previous_release")]
        let kind = PortfolioKind::AccountId(asset_issuer.account());

        // Create a new asset and mint some tokens.
        let asset_helper = AssetHelper::new_full(
            &tester.api,
            &mut asset_issuer,
            "TestAsset",
            1_000_000,
            BTreeSet::new(),
            false,
            Some(kind),
        )
        .await?;
        let asset_id = asset_helper.asset_id;

        // Investor opts in to mandatory receiver affirmation.
        tester
            .api
            .call()
            .settlement()
            .set_mandatory_receiver_affirmation(AffirmationRequirement::Required)?
            .execute(&mut investor)
            .await?;

        // Create an asset transfer from the asset issuer to the investor.
        let mut transfer_res = tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id, investor.account(), 10, None)?
            .submit_and_watch(&mut asset_issuer)
            .await?;

        // Get the pending transfer ID from the response.
        let pending_transfer_id = get_pending_transfer_id(&mut transfer_res)
            .await?
            .expect("Pending Transfer ID not found");

        // The sender rejects the transfer
        let mut reject_res = tester
            .api
            .call()
            .asset()
            .reject_asset_transfer(pending_transfer_id)?
            .submit_and_watch(&mut asset_issuer)
            .await?;

        // Wait for the sender rejection to complete.
        reject_res.ok().await?;

        Ok(())
    }

    /// Test for an asset transfer with pre-approved receiver affirmation.
    #[tokio::test]
    async fn asset_transfer_with_receiver_pre_approved() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["AssetIssuer1", "Investor1"])
            .await?
            .into_iter();
        let mut asset_issuer = users.next().expect("Asset issuer");
        let mut investor = users.next().expect("Investor");

        #[cfg(feature = "current_release")]
        let kind = AssetHolderKind::Account;
        #[cfg(feature = "previous_release")]
        let kind = PortfolioKind::AccountId(asset_issuer.account());

        // Create a new asset and mint some tokens.
        let asset_helper = AssetHelper::new_full(
            &tester.api,
            &mut asset_issuer,
            "TestAsset",
            1_000_000,
            BTreeSet::new(),
            false,
            Some(kind),
        )
        .await?;
        let asset_id = asset_helper.asset_id;

        // Receiver pre-approves asset transfers.
        let mut pre_approve_res = tester
            .api
            .call()
            .asset()
            .pre_approve_asset(asset_id)?
            .submit_and_watch(&mut investor)
            .await?;

        // Wait for pre-approval to complete.
        pre_approve_res.ok().await?;

        // Create an asset transfer from the asset issuer to the investor.
        let mut transfer_res = tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id, investor.account(), 10, None)?
            .submit_and_watch(&mut asset_issuer)
            .await?;

        // Try to get the pending transfer ID from the response.
        let pending_transfer_id = get_pending_transfer_id(&mut transfer_res).await?;

        assert!(
            pending_transfer_id.is_none(),
            "Pending Transfer ID should not be found for pre-approved transfer"
        );

        Ok(())
    }
}
