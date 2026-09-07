//! Scheduled & automatic settlement execution (requires `timed` feature: waits on blocks).
#[cfg(all(feature = "current_release", feature = "timed"))]
mod settlement_scheduling_tests {
    use std::collections::BTreeSet;

    use anyhow::{bail, Result};

    use integration::*;
    use polymesh_api::types::polymesh_primitives::{
        asset::{AssetHolder, AssetHolderKind},
        identity_id::{PortfolioId, PortfolioKind},
        settlement::{InstructionStatus, Leg, SettlementType},
    };

    const MAX_BLOCK_WAIT: u32 = 40;

    async fn create_asset_in_portfolio(
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
            Some(AssetHolderKind::DefaultPortfolio),
        )
        .await?;
        Ok(helper.asset_id)
    }

    fn pf(user: &User) -> PortfolioId {
        PortfolioId {
            did: user.did.expect("did"),
            kind: PortfolioKind::Default,
        }
    }

    fn holder(user: &User) -> AssetHolder {
        AssetHolder::Portfolio(pf(user))
    }

    async fn block_number(tester: &PolymeshTester) -> Result<u32> {
        Ok(tester.api.query().system().number().await?)
    }

    /// Poll until the instruction leaves Pending or we run out of blocks.
    async fn wait_for_completion(
        tester: &PolymeshTester,
        inst_id: polymesh_api::types::polymesh_primitives::settlement::InstructionId,
        start_block: u32,
    ) -> Result<InstructionStatus<u32>> {
        loop {
            let now = block_number(tester).await?;
            if now > start_block + MAX_BLOCK_WAIT {
                bail!("instruction did not complete within {MAX_BLOCK_WAIT} blocks");
            }
            let status = tester
                .api
                .query()
                .settlement()
                .instruction_statuses(inst_id)
                .await?;
            match status {
                InstructionStatus::Pending | InstructionStatus::Unknown => {
                    tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                }
                other => return Ok(other),
            }
        }
    }

    /// SettleOnAffirmation instructions execute automatically once all parties affirm.
    #[tokio::test]
    #[test_log::test]
    async fn settle_on_affirmation_auto_executes() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["SSOwner", "SSInv1"]).await?.into_iter();
        let mut owner = users.next().unwrap();
        let inv1 = users.next().unwrap();

        let asset_id =
            create_asset_in_portfolio(&mut tester, &mut owner, "SSAFFRM", 10_000).await?;

        let leg = Leg::Fungible {
            sender: holder(&owner),
            receiver: holder(&inv1),
            asset_id: asset_id.clone(),
            amount: 100,
        };
        let mut res = tester
            .api
            .call()
            .settlement()
            .add_instruction(
                None,
                SettlementType::SettleOnAffirmation,
                None,
                None,
                vec![leg],
                None,
            )?
            .submit_and_watch(&mut owner)
            .await?;
        res.ok().await?;
        let inst_id = get_instruction_id(&mut res).await?.expect("instruction id");
        let start = block_number(&tester).await?;

        // Current release auto-affirms the receiver; only the sender must affirm.
        tester
            .api
            .call()
            .settlement()
            .affirm_instruction(inst_id, BTreeSet::from([holder(&owner)]))?
            .execute(&mut owner)
            .await?
            .ok()
            .await?;

        // No manual execution needed.
        let status = wait_for_completion(&tester, inst_id, start).await?;
        assert!(
            matches!(status, InstructionStatus::Success(_)),
            "auto-settlement should succeed"
        );

        let bal = tester
            .api
            .query()
            .portfolio()
            .portfolio_asset_balances(pf(&inv1), asset_id)
            .await?;
        assert_eq!(bal, 100);

        Ok(())
    }

    /// SettleOnBlock instructions auto-execute at/after the target block.
    #[tokio::test]
    #[test_log::test]
    async fn settle_on_block_auto_executes() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["SSOwner2", "SSInv2"]).await?.into_iter();
        let mut owner = users.next().unwrap();
        let inv1 = users.next().unwrap();

        let asset_id =
            create_asset_in_portfolio(&mut tester, &mut owner, "SSBLOCK", 10_000).await?;

        // Target ~4 blocks out.
        let target_block = block_number(&tester).await? + 4;

        let leg = Leg::Fungible {
            sender: holder(&owner),
            receiver: holder(&inv1),
            asset_id: asset_id.clone(),
            amount: 200,
        };
        let mut res = tester
            .api
            .call()
            .settlement()
            .add_instruction(
                None,
                SettlementType::SettleOnBlock(target_block),
                None,
                None,
                vec![leg],
                None,
            )?
            .submit_and_watch(&mut owner)
            .await?;
        res.ok().await?;
        let inst_id = get_instruction_id(&mut res).await?.expect("instruction id");

        tester
            .api
            .call()
            .settlement()
            .affirm_instruction(inst_id, BTreeSet::from([holder(&owner)]))?
            .execute(&mut owner)
            .await?
            .ok()
            .await?;

        // The scheduler executes it around `target_block`.
        let status = wait_for_completion(&tester, inst_id, target_block.saturating_sub(4)).await?;
        assert!(matches!(status, InstructionStatus::Success(_)));

        let bal = tester
            .api
            .query()
            .portfolio()
            .portfolio_asset_balances(pf(&inv1), asset_id)
            .await?;
        assert_eq!(bal, 200);

        Ok(())
    }

    /// A single failing leg aborts the whole instruction (all-or-nothing legs).
    #[tokio::test]
    #[test_log::test]
    async fn failing_leg_aborts_instruction() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["SSOwner3", "SSInv3"]).await?.into_iter();
        let mut owner = users.next().unwrap();
        let inv1 = users.next().unwrap();

        let asset_ok =
            create_asset_in_portfolio(&mut tester, &mut owner, "SSFAILOK", 10_000).await?;
        let asset_bad =
            create_asset_in_portfolio(&mut tester, &mut owner, "SSFAILBD", 10_000).await?;

        // Two legs of the same instruction; freezing the second asset aborts the whole thing.
        let legs = vec![
            Leg::Fungible {
                sender: holder(&owner),
                receiver: holder(&inv1),
                asset_id: asset_ok.clone(),
                amount: 100,
            },
            Leg::Fungible {
                sender: holder(&owner),
                receiver: holder(&inv1),
                asset_id: asset_bad.clone(),
                amount: 100,
            },
        ];
        let mut res = tester
            .api
            .call()
            .settlement()
            .add_instruction(
                None,
                SettlementType::SettleManual(0),
                None,
                None,
                legs,
                None,
            )?
            .submit_and_watch(&mut owner)
            .await?;
        res.ok().await?;
        let inst_id = get_instruction_id(&mut res).await?.expect("instruction id");

        tester
            .api
            .call()
            .settlement()
            .affirm_instruction(inst_id, BTreeSet::from([holder(&owner)]))?
            .execute(&mut owner)
            .await?
            .ok()
            .await?;

        tester
            .api
            .call()
            .asset()
            .freeze(asset_bad.clone())?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // Manual execution fails because one asset is frozen.
        let res = tester
            .api
            .call()
            .settlement()
            .execute_manual_instruction(inst_id, None, 5, 0, 0, None)?
            .execute(&mut owner)
            .await;
        match res {
            Ok(mut r) => assert!(r.ok().await.is_err(), "execution must fail on bad leg"),
            Err(_) => {}
        }

        // Atomicity: nothing moved.
        let bal_ok = tester
            .api
            .query()
            .portfolio()
            .portfolio_asset_balances(pf(&inv1), asset_ok)
            .await?;
        let bal_bad = tester
            .api
            .query()
            .portfolio()
            .portfolio_asset_balances(pf(&inv1), asset_bad)
            .await?;
        assert_eq!(bal_ok, 0, "no tokens may move when any leg fails");
        assert_eq!(bal_bad, 0, "no tokens may move when any leg fails");

        Ok(())
    }
}
