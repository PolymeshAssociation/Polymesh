//! The `IPolymeshRuntime` precompile: general-purpose runtime extrinsics.
//!
//! Requires a running `eth-rpc` node, see `ETH_RPC_URL`.

// The Revive pallet was added in v8.
#![cfg(feature = "current_release")]

use anyhow::Result;

use alloy::primitives::{address, Address};
use alloy::sol_types::SolCall;

use integration::*;
use polymesh_api::types::polymesh_primitives::asset::{AssetName, AssetType};
use polymesh_precompiles::IPolymeshRuntime;

/// The address of `PolymeshRuntimeInterface`.
const POLYMESH_RUNTIME_ADDRESS: Address = address!("0x00000000000000000000000000000000FFFF0000");

/// `createAsset` registers a new asset under the caller's identity.
#[tokio::test]
#[test_log::test]
async fn polymesh_runtime_create_asset() -> Result<()> {
    let (mut tester, _node) = revive_tester().await?;
    let mut users = tester.users(&["PolymeshRuntimeCreateAsset"]).await?;
    let api = tester.api.clone();
    let issuer = &mut users[0];
    let issuer_did = issuer.did.expect("issuer DID");

    let mut caller = SubstrateCaller::new(&api, issuer).await?;
    let call = IPolymeshRuntime::assetCreateAssetCall {
        assetName: "PolyX Asset".to_string(),
        divisible: true,
        assetType: IPolymeshRuntime::AssetType {
            kind: IPolymeshRuntime::AssetTypeKind::EquityCommon,
            customTypeId: 0,
        },
        assetIdentifiers: vec![],
        fundingRoundName: String::new(),
    };
    let logs = caller
        .send_call(to_h160(&POLYMESH_RUNTIME_ADDRESS), call.abi_encode())
        .await?;

    let events: Vec<IPolymeshRuntime::AssetCreated> =
        decode_contract_logs(&logs, &to_h160(&POLYMESH_RUNTIME_ADDRESS))?;
    assert_eq!(events.len(), 1, "expected one AssetCreated event");
    assert_eq!(events[0].did.0, issuer_did.0);
    assert_eq!(events[0].assetName, "PolyX Asset");

    let asset_id = AssetId(events[0].assetId.0);
    let details = api
        .query()
        .asset()
        .assets(asset_id)
        .await?
        .expect("asset should exist on chain after creation");
    assert_eq!(details.owner_did, issuer_did);
    assert!(details.divisible);
    assert_eq!(details.asset_type, AssetType::EquityCommon);

    let name = api
        .query()
        .asset()
        .asset_names(asset_id)
        .await?
        .expect("asset name should exist on chain after creation");
    assert_eq!(name, AssetName(b"PolyX Asset".to_vec()));

    Ok(())
}

/// `registerTicker` registers a ticker symbol to the caller's identity.
#[tokio::test]
#[test_log::test]
async fn polymesh_runtime_register_ticker() -> Result<()> {
    let (mut tester, _node) = revive_tester().await?;
    let mut users = tester.users(&["PolymeshRuntimeRegisterTicker"]).await?;
    let api = tester.api.clone();
    let issuer = &mut users[0];
    let issuer_did = issuer.did.expect("issuer DID");

    let ticker = unique_ticker("POLYXRT");
    let ticker_str = String::from_utf8(ticker.0.to_vec())?;

    let mut caller = SubstrateCaller::new(&api, issuer).await?;
    let call = IPolymeshRuntime::assetRegisterTickerCall {
        ticker: ticker_str.clone(),
    };
    let logs = caller
        .send_call(to_h160(&POLYMESH_RUNTIME_ADDRESS), call.abi_encode())
        .await?;

    let events: Vec<IPolymeshRuntime::TickerRegistered> =
        decode_contract_logs(&logs, &to_h160(&POLYMESH_RUNTIME_ADDRESS))?;
    assert_eq!(events.len(), 1, "expected one TickerRegistered event");
    assert_eq!(events[0].did.0, issuer_did.0);
    assert_eq!(events[0].ticker, ticker_str);

    let registration = api
        .query()
        .asset()
        .unique_ticker_registration(ticker)
        .await?
        .expect("ticker registration should exist on chain after registering it");
    assert_eq!(registration.owner, issuer_did);
    assert!(registration.expiry.is_some());

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn polymesh_runtime_register_did() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let api = tester.api.clone();

    // A fresh wallet that has never been mapped or onboarded, so its fallback account has no DID.
    let wallet = node.new_wallet();
    let target_account = wallet.account();
    assert!(get_did(&api, target_account.clone()).await?.is_none());

    // `tester.cdd` is the chain's DID registrar (Alice on a `--dev` chain)
    let mut registrar = SubstrateCaller::new(&api, &mut tester.cdd).await?;
    let call = IPolymeshRuntime::identityRegisterDidCall {
        targetAccount: wallet.address,
    };
    let logs = registrar
        .send_call(to_h160(&POLYMESH_RUNTIME_ADDRESS), call.abi_encode())
        .await?;

    let events: Vec<IPolymeshRuntime::DidCreated> =
        decode_contract_logs(&logs, &to_h160(&POLYMESH_RUNTIME_ADDRESS))?;
    assert_eq!(events.len(), 1, "expected one DidCreated event");
    assert_eq!(events[0].targetAccount, wallet.address);

    let did = get_did(&api, target_account)
        .await?
        .expect("target account should have a DID after registration");
    assert_eq!(did.0, events[0].did.0);

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn polymesh_runtime_self_register_did() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let api = tester.api.clone();

    // A fresh wallet that has never been mapped or onboarded, so its fallback account has no DID.
    let mut wallet = node.new_wallet();
    wallet.fund(&mut tester, REVIVE_INIT_POLYX).await?;
    let target_account = wallet.account();
    assert!(get_did(&api, target_account.clone()).await?.is_none());

    let call = IPolymeshRuntime::identitySelfRegisterDidCall {};
    let logs = wallet
        .send_call(to_h160(&POLYMESH_RUNTIME_ADDRESS), call.abi_encode())
        .await?;

    let events: Vec<IPolymeshRuntime::DidCreated> =
        decode_contract_logs(&logs, &to_h160(&POLYMESH_RUNTIME_ADDRESS))?;
    assert_eq!(events.len(), 1, "expected one DidCreated event");
    assert_eq!(events[0].targetAccount, wallet.address);

    let did = get_did(&api, target_account)
        .await?
        .expect("caller account should have a DID after self-registration");
    assert_eq!(did.0, events[0].did.0);

    Ok(())
}
