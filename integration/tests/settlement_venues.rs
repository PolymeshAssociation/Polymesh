//! Settlement venues: CRUD + per-asset venue filtering.
#[cfg(feature = "current_release")]
mod settlement_venues_tests {
    use std::collections::BTreeSet;

    use anyhow::Result;

    use integration::*;
    use polymesh_api::types::polymesh_primitives::{
        asset::AssetHolderKind,
        settlement::{VenueDetails, VenueId, VenueType},
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

    async fn create_venue(
        tester: &PolymeshTester,
        user: &mut User,
        name: &str,
    ) -> Result<VenueId> {
        let mut res = tester
            .api
            .call()
            .settlement()
            .create_venue(VenueDetails(name.as_bytes().to_vec()), Default::default(), VenueType::Other)?
            .submit_and_watch(user)
            .await?;
        res.ok().await?;
        get_venue_id(&mut res)
            .await?
            .ok_or_else(|| anyhow::anyhow!("VenueCreated event not found"))
    }

    /// Venue details and type are updatable by the owner.
    #[tokio::test]
    #[test_log::test]
    async fn update_details_and_type() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["SVOwner"]).await?.into_iter();
        let mut owner = users.next().unwrap();

        let venue_id = create_venue(&tester, &mut owner, "SVOriginal").await?;

        // Update details.
        let new_details = VenueDetails(b"SVUpdated".to_vec());
        tester
            .api
            .call()
            .settlement()
            .update_venue_details(venue_id, new_details.clone())?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        let details = tester.api.query().settlement().details(venue_id).await?;
        assert_eq!(details, new_details, "venue details should be updated");

        // Update type to Exchange.
        tester
            .api
            .call()
            .settlement()
            .update_venue_type(venue_id, VenueType::Exchange)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        let venue = tester
            .api
            .query()
            .settlement()
            .venue_info(venue_id)
            .await?
            .expect("venue info");
        assert_eq!(venue.venue_type, VenueType::Exchange);

        Ok(())
    }

    /// Venue signers can be added & removed; only signers may affirm for the venue.
    #[tokio::test]
    #[test_log::test]
    async fn update_signers() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["SVOwner2", "SVSigner1", "SVSigner2"])
            .await?
            .into_iter();
        let mut owner = users.next().unwrap();
        let signer1 = users.next().unwrap();
        let signer2 = users.next().unwrap();

        let venue_id = create_venue(&tester, &mut owner, "SVSigners").await?;

        // Add both signers.
        tester
            .api
            .call()
            .settlement()
            .update_venue_signers(
                venue_id,
                BTreeSet::from([signer1.account(), signer2.account()]),
                true, // add
            )?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        let count1 = tester
            .api
            .query()
            .settlement()
            .number_of_venue_signers(venue_id)
            .await?;
        assert_eq!(count1, 2, "two added signers");

        // Remove signer2 again.
        tester
            .api
            .call()
            .settlement()
            .update_venue_signers(venue_id, BTreeSet::from([signer2.account()]), false)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        let count2 = tester
            .api
            .query()
            .settlement()
            .number_of_venue_signers(venue_id)
            .await?;
        assert_eq!(count2, 1, "signer2 removed");

        Ok(())
    }

    /// Per-asset venue allow-listing gates which venues may settle its instructions.
    #[tokio::test]
    #[test_log::test]
    async fn venue_filtering_allow_disallow() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["SVOwner3", "SVVenueUser"])
            .await?
            .into_iter();
        let mut asset_owner = users.next().unwrap();
        let mut venue_user = users.next().unwrap();

        let asset_id = create_asset(&mut tester, &mut asset_owner, "SVFILTER", 1_000).await?;

        let v1 = create_venue(&tester, &mut venue_user, "SVAllowed").await?;
        let v2 = create_venue(&tester, &mut venue_user, "SVBlocked").await?;

        // Enable allow-list filtering for the asset.
        tester
            .api
            .call()
            .settlement()
            .set_venue_filtering(asset_id.clone(), true)?
            .submit_and_watch(&mut asset_owner)
            .await?
            .ok()
            .await?;

        // Nothing allowed yet: v1 not in the list.
        let allowed_v1 = tester
            .api
            .query()
            .settlement()
            .venue_allow_list(asset_id.clone(), v1)
            .await?;
        assert!(!allowed_v1, "freshly filtered asset allows nobody");

        // Allow v1.
        tester
            .api
            .call()
            .settlement()
            .allow_venues(asset_id.clone(), vec![v1])?
            .submit_and_watch(&mut asset_owner)
            .await?
            .ok()
            .await?;
        assert!(
            tester
                .api
                .query()
                .settlement()
                .venue_allow_list(asset_id.clone(), v1)
                .await?,
            "v1 should now be allowed"
        );
        assert!(
            !tester
                .api
                .query()
                .settlement()
                .venue_allow_list(asset_id.clone(), v2)
                .await?,
            "v2 must remain blocked"
        );

        // Disallow v1 again.
        tester
            .api
            .call()
            .settlement()
            .disallow_venues(asset_id.clone(), vec![v1])?
            .submit_and_watch(&mut asset_owner)
            .await?
            .ok()
            .await?;
        assert!(
            !tester
                .api
                .query()
                .settlement()
                .venue_allow_list(asset_id, v1)
                .await?,
            "v1 disallowed"
        );

        Ok(())
    }
}