#![recursion_limit = "256"]
// Owner-controlled confidential asset auditor/mediator key updates.
#[cfg(feature = "current_release")]
mod confidential_asset_key_update_tests {
    use std::collections::{BTreeMap, BTreeSet};

    use anyhow::Result;
    use integration::confidential_assets_helper::*;
    use polymesh_dart::{LegBuilder, LegConfig, LegRef, SettlementBuilder};

    /// Updating keys retains historical asset roots and the mediator snapshot for existing legs.
    #[tokio::test]
    #[test_log::test]
    async fn update_keys_preserves_in_flight_settlement_snapshot() -> Result<()> {
        let tester = DartAssetTester::init(&[
            "KeyUpdateIssuer1",
            "KeyUpdateOldAuditor1",
            "KeyUpdateNewAuditor1",
            "KeyUpdateOldMediator1",
            "KeyUpdateNewMediator1",
            "KeyUpdateSender1",
            "KeyUpdateReceiver1",
            "KeyUpdateVenue1",
        ])
        .await?;
        let issuer = tester.user("KeyUpdateIssuer1").await;
        let old_auditor = tester.user("KeyUpdateOldAuditor1").await;
        let new_auditor = tester.user("KeyUpdateNewAuditor1").await;
        let old_mediator = tester.user("KeyUpdateOldMediator1").await;
        let new_mediator = tester.user("KeyUpdateNewMediator1").await;
        let sender = tester.user("KeyUpdateSender1").await;
        let receiver = tester.user("KeyUpdateReceiver1").await;
        let venue = tester.user("KeyUpdateVenue1").await;

        let asset = tester
            .create_asset(
                &issuer,
                "Key Update Asset 1",
                &[&old_mediator],
                &[&old_auditor],
                None,
            )
            .await?;

        let asset_tree = tester.asset_tree().await;
        let old_root_block = asset_tree.get_block_number().await?;
        let old_root = asset_tree.fetch_root(Some(old_root_block)).await?;
        let asset_state = asset.asset_state().await?;
        let asset_path = asset_tree
            .get_path_to_leaf(asset.id as _, 0, Some(old_root_block))
            .await?;
        let mut settlement =
            SettlementBuilder::<()>::new_root(b"Key update old root", old_root_block, old_root);
        settlement.add_path(asset.id, asset_path)?;
        settlement.add_leg(LegBuilder {
            sender: sender.public_keys().await,
            receiver: receiver.public_keys().await,
            asset: asset_state,
            amount: 1,
            config: LegConfig::default(),
            public_enc_keys: vec![],
        });
        let mut rng = rand::thread_rng();
        let proof = settlement.build(&mut rng)?;
        let settlement_ref = proof.settlement_ref();

        tester
            .onboard_confidential_roles(
                &[],
                &[&issuer, &new_auditor, &new_mediator],
                &[&new_mediator],
            )
            .await?;
        new_auditor.register_encryption_key().await?;
        new_mediator.register_account().await?;
        let mediator_keys = new_mediator.public_keys().await;
        let shared_key = create_keys().enc.clone();
        issuer
            .distribute_encryption_key(&shared_key, vec![mediator_keys.enc])
            .await?;
        issuer
            .update_asset_keys(
                asset.id,
                [(mediator_keys.acct, shared_key.public)].into(),
                [new_auditor.public_keys().await.enc].into(),
            )
            .await?;

        // This proof uses the pre-update asset root; it must remain accepted while that root is live.
        venue.create_settlement(proof).await?;
        let leg_ref = LegRef {
            settlement: settlement_ref,
            leg_id: 0,
        };
        old_mediator
            .mediator_affirmation(&tester, leg_ref, 0, true, Some((asset.id, 1)))
            .await?;

        Ok(())
    }

    /// Only the asset issuer can update keys, and updated auditor keys must pass role gating.
    #[tokio::test]
    #[test_log::test]
    async fn update_keys_requires_owner_and_auditor_claims() -> Result<()> {
        let tester = DartAssetTester::init(&[
            "KeyUpdateIssuer2",
            "KeyUpdateAuditor2",
            "KeyUpdateNoClaim2",
            "KeyUpdateOutsider2",
        ])
        .await?;
        let issuer = tester.user("KeyUpdateIssuer2").await;
        let auditor = tester.user("KeyUpdateAuditor2").await;
        let no_claim = tester.user("KeyUpdateNoClaim2").await;
        let outsider = tester.user("KeyUpdateOutsider2").await;
        let asset = tester
            .create_asset(&issuer, "Key Update Asset 2", &[], &[&auditor], None)
            .await?;

        let current_auditor_key = auditor.public_keys().await.enc;
        let not_owner = outsider
            .update_asset_keys(asset.id, BTreeMap::new(), [current_auditor_key].into())
            .await;
        assert_operation_fails_with(
            not_owner,
            "update keys as non-owner",
            "not the owner of the Confidential asset",
        );

        no_claim.register_encryption_key().await?;
        let claimless_update = issuer
            .update_asset_keys(
                asset.id,
                BTreeMap::new(),
                [no_claim.public_keys().await.enc].into(),
            )
            .await;
        assert_operation_fails_with(
            claimless_update,
            "update keys to claimless auditor",
            "does not hold the required claim",
        );

        outsider.register_account().await?;
        let outsider_keys = outsider.public_keys().await;
        let claimless_mediator_update = issuer
            .update_asset_keys(
                asset.id,
                [(outsider_keys.acct, outsider_keys.enc)].into(),
                [current_auditor_key].into(),
            )
            .await;
        assert_operation_fails_with(
            claimless_mediator_update,
            "update keys to claimless mediator",
            "does not hold the required claim",
        );

        Ok(())
    }

    /// Updated mediator keys must belong to the account or be shared with its registered key.
    #[tokio::test]
    #[test_log::test]
    async fn update_keys_rejects_undistributed_mediator_key() -> Result<()> {
        let tester = DartAssetTester::init(&[
            "KeyUpdateIssuer3",
            "KeyUpdateAuditor3",
            "KeyUpdateMediator3",
        ])
        .await?;
        let issuer = tester.user("KeyUpdateIssuer3").await;
        let auditor = tester.user("KeyUpdateAuditor3").await;
        let mediator = tester.user("KeyUpdateMediator3").await;
        let asset = tester
            .create_asset(&issuer, "Key Update Asset 3", &[], &[&auditor], None)
            .await?;

        mediator.register_account().await?;
        let mediator_account = mediator.public_keys().await.acct;
        let res = issuer
            .update_asset_keys(
                asset.id,
                [(mediator_account, auditor.public_keys().await.enc)].into(),
                BTreeSet::new(),
            )
            .await;
        assert_operation_fails_with(
            res,
            "update mediator to use an undistributed key",
            "hasn't been distributed to the mediator",
        );

        Ok(())
    }
}
