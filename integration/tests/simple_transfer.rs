use anyhow::Result;

use integration::*;

#[tokio::test]
#[test_log::test]
async fn simple_polyx_transfer() -> Result<()> {
    let mut tester = PolymeshTester::new().await?;
    let mut users = tester.users(&["User1", "User2"]).await?;

    let mut res = tester
        .api
        .call()
        .balances()
        .transfer_with_memo(users[1].account().into(), 13 * ONE_POLYX, None)?
        .execute(&mut users[0])
        .await?;
    let events = res.events().await?;
    println!("call1 events = {:#?}", events);
    Ok(())
}
