//! The ERC-3643 precompile interface for Polymesh native assets.
//!
//! Requires a running `eth-rpc` node, see `ETH_RPC_URL`.

// The Revive pallet was added in v8.
#![cfg(feature = "current_release")]

use anyhow::Result;

use alloy::primitives::{keccak256, Address, U256};
use alloy::sol_types::SolEvent;

use integration::*;
use polymesh_api::types::polymesh_primitives::{
    agent::AgentGroup, authorization::AuthorizationData, secondary_key::Signatory,
};
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

/// `removeAgent` removes the identity behind an address as an agent and emits `AgentRemoved`.
#[tokio::test]
#[test_log::test]
async fn erc3643_remove_agent() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester
        .users(&["Erc3643RemoveOwner", "Erc3643RemoveAgent"])
        .await?;
    let api = tester.api.clone();
    let (owners, agents) = users.split_at_mut(1);
    let owner = &mut owners[0];
    let agent = &mut agents[0];

    let (asset, erc3643) =
        create_erc20_asset(&api, &node, owner, "ERC3643 RemoveAgent", MINT).await?;
    let owner_address = eth_address_of(&api, owner).await?;
    let agent_address = eth_address_of(&api, agent).await?;

    // Becoming an agent takes the target's consent: authorise, then accept.
    let mut res = api
        .call()
        .identity()
        .add_authorization(
            Signatory::Identity(agent.did.unwrap()),
            AuthorizationData::BecomeAgent(asset.asset_id, AgentGroup::ExceptMeta),
            None,
        )?
        .execute(owner)
        .await?;
    let auth_id = get_auth_id(&mut res)
        .await?
        .expect("Missing BecomeAgent auth id");
    api.call()
        .external_agents()
        .accept_become_agent(auth_id)?
        .execute(agent)
        .await?;
    assert!(erc3643.is_agent(agent_address).await?);

    // An agent that may not manage agents cannot remove one.
    {
        let mut agent_caller = SubstrateCaller::new(&api, agent).await?;
        let remove_owner = ierc20::removeAgentCall {
            agent: owner_address,
        };
        assert!(erc3643.send(&mut agent_caller, remove_owner).await.is_err());
    }
    assert!(erc3643.is_agent(owner_address).await?);

    let mut caller = SubstrateCaller::new(&api, owner).await?;
    let remove_agent = ierc20::removeAgentCall {
        agent: agent_address,
    };
    let logs = erc3643.send(&mut caller, remove_agent.clone()).await?;
    assert_eq!(
        contract_event_order(&logs, &erc3643.h160()),
        [ierc20::AgentRemoved::SIGNATURE_HASH],
    );
    let events: Vec<ierc20::AgentRemoved> = decode_contract_logs(&logs, &erc3643.h160())?;
    assert_eq!(events[0].agent, agent_address);
    assert!(!erc3643.is_agent(agent_address).await?);

    // No longer an agent, and the last full agent cannot be removed.
    assert!(erc3643.send(&mut caller, remove_agent).await.is_err());
    let remove_owner = ierc20::removeAgentCall {
        agent: owner_address,
    };
    assert!(erc3643.send(&mut caller, remove_owner).await.is_err());
    assert!(erc3643.is_agent(owner_address).await?);

    Ok(())
}

/// `batchTransfer` emits one `Transfer` per element, and moves nothing if one element fails.
#[tokio::test]
#[test_log::test]
async fn erc3643_batch_transfer() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester
        .users(&["Erc3643BatchIssuer", "Erc3643BatchA", "Erc3643BatchB"])
        .await?;
    let api = tester.api.clone();
    let (issuers, holders) = users.split_at_mut(1);
    let issuer = &mut issuers[0];

    let (_, erc3643) = create_erc20_asset(&api, &node, issuer, "ERC3643 Batch", MINT).await?;
    let issuer_address = eth_address_of(&api, issuer).await?;
    let a = eth_address_of(&api, &mut holders[0]).await?;
    let b = eth_address_of(&api, &mut holders[1]).await?;
    // A batch reserves the worst case of every transfer before the first one runs.
    let mut caller = SubstrateCaller::new(&api, issuer)
        .await?
        .with_weight_limit(BATCH_WEIGHT_LIMIT);

    let logs = erc3643
        .send(
            &mut caller,
            ierc20::batchTransferCall {
                toList: vec![a, b, a],
                amounts: vec![U256::from(100), U256::from(200), U256::from(300)],
            },
        )
        .await?;
    assert_eq!(
        contract_event_order(&logs, &erc3643.h160()),
        [ierc20::Transfer::SIGNATURE_HASH; 3],
    );
    let transfers: Vec<ierc20::Transfer> = decode_contract_logs(&logs, &erc3643.h160())?;
    for (event, (to, value)) in transfers.iter().zip([(a, 100), (b, 200), (a, 300)]) {
        assert_eq!(event.from, issuer_address);
        assert_eq!(event.to, to);
        assert_eq!(event.value, U256::from(value));
    }
    assert_eq!(erc3643.balance_of(a).await?, 400);
    assert_eq!(erc3643.balance_of(b).await?, 200);

    // The second element is more than the caller holds, so the first is not applied either.
    let failing = ierc20::batchTransferCall {
        toList: vec![a, b],
        amounts: vec![U256::from(1), U256::from(MINT)],
    };
    assert!(erc3643.send(&mut caller, failing).await.is_err());
    assert_eq!(erc3643.balance_of(a).await?, 400);

    // Arrays of different lengths, or longer than 16 elements, revert.
    let mismatch = ierc20::batchTransferCall {
        toList: vec![a, b],
        amounts: vec![U256::from(1)],
    };
    assert!(erc3643.send(&mut caller, mismatch).await.is_err());
    let too_long = ierc20::batchTransferCall {
        toList: vec![a; 17],
        amounts: vec![U256::from(1); 17],
    };
    assert!(erc3643.send(&mut caller, too_long).await.is_err());
    assert_eq!(erc3643.balance_of(a).await?, 400);

    Ok(())
}

/// `batchForcedTransfer` emits each element's events in the order of a single `forcedTransfer`.
#[tokio::test]
#[test_log::test]
async fn erc3643_batch_forced_transfer() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester
        .users(&[
            "Erc3643BatchForceIssuer",
            "Erc3643BatchForceA",
            "Erc3643BatchForceB",
        ])
        .await?;
    let api = tester.api.clone();
    let (issuers, holders) = users.split_at_mut(1);
    let issuer = &mut issuers[0];

    let (_, erc3643) = create_erc20_asset(&api, &node, issuer, "ERC3643 BatchForce", MINT).await?;
    let issuer_address = eth_address_of(&api, issuer).await?;
    let a = eth_address_of(&api, &mut holders[0]).await?;
    let b = eth_address_of(&api, &mut holders[1]).await?;
    let mut caller = SubstrateCaller::new(&api, issuer).await?;

    erc3643.transfer(&mut caller, a, 1_000).await?;
    erc3643.transfer(&mut caller, b, 1_000).await?;
    // All of A's tokens are frozen, so taking some unfreezes them first.
    erc3643.set_frozen_tokens(&mut caller, a, 1_000).await?;

    let logs = erc3643
        .send(
            &mut caller,
            ierc20::batchForcedTransferCall {
                fromList: vec![a, b],
                toList: vec![issuer_address, issuer_address],
                amounts: vec![U256::from(400), U256::from(1_000)],
            },
        )
        .await?;
    assert_eq!(
        contract_event_order(&logs, &erc3643.h160()),
        [
            ierc20::TokensUnfrozen::SIGNATURE_HASH,
            ierc20::Frozen::SIGNATURE_HASH,
            ierc20::Transfer::SIGNATURE_HASH,
            ierc20::ForcedTransfer::SIGNATURE_HASH,
            ierc20::Transfer::SIGNATURE_HASH,
            ierc20::ForcedTransfer::SIGNATURE_HASH,
        ],
    );
    assert_eq!(erc3643.balance_of(a).await?, 600);
    assert_eq!(erc3643.get_frozen_tokens(a).await?, 600);
    assert_eq!(erc3643.balance_of(b).await?, 0);

    Ok(())
}

/// The batch freeze functions emit the events of the single calls for each element.
#[tokio::test]
#[test_log::test]
async fn erc3643_batch_freezes() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester
        .users(&[
            "Erc3643BatchFrzIssuer",
            "Erc3643BatchFrzA",
            "Erc3643BatchFrzB",
        ])
        .await?;
    let api = tester.api.clone();
    let (issuers, holders) = users.split_at_mut(1);
    let issuer = &mut issuers[0];

    let (_, erc3643) = create_erc20_asset(&api, &node, issuer, "ERC3643 BatchFrz", MINT).await?;
    let a = eth_address_of(&api, &mut holders[0]).await?;
    let b = eth_address_of(&api, &mut holders[1]).await?;
    let mut caller = SubstrateCaller::new(&api, issuer).await?;

    erc3643.transfer(&mut caller, a, 1_000).await?;
    erc3643.transfer(&mut caller, b, 1_000).await?;

    let logs = erc3643
        .send(
            &mut caller,
            ierc20::batchSetAddressFrozenCall {
                userAddresses: vec![a, b],
                freeze: vec![true, false],
            },
        )
        .await?;
    assert_eq!(
        contract_event_order(&logs, &erc3643.h160()),
        [ierc20::AddressFrozen::SIGNATURE_HASH; 2],
    );
    assert!(erc3643.is_frozen(a).await?);
    assert!(!erc3643.is_frozen(b).await?);

    let logs = erc3643
        .send(
            &mut caller,
            ierc20::batchFreezePartialTokensCall {
                userAddresses: vec![a, b],
                amounts: vec![U256::from(500), U256::from(700)],
            },
        )
        .await?;
    assert_eq!(
        contract_event_order(&logs, &erc3643.h160()),
        [
            ierc20::TokensFrozen::SIGNATURE_HASH,
            ierc20::Frozen::SIGNATURE_HASH,
            ierc20::TokensFrozen::SIGNATURE_HASH,
            ierc20::Frozen::SIGNATURE_HASH,
        ],
    );
    assert_eq!(erc3643.get_frozen_tokens(a).await?, 500);
    assert_eq!(erc3643.get_frozen_tokens(b).await?, 700);

    // The second element unfreezes more than is frozen, so the first is not applied either.
    let too_much = ierc20::batchUnfreezePartialTokensCall {
        userAddresses: vec![a, b],
        amounts: vec![U256::from(500), U256::from(701)],
    };
    assert!(erc3643.send(&mut caller, too_much).await.is_err());
    assert_eq!(erc3643.get_frozen_tokens(a).await?, 500);

    let logs = erc3643
        .send(
            &mut caller,
            ierc20::batchUnfreezePartialTokensCall {
                userAddresses: vec![a, b],
                amounts: vec![U256::from(200), U256::from(700)],
            },
        )
        .await?;
    assert_eq!(
        contract_event_order(&logs, &erc3643.h160()),
        [
            ierc20::TokensUnfrozen::SIGNATURE_HASH,
            ierc20::Frozen::SIGNATURE_HASH,
            ierc20::TokensUnfrozen::SIGNATURE_HASH,
            ierc20::Frozen::SIGNATURE_HASH,
        ],
    );
    assert_eq!(erc3643.get_frozen_tokens(a).await?, 300);
    assert_eq!(erc3643.get_frozen_tokens(b).await?, 0);

    Ok(())
}
