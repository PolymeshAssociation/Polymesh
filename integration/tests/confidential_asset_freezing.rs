#![recursion_limit = "256"]

#[cfg(feature = "current_release")]
mod confidential_asset_freezing_tests {
    use anyhow::{Context, Result};
    use codec::Encode;
    use integration::check_sudo_result;
    use integration::confidential_assets_helper::*;
    use polymesh_api::types::pallet_confidential_assets::FreezeOrigin;
    use polymesh_api_tester::PolymeshTester;

    async fn set_frozen_as_root(
        tester: &mut PolymeshTester,
        asset_id: u32,
        frozen: bool,
    ) -> Result<()> {
        let call = tester
            .api
            .call()
            .confidential_assets()
            .set_asset_frozen(asset_id, frozen)?;
        let sudo = tester.sudo.as_mut().context("Freeze tests require sudo")?;
        let mut result = tester
            .api
            .call()
            .sudo()
            .sudo(call.runtime_call().clone())?
            .submit_and_watch(sudo)
            .await?;
        check_sudo_result(&mut result).await?;
        Ok(())
    }

    #[tokio::test]
    #[test_log::test]
    async fn freeze_authority_and_leaf_commitments() -> Result<()> {
        let names = ["FreezeIssuer1", "FreezeAuditor1", "FreezeOutsider1"];
        let tester = DartAssetTester::init(&names).await?;
        let issuer = tester.user(names[0]).await;
        let auditor = tester.user(names[1]).await;
        let outsider = tester.user(names[2]).await;
        let asset = tester
            .create_asset(&issuer, "Freeze Authority Asset", &[], &[&auditor], None)
            .await?;
        let mut admin = PolymeshTester::new().await?;
        let api = tester.api().await;
        let query = api.query().confidential_assets();
        let original_leaf = query
            .asset_leaves(u64::from(asset.id))
            .await?
            .context("Asset leaf")?;
        assert!(query.asset_frozen(asset.id).await?.is_none());

        for frozen in [true, false] {
            let result = outsider.set_asset_frozen(asset.id, frozen).await;
            assert_operation_fails_with(
                result,
                "freeze state as non-owner",
                "not the owner of the Confidential asset",
            );
        }

        issuer.set_asset_frozen(asset.id, true).await?;
        assert!(matches!(
            query.asset_frozen(asset.id).await?,
            Some(FreezeOrigin::Issuer)
        ));
        let frozen_leaf = query
            .asset_leaves(u64::from(asset.id))
            .await?
            .context("Frozen leaf")?;
        assert_ne!(original_leaf.encode(), frozen_leaf.encode());
        let mut state = asset.asset_state().await?;
        state.frozen = true;
        assert_eq!(frozen_leaf.encode(), state.commitment()?.encode());

        issuer.set_asset_frozen(asset.id, false).await?;
        assert!(query.asset_frozen(asset.id).await?.is_none());
        assert_eq!(
            query
                .asset_leaves(u64::from(asset.id))
                .await?
                .unwrap()
                .encode(),
            original_leaf.encode()
        );

        issuer.set_asset_frozen(asset.id, true).await?;
        set_frozen_as_root(&mut admin, asset.id, true).await?;
        assert!(matches!(
            query.asset_frozen(asset.id).await?,
            Some(FreezeOrigin::Root)
        ));
        for frozen in [false, true] {
            let result = issuer.set_asset_frozen(asset.id, frozen).await;
            assert_operation_fails_with(
                result,
                "issuer modifying Root freeze",
                "Only Root may change a Root freeze",
            );
            assert!(matches!(
                query.asset_frozen(asset.id).await?,
                Some(FreezeOrigin::Root)
            ));
            assert_eq!(
                query
                    .asset_leaves(u64::from(asset.id))
                    .await?
                    .unwrap()
                    .encode(),
                frozen_leaf.encode()
            );
        }
        set_frozen_as_root(&mut admin, asset.id, false).await?;
        assert!(query.asset_frozen(asset.id).await?.is_none());
        assert_eq!(
            query
                .asset_leaves(u64::from(asset.id))
                .await?
                .unwrap()
                .encode(),
            original_leaf.encode()
        );
        Ok(())
    }
}
