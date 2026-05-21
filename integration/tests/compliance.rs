#[cfg(feature = "current_release")]
mod compliance_tests {
    use std::collections::BTreeSet;

    use anyhow::Result;

    use integration::*;
    use polymesh_api::types::polymesh_primitives::{
        condition::{Condition, ConditionType, TrustedFor, TrustedIssuer},
        identity_claim::{Claim, Scope},
    };

    /// Test adding and removing compliance requirements for an asset.
    ///
    /// Ported from `06_compliance.ts`.
    #[tokio::test]
    #[test_log::test]
    async fn compliance() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["ComplianceOwner", "ComplianceIssuer"])
            .await?
            .into_iter();
        let mut owner = users.next().expect("ComplianceOwner");
        let issuer = users.next().expect("ComplianceIssuer");

        let issuer_did = issuer.did.expect("ComplianceIssuer DID");

        let asset_helper = AssetHelper::new(
            &tester.api,
            &mut owner,
            "ComplianceAsset",
            1_000_000,
            BTreeSet::new(),
        )
        .await?;
        let asset_id = asset_helper.asset_id;

        // Build a compliance requirement: sender must have Accredited claim
        // issued by `issuer`.
        let trusted_issuer = TrustedIssuer {
            issuer: issuer_did,
            trusted_for: TrustedFor::Any,
        };

        let sender_condition = Condition {
            condition_type: ConditionType::IsPresent(Claim::Accredited(Scope::Asset(asset_id))),
            issuers: vec![trusted_issuer.clone()],
        };

        let receiver_condition = Condition {
            condition_type: ConditionType::IsPresent(Claim::Accredited(Scope::Asset(asset_id))),
            issuers: vec![trusted_issuer],
        };

        // Add a compliance requirement.
        tester
            .api
            .call()
            .compliance_manager()
            .add_compliance_requirement(
                asset_id,
                vec![sender_condition.clone()],
                vec![receiver_condition.clone()],
            )?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // Reset compliance requirements (removes all).
        tester
            .api
            .call()
            .compliance_manager()
            .reset_asset_compliance(asset_id)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // Add a new compliance requirement and then pause compliance.
        tester
            .api
            .call()
            .compliance_manager()
            .add_compliance_requirement(asset_id, vec![sender_condition], vec![receiver_condition])?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        tester
            .api
            .call()
            .compliance_manager()
            .pause_asset_compliance(asset_id)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // Resume compliance.
        tester
            .api
            .call()
            .compliance_manager()
            .resume_asset_compliance(asset_id)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        Ok(())
    }
}
