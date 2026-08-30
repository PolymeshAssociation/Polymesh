#[cfg(feature = "current_release")]
mod relayer_tests {
    use anyhow::Result;
    use integration::*;
    use polymesh_api::types::polymesh_primitives::{
        authorization::AuthorizationData,
        secondary_key::Signatory,
        settlement::{VenueDetails, VenueType},
    };

    const ONE_POLYX: u128 = 1_000_000;

    async fn get_subsidy(api: &Api, subsidized: &AccountId) -> Result<u128> {
        let subsidy = api.query().relayer().subsidies(*subsidized).await?;
        log::info!("Subsidy for {}: {:?}", subsidized, subsidy);
        Ok(subsidy.expect("Account Subsidy").remaining)
    }

    async fn get_free_balance(api: &Api, account: &AccountId) -> Result<u128> {
        Ok(api.query().system().account(*account).await?.data.free)
    }

    async fn test_subsidized_calls(api: &Api, subsidized: &mut User) -> Result<()> {
        let account = subsidized.account();
        // Get current subsidy remaining.
        let remaining = get_subsidy(&api, &account).await?;
        log::info!("Subsidy remaining: {}", remaining);

        // Make a subsidized call.
        let mut res1 = api
            .call()
            .settlement()
            .create_venue(
                VenueDetails(b"Test1".to_vec()),
                Default::default(),
                VenueType::Other,
            )?
            .submit_and_watch(subsidized)
            .await?;
        println!("venue1 = {:?}", get_venue_id(&mut res1).await?);

        // Get new subsidy remaining.
        let new_remaining = get_subsidy(&api, &account).await?;
        log::info!("Subsidy remaining after call: {}", new_remaining);

        // Try a non-subsidized call (should fail).
        log::info!("Test non-subsidized call");
        let res2 = api
            .call()
            .system()
            .remark(b"test".to_vec())?
            .submit_and_watch(subsidized)
            .await;
        assert!(res2.is_err(), "Non-subsidized call should fail");

        // Get final subsidy remaining.
        let final_remaining = get_subsidy(&api, &account).await?;
        log::info!("Final subsidy remaining: {}", final_remaining);
        assert_eq!(
            new_remaining, final_remaining,
            "Subsidy remaining should not change after failed call"
        );

        Ok(())
    }

    async fn test_eth_subsidized_calls(api: &Api, subsidized: &mut EthWallet) -> Result<()> {
        let account = subsidized.account();
        // Get current subsidy remaining.
        let remaining = get_subsidy(&api, &account).await?;
        log::info!("Subsidy remaining: {}", remaining);

        // Make a subsidized call.
        let create_venue = api.call().settlement().create_venue(
            VenueDetails(b"Test1".to_vec()),
            Default::default(),
            VenueType::Other,
        )?;
        let res1 = subsidized.send_runtime_call(&create_venue).await?;
        println!("create venue: res1 = {:?}", res1);

        // Get new subsidy remaining.
        let new_remaining = get_subsidy(&api, &account).await?;
        log::info!("Subsidy remaining after call: {}", new_remaining);

        // Try a non-subsidized call (should fail).
        log::info!("Test non-subsidized call");
        let remark = api.call().system().remark(b"test".to_vec())?;
        let res2 = subsidized.send_runtime_call(&remark).await;
        println!("create venue: res2 = {:?}", res2);
        assert!(res2.is_err(), "Non-subsidized call should fail");

        // Get final subsidy remaining.
        let final_remaining = get_subsidy(&api, &account).await?;
        log::info!("Final subsidy remaining: {}", final_remaining);
        assert_eq!(
            new_remaining, final_remaining,
            "Subsidy remaining should not change after failed call"
        );

        Ok(())
    }

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

        // Test that the subsidized user can make calls.
        test_subsidized_calls(&tester.api, &mut subsidized).await?;

        // Subsidizer updates the POLYX limit.
        tester
            .api
            .call()
            .relayer()
            .update_polyx_limit(subsidized_account.clone(), 100 * ONE_POLYX)?
            .submit_and_watch(&mut subsidizer)
            .await?
            .ok()
            .await?;

        // Subsidizer increases the POLYX limit.
        tester
            .api
            .call()
            .relayer()
            .increase_polyx_limit(subsidized_account.clone(), 70 * ONE_POLYX)?
            .submit_and_watch(&mut subsidizer)
            .await?
            .ok()
            .await?;

        // Subsidizer decreases the POLYX limit.
        tester
            .api
            .call()
            .relayer()
            .decrease_polyx_limit(subsidized_account.clone(), 100 * ONE_POLYX)?
            .submit_and_watch(&mut subsidizer)
            .await?
            .ok()
            .await?;

        // Test that the subsidized user can still make calls after limit changes.
        test_subsidized_calls(&tester.api, &mut subsidized).await?;

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

    #[tokio::test]
    #[test_log::test]
    async fn relayer_eth_account_subsidy() -> Result<()> {
        let (mut tester, node) = revive_tester().await?;
        let users = tester.users(&["Subsidizer", "EthUserPrimaryKey"]).await?;
        let mut subsidizer = users[0].clone();
        let mut eth_user = users[1].clone();
        let api = tester.api.clone();

        let subsidizer_account = subsidizer.account();

        // An eth wallet always acts as its fallback account, so it joins the identity through the
        // very interface under test rather than by signing a substrate extrinsic.
        let mut subsidized = node.new_wallet();
        let subsidized_account = subsidized.account();

        let mut res = api
            .call()
            .identity()
            .add_authorization(
                Signatory::Account(subsidized_account),
                AuthorizationData::JoinIdentity(PermissionsBuilder::whole().build()),
                None,
            )?
            .execute(&mut eth_user)
            .await?;
        let auth_id = get_auth_id(&mut res)
            .await?
            .expect("Missing JoinIdentity auth id");

        let join = api.call().identity().join_identity_as_key(auth_id)?;
        log::info!("ETH wallet join identity: auth_id={auth_id:?}");
        subsidized.send_runtime_call(&join).await?;

        // Subsidizer approves a subsidy for the subsidized user.
        api.call()
            .relayer()
            .approve_subsidy(subsidized_account.clone(), 100_000 * ONE_POLYX)?
            .submit_and_watch(&mut subsidizer)
            .await?
            .ok()
            .await?;

        // Subsidized user accepts the subsidy.
        let accept = api
            .call()
            .relayer()
            .accept_subsidy(subsidizer_account.clone())?;
        log::info!("ETH Wallet accept subsidy");
        let res = subsidized.send_runtime_call(&accept).await;
        println!("res = {res:?}");

        // Test ETH wallet subsidy.
        let subsidy_before = get_subsidy(&api, &subsidized_account).await?;
        let balance_before = get_free_balance(&api, &subsidizer_account).await?;
        test_eth_subsidized_calls(&api, &mut subsidized).await?;
        let subsidy_after = get_subsidy(&api, &subsidized_account).await?;
        let balance_after = get_free_balance(&api, &subsidizer_account).await?;

        assert_eq!(
            balance_before.saturating_sub(balance_after),
            subsidy_before.saturating_sub(subsidy_after),
            "Subsidizer balance decrease should match subsidy usage",
        );

        Ok(())
    }
}
