#![recursion_limit = "256"]
// Shared encryption keys (`distribute_encryption_key`) and their use as auditor/mediator keys.
#[cfg(feature = "current_release")]
mod confidential_key_distribution_tests {
    use std::collections::{BTreeMap, BTreeSet};

    use anyhow::Result;
    use integration::confidential_assets_helper::*;
    use polymesh_api::types::pallet_confidential_assets::RoleKind;
    use polymesh_dart::{AccountPublicKey, EncryptionPublicKey};

    const MISSING_CLAIM: &str = "does not hold the required claim";

    fn new_shared_key() -> EncryptionKeyPair {
        create_keys().enc.clone()
    }

    /// Register each user's encryption key and return the public keys.
    async fn register_recipients(recipients: &[&DartUser]) -> Result<Vec<EncryptionPublicKey>> {
        let mut keys = Vec::with_capacity(recipients.len());
        for recipient in recipients {
            recipient.register_encryption_key().await?;
            keys.push(recipient.public_keys().await.enc);
        }
        Ok(keys)
    }

    /// Create an asset directly with the given keys (no auto-onboarding).
    async fn create_asset_with_keys(
        issuer: &DartUser,
        name: &str,
        mediators: BTreeMap<AccountPublicKey, EncryptionPublicKey>,
        auditors: BTreeSet<EncryptionPublicKey>,
    ) -> Result<DartAssetId> {
        issuer.register_account().await?;
        issuer
            .create_asset(name, "TST", 0, "Shared key asset", mediators, auditors)
            .await
    }

    /// The asset issuer shares a new key with two auditors; the key is registered to the
    /// issuer and can be used as an asset auditor key.
    #[tokio::test]
    #[test_log::test]
    async fn issuer_distributes_shared_auditor_key() -> Result<()> {
        let tester = DartAssetTester::init(&["KdIssuer1", "KdAuditor1a", "KdAuditor1b"]).await?;
        let issuer = tester.user("KdIssuer1").await;
        let auditor_a = tester.user("KdAuditor1a").await;
        let auditor_b = tester.user("KdAuditor1b").await;
        tester
            .onboard_confidential_roles(&[&issuer], &[&issuer, &auditor_a, &auditor_b], &[])
            .await?;

        let recipients = register_recipients(&[&auditor_a, &auditor_b]).await?;
        let shared_key = new_shared_key();
        issuer
            .distribute_encryption_key(&shared_key, recipients.clone())
            .await?;

        assert_eq!(
            issuer.query_encryption_did(&shared_key.public).await?,
            Some(issuer.did().await),
            "shared key should be registered to the issuer"
        );
        assert_eq!(
            issuer
                .query_shared_key_recipients(&shared_key.public)
                .await?,
            recipients.into_iter().collect::<BTreeSet<_>>(),
        );

        create_asset_with_keys(
            &issuer,
            "Kd Asset 1",
            BTreeMap::new(),
            [shared_key.public].into(),
        )
        .await?;

        Ok(())
    }

    /// A DID with only the auditor claim can also distribute a shared key.
    #[tokio::test]
    #[test_log::test]
    async fn auditor_can_distribute_shared_key() -> Result<()> {
        let tester = DartAssetTester::init(&["KdDistributor2", "KdAuditor2"]).await?;
        let distributor = tester.user("KdDistributor2").await;
        let auditor = tester.user("KdAuditor2").await;
        tester
            .onboard_confidential_roles(&[], &[&distributor, &auditor], &[])
            .await?;

        let recipients = register_recipients(&[&auditor]).await?;
        distributor
            .distribute_encryption_key(&new_shared_key(), recipients)
            .await?;

        Ok(())
    }

    /// A caller without the auditor claim can't distribute a key.
    #[tokio::test]
    #[test_log::test]
    async fn distribute_without_claim_fails() -> Result<()> {
        let tester = DartAssetTester::init(&["KdNoClaim3", "KdAuditor3", "KdMediator3"]).await?;
        let caller = tester.user("KdNoClaim3").await;
        let auditor = tester.user("KdAuditor3").await;
        let mediator = tester.user("KdMediator3").await;
        // The mediator claim doesn't allow distributing keys.
        tester
            .onboard_confidential_roles(&[], &[&auditor], &[&caller, &mediator])
            .await?;

        let recipients = register_recipients(&[&auditor]).await?;
        let res = caller
            .distribute_encryption_key(&new_shared_key(), recipients)
            .await;
        assert_operation_fails_with(res, "distribute without claim", MISSING_CLAIM);

        Ok(())
    }

    /// Every recipient must hold the auditor claim.
    #[tokio::test]
    #[test_log::test]
    async fn distribute_to_claimless_recipient_fails() -> Result<()> {
        let tester =
            DartAssetTester::init(&["KdIssuer4", "KdAuditor4", "KdNoClaim4", "KdMediator4"])
                .await?;
        let issuer = tester.user("KdIssuer4").await;
        let auditor = tester.user("KdAuditor4").await;
        let no_claim = tester.user("KdNoClaim4").await;
        let mediator = tester.user("KdMediator4").await;
        tester
            .onboard_confidential_roles(&[&issuer], &[&issuer, &auditor], &[&mediator])
            .await?;

        // Claimless recipient.
        let recipients = register_recipients(&[&auditor, &no_claim]).await?;
        let res = issuer
            .distribute_encryption_key(&new_shared_key(), recipients)
            .await;
        assert_operation_fails_with(res, "distribute to claimless recipient", MISSING_CLAIM);

        // Recipient holding only the mediator claim.
        mediator.register_account().await?;
        let recipients = vec![
            auditor.public_keys().await.enc,
            mediator.public_keys().await.enc,
        ];
        let res = issuer
            .distribute_encryption_key(&new_shared_key(), recipients)
            .await;
        assert_operation_fails_with(res, "distribute to mediator-only recipient", MISSING_CLAIM);

        Ok(())
    }

    /// Every recipient key must be registered.
    #[tokio::test]
    #[test_log::test]
    async fn distribute_to_unregistered_recipient_fails() -> Result<()> {
        let tester = DartAssetTester::init(&["KdIssuer5"]).await?;
        let issuer = tester.user("KdIssuer5").await;
        tester
            .onboard_confidential_roles(&[&issuer], &[&issuer], &[])
            .await?;

        let res = issuer
            .distribute_encryption_key(&new_shared_key(), vec![new_shared_key().public])
            .await;
        assert_operation_fails_with(
            res,
            "distribute to unregistered recipient",
            "Encryption key for the Confidential account is missing",
        );

        Ok(())
    }

    /// A shared key can only be registered (distributed) once, and an already registered
    /// encryption key can't be distributed.
    #[tokio::test]
    #[test_log::test]
    async fn distribute_registered_key_fails() -> Result<()> {
        let tester = DartAssetTester::init(&["KdIssuer6", "KdAuditor6a", "KdAuditor6b"]).await?;
        let issuer = tester.user("KdIssuer6").await;
        let auditor_a = tester.user("KdAuditor6a").await;
        let auditor_b = tester.user("KdAuditor6b").await;
        tester
            .onboard_confidential_roles(&[&issuer], &[&issuer, &auditor_a, &auditor_b], &[])
            .await?;
        let recipients = register_recipients(&[&auditor_a, &auditor_b]).await?;

        let shared_key = new_shared_key();
        issuer
            .distribute_encryption_key(&shared_key, vec![recipients[0]])
            .await?;
        let res = issuer
            .distribute_encryption_key(&shared_key, vec![recipients[1]])
            .await;
        assert_operation_fails_with(
            res,
            "distribute an already shared key",
            "Encryption key already registered",
        );

        // The issuer's own (registered) account encryption key.
        issuer.register_account().await?;
        let own_key = issuer.keys().await.enc.clone();
        let res = issuer
            .distribute_encryption_key(&own_key, vec![recipients[1]])
            .await;
        assert_operation_fails_with(
            res,
            "distribute a registered account key",
            "Encryption key already registered",
        );

        Ok(())
    }

    /// Recipient claims are checked when sharing; subsequent auditor checks use the shared
    /// key owner's auditor claim like any other registered encryption key.
    #[tokio::test]
    #[test_log::test]
    async fn shared_auditor_key_uses_owner_claim_after_recipient_revocation() -> Result<()> {
        let tester = DartAssetTester::init(&["KdIssuer7", "KdAuditor7a", "KdAuditor7b"]).await?;
        let issuer = tester.user("KdIssuer7").await;
        let auditor_a = tester.user("KdAuditor7a").await;
        let auditor_b = tester.user("KdAuditor7b").await;
        tester
            .onboard_confidential_roles(&[&issuer], &[&issuer, &auditor_a, &auditor_b], &[])
            .await?;

        let recipients = register_recipients(&[&auditor_a, &auditor_b]).await?;
        let shared_key = new_shared_key();
        issuer
            .distribute_encryption_key(&shared_key, recipients)
            .await?;

        tester
            .revoke_confidential_role(&auditor_b, RoleKind::Auditor)
            .await?;

        create_asset_with_keys(
            &issuer,
            "Kd Asset 7",
            BTreeMap::new(),
            [shared_key.public].into(),
        )
        .await?;

        Ok(())
    }

    /// The shared key owner must still hold the asset creator or auditor claim when the
    /// key is used as an auditor key.
    #[tokio::test]
    #[test_log::test]
    async fn shared_auditor_key_rechecks_owner_claim() -> Result<()> {
        let tester = DartAssetTester::init(&["KdIssuer8", "KdDistributor8", "KdAuditor8"]).await?;
        let issuer = tester.user("KdIssuer8").await;
        let distributor = tester.user("KdDistributor8").await;
        let auditor = tester.user("KdAuditor8").await;
        tester
            .onboard_confidential_roles(&[&issuer], &[&issuer, &distributor, &auditor], &[])
            .await?;

        let recipients = register_recipients(&[&auditor]).await?;
        let shared_key = new_shared_key();
        distributor
            .distribute_encryption_key(&shared_key, recipients)
            .await?;

        tester
            .revoke_confidential_role(&distributor, RoleKind::Auditor)
            .await?;

        let res = create_asset_with_keys(
            &issuer,
            "Kd Asset 8",
            BTreeMap::new(),
            [shared_key.public].into(),
        )
        .await;
        assert_operation_fails_with(res, "shared auditor key with revoked owner", MISSING_CLAIM);

        Ok(())
    }

    /// A mediator can use a shared key that was distributed to their account's encryption key.
    #[tokio::test]
    #[test_log::test]
    async fn mediator_uses_distributed_shared_key() -> Result<()> {
        let tester = DartAssetTester::init(&["KdIssuer9", "KdMediator9"]).await?;
        let issuer = tester.user("KdIssuer9").await;
        let mediator = tester.user("KdMediator9").await;
        // Recipients need the auditor claim; mediators need the mediator claim.
        tester
            .onboard_confidential_roles(&[&issuer], &[&issuer, &mediator], &[&mediator])
            .await?;

        mediator.register_account().await?;
        let med_keys = mediator.public_keys().await;
        let shared_key = new_shared_key();
        issuer
            .distribute_encryption_key(&shared_key, vec![med_keys.enc])
            .await?;

        create_asset_with_keys(
            &issuer,
            "Kd Asset 9",
            [(med_keys.acct, shared_key.public)].into(),
            BTreeSet::new(),
        )
        .await?;

        Ok(())
    }

    /// A mediator can't use a shared key that wasn't distributed to their encryption key.
    #[tokio::test]
    #[test_log::test]
    async fn mediator_with_undistributed_shared_key_fails() -> Result<()> {
        let tester = DartAssetTester::init(&["KdIssuer10", "KdMediator10", "KdAuditor10"]).await?;
        let issuer = tester.user("KdIssuer10").await;
        let mediator = tester.user("KdMediator10").await;
        let auditor = tester.user("KdAuditor10").await;
        tester
            .onboard_confidential_roles(&[&issuer], &[&issuer, &mediator, &auditor], &[&mediator])
            .await?;

        mediator.register_account().await?;
        let recipients = register_recipients(&[&auditor]).await?;
        let shared_key = new_shared_key();
        issuer
            .distribute_encryption_key(&shared_key, recipients)
            .await?;

        let res = create_asset_with_keys(
            &issuer,
            "Kd Asset 10",
            [(mediator.public_keys().await.acct, shared_key.public)].into(),
            BTreeSet::new(),
        )
        .await;
        assert_operation_fails_with(
            res,
            "mediator with undistributed shared key",
            "hasn't been distributed to the mediator",
        );

        Ok(())
    }

    /// A mediator using a shared key must hold the mediator claim.
    #[tokio::test]
    #[test_log::test]
    async fn mediator_shared_key_without_mediator_claim_fails() -> Result<()> {
        let tester = DartAssetTester::init(&["KdIssuer11", "KdMediator11"]).await?;
        let issuer = tester.user("KdIssuer11").await;
        let mediator = tester.user("KdMediator11").await;
        // Only the auditor claim (enough to receive the key, not to mediate).
        tester
            .onboard_confidential_roles(&[&issuer], &[&issuer, &mediator], &[])
            .await?;

        mediator.register_account().await?;
        let med_keys = mediator.public_keys().await;
        let shared_key = new_shared_key();
        issuer
            .distribute_encryption_key(&shared_key, vec![med_keys.enc])
            .await?;

        let res = create_asset_with_keys(
            &issuer,
            "Kd Asset 11",
            [(med_keys.acct, shared_key.public)].into(),
            BTreeSet::new(),
        )
        .await;
        assert_operation_fails_with(
            res,
            "shared mediator key without mediator claim",
            MISSING_CLAIM,
        );

        Ok(())
    }
}
