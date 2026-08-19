//! The ERC-7943 precompile interface for Polymesh native assets.
//!
//! Requires a running `eth-rpc` node, see `ETH_RPC_URL`.

// The Revive pallet was added in v8.
#![cfg(feature = "current_release")]

use anyhow::Result;

use integration::*;

/// Initial supply issued to the asset owner's account.
const MINT: u128 = 1_000_000;

#[tokio::test]
#[test_log::test]
async fn erc3643_set_address_frozen() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester.users(&["Erc3643Issuer", "Erc3643Holder"]).await?;
    let api = tester.api.clone();
    let (issuers, holders) = users.split_at_mut(1);
    let issuer = &mut issuers[0];
    let holder = &mut holders[0];

    let (_, erc3643) = create_erc20_asset(&api, &node, issuer, "ERC3643", MINT).await?;

    let holder_address = eth_address_of(&api, holder).await?;
    let mut caller = SubstrateCaller::new(&api, issuer).await?;

    assert!(erc3643.can_send(holder_address).await.unwrap());

    erc3643
        .set_address_frozen(&mut caller, holder_address, true)
        .await?;

    assert!(!erc3643.can_send(holder_address).await.unwrap());

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn erc3643_set_symbol() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester.users(&["Erc3643Issuer"]).await?;
    let api = tester.api.clone();
    let issuer = &mut users[0];

    let (_, erc3643) = create_erc20_asset(&api, &node, issuer, "ERC3643 Symbol", MINT).await?;

    let mut caller = SubstrateCaller::new(&api, issuer).await?;

    assert_eq!(erc3643.symbol().await.unwrap(), "".to_string());

    erc3643
        .set_symbol(&mut caller, "NEW_SYMBOL".to_string())
        .await?;

    assert_eq!(erc3643.symbol().await.unwrap(), "NEW_SYMBOL".to_string());

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn erc3643_set_name() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester.users(&["Erc3643Issuer"]).await?;
    let api = tester.api.clone();
    let issuer = &mut users[0];

    let (_, erc3643) = create_erc20_asset(&api, &node, issuer, "ERC3643 Name", MINT).await?;

    let mut caller = SubstrateCaller::new(&api, issuer).await?;

    assert_eq!(erc3643.name().await.unwrap(), "ERC3643 Name".to_string());

    erc3643
        .set_name(&mut caller, "NEW_NAME".to_string())
        .await?;

    assert_eq!(erc3643.name().await.unwrap(), "NEW_NAME".to_string());

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn erc3643_pause_unpause() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester.users(&["Erc3643Issuer"]).await?;
    let api = tester.api.clone();
    let issuer = &mut users[0];

    let (_, erc3643) = create_erc20_asset(&api, &node, issuer, "ERC3643 Pause", MINT).await?;

    let issuer_address = eth_address_of(&api, issuer).await?;
    let mut caller = SubstrateCaller::new(&api, issuer).await?;

    assert!(erc3643.can_send(issuer_address).await.unwrap());

    erc3643.pause(&mut caller).await?;

    assert!(!erc3643.can_send(issuer_address).await.unwrap());

    erc3643.unpause(&mut caller).await?;

    assert!(erc3643.can_send(issuer_address).await.unwrap());

    Ok(())
}
