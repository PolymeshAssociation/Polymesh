//! The ERC-20 precompile interface for Polymesh native assets.
//!
//! Requires a running `eth-rpc` node, see `ETH_RPC_URL`.

// The Revive pallet was added in v8.
#![cfg(feature = "current_release")]

use anyhow::Result;

use alloy::primitives::{Address, U256};

use integration::*;
use polymesh_api::types::polymesh_primitives::settlement::AffirmationRequirement;
use polymesh_precompiles::IFungibleAsset as ierc20;

/// Initial supply issued to the asset owner's account.
const MINT: u128 = 1_000_000;

/// The zero address, used by the `Transfer` events of `mint` and `burn`.
const ZERO: Address = Address::ZERO;

/// The precompile metadata is read from the asset's chain state.
#[tokio::test]
#[test_log::test]
async fn erc20_metadata() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester.users(&["Erc20Meta"]).await?;
    let api = tester.api.clone();
    let issuer = &mut users[0];

    let (asset, erc20) = create_erc20_asset(&api, &node, issuer, "ERC20 Metadata", MINT).await?;
    let ticker = unique_ticker("MET");
    link_ticker(&api, issuer, asset.asset_id, ticker.clone()).await?;
    let issuer_address = eth_address_of(&api, issuer).await?;

    assert_eq!(erc20.name().await?, "ERC20 Metadata");
    assert_eq!(erc20.symbol().await?, String::from_utf8(ticker.0.to_vec())?);
    assert_eq!(erc20.decimals().await?, ERC20_DECIMALS);
    assert_eq!(erc20.total_supply().await?, MINT);
    // The whole supply was issued to the owner's account balance.
    assert_eq!(erc20.balance_of(issuer_address).await?, MINT);
    assert_eq!(erc20.allowance(issuer_address, ZERO).await?, 0);

    Ok(())
}

/// `transfer` moves tokens between two accounts mapped into `pallet_revive`.
#[tokio::test]
#[test_log::test]
async fn erc20_transfer_from_substrate() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester.users(&["Erc20SubIssuer", "Erc20SubHolder"]).await?;
    let api = tester.api.clone();
    let (issuers, holders) = users.split_at_mut(1);
    let issuer = &mut issuers[0];
    let holder = &mut holders[0];

    let (_asset, erc20) = create_erc20_asset(&api, &node, issuer, "ERC20 Sub", MINT).await?;
    let issuer_address = eth_address_of(&api, issuer).await?;
    let holder_address = eth_address_of(&api, holder).await?;
    let before = erc20.balance_of(holder_address).await?;

    let mut caller = SubstrateCaller::new(&api, issuer).await?;
    let logs = erc20.transfer(&mut caller, holder_address, 1_000).await?;

    let events: Vec<ierc20::Transfer> = decode_contract_logs(&logs, &erc20.h160())?;
    assert_eq!(events.len(), 1, "expected one Transfer event");
    assert_eq!(events[0].from, issuer_address);
    assert_eq!(events[0].to, holder_address);
    assert_eq!(events[0].value, U256::from(1_000));

    assert_eq!(erc20.balance_of(issuer_address).await?, MINT - 1_000);
    assert_eq!(erc20.balance_of(holder_address).await?, before + 1_000);
    // The supply didn't change.
    assert_eq!(erc20.total_supply().await?, MINT);

    Ok(())
}

/// The same `transfer`, driven by an Ethereum wallet through `eth-rpc`.
#[tokio::test]
#[test_log::test]
async fn erc20_transfer_from_eth_wallet() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester.users(&["Erc20EthIssuer"]).await?;
    let api = tester.api.clone();
    let issuer = &mut users[0];

    let (_asset, erc20) = create_erc20_asset(&api, &node, issuer, "ERC20 Eth", MINT).await?;
    let issuer_address = eth_address_of(&api, issuer).await?;

    // The wallet's account needs an identity before it can hold the asset.
    let mut wallet = node.new_wallet();
    wallet.onboard(&mut tester, REVIVE_INIT_POLYX).await?;

    // Fund the wallet from the issuer...
    let mut caller = SubstrateCaller::new(&api, issuer).await?;
    erc20.transfer(&mut caller, wallet.address, 5_000).await?;
    assert_eq!(erc20.balance_of(wallet.address).await?, 5_000);

    // ...then let the wallet send some of it back with an Ethereum transaction.
    let logs = erc20.transfer(&mut wallet, issuer_address, 2_000).await?;
    let events: Vec<ierc20::Transfer> = decode_contract_logs(&logs, &erc20.h160())?;
    assert_eq!(events.len(), 1, "expected one Transfer event");
    assert_eq!(events[0].from, wallet.address);
    assert_eq!(events[0].to, issuer_address);
    assert_eq!(events[0].value, U256::from(2_000));

    assert_eq!(erc20.balance_of(wallet.address).await?, 3_000);
    assert_eq!(erc20.balance_of(issuer_address).await?, MINT - 3_000);

    Ok(())
}

/// `approve` / `allowance` / `transferFrom` with a third-party spender.
#[tokio::test]
#[test_log::test]
async fn erc20_approve_and_transfer_from() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester
        .users(&["Erc20ApproveOwner", "Erc20ApproveSpender"])
        .await?;
    let api = tester.api.clone();
    let (owners, spenders) = users.split_at_mut(1);
    let owner = &mut owners[0];
    let spender = &mut spenders[0];

    let (_asset, erc20) = create_erc20_asset(&api, &node, owner, "ERC20 Approve", MINT).await?;
    let owner_address = eth_address_of(&api, owner).await?;
    let spender_address = eth_address_of(&api, spender).await?;

    // The recipient is an Ethereum wallet, to cover the mixed case.
    let mut recipient = node.new_wallet();
    recipient.onboard(&mut tester, REVIVE_INIT_POLYX).await?;

    let mut owner_caller = SubstrateCaller::new(&api, owner).await?;
    let logs = erc20
        .approve(&mut owner_caller, spender_address, 10_000)
        .await?;
    let events: Vec<ierc20::Approval> = decode_contract_logs(&logs, &erc20.h160())?;
    assert_eq!(events.len(), 1, "expected one Approval event");
    assert_eq!(events[0].owner, owner_address);
    assert_eq!(events[0].spender, spender_address);
    assert_eq!(events[0].value, U256::from(10_000));
    assert_eq!(
        erc20.allowance(owner_address, spender_address).await?,
        10_000
    );

    // The spender moves the owner's tokens to the recipient.
    let mut spender_caller = SubstrateCaller::new(&api, spender).await?;
    let logs = erc20
        .transfer_from(&mut spender_caller, owner_address, recipient.address, 4_000)
        .await?;
    let events: Vec<ierc20::Transfer> = decode_contract_logs(&logs, &erc20.h160())?;
    assert_eq!(events.len(), 1, "expected one Transfer event");
    assert_eq!(events[0].from, owner_address);
    assert_eq!(events[0].to, recipient.address);
    assert_eq!(events[0].value, U256::from(4_000));

    assert_eq!(erc20.balance_of(recipient.address).await?, 4_000);
    assert_eq!(erc20.balance_of(owner_address).await?, MINT - 4_000);
    // Spending the allowance reduces it, and the spender's own balance is untouched.
    assert_eq!(
        erc20.allowance(owner_address, spender_address).await?,
        6_000
    );
    assert_eq!(erc20.balance_of(spender_address).await?, 0);

    // Spending more than the remaining allowance fails.
    assert!(
        erc20
            .transfer_from(&mut spender_caller, owner_address, recipient.address, 6_001)
            .await
            .is_err(),
        "transferFrom should fail without enough allowance"
    );

    Ok(())
}

/// `mint` and `burn` change the asset's total supply.
#[tokio::test]
#[test_log::test]
async fn erc20_mint_and_burn() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester.users(&["Erc20Minter", "Erc20NotOwner"]).await?;
    let api = tester.api.clone();
    let (owners, others) = users.split_at_mut(1);
    let owner = &mut owners[0];
    let other = &mut others[0];

    let (_asset, erc20) = create_erc20_asset(&api, &node, owner, "ERC20 Supply", MINT).await?;
    let owner_address = eth_address_of(&api, owner).await?;

    let mut caller = SubstrateCaller::new(&api, owner).await?;
    let logs = erc20.mint(&mut caller, 500).await?;
    let events: Vec<ierc20::Transfer> = decode_contract_logs(&logs, &erc20.h160())?;
    assert_eq!(events.len(), 1, "expected one Transfer event");
    assert_eq!(events[0].from, ZERO);
    assert_eq!(events[0].to, owner_address);
    assert_eq!(events[0].value, U256::from(500));
    assert_eq!(erc20.total_supply().await?, MINT + 500);
    assert_eq!(erc20.balance_of(owner_address).await?, MINT + 500);

    let logs = erc20.burn(&mut caller, 200).await?;
    let events: Vec<ierc20::Transfer> = decode_contract_logs(&logs, &erc20.h160())?;
    assert_eq!(events.len(), 1, "expected one Transfer event");
    assert_eq!(events[0].from, owner_address);
    assert_eq!(events[0].to, ZERO);
    assert_eq!(events[0].value, U256::from(200));
    assert_eq!(erc20.total_supply().await?, MINT + 300);
    assert_eq!(erc20.balance_of(owner_address).await?, MINT + 300);

    // Only the asset owner can issue tokens.
    let mut other_caller = SubstrateCaller::new(&api, other).await?;
    assert!(
        erc20.mint(&mut other_caller, 1).await.is_err(),
        "only the asset owner may mint"
    );

    Ok(())
}

/// Calling the precompile of an asset that doesn't exist reverts.
#[tokio::test]
#[test_log::test]
async fn erc20_unknown_asset_reverts() -> Result<()> {
    let (_tester, node) = revive_tester().await?;

    // An index far beyond the next one to be assigned.
    let address = to_eth_address(&precompile_address(u32::MAX));

    let err = match node.call(address, &ierc20::totalSupplyCall {}).await {
        Ok(_) => panic!("totalSupply() of an unknown asset should revert"),
        Err(err) => err,
    };
    assert!(
        format!("{err:?}").contains("Asset not found"),
        "unexpected error: {err:?}"
    );

    Ok(())
}

/// Incomplete `transfer` call reverts.
#[tokio::test]
#[test_log::test]
async fn erc20_missing_affirmation_transfer_reverts() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester.users(&["Erc20SubIssuer", "Erc20SubHolder"]).await?;
    let api = tester.api.clone();
    let (issuers, holders) = users.split_at_mut(1);
    let issuer = &mut issuers[0];
    let holder = &mut holders[0];

    let (_asset, erc20) = create_erc20_asset(&api, &node, issuer, "ERC20 Sub", MINT).await?;
    erc20
        .api
        .call()
        .settlement()
        .set_mandatory_receiver_affirmation(AffirmationRequirement::Required)?
        .execute(holder)
        .await?;

    let _issuer_address = eth_address_of(&api, issuer).await?;
    let holder_address = eth_address_of(&api, holder).await?;

    let b_inst_id = erc20.api.query().settlement().instruction_counter().await?;

    let mut caller = SubstrateCaller::new(&api, issuer).await?;
    if erc20
        .transfer(&mut caller, holder_address, 1_000)
        .await
        .is_ok()
    {
        panic!("transfer() without receiver affirmation should revert");
    }

    let after_inst_id = erc20.api.query().settlement().instruction_counter().await?;
    assert_eq!(after_inst_id, b_inst_id);

    Ok(())
}
