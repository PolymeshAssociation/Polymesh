use anyhow::Result;

use integration::*;

#[tokio::test]
async fn secondary_keys_permissions() -> Result<()> {
    let mut tester = PolymeshTester::new().await?;
    let users = tester
        .users_with_secondary_keys(&[("SecondaryKeysPermissions", 10)])
        .await?;

    let mut results = Vec::new();
    for mut user in users {
        let pk = user.account();
        for sk in &mut user.secondary_keys {
            // Send some POLYX back to the primary key from the secondary key.
            let res = tester
                .api
                .call()
                .balances()
                .transfer(pk.into(), ONE_POLYX)?
                .submit_and_watch(sk)
                .await?;
            results.push(res);
        }
    }
    // Wait for all results.
    for mut res in results {
        println!("transfer res = {:#?}", res.ok().await);
    }
    Ok(())
}
