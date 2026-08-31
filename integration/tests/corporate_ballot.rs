//! Corporate ballot: attach, vote, window changes.
#[cfg(feature = "current_release")]
mod corporate_ballot_tests {
    use std::collections::BTreeSet;

    use anyhow::Result;

    use integration::*;
    use polymesh_api::types::pallet_corporate_actions::{CADetails, CAKind, RecordDateSpec};
    use polymesh_api::types::pallet_corporate_actions::ballot::{
        BallotMeta, BallotTimeRange, BallotTitle, BallotVote, ChoiceTitle, Motion, MotionInfoLink,
        MotionTitle,
    };
    use polymesh_api::types::polymesh_primitives::asset::AssetHolderKind;

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

    fn ballot_meta(title: &str) -> BallotMeta {
        BallotMeta {
            title: BallotTitle(title.as_bytes().to_vec()),
            motions: vec![Motion {
                title: MotionTitle(b"Approve?".to_vec()),
                info_link: MotionInfoLink(b"https://example.test".to_vec()),
                choices: vec![
                    ChoiceTitle(b"Yes".to_vec()),
                    ChoiceTitle(b"No".to_vec()),
                ],
            }],
        }
    }

    /// Attach a ballot to an IssuerNotice CA and cast a vote.
    #[tokio::test]
    #[test_log::test]
    async fn attach_and_vote() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["CbOwner", "CbVoter"])
            .await?
            .into_iter();
        let mut owner = users.next().unwrap();
        let mut voter = users.next().unwrap();

        let asset_id = create_asset(&mut tester, &mut owner, "CBVOTE", 1_000_000).await?;
        tester
            .api
            .call()
            .asset()
            .transfer_asset(asset_id.clone(), voter.account(), 100_000, None)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        let mut cp_res = tester
            .api
            .call()
            .checkpoint()
            .create_checkpoint(asset_id.clone())?
            .submit_and_watch(&mut owner)
            .await?;
        cp_res.ok().await?;
        let cp_id = get_checkpoint_id(&mut cp_res).await?.expect("checkpoint");

        let now = tester.api.query().timestamp().now().await?;
        let mut ca_res = tester
            .api
            .call()
            .corporate_action()
            .initiate_corporate_action(
                asset_id.clone(),
                CAKind::IssuerNotice,
                now,
                Some(RecordDateSpec::Existing(cp_id)),
                CADetails(b"agm".to_vec()),
                None,
                None,
                None,
            )?
            .submit_and_watch(&mut owner)
            .await?;
        ca_res.ok().await?;
        let ca_id = get_ca_id(&mut ca_res).await?.expect("ca id");

        let range = BallotTimeRange {
            start: now,
            end: now + 60_000,
        };
        tester
            .api
            .call()
            .corporate_ballot()
            .attach_ballot(ca_id.clone(), range, ballot_meta("AGM"), false)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        tester
            .api
            .call()
            .corporate_ballot()
            .vote(
                ca_id.clone(),
                vec![
                    BallotVote {
                        power: 100_000,
                        fallback: None,
                    },
                    BallotVote {
                        power: 0,
                        fallback: None,
                    },
                ],
            )?
            .submit_and_watch(&mut voter)
            .await?
            .ok()
            .await?;

        Ok(())
    }

    /// change_end / change_meta / change_rcv / remove_ballot.
    #[tokio::test]
    #[test_log::test]
    async fn ballot_admin_updates() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["CbAdminOwner"]).await?.into_iter();
        let mut owner = users.next().unwrap();

        let asset_id = create_asset(&mut tester, &mut owner, "CBADM", 1_000_000).await?;
        let mut cp_res = tester
            .api
            .call()
            .checkpoint()
            .create_checkpoint(asset_id.clone())?
            .submit_and_watch(&mut owner)
            .await?;
        cp_res.ok().await?;
        let cp_id = get_checkpoint_id(&mut cp_res).await?.expect("checkpoint");
        let now = tester.api.query().timestamp().now().await?;
        let mut ca_res = tester
            .api
            .call()
            .corporate_action()
            .initiate_corporate_action(
                asset_id,
                CAKind::IssuerNotice,
                now,
                Some(RecordDateSpec::Existing(cp_id)),
                CADetails(b"notice".to_vec()),
                None,
                None,
                None,
            )?
            .submit_and_watch(&mut owner)
            .await?;
        ca_res.ok().await?;
        let ca_id = get_ca_id(&mut ca_res).await?.expect("ca id");

        // Start in the future so admin updates are still allowed.
        let range = BallotTimeRange {
            start: now + 30_000,
            end: now + 90_000,
        };
        tester
            .api
            .call()
            .corporate_ballot()
            .attach_ballot(ca_id.clone(), range, ballot_meta("Draft"), false)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        tester
            .api
            .call()
            .corporate_ballot()
            .change_end(ca_id.clone(), now + 120_000)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;
        tester
            .api
            .call()
            .corporate_ballot()
            .change_meta(ca_id.clone(), ballot_meta("Updated"))?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;
        tester
            .api
            .call()
            .corporate_ballot()
            .change_rcv(ca_id.clone(), true)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;
        tester
            .api
            .call()
            .corporate_ballot()
            .remove_ballot(ca_id)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        Ok(())
    }
}