//! `Onboarder`, a contract that self-registers its own DID and creates an asset under it.
//!
//! Exercises a contract calling several `IPolymeshRuntime` precompile extrinsics from inside a
//! single contract call: unlike other test contracts (see `SwapHelper::deploy`), this one needs
//! no DID registrar to onboard it - it onboards itself with `identitySelfRegisterDid`.
//!
//! Requires a running `eth-rpc` node, see `ETH_RPC_URL`.

// The Revive pallet was added in v8.
#![cfg(feature = "current_release")]

use anyhow::Result;

use alloy::primitives::{address, Address};

use integration::contracts::{CodeKind, IOnboarder, ONBOARDER};
use integration::*;
use polymesh_api::types::polymesh_primitives::asset::{AssetName, AssetType};
use polymesh_precompiles::IPolymeshRuntime;

/// The address of `PolymeshRuntimeInterface`.
const POLYMESH_RUNTIME_ADDRESS: Address = address!("0x00000000000000000000000000000000FFFF0000");

async fn onboards_and_creates_asset(kind: CodeKind) -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester.users(&["OnboarderDeployer"]).await?;
    let api = tester.api.clone();
    let deployer = &mut users[0];

    let contract = Contract::deploy(&api, deployer, &ONBOARDER, kind, Vec::new()).await?;
    assert!(
        get_did(&api, contract.account()).await?.is_none(),
        "the contract should have no DID before onboarding itself"
    );

    let ticker = unique_ticker("ONBOARD");
    let ticker_str = String::from_utf8(ticker.0.to_vec())?;

    let mut res = contract
        .call(
            deployer,
            &IOnboarder::onboardAndCreateAssetCall {
                assetName: "Onboarder Asset".to_string(),
                divisible: true,
                ticker: ticker_str.clone(),
            },
        )
        .await?;

    // These events come from the precompile, not the contract, so they're addressed to
    // `POLYMESH_RUNTIME_ADDRESS` rather than `contract.address`.
    let did_events: Vec<IPolymeshRuntime::DidCreated> =
        decode_logs(&mut res, &to_h160(&POLYMESH_RUNTIME_ADDRESS)).await?;
    assert_eq!(did_events.len(), 1, "expected one DidCreated event");
    assert_eq!(
        did_events[0].targetAccount,
        to_eth_address(&contract.address)
    );

    let asset_events: Vec<IPolymeshRuntime::AssetCreated> =
        decode_logs(&mut res, &to_h160(&POLYMESH_RUNTIME_ADDRESS)).await?;
    assert_eq!(asset_events.len(), 1, "expected one AssetCreated event");
    assert_eq!(asset_events[0].did.0, did_events[0].did.0);
    assert_eq!(asset_events[0].assetName, "Onboarder Asset");

    let ticker_events: Vec<IPolymeshRuntime::TickerRegistered> =
        decode_logs(&mut res, &to_h160(&POLYMESH_RUNTIME_ADDRESS)).await?;
    assert_eq!(
        ticker_events.len(),
        1,
        "expected one TickerRegistered event"
    );
    assert_eq!(ticker_events[0].did.0, did_events[0].did.0);
    assert_eq!(ticker_events[0].ticker, ticker_str);

    // The identity, the asset and the ticker are all attributed to the contract's own account.
    let contract_did = get_did(&api, contract.account())
        .await?
        .expect("the contract should have a DID after onboarding itself");
    assert_eq!(contract_did.0, did_events[0].did.0);

    let asset_id = AssetId(asset_events[0].assetId.0);
    let details = api
        .query()
        .asset()
        .assets(asset_id)
        .await?
        .expect("asset should exist on chain after creation");
    assert_eq!(details.owner_did, contract_did);
    assert!(details.divisible);
    assert_eq!(details.asset_type, AssetType::EquityCommon);

    let name = api
        .query()
        .asset()
        .asset_names(asset_id)
        .await?
        .expect("asset name should exist on chain after creation");
    assert_eq!(name, AssetName(b"Onboarder Asset".to_vec()));

    let registration = api
        .query()
        .asset()
        .unique_ticker_registration(ticker)
        .await?
        .expect("ticker registration should exist on chain after registering it");
    assert_eq!(registration.owner, contract_did);

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn onboarder_evm() -> Result<()> {
    onboards_and_creates_asset(CodeKind::Evm).await
}

#[tokio::test]
#[test_log::test]
async fn onboarder_polkavm() -> Result<()> {
    onboards_and_creates_asset(CodeKind::PolkaVM).await
}
