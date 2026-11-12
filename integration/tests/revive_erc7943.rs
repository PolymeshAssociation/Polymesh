//! The ERC-7943 precompile interface for Polymesh native assets.
//!
//! Requires a running `eth-rpc` node, see `ETH_RPC_URL`.

// The Revive pallet was added in v8.
#![cfg(feature = "current_release")]

use anyhow::Result;

use alloy::primitives::U256;
use alloy::sol_types::SolEvent;

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

    let logs = erc7943
        .set_frozen_tokens(&mut caller, holder_address, 1_000)
        .await?;

    assert!(erc7943.get_frozen_tokens(holder_address).await.unwrap() == 1_000);

    // The ERC-3643 difference comes first, then the ERC-7943 total.
    assert_eq!(
        contract_event_order(&logs, &erc7943.h160()),
        [
            ierc20::TokensFrozen::SIGNATURE_HASH,
            ierc20::Frozen::SIGNATURE_HASH,
        ],
    );

    // ERC-7943 reports the new total, ERC-3643 the difference.
    let frozen: Vec<ierc20::Frozen> = decode_contract_logs(&logs, &erc7943.h160())?;
    assert_eq!(frozen.len(), 1, "expected one Frozen event");
    assert_eq!(frozen[0].account, holder_address);
    assert_eq!(frozen[0].amount, U256::from(1_000));
    let tokens_frozen: Vec<ierc20::TokensFrozen> = decode_contract_logs(&logs, &erc7943.h160())?;
    assert_eq!(tokens_frozen.len(), 1, "expected one TokensFrozen event");
    assert_eq!(tokens_frozen[0].amount, U256::from(1_000));

    // Lowering the frozen amount reports the difference as unfrozen.
    let logs = erc7943
        .set_frozen_tokens(&mut caller, holder_address, 400)
        .await?;
    assert_eq!(
        contract_event_order(&logs, &erc7943.h160()),
        [
            ierc20::TokensUnfrozen::SIGNATURE_HASH,
            ierc20::Frozen::SIGNATURE_HASH,
        ],
    );
    let unfrozen: Vec<ierc20::TokensUnfrozen> = decode_contract_logs(&logs, &erc7943.h160())?;
    assert_eq!(unfrozen.len(), 1, "expected one TokensUnfrozen event");
    assert_eq!(unfrozen[0].amount, U256::from(600));
    let frozen: Vec<ierc20::Frozen> = decode_contract_logs(&logs, &erc7943.h160())?;
    assert_eq!(frozen.len(), 1, "expected one Frozen event");
    assert_eq!(
        frozen[0].amount,
        U256::from(400),
        "Frozen carries the new total"
    );

    // Setting the same amount again still emits `Frozen`, as ERC-7943 requires, but no difference.
    let logs = erc7943
        .set_frozen_tokens(&mut caller, holder_address, 400)
        .await?;
    assert_eq!(
        contract_event_order(&logs, &erc7943.h160()),
        [ierc20::Frozen::SIGNATURE_HASH],
        "an unchanged amount emits only Frozen"
    );
    let frozen: Vec<ierc20::Frozen> = decode_contract_logs(&logs, &erc7943.h160())?;
    assert_eq!(frozen[0].amount, U256::from(400));

    Ok(())
}

/// The precompile reports `IERC165` and `IERC7943Fungible`, as ERC-7943 requires.
#[tokio::test]
#[test_log::test]
async fn erc7943_supports_interface() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester.users(&["Erc7943IfaceIssuer"]).await?;
    let api = tester.api.clone();
    let issuer = &mut users[0];

    let (_, erc7943) = create_erc20_asset(&api, &node, issuer, "ERC7943 Iface", MINT).await?;

    assert!(erc7943.supports_interface([0x01, 0xff, 0xc9, 0xa7]).await?);
    assert!(erc7943.supports_interface([0x3e, 0xdb, 0xb4, 0xc4]).await?);
    // `IERC20`
    assert!(erc7943.supports_interface([0x36, 0x37, 0x2b, 0x07]).await?);
    // ERC-165 requires `false` for `0xffffffff`.
    assert!(!erc7943.supports_interface([0xff, 0xff, 0xff, 0xff]).await?);

    Ok(())
}

/// `forcedTransfer` emits the ERC-20 `Transfer` then `ForcedTransfer`, and no freeze events when
/// the free balance covers the amount.
#[tokio::test]
#[test_log::test]
async fn erc7943_forced_transfer() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester
        .users(&["Erc7943ForceIssuer", "Erc7943ForceHolder"])
        .await?;
    let api = tester.api.clone();
    let (issuers, holders) = users.split_at_mut(1);
    let issuer = &mut issuers[0];
    let holder = &mut holders[0];

    let (_, erc7943) = create_erc20_asset(&api, &node, issuer, "ERC7943 Force", MINT).await?;
    let issuer_address = eth_address_of(&api, issuer).await?;
    let holder_address = eth_address_of(&api, holder).await?;

    let mut caller = SubstrateCaller::new(&api, issuer).await?;
    erc7943.transfer(&mut caller, holder_address, 100).await?;

    let logs = erc7943
        .forced_transfer(&mut caller, holder_address, issuer_address, 40)
        .await?;

    let forced: Vec<ierc20::ForcedTransfer> = decode_contract_logs(&logs, &erc7943.h160())?;
    assert_eq!(forced.len(), 1, "expected one ForcedTransfer event");
    assert_eq!(forced[0].from, holder_address);
    assert_eq!(forced[0].to, issuer_address);
    assert_eq!(forced[0].amount, U256::from(40));

    let transfers: Vec<ierc20::Transfer> = decode_contract_logs(&logs, &erc7943.h160())?;
    assert_eq!(transfers.len(), 1, "expected one Transfer event");
    assert_eq!(transfers[0].from, holder_address);
    assert_eq!(transfers[0].to, issuer_address);
    assert_eq!(transfers[0].value, U256::from(40));

    // Nothing was unfrozen, so no `Frozen` or `TokensUnfrozen`.
    assert_eq!(
        contract_event_order(&logs, &erc7943.h160()),
        [
            ierc20::Transfer::SIGNATURE_HASH,
            ierc20::ForcedTransfer::SIGNATURE_HASH,
        ],
    );

    assert_eq!(erc7943.balance_of(holder_address).await?, 60);

    Ok(())
}

/// When `forcedTransfer` has to unfreeze tokens, it logs the change before the `Transfer`.
#[tokio::test]
#[test_log::test]
async fn erc7943_forced_transfer_unfreezes() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester
        .users(&["Erc7943UnfreezeIssuer", "Erc7943UnfreezeHolder"])
        .await?;
    let api = tester.api.clone();
    let (issuers, holders) = users.split_at_mut(1);
    let issuer = &mut issuers[0];
    let holder = &mut holders[0];

    let (_, erc7943) = create_erc20_asset(&api, &node, issuer, "ERC7943 Unfreeze", MINT).await?;
    let issuer_address = eth_address_of(&api, issuer).await?;
    let holder_address = eth_address_of(&api, holder).await?;

    let mut caller = SubstrateCaller::new(&api, issuer).await?;
    erc7943.transfer(&mut caller, holder_address, 100).await?;
    erc7943
        .set_frozen_tokens(&mut caller, holder_address, 80)
        .await?;

    // 20 tokens are free, so taking 50 unfreezes 30 and leaves 50 frozen.
    let logs = erc7943
        .forced_transfer(&mut caller, holder_address, issuer_address, 50)
        .await?;

    let unfrozen: Vec<ierc20::TokensUnfrozen> = decode_contract_logs(&logs, &erc7943.h160())?;
    assert_eq!(unfrozen.len(), 1, "expected one TokensUnfrozen event");
    assert_eq!(unfrozen[0].account, holder_address);
    assert_eq!(unfrozen[0].amount, U256::from(30));

    let frozen: Vec<ierc20::Frozen> = decode_contract_logs(&logs, &erc7943.h160())?;
    assert_eq!(frozen.len(), 1, "expected one Frozen event");
    assert_eq!(frozen[0].account, holder_address);
    assert_eq!(frozen[0].amount, U256::from(50));

    // ERC-7943: the `Frozen` event must come before the base `Transfer` event.
    assert_eq!(
        contract_event_order(&logs, &erc7943.h160()),
        [
            ierc20::TokensUnfrozen::SIGNATURE_HASH,
            ierc20::Frozen::SIGNATURE_HASH,
            ierc20::Transfer::SIGNATURE_HASH,
            ierc20::ForcedTransfer::SIGNATURE_HASH,
        ],
    );

    assert_eq!(erc7943.get_frozen_tokens(holder_address).await?, 50);
    assert_eq!(erc7943.balance_of(holder_address).await?, 50);

    Ok(())
}
