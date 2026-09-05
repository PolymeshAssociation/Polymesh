//! Portfolio custody, creation permissions and pre-approvals.
#[cfg(feature = "current_release")]
mod portfolio_custody_tests {
    use std::collections::BTreeSet;

    use anyhow::Result;

    use integration::*;
    use polymesh_api::types::polymesh_primitives::{
        asset::AssetHolderKind,
        identity_id::{PortfolioId, PortfolioKind, PortfolioName, PortfolioNumber},
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

    fn user_pf(did: IdentityId, n: u64) -> PortfolioId {
        PortfolioId {
            did,
            kind: PortfolioKind::User(PortfolioNumber(n)),
        }
    }

    /// create_custody_portfolio hands custody to the creator until accepted/quit.
    #[tokio::test]
    #[test_log::test]
    async fn custody_portfolio_lifecycle() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["PCOwner", "PCCustodian"]).await?.into_iter();
        let mut owner = users.next().unwrap();
        let mut custodian = users.next().unwrap();

        let owner_did = owner.did.unwrap();
        let custodian_did = custodian.did.unwrap();

        tester
            .api
            .call()
            .portfolio()
            .allow_identity_to_create_portfolios(custodian_did)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // Custodian creates a portfolio owned by `owner` but custodied by itself.
        tester
            .api
            .call()
            .portfolio()
            .create_custody_portfolio(owner_did, PortfolioName(b"Custodied".to_vec()))?
            .submit_and_watch(&mut custodian)
            .await?
            .ok()
            .await?;

        // The first custodied portfolio under the owner gets number 1.
        let pid = user_pf(owner_did, 1);
        let custodian_of = tester
            .api
            .query()
            .portfolio()
            .portfolios_in_custody(custodian_did, pid.clone())
            .await?;
        assert!(
            custodian_of,
            "custodian should hold custody of the new portfolio"
        );

        // Custodian quits; ownership returns fully to the owner.
        tester
            .api
            .call()
            .portfolio()
            .quit_portfolio_custody(pid.clone())?
            .submit_and_watch(&mut custodian)
            .await?
            .ok()
            .await?;

        let still_custodied = tester
            .api
            .query()
            .portfolio()
            .portfolios_in_custody(custodian_did, pid)
            .await?;
        assert!(!still_custodied, "custody should be released after quit");

        Ok(())
    }

    /// Only identities granted permission can create custodied portfolios.
    #[tokio::test]
    #[test_log::test]
    async fn create_portfolios_permission() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["PCPOwner", "PCPDelegate"])
            .await?
            .into_iter();
        let mut owner = users.next().unwrap();
        let mut delegate = users.next().unwrap();

        let owner_did = owner.did.unwrap();
        let delegate_did = delegate.did.unwrap();

        // Grant permission...
        tester
            .api
            .call()
            .portfolio()
            .allow_identity_to_create_portfolios(delegate_did)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // ...delegate can now create a custodied portfolio under owner's DID.
        tester
            .api
            .call()
            .portfolio()
            .create_custody_portfolio(owner_did, PortfolioName(b"Delegated".to_vec()))?
            .submit_and_watch(&mut delegate)
            .await?
            .ok()
            .await?;

        // Revoke permission...
        tester
            .api
            .call()
            .portfolio()
            .revoke_create_portfolios_permission(delegate_did)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // ...further custodied creations fail.
        let mut res = tester
            .api
            .call()
            .portfolio()
            .create_custody_portfolio(owner_did, PortfolioName(b"Delegated2".to_vec()))?
            .submit_and_watch(&mut delegate)
            .await?;
        assert!(
            res.ok().await.is_err(),
            "revoked delegate must not create portfolios"
        );

        Ok(())
    }

    /// Pre-approving an asset for a portfolio skips receiver affirmation friction.
    #[tokio::test]
    #[test_log::test]
    async fn portfolio_pre_approval() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["PPAOwner", "PPAInv1"]).await?.into_iter();
        let mut owner = users.next().unwrap();
        let mut inv1 = users.next().unwrap();

        let asset_id = create_asset(&mut tester, &mut owner, "PPAAPPR", 10_000).await?;
        let inv_did = inv1.did.unwrap();
        let pf1 = PortfolioId {
            did: inv_did,
            kind: PortfolioKind::Default,
        };

        // Pre-approve the asset for the investor's default portfolio.
        tester
            .api
            .call()
            .portfolio()
            .pre_approve_portfolio(asset_id.clone(), pf1.clone())?
            .execute(&mut inv1)
            .await?
            .ok()
            .await?;

        let approved = tester
            .api
            .query()
            .portfolio()
            .pre_approved_portfolios(pf1.clone(), asset_id.clone())
            .await?;
        assert!(approved, "portfolio should be pre-approved");

        // Remove it again.
        tester
            .api
            .call()
            .portfolio()
            .remove_portfolio_pre_approval(asset_id.clone(), pf1.clone())?
            .execute(&mut inv1)
            .await?
            .ok()
            .await?;

        let approved = tester
            .api
            .query()
            .portfolio()
            .pre_approved_portfolios(pf1, asset_id)
            .await?;
        assert!(!approved, "pre-approval removed");

        Ok(())
    }

    /// Rename then delete (empty) custom portfolios.
    #[tokio::test]
    #[test_log::test]
    async fn rename_then_delete_portfolio() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["PRDUser"]).await?.into_iter();
        let mut user = users.next().unwrap();
        let _ = create_asset(&mut tester, &mut user, "PRDUMMY", 100).await?; // warm up fees

        // Create custom portfolio #1.
        tester
            .api
            .call()
            .portfolio()
            .create_portfolio(PortfolioName(b"Original".to_vec()))?
            .submit_and_watch(&mut user)
            .await?
            .ok()
            .await?;

        let did = user.did.unwrap();

        // Rename.
        tester
            .api
            .call()
            .portfolio()
            .rename_portfolio(PortfolioNumber(1), PortfolioName(b"Renamed".to_vec()))?
            .execute(&mut user)
            .await?
            .ok()
            .await?;

        let name = tester
            .api
            .query()
            .portfolio()
            .portfolios(did, PortfolioNumber(1))
            .await?
            .expect("portfolio name");
        assert_eq!(name, PortfolioName(b"Renamed".to_vec()));

        // Delete (must be empty).
        tester
            .api
            .call()
            .portfolio()
            .delete_portfolio(PortfolioNumber(1))?
            .execute(&mut user)
            .await?
            .ok()
            .await?;

        let name = tester
            .api
            .query()
            .portfolio()
            .portfolios(did, PortfolioNumber(1))
            .await?;
        assert!(name.is_none(), "deleted portfolio should have no name");

        Ok(())
    }
}
