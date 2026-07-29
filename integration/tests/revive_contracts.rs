//! Deploying and calling contracts through both transaction styles.
//!
//! Requires a running `eth-rpc` node, see `ETH_RPC_URL`.

// The Revive pallet was added in v8.
#![cfg(feature = "current_release")]

use anyhow::Result;

use alloy::primitives::U256;

use integration::contracts::{ctor, CodeKind, ICounter, COUNTER};
use integration::*;

/// Deploy with a Substrate extrinsic, then call it with a Substrate extrinsic.
async fn deploy_and_call_substrate(kind: CodeKind) -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester.users(&["Deployer"]).await?;
    let api = tester.api.clone();
    let user = &mut users[0];

    let contract = Contract::deploy(&api, user, &COUNTER, kind, ctor::counter(41)).await?;
    let address = to_eth_address(&contract.address);

    // The constructor argument was applied.
    let value = node.call(address, &ICounter::numberCall {}).await?;
    assert_eq!(value, U256::from(41));

    // A state-changing call emits the contract's event...
    let mut res = contract.call(user, &ICounter::incrementCall {}).await?;
    let logs: Vec<ICounter::Incremented> = decode_logs(&mut res, &contract.address).await?;
    assert_eq!(logs.len(), 1, "expected one Incremented event");
    assert_eq!(logs[0].newValue, U256::from(42));
    assert_eq!(logs[0].caller, to_eth_address(&address_of(&user.account())));

    // ...and updates the contract's storage.
    let value = node.call(address, &ICounter::numberCall {}).await?;
    assert_eq!(value, U256::from(42));

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn substrate_deploy_and_call_evm() -> Result<()> {
    deploy_and_call_substrate(CodeKind::Evm).await
}

#[tokio::test]
#[test_log::test]
async fn substrate_deploy_and_call_polkavm() -> Result<()> {
    deploy_and_call_substrate(CodeKind::PolkaVM).await
}

/// Deploy with an Ethereum transaction, then call it with an Ethereum transaction.
async fn deploy_and_call_eth(kind: CodeKind) -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let wallet = node.new_wallet();
    wallet.fund(&mut tester, REVIVE_INIT_POLYX).await?;

    let address = wallet.deploy(&COUNTER, kind, ctor::counter(7)).await?;

    let value = node.call(address, &ICounter::numberCall {}).await?;
    assert_eq!(value, U256::from(7));

    let receipt = wallet.call(address, &ICounter::incrementCall {}).await?;
    let logs: Vec<ICounter::Incremented> = decode_receipt_logs(&receipt, address)?;
    assert_eq!(logs.len(), 1, "expected one Incremented event");
    assert_eq!(logs[0].newValue, U256::from(8));
    assert_eq!(logs[0].caller, wallet.address);

    let value = node.call(address, &ICounter::numberCall {}).await?;
    assert_eq!(value, U256::from(8));

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn eth_deploy_and_call_evm() -> Result<()> {
    deploy_and_call_eth(CodeKind::Evm).await
}

#[tokio::test]
#[test_log::test]
async fn eth_deploy_and_call_polkavm() -> Result<()> {
    deploy_and_call_eth(CodeKind::PolkaVM).await
}

/// A contract deployed one way must be callable the other way, and both callers
/// must see the same storage.
#[tokio::test]
#[test_log::test]
async fn cross_style_calls() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester.users(&["CrossDeployer"]).await?;
    let api = tester.api.clone();

    let wallet = node.new_wallet();
    wallet.fund(&mut tester, REVIVE_INIT_POLYX).await?;

    // Deployed from Substrate, called from Ethereum.
    let sub_contract = Contract::deploy(
        &api,
        &mut users[0],
        &COUNTER,
        CodeKind::Evm,
        ctor::counter(0),
    )
    .await?;
    let sub_address = to_eth_address(&sub_contract.address);
    wallet
        .call(sub_address, &ICounter::setNumberCall { newValue: U256::from(100) })
        .await?;
    assert_eq!(
        node.call(sub_address, &ICounter::numberCall {}).await?,
        U256::from(100)
    );

    // Deployed from Ethereum, called from Substrate.
    let eth_address = wallet.deploy(&COUNTER, CodeKind::Evm, ctor::counter(0)).await?;
    let eth_contract = Contract::new(&api, to_h160(&eth_address));
    eth_contract
        .call(
            &mut users[0],
            &ICounter::setNumberCall { newValue: U256::from(200) },
        )
        .await?;
    assert_eq!(
        node.call(eth_address, &ICounter::numberCall {}).await?,
        U256::from(200)
    );

    Ok(())
}

/// The same call driven through the [`ContractCaller`] abstraction from both
/// transaction styles, hitting the same contract instance.
#[tokio::test]
#[test_log::test]
async fn shared_caller_abstraction() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester.users(&["SharedCaller"]).await?;
    let api = tester.api.clone();

    let mut wallet = node.new_wallet();
    wallet.fund(&mut tester, REVIVE_INIT_POLYX).await?;

    let contract = Contract::deploy(
        &api,
        &mut users[0],
        &COUNTER,
        CodeKind::Evm,
        ctor::counter(0),
    )
    .await?;
    let address = to_eth_address(&contract.address);

    let mut sub_caller = SubstrateCaller::new(&api, &mut users[0]).await?;
    contract
        .call_as(&mut sub_caller, &ICounter::incrementCall {})
        .await?;
    contract
        .call_as(&mut wallet, &ICounter::incrementCall {})
        .await?;

    assert_eq!(
        node.call(address, &ICounter::numberCall {}).await?,
        U256::from(2)
    );

    Ok(())
}

/// A reverting call must fail, and the revert reason must reach the caller.
#[tokio::test]
#[test_log::test]
async fn revert_is_reported() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester.users(&["Reverter"]).await?;
    let api = tester.api.clone();

    let contract = Contract::deploy(
        &api,
        &mut users[0],
        &COUNTER,
        CodeKind::Evm,
        ctor::counter(0),
    )
    .await?;
    let address = to_eth_address(&contract.address);

    // `eth_call` surfaces the revert string.
    let err = match node.call(address, &ICounter::boomCall {}).await {
        Ok(_) => panic!("boom() should revert"),
        Err(err) => err,
    };
    assert!(
        format!("{err:?}").contains("Counter: boom"),
        "unexpected error: {err:?}"
    );

    // The Substrate extrinsic fails too.
    assert!(
        contract
            .call(&mut users[0], &ICounter::boomCall {})
            .await
            .is_err(),
        "boom() should revert the extrinsic"
    );

    Ok(())
}
