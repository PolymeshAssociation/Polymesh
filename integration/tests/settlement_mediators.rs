//! Settlement mediators: mediated instructions, mediator affirm/reject, instruction locking.
#[cfg(feature = "current_release")]
mod settlement_mediators_tests {
    use std::collections::BTreeSet;

    use anyhow::Result;

    use integration::*;
    use polymesh_api::types::polymesh_primitives::{
        asset::{AssetHolder, AssetHolderKind},
        identity_id::{PortfolioId, PortfolioKind},
        settlement::{Leg, SettlementType},
    };

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

    /// A mediated instruction requires the mediator's affirmation.
    #[tokio::test]
    #[test_log::test]
    async fn mediated_instruction_affirm_flow() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["SMOwner", "SMInv1", "SMMed1"])
            .await?
            .into_iter();
        let mut owner = users.next().unwrap();
        let inv1 = users.next().unwrap();
        let mut mediator = users.next().unwrap();

        let asset_id =
            create_asset_in_portfolio(&mut tester, &mut owner, "SMEDIA", 10_000).await?;
        let mediator_did = mediator.did.unwrap();

        // Instruction with one leg + mediator attached at creation.
        let leg = Leg::Fungible {
            sender: AssetHolder::Portfolio(pf(&owner)),
            receiver: AssetHolder::Portfolio(pf(&inv1)),
            asset_id: asset_id.clone(),
            amount: 100,
        };
        let mut res = tester
            .api
            .call()
            .settlement()
            .add_instruction_with_mediators(
                None,
                SettlementType::SettleOnAffirmation,
                None,
                None,
                vec![leg],
                None,
                BTreeSet::from([mediator_did]),
            )?
            .submit_and_watch(&mut owner)
            .await?;
        res.ok().await?;
        let inst_id = get_instruction_id(&mut res).await?.expect("instruction id");

        // Mediator affirms (with expiry).
        tester
            .api
            .call()
            .settlement()
            .affirm_instruction_as_mediator(inst_id, None)?
            .execute(&mut mediator)
            .await?
            .ok()
            .await?;

        use polymesh_api::types::polymesh_primitives::settlement::MediatorAffirmationStatus;
        let status = tester
            .api
            .query()
            .settlement()
            .instruction_mediators_affirmations(inst_id, mediator_did)
            .await?;
        assert!(
            matches!(status, MediatorAffirmationStatus::Affirmed { .. }),
            "mediator affirmation should be recorded"
        );

        Ok(())
    }

    /// A mediator can reject a pending instruction outright.
    #[tokio::test]
    #[test_log::test]
    async fn mediator_rejection_rejects_instruction() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["SMROwner", "SMRInv1", "SMRMed1"])
            .await?
            .into_iter();
        let mut owner = users.next().unwrap();
        let inv1 = users.next().unwrap();
        let mut mediator = users.next().unwrap();

        let asset_id =
            create_asset_in_portfolio(&mut tester, &mut owner, "SMREJX", 10_000).await?;
        let mediator_did = mediator.did.unwrap();

        let leg = Leg::Fungible {
            sender: AssetHolder::Portfolio(pf(&owner)),
            receiver: AssetHolder::Portfolio(pf(&inv1)),
            asset_id: asset_id.clone(),
            amount: 50,
        };
        let mut res = tester
            .api
            .call()
            .settlement()
            .add_instruction_with_mediators(
                None,
                SettlementType::SettleOnAffirmation,
                None,
                None,
                vec![leg],
                None,
                BTreeSet::from([mediator_did]),
            )?
            .submit_and_watch(&mut owner)
            .await?;
        res.ok().await?;
        let inst_id = get_instruction_id(&mut res).await?.expect("instruction id");

        // Mediator rejects with an upper bound on asset count.
        tester
            .api
            .call()
            .settlement()
            .reject_instruction_as_mediator(
                inst_id,
                Some(polymesh_api::types::polymesh_primitives::settlement::AssetCount {
                    fungible: 4,
                    non_fungible: 0,
                    off_chain: 0,
                }),
            )?
            .execute(&mut mediator)
            .await?
            .ok()
            .await?;

        // Instruction status becomes rejected/failed.
        let status = tester
            .api
            .query()
            .settlement()
            .instruction_statuses(inst_id)
            .await?;
        assert!(
            matches!(
                status,
                polymesh_api::types::polymesh_primitives::settlement::InstructionStatus::Rejected(_)
                    | polymesh_api::types::polymesh_primitives::settlement::InstructionStatus::Failed
            ),
            "mediator rejection must move instruction out of Pending"
        );

        Ok(())
    }

    /// lock/unlock is a mediator-only path on SettleAfterLock instructions.
    #[tokio::test]
    #[test_log::test]
    async fn lock_and_unlock_instruction() -> Result<()> {
        use polymesh_api::types::polymesh_primitives::settlement::InstructionStatus;

        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["SMLOwner", "SMLInv1", "SMLMed"])
            .await?
            .into_iter();
        let mut owner = users.next().unwrap();
        let inv1 = users.next().unwrap();
        let mut mediator = users.next().unwrap();

        let asset_id =
            create_asset_in_portfolio(&mut tester, &mut owner, "SMLOCK", 10_000).await?;
        let mediator_did = mediator.did.unwrap();

        let sender = AssetHolder::Portfolio(pf(&owner));
        let leg = Leg::Fungible {
            sender: sender.clone(),
            receiver: AssetHolder::Portfolio(pf(&inv1)),
            asset_id: asset_id.clone(),
            amount: 25,
        };
        let mut res = tester
            .api
            .call()
            .settlement()
            .add_instruction_with_mediators(
                None,
                SettlementType::SettleAfterLock,
                None,
                None,
                vec![leg],
                None,
                BTreeSet::from([mediator_did]),
            )?
            .submit_and_watch(&mut owner)
            .await?;
        res.ok().await?;
        let inst_id = get_instruction_id(&mut res).await?.expect("instruction id");

        tester
            .api
            .call()
            .settlement()
            .affirm_instruction(inst_id, BTreeSet::from([sender]))?
            .execute(&mut owner)
            .await?
            .ok()
            .await?;
        tester
            .api
            .call()
            .settlement()
            .affirm_instruction_as_mediator(inst_id, None)?
            .execute(&mut mediator)
            .await?
            .ok()
            .await?;

        tester
            .api
            .call()
            .settlement()
            .lock_instruction(
                inst_id,
                sp_weights::Weight::from_parts(10_000_000_000, 10_000_000),
            )?
            .execute(&mut mediator)
            .await?
            .ok()
            .await?;
        assert!(
            matches!(
                tester
                    .api
                    .query()
                    .settlement()
                    .instruction_statuses(inst_id)
                    .await?,
                InstructionStatus::LockedForExecution
            ),
            "mediator lock must move instruction to LockedForExecution"
        );

        tester
            .api
            .call()
            .settlement()
            .unlock_instruction(inst_id)?
            .execute(&mut mediator)
            .await?
            .ok()
            .await?;
        assert!(
            matches!(
                tester
                    .api
                    .query()
                    .settlement()
                    .instruction_statuses(inst_id)
                    .await?,
                InstructionStatus::Pending
            ),
            "unlock must return the instruction to Pending"
        );

        Ok(())
    }
}