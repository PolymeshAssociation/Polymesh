#[cfg(feature = "current_release")]
mod portfolio_tests {
    use std::collections::BTreeSet;

    use anyhow::Result;
    use integration::*;
    use polymesh_api::types::polymesh_primitives::{
        identity_id::{PortfolioId, PortfolioKind, PortfolioName},
        portfolio::{Fund, FundDescription},
    };

    #[tokio::test]
    #[test_log::test]
    async fn portfolio_create_and_move() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["PortfolioOwner"])
            .await?
            .into_iter();
        let mut owner = users.next().expect("PortfolioOwner");

        // Create an asset with some tokens to move
        let asset_helper = AssetHelper::new(
            &tester.api,
            &mut owner,
            "PortfolioTestAsset",
            1_000_000,
            BTreeSet::new(),
        )
        .await?;
        let asset_id = asset_helper.asset_id;

        let owner_did = owner.did.expect("owner did");

        // Create a new portfolio
        let mut res = tester
            .api
            .call()
            .portfolio()
            .create_portfolio(PortfolioName(b"MyPortfolio".to_vec()))?
            .submit_and_watch(&mut owner)
            .await?;
        res.ok().await?;

        // The new portfolio will have number 1 (default is 0, first custom is 1)
        let default_portfolio = PortfolioId {
            did: owner_did,
            kind: PortfolioKind::Default,
        };
        let user_portfolio = PortfolioId {
            did: owner_did,
            kind: PortfolioKind::User(polymesh_api::types::polymesh_primitives::identity_id::PortfolioNumber(1)),
        };

        // Move some tokens from default portfolio to the new portfolio
        tester
            .api
            .call()
            .portfolio()
            .move_portfolio_funds(
                default_portfolio,
                user_portfolio,
                vec![Fund {
                    description: FundDescription::Fungible {
                        asset_id,
                        amount: 100_000,
                    },
                    memo: None,
                }],
            )?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        Ok(())
    }
}
