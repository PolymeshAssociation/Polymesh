//! The ERC-3643 precompile interface for Polymesh native assets.
//!
//! Requires a running `eth-rpc` node, see `ETH_RPC_URL`.

// The Revive pallet was added in v8.
#![cfg(feature = "current_release")]

use anyhow::Result;

use alloy::primitives::{keccak256, Address, U256};
use alloy::sol_types::SolEvent;

use integration::*;
use polymesh_precompiles::IFungibleAsset as ierc20;

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
    assert!(!erc3643.is_frozen(holder_address).await?);

    let logs = erc3643
        .set_address_frozen(&mut caller, holder_address, true)
        .await?;
    assert_eq!(
        contract_event_order(&logs, &erc3643.h160()),
        [ierc20::AddressFrozen::SIGNATURE_HASH],
    );

    assert!(!erc3643.can_send(holder_address).await.unwrap());
    assert!(erc3643.is_frozen(holder_address).await?);

    let logs = erc3643
        .set_address_frozen(&mut caller, holder_address, false)
        .await?;
    assert_eq!(
        contract_event_order(&logs, &erc3643.h160()),
        [ierc20::AddressFrozen::SIGNATURE_HASH],
    );
    assert!(!erc3643.is_frozen(holder_address).await?);

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

    let symbol = unique_symbol("ERC3643");
    erc3643.set_symbol(&mut caller, symbol.clone()).await?;

    assert_eq!(erc3643.symbol().await.unwrap(), symbol);

    Ok(())
}

/// `setSymbol` can be called repeatedly, and releases the ticker it replaces.
#[tokio::test]
#[test_log::test]
async fn erc3643_set_symbol_replaces_linked_ticker() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester.users(&["Erc3643SymbolReplacer"]).await?;
    let api = tester.api.clone();
    let issuer = &mut users[0];

    let (_, erc3643) = create_erc20_asset(&api, &node, issuer, "ERC3643 Replace", MINT).await?;
    let mut caller = SubstrateCaller::new(&api, issuer).await?;

    let first = unique_symbol("FIRST");
    erc3643.set_symbol(&mut caller, first.clone()).await?;
    assert_eq!(erc3643.symbol().await?, first);

    let second = unique_symbol("SECOND");
    erc3643.set_symbol(&mut caller, second.clone()).await?;
    assert_eq!(erc3643.symbol().await?, second);

    // The first ticker was released, so it can be registered again.
    erc3643.set_symbol(&mut caller, first.clone()).await?;
    assert_eq!(erc3643.symbol().await?, first);

    Ok(())
}

/// Setting the current symbol again succeeds for an agent, and still reverts for anyone else.
#[tokio::test]
#[test_log::test]
async fn erc3643_set_symbol_to_current_symbol() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester
        .users(&["Erc3643SameSymbol", "Erc3643SameSymbolOther"])
        .await?;
    let api = tester.api.clone();
    let (issuers, others) = users.split_at_mut(1);
    let issuer = &mut issuers[0];
    let other = &mut others[0];

    let (_, erc3643) = create_erc20_asset(&api, &node, issuer, "ERC3643 Same", MINT).await?;
    let mut caller = SubstrateCaller::new(&api, issuer).await?;

    let symbol = unique_symbol("SAME");
    // `UpdatedTokenInformation` is emitted when the symbol changes, and when it does not.
    for _ in 0..2 {
        let logs = erc3643.set_symbol(&mut caller, symbol.clone()).await?;
        assert_eq!(
            contract_event_order(&logs, &erc3643.h160()),
            [ierc20::UpdatedTokenInformation::SIGNATURE_HASH],
        );
        let events: Vec<ierc20::UpdatedTokenInformation> =
            decode_contract_logs(&logs, &erc3643.h160())?;
        assert_eq!(events[0].newName, keccak256("ERC3643 Same"));
        assert_eq!(events[0].newSymbol, keccak256(&symbol));
    }
    assert_eq!(erc3643.symbol().await?, symbol);

    let mut other_caller = SubstrateCaller::new(&api, other).await?;
    assert!(
        erc3643
            .set_symbol(&mut other_caller, symbol.clone())
            .await
            .is_err(),
        "a non-agent must not be able to call setSymbol"
    );

    Ok(())
}

/// A failed `setSymbol` leaves the current symbol linked.
#[tokio::test]
#[test_log::test]
async fn erc3643_set_symbol_taken_ticker_keeps_current_symbol() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester
        .users(&["Erc3643TakenSymbol", "Erc3643TakenSymbolOther"])
        .await?;
    let api = tester.api.clone();
    let (issuers, others) = users.split_at_mut(1);
    let issuer = &mut issuers[0];
    let other = &mut others[0];

    let (_, erc3643) = create_erc20_asset(&api, &node, issuer, "ERC3643 Taken", MINT).await?;
    let (_, other_erc3643) =
        create_erc20_asset(&api, &node, other, "ERC3643 Taken Other", MINT).await?;
    let mut caller = SubstrateCaller::new(&api, issuer).await?;
    let mut other_caller = SubstrateCaller::new(&api, other).await?;

    let symbol = unique_symbol("KEEP");
    erc3643.set_symbol(&mut caller, symbol.clone()).await?;
    let taken = unique_symbol("TAKEN");
    other_erc3643
        .set_symbol(&mut other_caller, taken.clone())
        .await?;

    assert!(erc3643.set_symbol(&mut caller, taken).await.is_err());
    assert_eq!(erc3643.symbol().await?, symbol);

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

    let logs = erc3643
        .set_name(&mut caller, "NEW_NAME".to_string())
        .await?;

    assert_eq!(erc3643.name().await.unwrap(), "NEW_NAME".to_string());

    // The name and symbol are indexed strings, so the event carries their hashes.
    assert_eq!(
        contract_event_order(&logs, &erc3643.h160()),
        [ierc20::UpdatedTokenInformation::SIGNATURE_HASH],
    );
    let events: Vec<ierc20::UpdatedTokenInformation> =
        decode_contract_logs(&logs, &erc3643.h160())?;
    assert_eq!(events[0].newName, keccak256("NEW_NAME"));
    assert_eq!(events[0].newSymbol, keccak256(""));
    assert_eq!(events[0].newDecimals, ERC20_DECIMALS);
    assert_eq!(events[0].newVersion, "");
    assert_eq!(events[0].newOnchainID, Address::ZERO);

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn erc3643_freeze_partial_tokens() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester.users(&["Erc3643Issuer", "Erc3643Holder"]).await?;
    let api = tester.api.clone();
    let (issuers, holders) = users.split_at_mut(1);
    let issuer = &mut issuers[0];
    let holder = &mut holders[0];

    let (_, erc3643) = create_erc20_asset(&api, &node, issuer, "ERC3643 Freeze", MINT).await?;

    let holder_address = eth_address_of(&api, holder).await?;
    let mut caller = SubstrateCaller::new(&api, issuer).await?;

    erc3643.transfer(&mut caller, holder_address, 150).await?;

    assert_eq!(erc3643.get_frozen_tokens(holder_address).await.unwrap(), 0);

    erc3643
        .freeze_partial_tokens(&mut caller, holder_address, 100)
        .await?;

    assert_eq!(
        erc3643.get_frozen_tokens(holder_address).await.unwrap(),
        100
    );

    // Freezing again adds to the amount already frozen, rather than replacing it.
    let logs = erc3643
        .freeze_partial_tokens(&mut caller, holder_address, 50)
        .await?;

    // ERC-3643 reports the difference, ERC-7943 the new total, in that order.
    assert_eq!(
        contract_event_order(&logs, &erc3643.h160()),
        [
            ierc20::TokensFrozen::SIGNATURE_HASH,
            ierc20::Frozen::SIGNATURE_HASH
        ],
    );
    let tokens_frozen: Vec<ierc20::TokensFrozen> = decode_contract_logs(&logs, &erc3643.h160())?;
    assert_eq!(tokens_frozen.len(), 1, "expected one TokensFrozen event");
    assert_eq!(tokens_frozen[0].amount, U256::from(50));
    let frozen: Vec<ierc20::Frozen> = decode_contract_logs(&logs, &erc3643.h160())?;
    assert_eq!(frozen.len(), 1, "expected one Frozen event");
    assert_eq!(frozen[0].amount, U256::from(150));

    assert_eq!(
        erc3643.get_frozen_tokens(holder_address).await.unwrap(),
        150
    );

    // ERC-3643 emits `TokensFrozen` on every call, even for 0, as T-REX does.
    let logs = erc3643
        .freeze_partial_tokens(&mut caller, holder_address, 0)
        .await?;
    assert_eq!(
        contract_event_order(&logs, &erc3643.h160()),
        [
            ierc20::TokensFrozen::SIGNATURE_HASH,
            ierc20::Frozen::SIGNATURE_HASH
        ],
    );
    let tokens_frozen: Vec<ierc20::TokensFrozen> = decode_contract_logs(&logs, &erc3643.h160())?;
    assert_eq!(tokens_frozen.len(), 1, "expected one TokensFrozen event");
    assert_eq!(tokens_frozen[0].amount, U256::ZERO);
    let frozen: Vec<ierc20::Frozen> = decode_contract_logs(&logs, &erc3643.h160())?;
    assert_eq!(frozen.len(), 1, "expected one Frozen event");
    assert_eq!(frozen[0].amount, U256::from(150));

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn erc3643_unfreeze_partial_tokens() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester.users(&["Erc3643Issuer", "Erc3643Holder"]).await?;
    let api = tester.api.clone();
    let (issuers, holders) = users.split_at_mut(1);
    let issuer = &mut issuers[0];
    let holder = &mut holders[0];

    let (_, erc3643) = create_erc20_asset(&api, &node, issuer, "ERC3643 Unfreeze", MINT).await?;

    let holder_address = eth_address_of(&api, holder).await?;
    let mut caller = SubstrateCaller::new(&api, issuer).await?;

    erc3643.transfer(&mut caller, holder_address, 100).await?;

    erc3643
        .freeze_partial_tokens(&mut caller, holder_address, 100)
        .await?;

    assert_eq!(
        erc3643.get_frozen_tokens(holder_address).await.unwrap(),
        100
    );

    let logs = erc3643
        .unfreeze_partial_tokens(&mut caller, holder_address, 40)
        .await?;

    assert_eq!(erc3643.get_frozen_tokens(holder_address).await.unwrap(), 60);

    // ERC-3643 reports the difference, ERC-7943 the new total, in that order.
    assert_eq!(
        contract_event_order(&logs, &erc3643.h160()),
        [
            ierc20::TokensUnfrozen::SIGNATURE_HASH,
            ierc20::Frozen::SIGNATURE_HASH
        ],
    );
    let unfrozen: Vec<ierc20::TokensUnfrozen> = decode_contract_logs(&logs, &erc3643.h160())?;
    assert_eq!(unfrozen.len(), 1, "expected one TokensUnfrozen event");
    assert_eq!(unfrozen[0].amount, U256::from(40));
    let frozen: Vec<ierc20::Frozen> = decode_contract_logs(&logs, &erc3643.h160())?;
    assert_eq!(frozen.len(), 1, "expected one Frozen event");
    assert_eq!(frozen[0].amount, U256::from(60));

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
    assert!(!erc3643.paused().await?);

    let logs = erc3643.pause(&mut caller).await?;
    assert_eq!(
        contract_event_order(&logs, &erc3643.h160()),
        [ierc20::Paused::SIGNATURE_HASH],
    );

    assert!(!erc3643.can_send(issuer_address).await.unwrap());
    assert!(erc3643.paused().await?);

    let logs = erc3643.unpause(&mut caller).await?;
    assert_eq!(
        contract_event_order(&logs, &erc3643.h160()),
        [ierc20::Unpaused::SIGNATURE_HASH],
    );

    assert!(erc3643.can_send(issuer_address).await.unwrap());
    assert!(!erc3643.paused().await?);

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn erc3643_is_agent() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester.users(&["Erc3643Owner", "Erc3643NonAgent"]).await?;
    let api = tester.api.clone();
    let (owners, non_agents) = users.split_at_mut(1);
    let owner = &mut owners[0];
    let non_agent = &mut non_agents[0];

    let (_, erc3643) = create_erc20_asset(&api, &node, owner, "ERC3643 Agent", MINT).await?;

    let owner_address = eth_address_of(&api, owner).await?;
    let non_agent_address = eth_address_of(&api, non_agent).await?;

    // The asset owner is a Full agent from creation.
    assert!(erc3643.is_agent(owner_address).await.unwrap());
    // An unrelated identity is not an agent of the asset.
    assert!(!erc3643.is_agent(non_agent_address).await.unwrap());

    Ok(())
}
