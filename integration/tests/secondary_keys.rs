use std::sync::Arc;

use anyhow::{anyhow, Result};

use tokio::task::JoinHandle;

use polymesh_api::{
    types::polymesh_primitives::{
        authorization::AuthorizationData,
        secondary_key::Signatory,
        settlement::{VenueDetails, VenueType},
    },
    TransactionResults, WrappedCall,
};

use integration::*;

async fn test_sk_call(
    users: &mut [User],
    call: Arc<WrappedCall>,
    expect_ok: bool,
) -> Result<Vec<TransactionResults>> {
    let mut results = Vec::new();
    for user in users {
        for sk in &mut user.secondary_keys {
            // Send some POLYX back to the primary key from the secondary key.
            let res = call.submit_and_watch(sk).await?;
            results.push(res);
        }
    }
    // Wait for all results.
    for res in &mut results {
        let result = res.ok().await;
        if expect_ok {
            assert!(result.is_ok());
        } else {
            assert!(result.is_err());
        }
    }
    Ok(results)
}

async fn test_sk_calls(
    api: &Api,
    users: &mut Vec<User>,
    calls: &[(&Arc<WrappedCall>, bool)],
) -> Result<()> {
    let mut tasks: Vec<JoinHandle<Result<()>>> = Vec::new();

    let mut expect_all_ok = true;
    let mut batch = Vec::with_capacity(calls.len());
    let mut expected = Vec::with_capacity(calls.len());
    // Test each call.
    for (call, expect_ok) in calls {
        if !expect_ok {
            expect_all_ok = false;
        }
        expected.push(*expect_ok);
        batch.push(call.runtime_call().clone());
        let mut users = users.clone();
        let call = (*call).clone();
        let expect_ok = *expect_ok;
        tasks.push(tokio::spawn(async move {
            test_sk_call(&mut users, call, expect_ok).await?;
            Ok(())
        }));
    }
    if expect_all_ok {
        let call = Arc::new(api.call().utility().batch_all(batch)?);
        let mut users = users.clone();
        tasks.push(tokio::spawn(async move {
            test_sk_call(&mut users, call, true).await?;
            Ok(())
        }));
    } else {
        // `batch_all` should fail, since at least one call is expected to fail.
        {
            let call = Arc::new(api.call().utility().batch_all(batch.clone())?);
            let mut users = users.clone();
            tasks.push(tokio::spawn(async move {
                test_sk_call(&mut users, call, false).await?;
                Ok(())
            }));
        }
        // Use `force_batch` and check the results for each call in the batch.
        let call = Arc::new(api.call().utility().force_batch(batch)?);
        let results = test_sk_call(users, call, true).await?;
        for mut res in results {
            let events = res
                .events()
                .await?
                .ok_or_else(|| anyhow!("Failed to get batch events"))?;
            let calls_ok = events
                .0
                .iter()
                .filter_map(|rec| match rec.event {
                    RuntimeEvent::Utility(UtilityEvent::ItemCompleted) => Some(true),
                    RuntimeEvent::Utility(UtilityEvent::ItemFailed { .. }) => Some(false),
                    _ => None,
                })
                .collect::<Vec<bool>>();
            assert_eq!(calls_ok, expected);
        }
    }
    // Wait for tasks to finish.
    for task in tasks {
        task.await??;
    }
    Ok(())
}

#[tokio::test]
async fn secondary_keys_permissions() -> Result<()> {
    let mut tester = PolymeshTester::new().await?;
    const SK_COUNT: usize = 10;
    let mut users = tester
        .users_with_secondary_keys(&[("SecondaryKeysPermissions", SK_COUNT)])
        .await?;

    // Prepare a POLYX transfer to one user.
    let user0_pk = users[0].account();
    let balance_transfer_call =
        Arc::new(tester.api.call().balances().transfer(user0_pk.into(), 1)?);
    // Prepare `system.remark` call.
    let remark_call = Arc::new(tester.api.call().system().remark(vec![])?);
    // Prepare `settlement.create_venue` call.
    let create_venue_call = Arc::new(tester.api.call().settlement().create_venue(
        VenueDetails(vec![]),
        vec![],
        VenueType::Other,
    )?);
    // Prepare `identity.add_authorization` call.
    let add_auth_call = Arc::new(tester.api.call().identity().add_authorization(
        Signatory::Account(user0_pk),
        AuthorizationData::RotatePrimaryKey,
        None,
    )?);

    // Ensure all secondary keys start with `Whole` permissions.
    let whole = PermissionsBuilder::whole();
    for user in &users {
        for sk in 0..SK_COUNT {
            user.ensure_key_permissions(sk, &whole).await?;
        }
    }

    test_sk_calls(
        &tester.api,
        &mut users,
        &[
            // Ensure the secndary keys can do simple calls.
            (&balance_transfer_call, true),
            (&remark_call, true),
            // The keys should be allowed to create a venue.
            (&create_venue_call, true),
            // The keys should be allowed to add an authorization.
            (&add_auth_call, true),
        ],
    )
    .await?;

    // Remove all permissions from the secondary keys.
    let mut perms = PermissionsBuilder::empty();
    for user in &mut users {
        user.set_all_keys_permissions(&perms).await?;
    }

    test_sk_calls(
        &tester.api,
        &mut users,
        &[
            // Ensure the secndary keys can still do `balance.transfer` and `system.remark` calls.
            // These calls are not restricted by key permissions.
            (&balance_transfer_call, true),
            (&remark_call, true),
            // The keys shouldn't be allowed to create venues.
            (&create_venue_call, false),
            // The keys shouldn't be allowed to add an authorization.
            (&add_auth_call, false),
        ],
    )
    .await?;

    // Allow the `Settlement` pallet.
    perms.allow_pallet("Settlement");
    for user in &mut users {
        user.set_all_keys_permissions(&perms).await?;
    }

    test_sk_calls(
        &tester.api,
        &mut users,
        &[
            // Ensure the secndary keys can still do `balance.transfer` and `system.remark` calls.
            // These calls are not restricted by key permissions.
            (&balance_transfer_call, true),
            (&remark_call, true),
            // The keys should be allowed to create venues.
            (&create_venue_call, true),
            // The keys shouldn't be allowed to add an authorization.
            (&add_auth_call, false),
        ],
    )
    .await?;

    Ok(())
}
