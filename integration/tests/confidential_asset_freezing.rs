#![recursion_limit = "256"]

#[cfg(feature = "current_release")]
mod confidential_asset_freezing_tests {
    use anyhow::{Context, Result};
    use codec::Encode;
    use integration::check_sudo_result;
    use integration::confidential_assets_helper::*;
    use polymesh_api::types::pallet_confidential_assets::FreezeOrigin;
    use polymesh_api_tester::PolymeshTester;
    use polymesh_dart::{
        InstantSettlementProof, LegBuilder, LegConfig, SettlementBuilder, SettlementProof,
    };

    async fn settlement_proof(
        tester: &DartAssetTester,
        asset: &DartTestAsset,
        sender: &DartUser,
        receiver: &DartUser,
        revealed: bool,
        memo: &[u8],
    ) -> Result<SettlementProof<()>> {
        let tree = tester.asset_tree().await;
        let block = tree.get_block_number().await?;
        let root = tree.fetch_root(Some(block)).await?;
        let mut builder = SettlementBuilder::<()>::new_root(memo, block, root);
        if !revealed {
            builder.add_path(
                asset.id,
                tree.get_path_to_leaf(asset.id as _, 0, Some(block)).await?,
            )?;
        }
        builder.add_leg(LegBuilder {
            sender: sender.public_keys().await,
            receiver: receiver.public_keys().await,
            asset: asset.asset_state().await?,
            amount: 1,
            config: LegConfig {
                reveal_asset_id: revealed,
                ..Default::default()
            },
            public_enc_keys: vec![],
        });
        Ok(builder.build(&mut rand::thread_rng())?)
    }

    async fn assert_leaf_event(
        result: &mut TransactionResults,
        asset: &DartTestAsset,
        frozen: bool,
    ) -> Result<()> {
        let events = result
            .events()
            .await?
            .context("Freeze transaction events")?;
        let mut state = asset.asset_state().await?;
        state.frozen = frozen;
        let expected = state.commitment()?.encode();
        let leaves = events
            .0
            .iter()
            .filter_map(|record| match &record.event {
                RuntimeEvent::ConfidentialAssets(
                    ConfidentialAssetsEvent::AssetStateLeafUpdated {
                        leaf_index,
                        asset_leaf,
                    },
                ) if *leaf_index == u64::from(asset.id) => Some(asset_leaf.encode()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(leaves, vec![expected]);
        Ok(())
    }

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

        let mut result = issuer.set_asset_frozen(asset.id, true).await?;
        assert_leaf_event(&mut result, &asset, true).await?;
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

        let mut result = issuer.set_asset_frozen(asset.id, false).await?;
        assert_leaf_event(&mut result, &asset, false).await?;
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

    #[tokio::test]
    #[test_log::test]
    async fn key_updates_preserve_root_freeze() -> Result<()> {
        let tester =
            DartAssetTester::init(&["FreezeIssuer2", "FreezeAuditor2", "FreezeNewAuditor2"])
                .await?;
        let issuer = tester.user("FreezeIssuer2").await;
        let auditor = tester.user("FreezeAuditor2").await;
        let new_auditor = tester.user("FreezeNewAuditor2").await;
        let asset = tester
            .create_asset(&issuer, "Freeze Key Update Asset", &[], &[&auditor], None)
            .await?;
        let mut admin = PolymeshTester::new().await?;
        set_frozen_as_root(&mut admin, asset.id, true).await?;
        tester
            .onboard_confidential_roles(&[], &[&new_auditor], &[])
            .await?;
        new_auditor.register_encryption_key().await?;
        let enc_key = new_auditor.public_keys().await.enc;
        issuer
            .update_asset_keys(asset.id, Default::default(), [enc_key].into())
            .await?;
        let api = tester.api().await;
        assert!(matches!(
            api.query()
                .confidential_assets()
                .asset_frozen(asset.id)
                .await?,
            Some(FreezeOrigin::Root)
        ));
        let mut expected = polymesh_dart::AssetState::new::<()>(asset.id, &[], &[enc_key])?;
        expected.frozen = true;
        let leaf = api
            .query()
            .confidential_assets()
            .asset_leaves(u64::from(asset.id))
            .await?
            .unwrap();
        assert_eq!(leaf.encode(), expected.commitment()?.encode());
        assert_operation_fails_with(
            issuer.set_asset_frozen(asset.id, false).await,
            "unfreeze after key repair",
            "Only Root may change a Root freeze",
        );
        set_frozen_as_root(&mut admin, asset.id, false).await?;
        expected.frozen = false;
        let leaf = api
            .query()
            .confidential_assets()
            .asset_leaves(u64::from(asset.id))
            .await?
            .unwrap();
        assert_eq!(leaf.encode(), expected.commitment()?.encode());
        Ok(())
    }

    #[tokio::test]
    #[test_log::test]
    async fn revealed_settlement_rejected_while_frozen_and_restored_on_unfreeze() -> Result<()> {
        let tester = DartAssetTester::init(&[
            "FreezeIssuer3",
            "FreezeAuditor3",
            "FreezeReceiver3",
            "FreezeVenue3",
        ])
        .await?;
        let issuer = tester.user("FreezeIssuer3").await;
        let auditor = tester.user("FreezeAuditor3").await;
        let receiver = tester.user("FreezeReceiver3").await;
        let venue = tester.user("FreezeVenue3").await;
        let asset = tester
            .create_asset(&issuer, "Freeze Revealed Asset", &[], &[&auditor], None)
            .await?;
        let proof = settlement_proof(
            &tester,
            &asset,
            &issuer,
            &receiver,
            true,
            b"Freeze revealed settlement",
        )
        .await?;
        issuer.set_asset_frozen(asset.id, true).await?;
        assert_operation_fails_with(
            venue.create_settlement(proof.clone()).await,
            "revealed settlement while frozen",
            "A frozen asset cannot be used in a new settlement",
        );
        assert_operation_fails_with(
            venue
                .execute_instant_settlement(InstantSettlementProof {
                    settlement: proof.clone(),
                    leg_affirmations: Default::default(),
                })
                .await,
            "instant revealed settlement while frozen",
            "A frozen asset cannot be used in a new settlement",
        );
        let api = tester.api().await;
        assert!(api
            .query()
            .confidential_assets()
            .settlement_state(to_scale(&proof.settlement_ref()))
            .await?
            .is_none());
        issuer.set_asset_frozen(asset.id, false).await?;
        assert_eq!(
            venue.create_settlement(proof.clone()).await?,
            proof.settlement_ref()
        );
        Ok(())
    }

    #[tokio::test]
    #[test_log::test]
    async fn hidden_pre_freeze_root_remains_usable_while_live() -> Result<()> {
        let tester = DartAssetTester::init(&[
            "FreezeIssuer5",
            "FreezeAuditor5",
            "FreezeReceiver5",
            "FreezeVenue5",
        ])
        .await?;
        let issuer = tester.user("FreezeIssuer5").await;
        let auditor = tester.user("FreezeAuditor5").await;
        let receiver = tester.user("FreezeReceiver5").await;
        let venue = tester.user("FreezeVenue5").await;
        let asset = tester
            .create_asset(
                &issuer,
                "Freeze Historical Root Asset",
                &[],
                &[&auditor],
                None,
            )
            .await?;
        let proof = settlement_proof(
            &tester,
            &asset,
            &issuer,
            &receiver,
            false,
            b"Freeze historical root",
        )
        .await?;
        issuer.set_asset_frozen(asset.id, true).await?;
        assert_eq!(
            venue.create_settlement(proof.clone()).await?,
            proof.settlement_ref()
        );
        let api = tester.api().await;
        assert!(matches!(
            api.query()
                .confidential_assets()
                .asset_frozen(asset.id)
                .await?,
            Some(FreezeOrigin::Issuer),
        ));
        Ok(())
    }

    #[tokio::test]
    #[test_log::test]
    async fn in_flight_settlement_completes_after_freeze() -> Result<()> {
        let tester = DartAssetTester::init(&[
            "FreezeIssuer4",
            "FreezeAuditor4",
            "FreezeMediator4",
            "FreezeReceiver4",
            "FreezeVenue4",
        ])
        .await?;
        let issuer = tester.user("FreezeIssuer4").await;
        let auditor = tester.user("FreezeAuditor4").await;
        let mediator = tester.user("FreezeMediator4").await;
        let receiver = tester.user("FreezeReceiver4").await;
        let venue = tester.user("FreezeVenue4").await;
        let asset = tester
            .create_asset(
                &issuer,
                "Freeze In Flight Asset",
                &[&mediator],
                &[&auditor],
                Some(10),
            )
            .await?;
        receiver.register_account().await?;
        receiver.register_account_asset(asset.id).await?;
        let settlement = DartSettlementState::new(
            &tester,
            &venue,
            &[DartLeg {
                sender: issuer.clone(),
                receiver: receiver.clone(),
                asset_id: asset.id,
                amount: 1,
                config: LegConfig::default(),
            }],
            Some(b"Freeze in flight"),
        )
        .await?;
        issuer.set_asset_frozen(asset.id, true).await?;
        settlement.senders_affirm_legs(&tester).await?;
        settlement.receivers_affirm_legs(&tester).await?;
        settlement.mediators_affirm_legs(&tester, true).await?;
        for leg in &settlement.legs {
            leg.sender
                .sender_counter_update(&tester, leg.leg_ref)
                .await?;
        }
        settlement.receivers_claim_assets(&tester).await?;
        let api = tester.api().await;
        let status = api
            .query()
            .confidential_assets()
            .settlement_state(to_scale(&settlement.settlement_ref))
            .await?;
        assert!(matches!(status, Some(polymesh_api::types::pallet_confidential_assets::settlement::SettlementStatus::Finalized)));
        assert!(matches!(
            api.query()
                .confidential_assets()
                .asset_frozen(asset.id)
                .await?,
            Some(FreezeOrigin::Issuer)
        ));
        Ok(())
    }
}
