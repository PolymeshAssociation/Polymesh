#[cfg(feature = "current_release")]
mod corporate_actions_tests {
    use std::collections::BTreeSet;

    use anyhow::Result;
    use integration::*;
    use polymesh_api::types::pallet_corporate_actions::{CADetails, CAKind};

    #[tokio::test]
    #[test_log::test]
    async fn corporate_actions() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["CAOwner", "CAHolder"]).await?.into_iter();
        let mut owner = users.next().expect("CAOwner");
        let mut _holder = users.next().expect("CAHolder");

        // Create asset for corporate actions
        let asset_helper = AssetHelper::new(
            &tester.api,
            &mut owner,
            "CATestAsset",
            1_000_000,
            BTreeSet::new(),
        )
        .await?;
        let asset_id = asset_helper.asset_id;

        let now = tester.api.query().timestamp().now().await?;
        let decl_date = now;

        // Initiate a corporate action
        let mut res = tester
            .api
            .call()
            .corporate_action()
            .initiate_corporate_action(
                asset_id,
                CAKind::IssuerNotice,
                decl_date,
                None,
                CADetails(b"Test corporate action".to_vec()),
                None,
                None,
                None,
            )?
            .submit_and_watch(&mut owner)
            .await?;
        res.ok().await?;
        let ca_id = get_ca_id(&mut res)
            .await?
            .expect("CAId from CAInitiated event");

        // Remove the corporate action
        tester
            .api
            .call()
            .corporate_action()
            .remove_ca(ca_id)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        Ok(())
    }

    #[tokio::test]
    #[test_log::test]
    async fn corporate_actions_set_defaults() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["CADefaultOwner"]).await?.into_iter();
        let mut owner = users.next().expect("CADefaultOwner");

        let asset_helper = AssetHelper::new(
            &tester.api,
            &mut owner,
            "CADefaultsAsset",
            500_000,
            BTreeSet::new(),
        )
        .await?;
        let asset_id = asset_helper.asset_id;

        // Set default withholding tax to 10% (100_000 / 1_000_000)
        // Use sp_arithmetic Permill converted to the API's Permill type.
        let permill = sp_arithmetic::per_things::Permill::from_percent(10);
        tester
            .api
            .call()
            .corporate_action()
            .set_default_withholding_tax(asset_id, permill.into())?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        Ok(())
    }
}
