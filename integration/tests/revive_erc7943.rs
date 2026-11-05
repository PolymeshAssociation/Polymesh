//! The ERC-7943 precompile interface for Polymesh native assets.
//!
//! Requires a running `eth-rpc` node, see `ETH_RPC_URL`.

// The Revive pallet was added in v8.
#![cfg(feature = "current_release")]

use anyhow::Result;

use alloy::primitives::U256;

use integration::*;
use polymesh_api::types::polymesh_primitives::condition::{Condition, ConditionType};
use polymesh_api::types::polymesh_primitives::condition::{TrustedFor, TrustedIssuer};
use polymesh_api::types::polymesh_primitives::identity_claim::{Claim, ClaimType, Scope};
use polymesh_api::types::polymesh_primitives::jurisdiction::CountryCode;
use polymesh_precompiles::IFungibleAsset as ierc20;

/// Initial supply issued to the asset owner's account.
const MINT: u128 = 1_000_000;

/// Can transfer returns false after receiver fails compliance check.
#[tokio::test]
#[test_log::test]
async fn erc7943_can_transfer_receiver_fails_compliance() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester
        .users(&["Erc7943SubIssuer", "Erc7943SubHolder"])
        .await?;
    let api = tester.api.clone();
    let (issuers, holders) = users.split_at_mut(1);
    let issuer = &mut issuers[0];
    let holder = &mut holders[0];

    let (asset_helper, erc7943) =
        create_erc20_asset(&api, &node, issuer, "ERC7943 Sub", MINT).await?;
    erc7943
        .api
        .call()
        .compliance_manager()
        .add_compliance_requirement(
            asset_helper.asset_id,
            Default::default(),
            vec![Condition {
                condition_type: ConditionType::IsPresent(Claim::Jurisdiction(
                    CountryCode::BR,
                    Scope::Identity(issuer.did.unwrap()),
                )),
                issuers: vec![TrustedIssuer {
                    issuer: issuer.did.unwrap(),
                    trusted_for: TrustedFor::Specific(vec![ClaimType::Jurisdiction]),
                }],
            }],
        )?
        .execute(issuer)
        .await?;

    let issuer_address = eth_address_of(&api, issuer).await?;
    let holder_address = eth_address_of(&api, holder).await?;

    assert!(!erc7943
        .can_transfer(issuer_address, holder_address, 1_000)
        .await
        .unwrap());

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn erc7943_set_get_frozen_tokens() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester
        .users(&["Erc7943SubIssuer", "Erc7943SubHolder"])
        .await?;
    let api = tester.api.clone();
    let (issuers, holders) = users.split_at_mut(1);
    let issuer = &mut issuers[0];
    let holder = &mut holders[0];

    let (_, erc7943) = create_erc20_asset(&api, &node, issuer, "ERC7943 SetFrozen", MINT).await?;
    let holder_address = eth_address_of(&api, holder).await?;

    let mut caller = SubstrateCaller::new(&api, issuer).await?;

    erc7943
        .set_frozen_tokens(&mut caller, holder_address, 1_000)
        .await?;

    assert!(erc7943.get_frozen_tokens(holder_address).await.unwrap() == 1_000);

    Ok(())
}

/// `forcedTransfer` claws tokens back to the agent, emitting both `ForcedTransfer` and `Transfer`.
#[tokio::test]
#[test_log::test]
async fn erc7943_forced_transfer() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester
        .users(&["Erc7943ForceAgent", "Erc7943ForceHolder"])
        .await?;
    let api = tester.api.clone();
    let (issuers, holders) = users.split_at_mut(1);
    let issuer = &mut issuers[0];
    let holder = &mut holders[0];

    let (_, erc7943) = create_erc20_asset(&api, &node, issuer, "ERC7943 Forced", MINT).await?;
    let issuer_address = eth_address_of(&api, issuer).await?;
    let holder_address = eth_address_of(&api, holder).await?;

    let mut caller = SubstrateCaller::new(&api, issuer).await?;
    erc7943.transfer(&mut caller, holder_address, 1_000).await?;
    assert_eq!(erc7943.balance_of(holder_address).await?, 1_000);

    // The agent takes part of it back.
    let logs = erc7943
        .forced_transfer(&mut caller, holder_address, issuer_address, 400)
        .await?;

    let forced: Vec<ierc20::ForcedTransfer> = decode_contract_logs(&logs, &erc7943.h160())?;
    assert_eq!(forced.len(), 1, "expected one ForcedTransfer event");
    assert_eq!(forced[0].from, holder_address);
    assert_eq!(forced[0].to, issuer_address);
    assert_eq!(forced[0].amount, U256::from(400));

    let transfers: Vec<ierc20::Transfer> = decode_contract_logs(&logs, &erc7943.h160())?;
    assert_eq!(transfers.len(), 1, "expected one Transfer event");
    assert_eq!(transfers[0].from, holder_address);
    assert_eq!(transfers[0].to, issuer_address);
    assert_eq!(transfers[0].value, U256::from(400));

    assert_eq!(erc7943.balance_of(holder_address).await?, 600);
    assert_eq!(erc7943.balance_of(issuer_address).await?, MINT - 600);

    Ok(())
}
