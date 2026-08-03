#[cfg(feature = "current_release")]
mod relayer_tests {
    use anyhow::Result;
    use integration::*;

    const ONE_POLYX: u128 = 1_000_000;

    #[tokio::test]
    #[test_log::test]
    async fn relayer_subsidy_lifecycle() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let users = tester.users(&["Subsidizer", "Subsidized"]).await?;
        let mut subsidizer = users[0].clone();
        let mut subsidized = users[1].clone();

        let subsidizer_account = subsidizer.account();
        let subsidized_account = subsidized.account();

        // Subsidizer approves a subsidy for the subsidized user.
        tester
            .api
            .call()
            .relayer()
            .approve_subsidy(subsidized_account.clone(), 100_000 * ONE_POLYX)?
            .submit_and_watch(&mut subsidizer)
            .await?
            .ok()
            .await?;

        // Subsidized user accepts the subsidy.
        tester
            .api
            .call()
            .relayer()
            .accept_subsidy(subsidizer_account.clone())?
            .submit_and_watch(&mut subsidized)
            .await?
            .ok()
            .await?;

        // Subsidizer updates the POLYX limit.
        tester
            .api
            .call()
            .relayer()
            .update_polyx_limit(subsidized_account.clone(), 500_000 * ONE_POLYX)?
            .submit_and_watch(&mut subsidizer)
            .await?
            .ok()
            .await?;

        // Subsidizer increases the POLYX limit.
        tester
            .api
            .call()
            .relayer()
            .increase_polyx_limit(subsidized_account.clone(), 70_000 * ONE_POLYX)?
            .submit_and_watch(&mut subsidizer)
            .await?
            .ok()
            .await?;

        // Subsidizer decreases the POLYX limit.
        tester
            .api
            .call()
            .relayer()
            .decrease_polyx_limit(subsidized_account.clone(), 30_000 * ONE_POLYX)?
            .submit_and_watch(&mut subsidizer)
            .await?
            .ok()
            .await?;

        // Remove the subsidy.
        tester
            .api
            .call()
            .relayer()
            .remove_subsidy(subsidized_account, subsidizer_account)?
            .submit_and_watch(&mut subsidizer)
            .await?
            .ok()
            .await?;

        Ok(())
    }
}
