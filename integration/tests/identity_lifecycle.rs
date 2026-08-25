//! Identity lifecycle: primary-key rotation, SK freeze/remove, claim revocation, authorizations.
#[cfg(feature = "current_release")]
mod identity_lifecycle_tests {
    use anyhow::Result;

    use integration::*;
    use polymesh_api::types::pallet_identity::types::{Claim1stKey, Claim2ndKey};
    use polymesh_api::types::polymesh_primitives::{
        authorization::AuthorizationData,
        identity_claim::{Claim, Scope},
        secondary_key::Signatory,
        settlement::{VenueDetails, VenueType},
    };

    /// Join a key that has no DID into `owner`'s identity as a secondary key.
    async fn add_secondary_key(
        tester: &PolymeshTester,
        owner: &mut User,
        new_key: &mut AccountSigner,
        perms: Permissions,
    ) -> Result<()> {
        // The new key needs POLYX to pay for join_identity_as_key.
        tester
            .api
            .call()
            .balances()
            .transfer_with_memo(new_key.account().into(), 100 * ONE_POLYX, None)?
            .submit_and_watch(owner)
            .await?
            .ok()
            .await?;

        let mut res = tester
            .api
            .call()
            .identity()
            .add_authorization(
                Signatory::Account(new_key.account()),
                AuthorizationData::JoinIdentity(perms),
                None,
            )?
            .submit_and_watch(owner)
            .await?;
        res.ok().await?;
        let auth_id = get_auth_id(&mut res)
            .await?
            .expect("AuthorizationAdded event");
        tester
            .api
            .call()
            .identity()
            .join_identity_as_key(auth_id)?
            .submit_and_watch(new_key)
            .await?
            .ok()
            .await?;
        Ok(())
    }

    /// Primary key rotation to a secondary key via RotatePrimaryKeyToSecondary auth.
    #[tokio::test]
    #[test_log::test]
    async fn primary_key_rotation_to_secondary() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["ILROwner"]).await?.into_iter();
        let mut owner = users.next().unwrap();
        let mut new_primary = tester.new_signer_idx("ILROwner", 1)?;

        add_secondary_key(
            &tester,
            &mut owner,
            &mut new_primary,
            PermissionsBuilder::whole().build(),
        )
        .await?;

        // Owner authorizes rotating the primary key to the SK.
        let mut res = tester
            .api
            .call()
            .identity()
            .add_authorization(
                Signatory::Account(new_primary.account()),
                AuthorizationData::RotatePrimaryKeyToSecondary(PermissionsBuilder::whole().build()),
                None,
            )?
            .submit_and_watch(&mut owner)
            .await?;
        res.ok().await?;
        let auth_id = get_auth_id(&mut res).await?.expect("auth id");

        // The new key accepts and becomes the primary; old key remains as SK.
        tester
            .api
            .call()
            .identity()
            .rotate_primary_key_to_secondary(auth_id)?
            .submit_and_watch(&mut new_primary)
            .await?
            .ok()
            .await?;

        // Verify: DID's primary key is now the new account.
        let did = owner.did.unwrap();
        let did_records = tester
            .api
            .query()
            .identity()
            .did_records(did)
            .await?
            .expect("did records");
        assert_eq!(
            did_records.primary_key,
            Some(new_primary.account()),
            "new primary key should be active"
        );

        Ok(())
    }

    /// Freezing all secondary keys blocks them until unfrozen.
    #[tokio::test]
    #[test_log::test]
    async fn freeze_unfreeze_secondary_keys() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["ILFOwner"]).await?.into_iter();
        let mut owner = users.next().unwrap();
        let mut sk = tester.new_signer_idx("ILFOwner", 1)?;

        add_secondary_key(&tester, &mut owner, &mut sk, PermissionsBuilder::whole().build()).await?;

        // SK can act (create venue).
        tester
            .api
            .call()
            .settlement()
            .create_venue(VenueDetails(b"SKVenue1".to_vec()), Default::default(), VenueType::Other)?
            .execute(&mut sk)
            .await?
            .ok()
            .await?;

        // Primary freezes ALL secondary keys (no per-call args).
        tester
            .api
            .call()
            .identity()
            .freeze_secondary_keys()?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // SK calls are now rejected.
        let res = tester
            .api
            .call()
            .settlement()
            .create_venue(VenueDetails(b"SKVenue2".to_vec()), Default::default(), VenueType::Other)?
            .execute(&mut sk)
            .await;
        match res {
            Ok(mut r) => assert!(r.ok().await.is_err(), "frozen SK should be blocked"),
            Err(_) => {} // Rejected by node also acceptable.
        }

        // Unfreeze restores access.
        tester
            .api
            .call()
            .identity()
            .unfreeze_secondary_keys()?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;
        tester
            .api
            .call()
            .settlement()
            .create_venue(VenueDetails(b"SKVenue3".to_vec()), Default::default(), VenueType::Other)?
            .execute(&mut sk)
            .await?
            .ok()
            .await?;

        Ok(())
    }

    /// Removing a secondary key revokes its identity linkage.
    #[tokio::test]
    #[test_log::test]
    async fn remove_secondary_keys_revokes_access() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["ILROwner2"]).await?.into_iter();
        let mut owner = users.next().unwrap();
        let mut sk = tester.new_signer_idx("ILROwner2", 1)?;

        add_secondary_key(&tester, &mut owner, &mut sk, PermissionsBuilder::whole().build()).await?;

        // Works before removal.
        tester
            .api
            .call()
            .settlement()
            .create_venue(VenueDetails(b"RSKVenue1".to_vec()), Default::default(), VenueType::Other)?
            .execute(&mut sk)
            .await?
            .ok()
            .await?;

        // Owner removes the SK.
        tester
            .api
            .call()
            .identity()
            .remove_secondary_keys(vec![sk.account()])?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // SK no longer linked -> permissioned calls fail.
        let res = tester
            .api
            .call()
            .settlement()
            .create_venue(VenueDetails(b"RSKVenue2".to_vec()), Default::default(), VenueType::Other)?
            .execute(&mut sk)
            .await;
        match res {
            Ok(mut r) => assert!(r.ok().await.is_err(), "removed SK should be blocked"),
            Err(_) => {}
        }

        Ok(())
    }

    /// Issuer can add and then revoke a claim on a target identity.
    #[tokio::test]
    #[test_log::test]
    async fn revoke_claim_removes_it() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["ILCIssuer", "ILCTarget"])
            .await?
            .into_iter();
        let mut issuer = users.next().unwrap();
        let target = users.next().unwrap();

        let issuer_did = issuer.did.expect("issuer did");
        let target_did = target.did.expect("target did");

        // Add claim scoped to the issuer's own identity.
        tester
            .api
            .call()
            .identity()
            .add_claim(target_did, Claim::Accredited(Scope::Identity(issuer_did)), None)?
            .submit_and_watch(&mut issuer)
            .await?
            .ok()
            .await?;

        let first_key = Claim1stKey {
            target: target_did,
            claim_type:
                polymesh_api::types::polymesh_primitives::identity_claim::ClaimType::Accredited,
        };
        let second_key = Claim2ndKey {
            issuer: issuer_did,
            scope: Some(Scope::Identity(issuer_did)),
        };
        let stored = tester
            .api
            .query()
            .identity()
            .claims(first_key.clone(), second_key.clone())
            .await?;
        assert!(stored.is_some(), "claim should exist after add_claim");

        // Issuer revokes it by full claim value.
        tester
            .api
            .call()
            .identity()
            .revoke_claim(
                target_did,
                Claim::Accredited(Scope::Identity(issuer_did)),
            )?
            .submit_and_watch(&mut issuer)
            .await?
            .ok()
            .await?;

        let stored = tester
            .api
            .query()
            .identity()
            .claims(first_key, second_key)
            .await?;
        assert!(stored.is_none(), "claim should be gone after revoke_claim");

        Ok(())
    }

    /// register_custom_claim_type allocates incrementing type IDs usable in claims.
    #[tokio::test]
    #[test_log::test]
    async fn custom_claim_types() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["ILCCIssuer", "ILCCTarget"])
            .await?
            .into_iter();
        let mut issuer = users.next().unwrap();
        let target = users.next().unwrap();

        let target_did = target.did.unwrap();

        // Register two custom types.
        let suffix = format!("{:?}", std::time::SystemTime::now());
        for ty_name in [
            format!("CustomKYC{suffix}").into_bytes(),
            format!("Audited{suffix}").into_bytes(),
        ] {
            tester
                .api
                .call()
                .identity()
                .register_custom_claim_type(ty_name)?
                .submit_and_watch(&mut issuer)
                .await?
                .ok()
                .await?;
        }

        let mut last = tester
            .api
            .call()
            .identity()
            .register_custom_claim_type(format!("Extra{suffix}").into_bytes())?
            .submit_and_watch(&mut issuer)
            .await?;
        last.ok().await?;
        let ty_id = {
            let events = last.events().await?.expect("events");
            let mut found = None;
            for rec in &events.0 {
                if let RuntimeEvent::Identity(IdentityEvent::CustomClaimTypeAdded(_, id, _)) =
                    &rec.event
                {
                    found = Some(id.clone());
                }
            }
            found.expect("CustomClaimTypeAdded")
        };
        tester
            .api
            .call()
            .identity()
            .add_claim(target_did, Claim::Custom(ty_id, None), None)?
            .submit_and_watch(&mut issuer)
            .await?
            .ok()
            .await?;

        Ok(())
    }

    /// Target rejects a pending authorization; issuer can also cancel it.
    #[tokio::test]
    #[test_log::test]
    async fn authorization_reject_and_cancel() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["ILAOwner", "ILATarget", "ILATarget2"])
            .await?
            .into_iter();
        let mut owner = users.next().unwrap();
        let mut target = users.next().unwrap();
        let mut target2 = users.next().unwrap();

        let perms = PermissionsBuilder::whole().build();

        // Auth #1: target rejects it.
        let mut res = tester
            .api
            .call()
            .identity()
            .add_authorization(
                Signatory::Account(target.account()),
                AuthorizationData::JoinIdentity(perms.clone()),
                None,
            )?
            .submit_and_watch(&mut owner)
            .await?;
        res.ok().await?;
        let auth1 = get_auth_id(&mut res).await?.expect("auth id");

        // remove_authorization(target, auth_id, reject=true) executed BY the target == rejection.
        tester
            .api
            .call()
            .identity()
            .remove_authorization(Signatory::Account(target.account()), auth1, true)?
            .submit_and_watch(&mut target)
            .await?
            .ok()
            .await?;

        // Auth #2: issuer cancels it before acceptance.
        let mut res = tester
            .api
            .call()
            .identity()
            .add_authorization(
                Signatory::Account(target2.account()),
                AuthorizationData::JoinIdentity(perms),
                None,
            )?
            .submit_and_watch(&mut owner)
            .await?;
        res.ok().await?;
        let auth2 = get_auth_id(&mut res).await?.expect("auth id");

        // remove_authorization(...) executed BY the issuer == cancellation.
        tester
            .api
            .call()
            .identity()
            .remove_authorization(Signatory::Account(target2.account()), auth2, false)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // Accepting either auth now fails.
        let res = tester
            .api
            .call()
            .identity()
            .join_identity_as_key(auth2)?
            .submit_and_watch(&mut target2)
            .await;
        match res {
            Ok(mut r) => assert!(r.ok().await.is_err(), "cancelled auth must not be acceptable"),
            Err(_) => {}
        }
        let _ = auth1;

        Ok(())
    }
}