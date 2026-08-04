//! The ERC-20 precompile interface for Polymesh native assets.
//!
//! Requires a running `eth-rpc` node, see `ETH_RPC_URL`.

// The Revive pallet was added in v8.
#![cfg(feature = "current_release")]

use anyhow::Result;

use alloy::primitives::{Address, U256};

use integration::*;
use polymesh_api::types::polymesh_primitives::settlement::AffirmationRequirement;
use polymesh_precompiles::IFungibleAsset as ierc7943;

/// Initial supply issued to the asset owner's account.
const MINT: u128 = 1_000_000;

/// The zero address, used by the `Transfer` events of `mint` and `burn`.
const ZERO: Address = Address::ZERO;

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

    let (_asset, erc7943) = create_erc20_asset(&api, &node, issuer, "ERC7943 Sub", MINT).await?;
    erc7943
        .api
        .call()
        .settlement()
        .set_mandatory_receiver_affirmation(AffirmationRequirement::Required)?
        .execute(holder)
        .await?;

    let _issuer_address = eth_address_of(&api, issuer).await?;
    let holder_address = eth_address_of(&api, holder).await?;

    let mut caller = SubstrateCaller::new(&api, issuer).await?;
    if erc7943
        .transfer(&mut caller, holder_address, 1_000)
        .await
        .is_ok()
    {
        panic!("transfer() without receiver affirmation should revert");
    }

    Ok(())
}
