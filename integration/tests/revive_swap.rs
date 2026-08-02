//! A minimal swap contract trading Polymesh native assets and Solidity tokens.
//!
//! Requires a running `eth-rpc` node, see `ETH_RPC_URL`.

// The Revive pallet was added in v8.
#![cfg(feature = "current_release")]

use anyhow::Result;

use alloy::primitives::U256;

use integration::contracts::{ctor, CodeKind, ITestERC20, TEST_ERC20};
use integration::*;

/// Initial supply of each native asset.
const MINT: u128 = 1_000_000;
/// How much of the output token the contract holds.
const LIQUIDITY: u128 = 100_000;
/// How much of the input token the trader gets to play with.
const TRADER_FUNDS: u128 = 10_000;
/// How much is swapped.
const SWAP_IN: u128 = 1_000;
/// `amountB = amountA * RATE_NUM / RATE_DEN`.
const RATE_NUM: u128 = 2;
const RATE_DEN: u128 = 1;

/// Swap between two native assets, driven by either transaction style.
async fn native_swap(kind: CodeKind, eth_trader: bool) -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester.users(&["SwapIssuer", "SwapTrader"]).await?;
    let api = tester.api.clone();
    let (issuers, traders) = users.split_at_mut(1);
    let issuer = &mut issuers[0];
    let trader = &mut traders[0];

    let (_asset_a, erc20_a) = create_erc20_asset(&api, &node, issuer, "Swap Token A", MINT).await?;
    let (_asset_b, erc20_b) = create_erc20_asset(&api, &node, issuer, "Swap Token B", MINT).await?;

    let swap = SwapHelper::deploy(
        &mut tester,
        &node,
        issuer,
        kind,
        &erc20_a.token,
        &erc20_b.token,
        RATE_NUM,
        RATE_DEN,
    )
    .await?;

    // The trader is either a Substrate account or an Ethereum wallet. Both need
    // an identity to hold the assets.
    let mut wallet = node.new_wallet();
    let trader_address = if eth_trader {
        wallet.onboard(&mut tester, REVIVE_INIT_POLYX).await?;
        wallet.address
    } else {
        eth_address_of(&api, trader).await?
    };

    // Fund the contract with token B and the trader with token A.
    let mut issuer_caller = SubstrateCaller::new(&api, issuer).await?;
    erc20_b
        .transfer(&mut issuer_caller, swap.address, LIQUIDITY)
        .await?;
    erc20_a
        .transfer(&mut issuer_caller, trader_address, TRADER_FUNDS)
        .await?;
    assert_eq!(swap.liquidity().await?, (0, LIQUIDITY));

    let amount_out = swap.quote_a_to_b(SWAP_IN).await?;
    assert_eq!(amount_out, SWAP_IN * RATE_NUM / RATE_DEN);

    let mut sub_caller;
    let caller: &mut dyn ContractCaller = if eth_trader {
        &mut wallet
    } else {
        sub_caller = SubstrateCaller::new(&api, trader).await?;
        &mut sub_caller
    };

    // Swapping without an allowance fails.
    assert!(
        swap.swap_a_to_b(caller, SWAP_IN).await.is_err(),
        "swap should fail without an allowance"
    );

    erc20_a.approve(caller, swap.address, SWAP_IN).await?;
    let logs = swap.swap_a_to_b(caller, SWAP_IN).await?;

    let events = swap.swap_events(&logs)?;
    assert_eq!(events.len(), 1, "expected one Swap event");
    assert_eq!(events[0].caller, trader_address);
    assert_eq!(events[0].tokenIn, erc20_a.address);
    assert_eq!(events[0].tokenOut, erc20_b.address);
    assert_eq!(events[0].amountIn, U256::from(SWAP_IN));
    assert_eq!(events[0].amountOut, U256::from(amount_out));

    // Both tokens moved, in both directions.
    assert_eq!(
        erc20_a.balance_of(trader_address).await?,
        TRADER_FUNDS - SWAP_IN
    );
    assert_eq!(erc20_b.balance_of(trader_address).await?, amount_out);
    assert_eq!(swap.liquidity().await?, (SWAP_IN, LIQUIDITY - amount_out));
    // The allowance was spent by the contract.
    assert_eq!(erc20_a.allowance(trader_address, swap.address).await?, 0);

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn native_swap_from_substrate_evm() -> Result<()> {
    native_swap(CodeKind::Evm, false).await
}

#[tokio::test]
#[test_log::test]
async fn native_swap_from_substrate_polkavm() -> Result<()> {
    native_swap(CodeKind::PolkaVM, false).await
}

#[tokio::test]
#[test_log::test]
async fn native_swap_from_eth_wallet() -> Result<()> {
    native_swap(CodeKind::Evm, true).await
}

/// Swap a native asset against a Solidity ERC-20, in both directions.
#[tokio::test]
#[test_log::test]
async fn native_to_solidity_swap() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester
        .users(&["MixedSwapIssuer", "MixedSwapTrader"])
        .await?;
    let api = tester.api.clone();
    let (issuers, traders) = users.split_at_mut(1);
    let issuer = &mut issuers[0];
    let trader = &mut traders[0];

    // Token A is a native asset, token B a plain Solidity ERC-20.
    let (_asset_a, erc20_a) = create_erc20_asset(&api, &node, issuer, "Mixed Native", MINT).await?;
    let token_b = Contract::deploy(
        &api,
        issuer,
        &TEST_ERC20,
        CodeKind::Evm,
        ctor::test_erc20("Mixed Solidity", "MIX"),
    )
    .await?;
    let token_b = Token::new(&node, to_eth_address(&token_b.address));

    let swap = SwapHelper::deploy(
        &mut tester,
        &node,
        issuer,
        CodeKind::Evm,
        &erc20_a.token,
        &token_b,
        RATE_NUM,
        RATE_DEN,
    )
    .await?;

    let trader_address = eth_address_of(&api, trader).await?;

    // The Solidity token has an open mint, the native asset is transferred.
    let mut issuer_caller = SubstrateCaller::new(&api, issuer).await?;
    token_b
        .send(
            &mut issuer_caller,
            ITestERC20::mintCall {
                to: swap.address,
                value: U256::from(LIQUIDITY),
            },
        )
        .await?;
    erc20_a
        .transfer(&mut issuer_caller, trader_address, TRADER_FUNDS)
        .await?;
    assert_eq!(swap.liquidity().await?, (0, LIQUIDITY));

    // Native asset in, Solidity token out.
    let mut caller = SubstrateCaller::new(&api, trader).await?;
    let amount_out = swap.quote_a_to_b(SWAP_IN).await?;
    erc20_a.approve(&mut caller, swap.address, SWAP_IN).await?;
    let logs = swap.swap_a_to_b(&mut caller, SWAP_IN).await?;

    let events = swap.swap_events(&logs)?;
    assert_eq!(events.len(), 1, "expected one Swap event");
    assert_eq!(events[0].tokenIn, erc20_a.address);
    assert_eq!(events[0].tokenOut, token_b.address);
    assert_eq!(events[0].amountOut, U256::from(amount_out));

    assert_eq!(
        erc20_a.balance_of(trader_address).await?,
        TRADER_FUNDS - SWAP_IN
    );
    assert_eq!(token_b.balance_of(trader_address).await?, amount_out);

    // Solidity token in, native asset out.
    let back = swap.quote_b_to_a(amount_out).await?;
    assert_eq!(back, SWAP_IN);
    token_b
        .approve(&mut caller, swap.address, amount_out)
        .await?;
    let logs = swap.swap_b_to_a(&mut caller, amount_out).await?;

    let events = swap.swap_events(&logs)?;
    assert_eq!(events.len(), 1, "expected one Swap event");
    assert_eq!(events[0].tokenIn, token_b.address);
    assert_eq!(events[0].tokenOut, erc20_a.address);
    assert_eq!(events[0].amountOut, U256::from(back));

    // The trader is back where it started, and so is the contract.
    assert_eq!(erc20_a.balance_of(trader_address).await?, TRADER_FUNDS);
    assert_eq!(token_b.balance_of(trader_address).await?, 0);
    assert_eq!(swap.liquidity().await?, (0, LIQUIDITY));

    Ok(())
}

/// The swap is atomic: if it can't pay out, nothing moves.
#[tokio::test]
#[test_log::test]
async fn swap_without_liquidity_is_atomic() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester.users(&["DrySwapIssuer", "DrySwapTrader"]).await?;
    let api = tester.api.clone();
    let (issuers, traders) = users.split_at_mut(1);
    let issuer = &mut issuers[0];
    let trader = &mut traders[0];

    let (_asset_a, erc20_a) = create_erc20_asset(&api, &node, issuer, "Dry Token A", MINT).await?;
    let (_asset_b, erc20_b) = create_erc20_asset(&api, &node, issuer, "Dry Token B", MINT).await?;

    // No liquidity is added to this contract.
    let swap = SwapHelper::deploy(
        &mut tester,
        &node,
        issuer,
        CodeKind::Evm,
        &erc20_a.token,
        &erc20_b.token,
        RATE_NUM,
        RATE_DEN,
    )
    .await?;

    let trader_address = eth_address_of(&api, trader).await?;
    let mut issuer_caller = SubstrateCaller::new(&api, issuer).await?;
    erc20_a
        .transfer(&mut issuer_caller, trader_address, TRADER_FUNDS)
        .await?;

    let mut caller = SubstrateCaller::new(&api, trader).await?;
    erc20_a.approve(&mut caller, swap.address, SWAP_IN).await?;
    assert!(
        swap.swap_a_to_b(&mut caller, SWAP_IN).await.is_err(),
        "swap should fail without liquidity"
    );

    // The trader kept its tokens and its allowance.
    assert_eq!(erc20_a.balance_of(trader_address).await?, TRADER_FUNDS);
    assert_eq!(
        erc20_a.allowance(trader_address, swap.address).await?,
        SWAP_IN
    );
    assert_eq!(swap.liquidity().await?, (0, 0));

    Ok(())
}
