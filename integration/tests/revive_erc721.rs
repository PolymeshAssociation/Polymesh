//! The ERC-721 precompile interface for Polymesh NFT collections.
//!
//! Requires a running `eth-rpc` node, see `ETH_RPC_URL`.

// The Revive pallet was added in v8.
#![cfg(feature = "current_release")]

use anyhow::Result;

use alloy::primitives::{Address, U256};

use integration::*;
use polymesh_precompiles::INonFungibleAsset as ierc721;

/// The zero address, used by the `Transfer` events of `mint` and `burn`.
const ZERO: Address = Address::ZERO;

/// ERC-165 interface ids, from the EIP-721 specification.
const ERC165_ID: [u8; 4] = [0x01, 0xff, 0xc9, 0xa7];
const ERC721_ID: [u8; 4] = [0x80, 0xac, 0x58, 0xcd];
const ERC721_METADATA_ID: [u8; 4] = [0x5b, 0x5e, 0x13, 0x9f];

/// The precompile metadata is read from the collection's chain state.
#[tokio::test]
#[test_log::test]
async fn erc721_metadata() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester.users(&["Erc721Meta"]).await?;
    let api = tester.api.clone();
    let owner = &mut users[0];

    let nft = create_erc721_collection(&api, &node, owner, "ERC721 Metadata").await?;
    let ticker = unique_ticker("NFT");
    link_ticker(&api, owner, nft.asset_id, ticker.clone()).await?;

    assert_eq!(nft.name().await?, "ERC721 Metadata");
    assert_eq!(nft.symbol().await?, String::from_utf8(ticker.0.to_vec())?);
    assert_eq!(nft.total_supply().await?, 0);

    // ERC-165 introspection.
    assert!(nft.supports_interface(ERC165_ID).await?);
    assert!(nft.supports_interface(ERC721_ID).await?);
    assert!(nft.supports_interface(ERC721_METADATA_ID).await?);
    // An interface we do not implement.
    assert!(!nft.supports_interface([0xde, 0xad, 0xbe, 0xef]).await?);

    Ok(())
}

/// `safeTransferFrom` to an externally-owned account behaves like `transferFrom`.
#[tokio::test]
#[test_log::test]
async fn erc721_safe_transfer_from_eoa() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester.users(&["Erc721SafeFrom", "Erc721SafeTo"]).await?;
    let api = tester.api.clone();
    let (owners, holders) = users.split_at_mut(1);
    let owner = &mut owners[0];
    let holder = &mut holders[0];

    let nft = create_erc721_collection(&api, &node, owner, "ERC721 Safe").await?;
    let owner_address = eth_address_of(&api, owner).await?;
    let holder_address = eth_address_of(&api, holder).await?;

    let mut caller = SubstrateCaller::new(&api, owner).await?;
    nft.mint(&mut caller, vec![]).await?;

    let logs = nft
        .safe_transfer_from(&mut caller, owner_address, holder_address, 1)
        .await?;
    let events: Vec<ierc721::Transfer> = decode_contract_logs(&logs, &nft.h160())?;
    assert_eq!(events.len(), 1, "expected one Transfer event");
    assert_eq!(events[0].to, holder_address);

    assert_eq!(nft.owner_of(1).await?, holder_address);

    Ok(())
}

/// `safeTransferFrom` refuses a receiver with code, while `transferFrom` still allows it.
///
/// The precompile cannot invoke `onERC721Received`, so rather than silently drop ERC-721's
/// no-stranded-tokens guarantee it rejects every contract receiver.
#[tokio::test]
#[test_log::test]
async fn erc721_safe_transfer_from_rejects_contract() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester.users(&["Erc721SafeContract"]).await?;
    let api = tester.api.clone();
    let owner = &mut users[0];

    let nft = create_erc721_collection(&api, &node, owner, "ERC721 SafeContract").await?;
    let owner_address = eth_address_of(&api, owner).await?;

    let mut caller = SubstrateCaller::new(&api, owner).await?;
    nft.mint(&mut caller, vec![]).await?;

    // Any address with code will do; another precompile is the cheapest one to hand.
    let contract_address = to_eth_address(&precompile_address(nft.asset_id));

    // `eth_call` surfaces the revert reason; a submitted extrinsic does not.
    let err = match nft
        .try_call(
            owner_address,
            &ierc721::safeTransferFrom_0Call {
                from: owner_address,
                to: contract_address,
                tokenId: U256::from(1),
            },
        )
        .await
    {
        Ok(_) => panic!("safeTransferFrom() to a contract should revert"),
        Err(err) => err,
    };
    assert!(
        format!("{err:?}").contains("externally-owned accounts"),
        "unexpected error: {err:?}"
    );

    // And submitting it really does fail rather than moving the NFT.
    assert!(
        nft.safe_transfer_from(&mut caller, owner_address, contract_address, 1)
            .await
            .is_err(),
        "safeTransferFrom() to a contract should not be submittable"
    );
    assert_eq!(nft.owner_of(1).await?, owner_address);

    Ok(())
}

/// `transferFrom` by the owner moves the NFT and updates both balances.
#[tokio::test]
#[test_log::test]
async fn erc721_transfer_from_owner() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester.users(&["Erc721XferFrom", "Erc721XferTo"]).await?;
    let api = tester.api.clone();
    let (owners, holders) = users.split_at_mut(1);
    let owner = &mut owners[0];
    let holder = &mut holders[0];

    let nft = create_erc721_collection(&api, &node, owner, "ERC721 Transfer").await?;
    let owner_address = eth_address_of(&api, owner).await?;
    let holder_address = eth_address_of(&api, holder).await?;

    let mut caller = SubstrateCaller::new(&api, owner).await?;
    nft.mint(&mut caller, vec![]).await?;

    let logs = nft
        .transfer_from(&mut caller, owner_address, holder_address, 1)
        .await?;

    let events: Vec<ierc721::Transfer> = decode_contract_logs(&logs, &nft.h160())?;
    assert_eq!(events.len(), 1, "expected one Transfer event");
    assert_eq!(events[0].from, owner_address);
    assert_eq!(events[0].to, holder_address);
    assert_eq!(events[0].tokenId, U256::from(1));

    assert_eq!(nft.owner_of(1).await?, holder_address);
    assert_eq!(nft.balance_of(owner_address).await?, 0);
    assert_eq!(nft.balance_of(holder_address).await?, 1);
    // The supply didn't change.
    assert_eq!(nft.total_supply().await?, 1);

    Ok(())
}

/// A per-token `approve` lets a third party move exactly that NFT, and the approval does not
/// survive the transfer.
#[tokio::test]
#[test_log::test]
async fn erc721_approve_and_transfer_from() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester
        .users(&["Erc721ApprOwner", "Erc721ApprSpender"])
        .await?;
    let api = tester.api.clone();
    let (owners, spenders) = users.split_at_mut(1);
    let owner = &mut owners[0];
    let spender = &mut spenders[0];

    let nft = create_erc721_collection(&api, &node, owner, "ERC721 Approve").await?;
    let owner_address = eth_address_of(&api, owner).await?;
    let spender_address = eth_address_of(&api, spender).await?;

    let mut owner_caller = SubstrateCaller::new(&api, owner).await?;
    nft.mint(&mut owner_caller, vec![]).await?;

    // No approval yet.
    assert_eq!(nft.get_approved(1).await?, ZERO);

    let logs = nft.approve(&mut owner_caller, spender_address, 1).await?;
    let events: Vec<ierc721::Approval> = decode_contract_logs(&logs, &nft.h160())?;
    assert_eq!(events.len(), 1, "expected one Approval event");
    assert_eq!(events[0].owner, owner_address);
    assert_eq!(events[0].approved, spender_address);
    assert_eq!(events[0].tokenId, U256::from(1));

    assert_eq!(nft.get_approved(1).await?, spender_address);

    // The spender moves the NFT to itself.
    let mut spender_caller = SubstrateCaller::new(&api, spender).await?;
    nft.transfer_from(&mut spender_caller, owner_address, spender_address, 1)
        .await?;

    assert_eq!(nft.owner_of(1).await?, spender_address);
    // ERC-721 requires the approval to be cleared by the transfer.
    assert_eq!(nft.get_approved(1).await?, ZERO);

    Ok(())
}

/// `setApprovalForAll` authorizes every NFT of the collection and is not consumed on use.
#[tokio::test]
#[test_log::test]
async fn erc721_approval_for_all() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester.users(&["Erc721OpOwner", "Erc721Operator"]).await?;
    let api = tester.api.clone();
    let (owners, operators) = users.split_at_mut(1);
    let owner = &mut owners[0];
    let operator = &mut operators[0];

    let nft = create_erc721_collection(&api, &node, owner, "ERC721 Operator").await?;
    let owner_address = eth_address_of(&api, owner).await?;
    let operator_address = eth_address_of(&api, operator).await?;

    let mut owner_caller = SubstrateCaller::new(&api, owner).await?;
    nft.mint(&mut owner_caller, vec![]).await?;
    nft.mint(&mut owner_caller, vec![]).await?;

    assert!(
        !nft.is_approved_for_all(owner_address, operator_address)
            .await?
    );

    let logs = nft
        .set_approval_for_all(&mut owner_caller, operator_address, true)
        .await?;
    let events: Vec<ierc721::ApprovalForAll> = decode_contract_logs(&logs, &nft.h160())?;
    assert_eq!(events.len(), 1, "expected one ApprovalForAll event");
    assert_eq!(events[0].owner, owner_address);
    assert_eq!(events[0].operator, operator_address);
    assert!(events[0].approved);

    assert!(
        nft.is_approved_for_all(owner_address, operator_address)
            .await?
    );

    // The operator moves both NFTs without any per-token approval.
    let mut operator_caller = SubstrateCaller::new(&api, operator).await?;
    for token_id in [1u64, 2u64] {
        nft.transfer_from(
            &mut operator_caller,
            owner_address,
            operator_address,
            token_id,
        )
        .await?;
    }

    assert_eq!(nft.balance_of(operator_address).await?, 2);
    assert_eq!(nft.balance_of(owner_address).await?, 0);
    // The operator approval survived.
    assert!(
        nft.is_approved_for_all(owner_address, operator_address)
            .await?
    );

    Ok(())
}

/// Moving someone else's NFT without an approval reverts.
#[tokio::test]
#[test_log::test]
async fn erc721_transfer_without_approval_reverts() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester
        .users(&["Erc721NoApprOwner", "Erc721NoApprSpender"])
        .await?;
    let api = tester.api.clone();
    let (owners, spenders) = users.split_at_mut(1);
    let owner = &mut owners[0];
    let spender = &mut spenders[0];

    let nft = create_erc721_collection(&api, &node, owner, "ERC721 NoApproval").await?;
    let owner_address = eth_address_of(&api, owner).await?;
    let spender_address = eth_address_of(&api, spender).await?;

    let mut owner_caller = SubstrateCaller::new(&api, owner).await?;
    nft.mint(&mut owner_caller, vec![]).await?;

    let mut spender_caller = SubstrateCaller::new(&api, spender).await?;
    if nft
        .transfer_from(&mut spender_caller, owner_address, spender_address, 1)
        .await
        .is_ok()
    {
        panic!("transferFrom() without an approval should revert");
    }

    // The NFT didn't move.
    assert_eq!(nft.owner_of(1).await?, owner_address);

    Ok(())
}

/// `forcedTransfer` claws an NFT back to the collection's agent.
#[tokio::test]
#[test_log::test]
async fn erc721_forced_transfer() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester
        .users(&["Erc721ForceAgent", "Erc721ForceHolder"])
        .await?;
    let api = tester.api.clone();
    let (owners, holders) = users.split_at_mut(1);
    let owner = &mut owners[0];
    let holder = &mut holders[0];

    let nft = create_erc721_collection(&api, &node, owner, "ERC721 Forced").await?;
    let owner_address = eth_address_of(&api, owner).await?;
    let holder_address = eth_address_of(&api, holder).await?;

    let mut caller = SubstrateCaller::new(&api, owner).await?;
    nft.mint(&mut caller, vec![]).await?;
    nft.transfer_from(&mut caller, owner_address, holder_address, 1)
        .await?;
    assert_eq!(nft.owner_of(1).await?, holder_address);

    // The agent takes it back.
    let logs = nft.forced_transfer(&mut caller, holder_address, 1).await?;
    let events: Vec<ierc721::ForcedTransfer> = decode_contract_logs(&logs, &nft.h160())?;
    assert_eq!(events.len(), 1, "expected one ForcedTransfer event");
    assert_eq!(events[0].from, holder_address);
    assert_eq!(events[0].to, owner_address);
    assert_eq!(events[0].tokenId, U256::from(1));

    assert_eq!(nft.owner_of(1).await?, owner_address);

    Ok(())
}

/// The ERC-7943 view methods report the collection's transfer restrictions.
#[tokio::test]
#[test_log::test]
async fn erc721_transfer_restrictions() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester.users(&["Erc7943NftFrom", "Erc7943NftTo"]).await?;
    let api = tester.api.clone();
    let (owners, holders) = users.split_at_mut(1);
    let owner = &mut owners[0];
    let holder = &mut holders[0];

    let nft = create_erc721_collection(&api, &node, owner, "ERC721 Restrictions").await?;
    let owner_address = eth_address_of(&api, owner).await?;
    let holder_address = eth_address_of(&api, holder).await?;

    let mut caller = SubstrateCaller::new(&api, owner).await?;
    nft.mint(&mut caller, vec![]).await?;

    assert!(nft.can_send(owner_address).await?);
    assert!(nft.can_receive(holder_address).await?);
    assert!(nft.can_transfer(owner_address, holder_address, 1).await?);

    Ok(())
}

/// `ownerOf` reverts for a token that was never issued.
#[tokio::test]
#[test_log::test]
async fn erc721_unknown_token_reverts() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester.users(&["Erc721Unknown"]).await?;
    let api = tester.api.clone();
    let owner = &mut users[0];

    let nft = create_erc721_collection(&api, &node, owner, "ERC721 Unknown").await?;

    let err = match nft.owner_of(999).await {
        Ok(_) => panic!("ownerOf() of a non-existent token should revert"),
        Err(err) => err,
    };
    assert!(
        format!("{err:?}").contains("NFT does not exist"),
        "unexpected error: {err:?}"
    );

    Ok(())
}

/// Calling the NFT precompile of a collection that doesn't exist reverts.
#[tokio::test]
#[test_log::test]
async fn erc721_unknown_collection_reverts() -> Result<()> {
    let (_tester, node) = revive_tester().await?;

    let unknown_asset_id = "ffffffff-ffff-ffff-ffff-ffffffffffff"
        .parse()
        .expect("valid UUID literal");
    let address = to_eth_address(&nft_precompile_address(unknown_asset_id));

    let err = match node.call(address, &ierc721::totalSupplyCall {}).await {
        Ok(_) => panic!("totalSupply() of an unknown collection should revert"),
        Err(err) => err,
    };
    assert!(
        format!("{err:?}").contains("Asset not found"),
        "unexpected error: {err:?}"
    );

    Ok(())
}

/// The NFT precompile rejects a fungible asset, and the ERC-20 precompile rejects a collection.
///
/// The two precompiles live at different addresses derived from the same asset id, so this
/// guards against reaching an asset through the wrong interface.
#[tokio::test]
#[test_log::test]
async fn erc721_rejects_fungible_asset() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester.users(&["Erc721WrongKind"]).await?;
    let api = tester.api.clone();
    let owner = &mut users[0];

    // A fungible asset reached through the NFT precompile.
    let (asset, _erc20) =
        create_erc20_asset(&api, &node, owner, "ERC721 Wrong Kind", 1_000).await?;
    let as_nft = Erc721Collection::new(&api, &node, asset.asset_id).await?;
    let err = match as_nft.total_supply().await {
        Ok(_) => panic!("the NFT precompile should reject a fungible asset"),
        Err(err) => err,
    };
    assert!(
        format!("{err:?}").contains("Asset is not non-fungible"),
        "unexpected error: {err:?}"
    );

    // An NFT collection reached through the ERC-20 precompile.
    let nft = create_erc721_collection(&api, &node, owner, "ERC721 Wrong Kind 2").await?;
    let as_erc20 = Erc20Asset::new(&api, &node, nft.asset_id).await?;
    let err = match as_erc20.total_supply().await {
        Ok(_) => panic!("the ERC-20 precompile should reject an NFT collection"),
        Err(err) => err,
    };
    assert!(
        format!("{err:?}").contains("Asset is not fungible"),
        "unexpected error: {err:?}"
    );

    Ok(())
}

/// `tokenURI` resolves the per-NFT `tokenUri`, falls back to the collection `baseTokenUri`, and
/// substitutes `{tokenId}`.
///
/// This also pins down the runtime's `TokenUriMetadataKey` / `BaseTokenUriMetadataKey`
/// constants: they must match the global keys the chain actually registered at genesis, so the
/// test looks the keys up by name rather than hardcoding them.
#[tokio::test]
#[test_log::test]
async fn erc721_token_uri() -> Result<()> {
    use polymesh_api::types::polymesh_primitives::asset_metadata::{
        AssetMetadataKey, AssetMetadataName, AssetMetadataValue,
    };

    let (mut tester, node) = revive_tester().await?;
    let mut users = tester.users(&["Erc721TokenUri"]).await?;
    let api = tester.api.clone();
    let owner = &mut users[0];

    // Resolve the global metadata keys registered at genesis from `src/data/asset_metadata.json`.
    let global_key = |name: &str| {
        let api = api.clone();
        let name = AssetMetadataName(name.as_bytes().to_vec());
        async move {
            api.query()
                .asset()
                .asset_metadata_global_name_to_key(name)
                .await?
                .ok_or_else(|| anyhow::anyhow!("global metadata key not registered"))
        }
    };
    let token_uri_key = global_key("tokenUri").await?;
    let base_token_uri_key = global_key("baseTokenUri").await?;

    // These are the values wired into `pallet_precompiles::Config` for every runtime.
    assert_eq!(token_uri_key.0, 3, "tokenUri global key changed");
    assert_eq!(base_token_uri_key.0, 1, "baseTokenUri global key changed");

    let nft = create_erc721_collection(&api, &node, owner, "ERC721 TokenUri").await?;
    let mut caller = SubstrateCaller::new(&api, owner).await?;
    nft.mint(&mut caller, vec![]).await?;

    // Nothing set yet.
    assert_eq!(nft.token_uri(1).await?, "");

    // Collection-level base URI, with a placeholder.
    api.call()
        .asset()
        .set_asset_metadata(
            nft.asset_id,
            AssetMetadataKey::Global(base_token_uri_key.clone()),
            AssetMetadataValue(b"https://example.com/nft/{tokenId}.json".to_vec()),
            None,
        )?
        .execute(owner)
        .await?
        .ok()
        .await?;

    assert_eq!(
        nft.token_uri(1).await?,
        "https://example.com/nft/1.json",
        "base URI placeholder should be substituted"
    );

    // A base URI without a placeholder gets the id appended.
    api.call()
        .asset()
        .set_asset_metadata(
            nft.asset_id,
            AssetMetadataKey::Global(base_token_uri_key),
            AssetMetadataValue(b"https://example.com/base/".to_vec()),
            None,
        )?
        .execute(owner)
        .await?
        .ok()
        .await?;

    assert_eq!(
        nft.token_uri(1).await?,
        "https://example.com/base/1",
        "token id should be appended when there is no placeholder"
    );

    Ok(())
}
