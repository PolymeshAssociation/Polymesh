//! Checkpoint pallet: manual + scheduled checkpoints.
#[cfg(feature = "current_release")]
mod checkpoints_tests {
    use std::collections::BTreeSet;

    use anyhow::Result;

    use integration::*;
    use polymesh_api::types::polymesh_primitives::{
        asset::AssetHolderKind, checkpoint::ScheduleCheckpoints,
    };

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

    /// Manual checkpoint creation records a snapshot id.
    #[tokio::test]
    #[test_log::test]
    async fn manual_checkpoint() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["CpOwner", "CpInv"]).await?.into_iter();
        let mut owner = users.next().unwrap();
        let inv = users.next().unwrap();

        let asset_id = create_asset(&mut tester, &mut owner, "CPMAN", 1_000_000).await?;

        tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id.clone(), inv.account(), 100, None)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        let mut res = tester
            .api
            .call()
            .checkpoint()
            .create_checkpoint(asset_id.clone())?
            .submit_and_watch(&mut owner)
            .await?;
        res.ok().await?;
        let cp_id = get_checkpoint_id(&mut res)
            .await?
            .expect("CheckpointCreated event");
        assert!(cp_id.0 >= 1, "checkpoint id should be allocated");

        Ok(())
    }

    /// A near-term scheduled checkpoint is accepted and stored.
    #[cfg(feature = "timed")]
    #[tokio::test]
    #[test_log::test]
    async fn scheduled_checkpoint() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["CpSchedOwner"]).await?.into_iter();
        let mut owner = users.next().unwrap();

        let asset_id = create_asset(&mut tester, &mut owner, "CPSCHD", 1_000_000).await?;
        let now = tester.api.query().timestamp().now().await?;
        // A few seconds in the future so the schedule is pending.
        let schedule = ScheduleCheckpoints {
            pending: BTreeSet::from([now + 8_000]),
        };

        tester
            .api
            .call()
            .checkpoint()
            .create_schedule(asset_id.clone(), schedule)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        Ok(())
    }

    /// Creating a schedule with an empty pending set is rejected.
    #[tokio::test]
    #[test_log::test]
    async fn empty_schedule_rejected() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["CpEmptyOwner"]).await?.into_iter();
        let mut owner = users.next().unwrap();

        let asset_id = create_asset(&mut tester, &mut owner, "CPEMPT", 1_000_000).await?;
        let empty = ScheduleCheckpoints {
            pending: BTreeSet::new(),
        };
        let mut res = tester
            .api
            .call()
            .checkpoint()
            .create_schedule(asset_id, empty)?
            .submit_and_watch(&mut owner)
            .await?;
        assert!(
            res.ok().await.is_err(),
            "empty checkpoint schedule should be rejected"
        );

        Ok(())
    }
}
