//! Protocol fees and treasury disbursement/reimbursement.
#[cfg(feature = "current_release")]
mod economics_tests {
    use anyhow::Result;

    use integration::*;
    use polymesh_api::types::polymesh_primitives::{
        protocol_fee::ProtocolOp,
        Beneficiary,
    };

    async fn free_balance(tester: &PolymeshTester, who: &AccountId) -> Result<u128> {
        let info = tester.api.query().system().account(who.clone()).await?;
        Ok(info.data.free)
    }

    /// Ticker registration charges a protocol fee (balance decreases by more than just the tx fee).
    #[tokio::test]
    #[test_log::test]
    async fn ticker_registration_charges_protocol_fee() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["EcoOwner"]).await?.into_iter();
        let mut owner = users.next().unwrap();

        let before = free_balance(&tester, &owner.account()).await?;
        let ticker = unique_ticker("ECO");
        tester
            .api
            .call()
            .asset()
            .register_unique_ticker(ticker)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;
        let after = free_balance(&tester, &owner.account()).await?;
        assert!(after < before, "ticker registration should cost POLYX");
        // Protocol fee for ticker registration is 500 POLYX on the develop chain spec.
        assert!(
            before - after >= 500 * ONE_POLYX,
            "expected at least the 500 POLYX protocol fee, delta={}",
            before - after
        );

        Ok(())
    }

    /// Root can change the base protocol fee; subsequent registrations pick it up.
    #[tokio::test]
    #[test_log::test]
    async fn sudo_changes_base_fee() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["EcoFeeOwner"]).await?.into_iter();
        let mut owner = users.next().unwrap();
        let mut sudo = tester.sudo.clone().expect("dev chain has sudo");

        // Raise ticker registration fee to 1_000 POLYX.
        let set_fee = tester
            .api
            .call()
            .protocol_fee()
            .change_base_fee(ProtocolOp::AssetRegisterTicker, 1_000 * ONE_POLYX)?
            .into_runtime_call();
        tester
            .api
            .call()
            .sudo()
            .sudo(set_fee)?
            .submit_and_watch(&mut sudo)
            .await?
            .ok()
            .await?;

        let before = free_balance(&tester, &owner.account()).await?;
        tester
            .api
            .call()
            .asset()
            .register_unique_ticker(unique_ticker("EC2"))?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;
        let after = free_balance(&tester, &owner.account()).await?;
        assert!(
            before - after >= 1_000 * ONE_POLYX,
            "new base fee should apply, delta={}",
            before - after
        );

        // Restore original fee so later tests are not affected.
        let restore = tester
            .api
            .call()
            .protocol_fee()
            .change_base_fee(ProtocolOp::AssetRegisterTicker, 500 * ONE_POLYX)?
            .into_runtime_call();
        tester
            .api
            .call()
            .sudo()
            .sudo(restore)?
            .submit_and_watch(&mut sudo)
            .await?
            .ok()
            .await?;

        Ok(())
    }

    /// Treasury reimbursement (donate in) and root disbursement (pay out).
    #[tokio::test]
    #[test_log::test]
    async fn treasury_reimburse_and_disburse() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["EcoDonor", "EcoBene1"])
            .await?
            .into_iter();
        let mut donor = users.next().unwrap();
        let bene = users.next().unwrap();
        let mut sudo = tester.sudo.clone().expect("dev chain has sudo");

        let donate = 200 * ONE_POLYX;
        let donor_before = free_balance(&tester, &donor.account()).await?;
        tester
            .api
            .call()
            .treasury()
            .reimbursement(donate)?
            .submit_and_watch(&mut donor)
            .await?
            .ok()
            .await?;
        let donor_after = free_balance(&tester, &donor.account()).await?;
        assert!(donor_after + donate <= donor_before);

        let bene_before = free_balance(&tester, &bene.account()).await?;
        let payout = 50 * ONE_POLYX;
        let call = tester
            .api
            .call()
            .treasury()
            .disbursement(vec![Beneficiary {
                id: bene.did.expect("bene did"),
                amount: payout,
            }])?
            .into_runtime_call();
        tester
            .api
            .call()
            .sudo()
            .sudo(call)?
            .submit_and_watch(&mut sudo)
            .await?
            .ok()
            .await?;
        let bene_after = free_balance(&tester, &bene.account()).await?;
        assert!(
            bene_after >= bene_before + payout,
            "beneficiary should receive the disbursement"
        );

        Ok(())
    }
}