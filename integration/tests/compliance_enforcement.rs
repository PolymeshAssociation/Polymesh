//! End-to-end compliance enforcement: transfers blocked/allowed by claims.
#[cfg(feature = "current_release")]
mod compliance_enforcement_tests {
    use std::collections::BTreeSet;

    use anyhow::Result;

    use integration::*;
    use polymesh_api::types::polymesh_primitives::{
        asset::AssetHolderKind,
        condition::{Condition, ConditionType, TrustedFor, TrustedIssuer},
        identity_claim::{Claim, Scope},
    };

    async fn create_asset(
        tester: &mut PolymeshTester,
        owner: &mut User,
        ticker: &str,
        amount: u128,
    ) -> Result<AssetId> {
        let helper = AssetHelper::new_full(
            &tester.api,
            owner,
            ticker,
            amount,
            BTreeSet::new(),
            false,
            Some(AssetHolderKind::Account),
        )
        .await?;
        Ok(helper.asset_id)
    }

    /// Transfer blocked until both sender & receiver hold the required claim.
    #[tokio::test]
    #[test_log::test]
    async fn transfer_blocked_until_claims_added() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["CEOwner", "CEIssuer", "CEInvestor1", "CEInvestor2"])
            .await?
            .into_iter();
        let mut owner = users.next().unwrap();
        let mut issuer = users.next().unwrap();
        let investor1 = users.next().unwrap();
        let _investor2 = users.next().unwrap();

        let issuer_did = issuer.did.expect("issuer did");
        let owner_did = owner.did.expect("owner did");
        let inv1_did = investor1.did.expect("investor1 did");

        let asset_id = create_asset(&mut tester, &mut owner, "CEBLOCK", 1_000_000).await?;

        // Requirement: sender AND receiver must have Accredited (asset-scoped) claim.
        let cond = Condition {
            condition_type: ConditionType::IsPresent(Claim::Accredited(Scope::Asset(asset_id))),
            issuers: vec![TrustedIssuer {
                issuer: issuer_did,
                trusted_for: TrustedFor::Any,
            }],
        };
        tester
            .api
            .call()
            .compliance_manager()
            .add_compliance_requirement(asset_id.clone(), vec![cond.clone()], vec![cond])?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // No claims yet -> transfer must fail.
        let mut res = tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id.clone(), investor1.account(), 100, None)?
            .submit_and_watch(&mut owner)
            .await?;
        assert!(res.ok().await.is_err(), "transfer without claims should fail");

        // Sender-only claim is not enough.
        tester
            .api
            .call()
            .identity()
            .add_claim(owner_did, Claim::Accredited(Scope::Asset(asset_id.clone())), None)?
            .submit_and_watch(&mut issuer)
            .await?
            .ok()
            .await?;
        let mut res = tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id.clone(), investor1.account(), 100, None)?
            .submit_and_watch(&mut owner)
            .await?;
        assert!(res.ok().await.is_err(), "transfer should still fail (receiver lacks claim)");

        // Receiver claim added -> transfer succeeds.
        tester
            .api
            .call()
            .identity()
            .add_claim(inv1_did, Claim::Accredited(Scope::Asset(asset_id.clone())), None)?
            .submit_and_watch(&mut issuer)
            .await?
            .ok()
            .await?;
        tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id.clone(), investor1.account(), 100, None)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        Ok(())
    }

    /// Expired claims no longer satisfy requirements.
    #[tokio::test]
    #[test_log::test]
    async fn expired_claims_block_transfer() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["CExOwner", "CExIssuer", "CExInvestor1", "CExInvestor2"])
            .await?
            .into_iter();
        let mut owner = users.next().unwrap();
        let mut issuer = users.next().unwrap();
        let investor1 = users.next().unwrap();
        let investor2 = users.next().unwrap();

        let issuer_did = issuer.did.expect("issuer did");
        let owner_did = owner.did.expect("owner did");
        let inv1_did = investor1.did.expect("investor1 did");

        let asset_id = create_asset(&mut tester, &mut owner, "CEXPIRY", 1_000_000).await?;

        let cond = Condition {
            condition_type: ConditionType::IsPresent(Claim::Affiliate(Scope::Asset(asset_id))),
            issuers: vec![TrustedIssuer {
                issuer: issuer_did,
                trusted_for: TrustedFor::Any,
            }],
        };
        tester
            .api
            .call()
            .compliance_manager()
            .add_compliance_requirement(asset_id.clone(), vec![cond.clone()], vec![cond])?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // Claims valid for ~3 seconds only.
        let now = tester.api.query().timestamp().now().await?;
        let expiry = now + 20_000;
        for did in [owner_did, inv1_did] {
            tester
                .api
                .call()
                .identity()
                .add_claim(did, Claim::Affiliate(Scope::Asset(asset_id.clone())), Some(expiry))?
                .submit_and_watch(&mut issuer)
                .await?
                .ok()
                .await?;
        }

        // Valid while unexpired.
        tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id.clone(), investor1.account(), 100, None)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // Wait out the expiry.
        tokio::time::sleep(std::time::Duration::from_secs(22)).await;

        // New transfer to a fresh receiver must fail now.
        let mut res = tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id.clone(), investor2.account(), 100, None)?
            .submit_and_watch(&mut owner)
            .await?;
        assert!(res.ok().await.is_err(), "transfer after claim expiry should fail");

        Ok(())
    }

    /// Pausing compliance lets transfers through; resume re-enables checks.
    #[tokio::test]
    #[test_log::test]
    async fn pause_resume_allows_and_blocks() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["CPaOwner", "CPaIssuer", "CPaInvestor1"])
            .await?
            .into_iter();
        let mut owner = users.next().unwrap();
        let mut issuer = users.next().unwrap();
        let investor1 = users.next().unwrap();

        let issuer_did = issuer.did.expect("issuer did");
        let owner_did = owner.did.expect("owner did");
        let inv1_did = investor1.did.expect("investor1 did");

        let asset_id = create_asset(&mut tester, &mut owner, "CPAUSE", 1_000_000).await?;

        let cond = Condition {
            condition_type: ConditionType::IsPresent(Claim::KnowYourCustomer(Scope::Asset(
                asset_id.clone(),
            ))),
            issuers: vec![TrustedIssuer {
                issuer: issuer_did,
                trusted_for: TrustedFor::Any,
            }],
        };
        tester
            .api
            .call()
            .compliance_manager()
            .add_compliance_requirement(asset_id.clone(), vec![cond.clone()], vec![cond])?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // No KYC claims yet -> blocked.
        let mut res = tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id.clone(), investor1.account(), 10, None)?
            .submit_and_watch(&mut owner)
            .await?;
        assert!(res.ok().await.is_err());

        // Pause compliance -> transfer allowed despite missing claims.
        tester
            .api
            .call()
            .compliance_manager()
            .pause_asset_compliance(asset_id.clone())?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;
        tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id.clone(), investor1.account(), 10, None)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // Resume -> blocked again (still no claims).
        tester
            .api
            .call()
            .compliance_manager()
            .resume_asset_compliance(asset_id.clone())?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;
        let mut res = tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id.clone(), investor1.account(), 10, None)?
            .submit_and_watch(&mut owner)
            .await?;
        assert!(res.ok().await.is_err());

        // Add KYC claims for both sender and receiver and verify transfer succeeds.
        for did in [owner_did, inv1_did] {
            tester
                .api
                .call()
                .identity()
                .add_claim(did, Claim::KnowYourCustomer(Scope::Asset(asset_id.clone())), None)?
                .submit_and_watch(&mut issuer)
                .await?
                .ok()
                .await?;
        }
        tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id.clone(), investor1.account(), 10, None)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        Ok(())
    }

    /// change/remove/replace requirement updates affect subsequent transfers.
    #[tokio::test]
    #[test_log::test]
    async fn change_remove_replace_requirements() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["CRROwner", "CRRIssuer", "CRRInvestor1"])
            .await?
            .into_iter();
        let mut owner = users.next().unwrap();
        let mut issuer = users.next().unwrap();
        let investor1 = users.next().unwrap();

        let issuer_did = issuer.did.expect("issuer did");

        let asset_id = create_asset(&mut tester, &mut owner, "CRRMOD", 1_000_000).await?;

        let accredited_cond = || Condition {
            condition_type: ConditionType::IsPresent(Claim::Accredited(Scope::Asset(asset_id))),
            issuers: vec![TrustedIssuer {
                issuer: issuer_did,
                trusted_for: TrustedFor::Any,
            }],
        };

        tester
            .api
            .call()
            .compliance_manager()
            .add_compliance_requirement(asset_id.clone(), vec![], vec![accredited_cond()])?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // Receiver lacks Accredited -> fails.
        let mut res = tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id.clone(), investor1.account(), 10, None)?
            .submit_and_watch(&mut owner)
            .await?;
        assert!(res.ok().await.is_err());

        // Remove the only requirement (id 1) -> succeeds.
        tester
            .api
            .call()
            .compliance_manager()
            .remove_compliance_requirement(asset_id.clone(), 1)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;
        tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id.clone(), investor1.account(), 10, None)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // Replace whole compliance with a receiver-Accredited requirement again.
        let req = polymesh_api::types::polymesh_primitives::compliance_manager::ComplianceRequirement {
            id: 5,
            sender_conditions: vec![],
            receiver_conditions: vec![accredited_cond()],
        };
        tester
            .api
            .call()
            .compliance_manager()
            .replace_asset_compliance(asset_id.clone(), vec![req])?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        let mut res = tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id.clone(), investor1.account(), 10, None)?
            .submit_and_watch(&mut owner)
            .await?;
        assert!(res.ok().await.is_err(), "replaced compliance should block again");

        // Satisfy it via a claim from the trusted issuer.
        tester
            .api
            .call()
            .identity()
            .add_claim(
                investor1.did.unwrap(),
                Claim::Accredited(Scope::Asset(asset_id.clone())),
                None,
            )?
            .submit_and_watch(&mut issuer)
            .await?
            .ok()
            .await?;
        tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id.clone(), investor1.account(), 10, None)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        Ok(())
    }

    /// Claims from issuers that are not trusted do not satisfy conditions.
    #[tokio::test]
    #[test_log::test]
    async fn untrusted_issuer_claim_does_not_satisfy() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["CTIOwner", "CTIIssuerA", "CTIIssuerB", "CTIInvestor1"])
            .await?
            .into_iter();
        let mut owner = users.next().unwrap();
        let mut issuer_a = users.next().unwrap();
        let mut issuer_b = users.next().unwrap();
        let investor1 = users.next().unwrap();

        let issuer_a_did = issuer_a.did.expect("issuer A did");
        let inv1_did = investor1.did.expect("investor1 did");

        let asset_id = create_asset(&mut tester, &mut owner, "CTISCOP", 1_000_000).await?;

        // Requirement trusts ONLY IssuerA.
        let cond = Condition {
            condition_type: ConditionType::IsPresent(Claim::Accredited(Scope::Asset(asset_id))),
            issuers: vec![TrustedIssuer {
                issuer: issuer_a_did,
                trusted_for: TrustedFor::Any,
            }],
        };
        tester
            .api
            .call()
            .compliance_manager()
            .add_compliance_requirement(asset_id.clone(), vec![], vec![cond])?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // Claim from untrusted IssuerB doesn't help.
        tester
            .api
            .call()
            .identity()
            .add_claim(inv1_did, Claim::Accredited(Scope::Asset(asset_id.clone())), None)?
            .submit_and_watch(&mut issuer_b)
            .await?
            .ok()
            .await?;
        let mut res = tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id.clone(), investor1.account(), 10, None)?
            .submit_and_watch(&mut owner)
            .await?;
        assert!(res.ok().await.is_err(), "untrusted issuer's claim should not satisfy");

        // Claim from trusted IssuerA works.
        tester
            .api
            .call()
            .identity()
            .add_claim(inv1_did, Claim::Accredited(Scope::Asset(asset_id.clone())), None)?
            .submit_and_watch(&mut issuer_a)
            .await?
            .ok()
            .await?;
        tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id.clone(), investor1.account(), 10, None)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        Ok(())
    }
}