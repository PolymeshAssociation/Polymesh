use std::sync::Arc;

use anyhow::Result;

use tokio::task::JoinHandle;

use polymesh_api::{
    client::Error as ClientError,
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
) -> Result<Vec<Result<TransactionResults, ClientError>>> {
    let mut results = Vec::new();
    for user in users {
        for sk in &mut user.secondary_keys {
            // Send some POLYX back to the primary key from the secondary key.
            let res = call.submit_and_watch(sk).await;
            results.push(res);
        }
    }
    // Wait for all results.
    for res in &mut results {
        if expect_ok {
            // The transaction should be accepted by the RPC node.  (valid transaction)
            let res = res.as_mut().expect("Transaction rejected by node");
            // The transaction should execute without error.
            let result = res.ok().await;
            assert!(result.is_ok());
        } else {
            match res {
              Ok(ref mut res) => {
                let result = res.ok().await;
                assert!(result.is_err());
              }
              Err(err) => {
                // TODO: Check error type.
                eprintln!("Transaction reject by node: {err:?}")
              }
            }
        }
    }
    Ok(results)
}

async fn test_sk_calls(
    api: &Api,
    users: &mut Vec<User>,
    calls: &[(&Arc<WrappedCall>, bool)],
    only_batch: bool,
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
        if only_batch {
            continue;
        }
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
        for res in results {
            let mut res = res.expect("Valid transaction");
            let calls_ok = get_batch_results(&mut res).await?;
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
        false,
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
        false,
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
        false,
    )
    .await?;

    Ok(())
}

#[tokio::test]
async fn secondary_key_change_identity() -> Result<()> {
    let mut tester = PolymeshTester::new().await?;
    let mut users = tester
        .users_with_secondary_keys(&[("SecondaryKeyChangeDID1", 1), ("SecondaryKeyChangeDID2", 0)])
        .await?;

    let user0_pk = users[0].account();
    let user0_sk0 = users[0].get_sk(0)?.account();

    // Create JoinIdentity auth for sk0 to join DID2 with no-permissions.
    let mut res = tester
        .api
        .call()
        .identity()
        .add_authorization(
            Signatory::Account(user0_sk0),
            AuthorizationData::JoinIdentity(PermissionsBuilder::empty().build()),
            None,
        )?
        .execute(&mut users[1])
        .await?;
    let auth_id = get_auth_id(&mut res)
        .await?
        .expect("Missing JoinIdentity auth id");

    // Prepare a POLYX transfer to one user.
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
    // Prepare `identity.leave_identity_as_key` call.
    let leave_did1_call = Arc::new(tester.api.call().identity().leave_identity_as_key()?);
    // Prepare `identity.join_identity_as_key` call.
    let join_did2_call = Arc::new(tester.api.call().identity().join_identity_as_key(auth_id)?);

    test_sk_calls(
        &tester.api,
        &mut users,
        &[
            // Ensure the secondary key can still do `balance.transfer` and `system.remark` calls.
            // These calls are not restricted by key permissions.
            (&balance_transfer_call, true),
            (&remark_call, true),
            // The secondary key should be allowed to create venues.
            (&create_venue_call, true),
            // The secondary key should be allowed to add an authorization.
            (&add_auth_call, true),
            // The secondary key should be allowed to leave DID1.
            (&leave_did1_call, true), // The secondary key should have no identity here.
            // The key should still be able to do POLYX transfer and `system.remark` calls.
            (&balance_transfer_call, true),
            (&remark_call, true),
            // The key shouldn't be allowed to create venues.  Has no identity.
            (&create_venue_call, false),
            // The key shouldn't be allowed to add an authorization.  Has no identity.
            (&add_auth_call, false),
            // The key should be allowed to join DID2.
            (&join_did2_call, true), // The key is now a secondar key of DID2 with no permissions.
            // The secondary key should still be able to do POLYX transfer and `system.remark` calls.
            (&balance_transfer_call, true),
            (&remark_call, true),
            // The secondary key shouldn't be allowed to create venues.  Has no call permissions.
            (&create_venue_call, false),
            // The secondary key shouldn't be allowed to add an authorization.  Has no call permissions.
            (&add_auth_call, false),
        ],
        true,
    )
    .await?;

    Ok(())
}
