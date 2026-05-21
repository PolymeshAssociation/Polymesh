#[cfg(feature = "current_release")]
mod claim_management_tests {
    use std::collections::BTreeSet;

    use anyhow::Result;

    use integration::*;
    use polymesh_api::types::polymesh_primitives::identity_claim::{Claim, Scope};

    /// Test adding various types of claims to an identity.
    ///
    /// Ported from `05_claim_management.ts`.
    #[tokio::test]
    #[test_log::test]
    async fn claim_management() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["ClaimIssuer1", "ClaimIssuer2", "ClaimTarget"])
            .await?
            .into_iter();
        let mut issuer1 = users.next().expect("ClaimIssuer1");
        let mut issuer2 = users.next().expect("ClaimIssuer2");
        let target = users.next().expect("ClaimTarget");

        let target_did = target.did.expect("ClaimTarget DID");
        let issuer2_did = issuer2.did.expect("ClaimIssuer2 DID");

        // Create an asset owned by issuer1.
        let asset_helper = AssetHelper::new(
            &tester.api,
            &mut issuer1,
            "ClaimTestAsset",
            1_000_000,
            BTreeSet::new(),
        )
        .await?;
        let asset_id = asset_helper.asset_id;

        // issuer1 adds an Exempted claim for target (scoped to issuer2's identity).
        tester
            .api
            .call()
            .identity()
            .add_claim(
                target_did,
                Claim::Exempted(Scope::Identity(issuer2_did)),
                None,
            )?
            .submit_and_watch(&mut issuer1)
            .await?
            .ok()
            .await?;

        // issuer1 adds a SellLockup claim for target (scoped to the asset).
        tester
            .api
            .call()
            .identity()
            .add_claim(target_did, Claim::SellLockup(Scope::Asset(asset_id)), None)?
            .submit_and_watch(&mut issuer1)
            .await?
            .ok()
            .await?;

        // issuer2 adds an Accredited claim for target (scoped to the asset).
        tester
            .api
            .call()
            .identity()
            .add_claim(target_did, Claim::Accredited(Scope::Asset(asset_id)), None)?
            .submit_and_watch(&mut issuer2)
            .await?
            .ok()
            .await?;

        // issuer2 adds an Affiliate claim for target with an expiry (scoped to the asset).
        let now_ms = tester.api.query().timestamp().now().await?;
        let expiry_ms = now_ms + 60 * 60 * 1000; // 1 hour from now.
        tester
            .api
            .call()
            .identity()
            .add_claim(
                target_did,
                Claim::Affiliate(Scope::Asset(asset_id)),
                Some(expiry_ms),
            )?
            .submit_and_watch(&mut issuer2)
            .await?
            .ok()
            .await?;

        Ok(())
    }
}
