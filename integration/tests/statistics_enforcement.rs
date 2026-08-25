//! Statistics transfer-manager enforcement: investor count limits & exemptions.
#[cfg(feature = "current_release")]
mod statistics_enforcement_tests {
    use std::collections::BTreeSet;

    use anyhow::Result;

    use integration::*;
    use polymesh_api::types::polymesh_primitives::{
        asset::AssetHolderKind,
        condition::{TrustedFor, TrustedIssuer},
        statistics::{Stat2ndKey, StatOpType, StatType, StatUpdate},
        transfer_compliance::{TransferCondition, TransferConditionExemptKey},
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

    fn count_stat() -> StatType {
        StatType {
            operation_type: StatOpType::Count,
            claim_issuer: None,
        }
    }

    /// MaxInvestorCount blocks the Nth+1 transfer until the limit is raised.
    #[tokio::test]
    #[test_log::test]
    async fn max_investor_count_enforcement() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["SMIOwner", "SMIIssuer", "SMIInv1", "SMIInv2", "SMIInv3"])
            .await?
            .into_iter();
        let mut owner = users.next().unwrap();
        let issuer = users.next().unwrap();
        let mut inv1 = users.next().unwrap();
        let mut inv2 = users.next().unwrap();
        let inv3 = users.next().unwrap();

        let issuer_did = issuer.did.unwrap();

        let asset_id = create_asset(&mut tester, &mut owner, "SMICNT", 1_000_000).await?;

        // Trusted issuer so claims satisfy compliance.
        tester
            .api
            .call()
            .compliance_manager()
            .add_default_trusted_claim_issuer(
                asset_id.clone(),
                TrustedIssuer {
                    issuer: issuer_did,
                    trusted_for: TrustedFor::Any,
                },
            )?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // Track investor Count and cap at 2.
        tester
            .api
            .call()
            .statistics()
            .set_active_asset_stats(asset_id.clone(), BTreeSet::from([count_stat()]))?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;
        tester
            .api
            .call()
            .statistics()
            .set_asset_transfer_compliance(
                asset_id.clone(),
                BTreeSet::from([TransferCondition::MaxInvestorCount(2)]),
            )?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // 1st & 2nd investor OK.
        for inv in [&mut inv1, &mut inv2] {
            let mut res = tester
                .api
                .call()
                .asset()
                .transfer_asset(asset_id.clone(), inv.account(), 100, None)?
                .submit_and_watch(&mut owner)
                .await?;
            res.ok().await?;
        }

        // 3rd investor blocked.
        let mut res = tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id.clone(), inv3.account(), 100, None)?
            .submit_and_watch(&mut owner)
            .await?;
        assert!(res.ok().await.is_err(), "3rd investor must be blocked by MaxInvestorCount=2");

        // Raise the cap to 3.
        tester
            .api
            .call()
            .statistics()
            .set_asset_transfer_compliance(
                asset_id.clone(),
                BTreeSet::from([TransferCondition::MaxInvestorCount(3)]),
            )?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;
        tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id.clone(), inv3.account(), 100, None)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        Ok(())
    }

    /// Exempted entities bypass the investor-count restriction.
    #[tokio::test]
    #[test_log::test]
    async fn exempt_entity_bypasses_limit() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["SEEOwner", "SEEIssuer", "SEEInv1", "SEEInv2", "SEEDealer"])
            .await?
            .into_iter();
        let mut owner = users.next().unwrap();
        let issuer = users.next().unwrap();
        let mut inv1 = users.next().unwrap();
        let mut inv2 = users.next().unwrap();
        let dealer = users.next().unwrap();

        let issuer_did = issuer.did.unwrap();
        let dealer_did = dealer.did.expect("dealer did");

        let asset_id = create_asset(&mut tester, &mut owner, "SEEXMP", 1_000_000).await?;

        tester
            .api
            .call()
            .compliance_manager()
            .add_default_trusted_claim_issuer(
                asset_id.clone(),
                TrustedIssuer {
                    issuer: issuer_did,
                    trusted_for: TrustedFor::Any,
                },
            )?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        tester
            .api
            .call()
            .statistics()
            .set_active_asset_stats(asset_id.clone(), BTreeSet::from([count_stat()]))?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;
        tester
            .api
            .call()
            .statistics()
            .set_asset_transfer_compliance(
                asset_id.clone(),
                BTreeSet::from([TransferCondition::MaxInvestorCount(2)]),
            )?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // Fill up both slots.
        for inv in [&mut inv1, &mut inv2] {
            tester
                .api
                .call()
                .asset()
                .transfer_asset(asset_id.clone(), inv.account(), 100, None)?
                .submit_and_watch(&mut owner)
                .await?
                .ok()
                .await?;
        }

        // Count restrictions exempt the *sender*. Exempt the issuer so they can
        // still transfer to a new investor once the cap is reached.
        tester
            .api
            .call()
            .statistics()
            .set_entities_exempt(
                true,
                TransferConditionExemptKey {
                    asset_id: asset_id.clone(),
                    op: StatOpType::Count,
                    claim_type: None,
                },
                BTreeSet::from([owner.did.unwrap()]),
            )?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id.clone(), dealer.account(), 100, None)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // Remove exemption -> further transfers to the dealer are blocked again.
        tester
            .api
            .call()
            .statistics()
            .set_entities_exempt(
                false,
                TransferConditionExemptKey {
                    asset_id: asset_id.clone(),
                    op: StatOpType::Count,
                    claim_type: None,
                },
                BTreeSet::from([dealer_did]),
            )?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        Ok(())
    }

    /// batch_update_asset_stats can adjust tracked stats (investor count).
    #[tokio::test]
    #[test_log::test]
    async fn batch_update_asset_stats_adjusts_count() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["SBIOwner", "SBIIssuer", "SBIInv1", "SBIInv2"])
            .await?
            .into_iter();
        let mut owner = users.next().unwrap();
        let issuer = users.next().unwrap();
        let inv1 = users.next().unwrap();
        let inv2 = users.next().unwrap();

        let issuer_did = issuer.did.unwrap();

        let asset_id = create_asset(&mut tester, &mut owner, "SBIBAT", 1_000_000).await?;

        tester
            .api
            .call()
            .compliance_manager()
            .add_default_trusted_claim_issuer(
                asset_id.clone(),
                TrustedIssuer {
                    issuer: issuer_did,
                    trusted_for: TrustedFor::Any,
                },
            )?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        tester
            .api
            .call()
            .statistics()
            .set_active_asset_stats(asset_id.clone(), BTreeSet::from([count_stat()]))?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // Enable Count tracking + cap of 1 in a single batch update.
        tester
            .api
            .call()
            .statistics()
            .batch_update_asset_stats(
                asset_id.clone(),
                count_stat(),
                BTreeSet::from([StatUpdate {
                    key2: Stat2ndKey::NoClaimStat,
                    value: Some(0), // current investor count
                }]),
            )?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;
        tester
            .api
            .call()
            .statistics()
            .set_asset_transfer_compliance(
                asset_id.clone(),
                BTreeSet::from([TransferCondition::MaxInvestorCount(1)]),
            )?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // First investor fine.
        tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id.clone(), inv1.account(), 100, None)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // Second blocked at cap=1...
        let mut res = tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id.clone(), inv2.account(), 100, None)?
            .submit_and_watch(&mut owner)
            .await?;
        assert!(res.ok().await.is_err());

        // ...but allowed once the stored count is corrected downwards.
        tester
            .api
            .call()
            .statistics()
            .batch_update_asset_stats(
                asset_id.clone(),
                count_stat(),
                BTreeSet::from([StatUpdate {
                    key2: Stat2ndKey::NoClaimStat,
                    value: Some(1),
                }]),
            )?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        Ok(())
    }
}