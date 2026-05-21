#[cfg(feature = "current_release")]
mod statistics_tests {
    use std::collections::BTreeSet;

    use anyhow::Result;
    use integration::*;
    use polymesh_api::types::polymesh_primitives::{
        condition::{TrustedFor, TrustedIssuer},
        statistics::{StatOpType, StatType},
        transfer_compliance::TransferCondition,
    };

    /// Test adding a default trusted claim issuer and enabling investor count
    /// statistics with a MaxInvestorCount transfer compliance condition.
    ///
    /// Ported from `15_portfolio.ts` (statistics section).
    #[tokio::test]
    #[test_log::test]
    async fn statistics_and_transfer_compliance() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["StatsOwner", "StatsIssuer"])
            .await?
            .into_iter();
        let mut owner = users.next().expect("StatsOwner");
        let issuer = users.next().expect("StatsIssuer");

        let issuer_did = issuer.did.expect("StatsIssuer DID");

        let asset_helper = AssetHelper::new(
            &tester.api,
            &mut owner,
            "StatsTestAsset",
            1_000_000,
            BTreeSet::new(),
        )
        .await?;
        let asset_id = asset_helper.asset_id;

        // Add a default trusted claim issuer for the asset.
        let trusted_issuer = TrustedIssuer {
            issuer: issuer_did,
            trusted_for: TrustedFor::Any,
        };
        tester
            .api
            .call()
            .compliance_manager()
            .add_default_trusted_claim_issuer(asset_id, trusted_issuer)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // Enable investor count statistics for the asset.
        let count_stat = StatType {
            operation_type: StatOpType::Count,
            claim_issuer: None,
        };
        tester
            .api
            .call()
            .statistics()
            .set_active_asset_stats(asset_id, BTreeSet::from([count_stat]))?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // Set a MaxInvestorCount transfer compliance condition.
        tester
            .api
            .call()
            .statistics()
            .set_asset_transfer_compliance(
                asset_id,
                BTreeSet::from([TransferCondition::MaxInvestorCount(10)]),
            )?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        Ok(())
    }
}
