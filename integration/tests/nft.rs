//! NFT collection, issue, transfer, redeem.
#[cfg(feature = "current_release")]
mod nft_tests {
    use anyhow::Result;

    use integration::*;
    use polymesh_api::types::polymesh_primitives::{
        asset::{AssetName, AssetType, NonFungibleType},
        asset_metadata::{
            AssetMetadataKey, AssetMetadataLocalKey, AssetMetadataName, AssetMetadataSpec,
            AssetMetadataValue,
        },
        nft::{NFTCollectionKeys, NFTId, NFTMetadataAttribute, NFTs},
    };

    async fn create_nft_asset(
        tester: &PolymeshTester,
        owner: &mut User,
        name: &str,
    ) -> Result<AssetId> {
        let mut res = tester
            .api
            .call()
            .asset()
            .create_asset(
                AssetName(name.as_bytes().to_vec()),
                false,
                AssetType::NonFungible(NonFungibleType::Derivative),
                vec![],
                None,
            )?
            .submit_and_watch(owner)
            .await?;
        res.ok().await?;
        get_asset_id(&mut res)
            .await?
            .ok_or_else(|| anyhow::anyhow!("AssetCreated event missing"))
    }

    async fn register_local_key(
        tester: &PolymeshTester,
        owner: &mut User,
        asset_id: AssetId,
        name: &str,
    ) -> Result<()> {
        tester
            .api
            .call()
            .asset()
            .register_asset_metadata_local_type(
                asset_id,
                AssetMetadataName(name.as_bytes().to_vec()),
                AssetMetadataSpec {
                    url: None,
                    description: None,
                    type_def: None,
                },
            )?
            .submit_and_watch(owner)
            .await?
            .ok()
            .await?;
        Ok(())
    }

    /// Create a collection, issue an NFT, transfer it, then redeem it.
    #[tokio::test]
    #[test_log::test]
    async fn collection_issue_transfer_redeem() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["NftIssuer", "NftHolder"]).await?.into_iter();
        let mut issuer = users.next().unwrap();
        let holder = users.next().unwrap();

        let asset_id = create_nft_asset(&tester, &mut issuer, "NftCol").await?;
        register_local_key(&tester, &mut issuer, asset_id.clone(), "image").await?;

        let keys = NFTCollectionKeys(vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))]);
        tester
            .api
            .call()
            .nft()
            .create_nft_collection(Some(asset_id.clone()), None, keys)?
            .submit_and_watch(&mut issuer)
            .await?
            .ok()
            .await?;

        use polymesh_api::types::polymesh_primitives::asset::{AssetHolder, AssetHolderKind};

        tester
            .api
            .call()
            .nft()
            .issue_nft(
                asset_id.clone(),
                vec![NFTMetadataAttribute {
                    key: AssetMetadataKey::Local(AssetMetadataLocalKey(1)),
                    value: AssetMetadataValue(b"ipfs://img".to_vec()),
                }],
                AssetHolderKind::Account,
            )?
            .submit_and_watch(&mut issuer)
            .await?
            .ok()
            .await?;

        let nfts = NFTs {
            asset_id: asset_id.clone(),
            ids: vec![NFTId(1)],
        };
        tester
            .api
            .call()
            .nft()
            .transfer_nft(nfts.clone(), holder.account(), None)?
            .submit_and_watch(&mut issuer)
            .await?
            .ok()
            .await?;

        tester
            .api
            .call()
            .nft()
            .controller_transfer(
                nfts,
                AssetHolder::Account(holder.account()),
                AssetHolderKind::Account,
            )?
            .submit_and_watch(&mut issuer)
            .await?
            .ok()
            .await?;

        tester
            .api
            .call()
            .nft()
            .redeem_nft(asset_id, NFTId(1), AssetHolderKind::Account, None)?
            .submit_and_watch(&mut issuer)
            .await?
            .ok()
            .await?;

        Ok(())
    }
}
