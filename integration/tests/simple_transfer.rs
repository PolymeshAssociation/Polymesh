use anyhow::Result;

use sp_keyring::AccountKeyring;

use integration::*;

async fn do_simple_polyx_transfer() -> Result<()> {
  let mut alice = PolymeshAccount::alice();

  let api = client_api().await?;

  let dest = AccountKeyring::Bob.to_account_id().into();
  let mut res = api
    .call()
    .balances()
    .transfer(dest, 123_012_345)?
    .sign_submit_and_watch(&mut alice)
    .await?;
  let events = res.events().await?;
  println!("call1 events = {:#?}", events);
  Ok(())
}

#[tokio::test]
async fn simple_polyx_transfer_01() -> Result<()> {
  do_simple_polyx_transfer().await
}

#[tokio::test]
async fn simple_polyx_transfer_02() -> Result<()> {
  do_simple_polyx_transfer().await
}

#[tokio::test]
async fn simple_polyx_transfer_03() -> Result<()> {
  do_simple_polyx_transfer().await
}

#[tokio::test]
async fn simple_polyx_transfer_04() -> Result<()> {
  do_simple_polyx_transfer().await
}

#[tokio::test]
async fn simple_polyx_transfer_05() -> Result<()> {
  do_simple_polyx_transfer().await
}
