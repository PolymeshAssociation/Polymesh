use anyhow::{anyhow, Result};

use polymesh_api::{
    types::polymesh_primitives::{
        authorization::AuthorizationData,
        secondary_key::Signatory,
        settlement::{VenueDetails, VenueType},
    },
    TransactionResults, WrappedCall,
};
use sp_weights::Weight;

use integration::*;

async fn get_ms_proposal_id(res: &mut TransactionResults) -> Result<u64> {
    let events = res.events().await?.expect("Failed to create MS proposal");
    for rec in &events.0 {
        match &rec.event {
            RuntimeEvent::MultiSig(MultiSigEvent::ProposalAdded { proposal_id, .. }) => {
                return Ok(*proposal_id);
            }
            _ => (),
        }
    }
    Err(anyhow!("Failed to get new MS proposal ID."))
}

async fn ms_proposal_executed(res: &mut TransactionResults) -> Result<Option<bool>> {
    let events = res.events().await?.expect("Failed to approve MS proposal");
    for rec in &events.0 {
        match &rec.event {
            RuntimeEvent::MultiSig(MultiSigEvent::ProposalExecuted { result, .. }) => {
                return Ok(Some(result.is_ok()));
            }
            _ => (),
        }
    }
    Ok(None)
}

pub struct MuliSigState {
    api: Api,
    creator: AccountSigner,
    account: AccountId,
    signers: Vec<AccountSigner>,
    sigs_required: u64,
}

impl MuliSigState {
    pub async fn create_and_join_creator(
        tester: &mut PolymeshTester,
        creator: &AccountSigner,
        n_signers: usize,
        sigs_required: u64,
        as_primary: bool,
        perms: Option<Permissions>,
    ) -> Result<MuliSigState> {
        let mut ms = Self::create(tester, creator, n_signers, sigs_required, perms).await?;
        if as_primary {
            let mut res = ms.make_primary().await?;
            res.wait_in_block().await?;
        }

        Ok(ms)
    }

    pub async fn create(
        tester: &mut PolymeshTester,
        creator: &AccountSigner,
        n_signers: usize,
        sigs_required: u64,
        perms: Option<Permissions>,
    ) -> Result<MuliSigState> {
        let mut creator = creator.clone();
        let mut signers = Vec::with_capacity(n_signers);
        let signer_name_prefix = format!("{:?}Signer", creator.account());
        for idx in 0..n_signers {
            signers.push(tester.new_signer_idx(&signer_name_prefix, idx)?);
        }

        let mut res = tester
            .api
            .call()
            .multi_sig()
            .create_multisig(
                signers.iter().map(|s| s.account()).collect(),
                sigs_required,
                perms,
            )?
            .submit_and_watch(&mut creator)
            .await?;
        // Wait for finalization.
        res.wait_finalized().await?;

        let mut results = Vec::new();

        // Get MS address and join signers to MS.
        let mut account = None;
        let events = res
            .events()
            .await?
            .expect("Failed to get events from MS creation");
        for rec in &events.0 {
            match &rec.event {
                RuntimeEvent::Identity(IdentityEvent::AuthorizationAdded(
                    _,
                    _,
                    Some(account),
                    auth_id,
                    ..,
                )) => {
                    for signer in &mut signers {
                        // Find matching signer.
                        if signer.account() == *account {
                            // Join MS by accepting the authorization.
                            let res = tester
                                .api
                                .call()
                                .multi_sig()
                                .accept_multisig_signer(*auth_id)?
                                .submit_and_watch(signer)
                                .await?;
                            results.push(res);
                        }
                    }
                }
                RuntimeEvent::MultiSig(MultiSigEvent::MultiSigCreated { multisig, .. }) => {
                    account = Some(*multisig);
                }
                _ => (),
            }
        }

        for mut res in results {
            res.wait_in_block().await?;
        }

        Ok(MuliSigState {
            api: tester.api.clone(),
            creator,
            account: account.expect("Failed to get MS address"),
            signers,
            sigs_required,
        })
    }

    pub async fn leave_did(&mut self) -> Result<TransactionResults> {
        let leave_did_call = self.api.call().identity().leave_identity_as_key()?;
        self.run_proposal(leave_did_call).await
    }

    pub async fn remove_payer(&mut self) -> Result<TransactionResults> {
        let remove_payer_call = self.api.call().multi_sig().remove_payer()?;
        self.run_proposal(remove_payer_call).await
    }

    pub async fn remove_payer_via_payer(&mut self) -> Result<TransactionResults> {
        let res = self
            .api
            .call()
            .multi_sig()
            .remove_payer_via_payer(self.account.clone())?
            .execute(&mut self.creator)
            .await?;
        Ok(res)
    }

    pub async fn make_primary(&mut self) -> Result<TransactionResults> {
        let mut res = self
            .api
            .call()
            .identity()
            .add_authorization(
                Signatory::Account(self.account.clone()),
                AuthorizationData::RotatePrimaryKey,
                None,
            )?
            .execute(&mut self.creator)
            .await?;
        let auth_id = get_auth_id(&mut res)
            .await?
            .expect("Missing RotatePrimaryKey auth id");

        let rotate_primary_call = self
            .api
            .call()
            .identity()
            .accept_primary_key(auth_id, None)?;
        self.run_proposal(rotate_primary_call).await
    }

    pub async fn make_secondary(
        &mut self,
        permissions: Option<Permissions>,
    ) -> Result<TransactionResults> {
        let perms = permissions.unwrap_or_else(|| PermissionsBuilder::empty().build());
        let mut res = self
            .api
            .call()
            .identity()
            .add_authorization(
                Signatory::Account(self.account.clone()),
                AuthorizationData::JoinIdentity(perms),
                None,
            )?
            .execute(&mut self.creator)
            .await?;
        let auth_id = get_auth_id(&mut res)
            .await?
            .expect("Missing JoinIdentity auth id");

        let join_did_call = self.api.call().identity().join_identity_as_key(auth_id)?;
        self.run_proposal(join_did_call).await
    }

    pub async fn create_proposal(&mut self, proposal: WrappedCall) -> Result<TransactionResults> {
        let res = self
            .api
            .call()
            .multi_sig()
            .create_proposal(self.account.clone(), proposal.runtime_call().clone(), None)?
            .submit_and_watch(&mut self.signers[0])
            .await?;
        Ok(res)
    }

    pub async fn join_identity(&mut self, auth_id: u64) -> Result<TransactionResults> {
        let approve_join_call = self
            .api
            .call()
            .multi_sig()
            .approve_join_identity(self.account.clone(), auth_id)?;
        let mut results = Vec::new();
        for signer in &mut self.signers[0..self.sigs_required as usize] {
            let res = approve_join_call.submit_and_watch(signer).await?;
            results.push(res);
        }
        // Find which approval call executed the proposal.
        for mut res in results {
            res.wait_finalized().await?;
            match ms_proposal_executed(&mut res).await? {
                Some(true) => {
                    return Ok(res);
                }
                Some(false) => {
                    return Err(anyhow!("MS proposal returned error"));
                }
                None => (),
            }
        }
        Err(anyhow!("Failed to execute MS proposal"))
    }

    pub async fn run_proposal(&mut self, proposal: WrappedCall) -> Result<TransactionResults> {
        // Create proposal with the first signer.
        let mut res = self.create_proposal(proposal).await?;
        res.wait_finalized().await?;
        if self.sigs_required > 1 {
            let id = get_ms_proposal_id(&mut res).await?;
            let weight = Weight::from_parts(10_000_000_000, 0);
            let approve_call =
                self.api
                    .call()
                    .multi_sig()
                    .approve(self.account.clone(), id, Some(weight))?;
            let mut results = Vec::new();
            for signer in &mut self.signers[1..self.sigs_required as usize] {
                let res = approve_call.submit_and_watch(signer).await?;
                results.push(res);
            }
            // Find which approval call executed the proposal.
            for mut res in results {
                res.wait_finalized().await?;
                match ms_proposal_executed(&mut res).await? {
                    Some(true) => {
                        return Ok(res);
                    }
                    Some(false) => {
                        return Err(anyhow!("MS proposal returned error"));
                    }
                    None => (),
                }
            }
            Err(anyhow!("Failed to execute MS proposal"))
        } else {
            Ok(res)
        }
    }

    pub async fn set_ms_key_permissions(
        &mut self,
        permissions: impl Into<Permissions> + Send,
    ) -> Result<TransactionResults> {
        let permissions = permissions.into();
        let res = self
            .api
            .call()
            .identity()
            .set_secondary_key_permissions(self.account, permissions)?
            .submit_and_watch(&mut self.creator)
            .await?;
        Ok(res)
    }

    pub async fn ensure_ms_key_permissions(
        &self,
        permissions: impl Into<Permissions> + Send,
    ) -> Result<()> {
        let permissions = permissions.into();
        let asset = self
            .api
            .query()
            .identity()
            .key_asset_permissions(self.account)
            .await?
            .ok_or_else(|| anyhow!("Missing asset permissions"))?;
        assert_eq!(permissions.asset, asset);
        let portfolio = self
            .api
            .query()
            .identity()
            .key_portfolio_permissions(self.account)
            .await?
            .ok_or_else(|| anyhow!("Missing portfolio permissions"))?;
        assert_eq!(permissions.portfolio, portfolio);
        let extrinsic = self
            .api
            .query()
            .identity()
            .key_extrinsic_permissions(self.account)
            .await?
            .ok_or_else(|| anyhow!("Missing extrinsic permissions"))?;
        assert_eq!(permissions.extrinsic, extrinsic);
        Ok(())
    }
}

#[tokio::test]
async fn multisig_as_secondary_key_change_identity() -> Result<()> {
    let mut tester = PolymeshTester::new().await?;
    let users = tester
        .users(&["MultiSigSecondaryDID1", "MultiSigSecondaryDID2"])
        .await?;
    let did1 = users[0].clone();
    let mut did2 = users[1].clone();

    // Use the primary key of did1 to create a MS and join it do did1 as a secondary key.
    let whole = PermissionsBuilder::whole();
    let mut ms = MuliSigState::create_and_join_creator(
        &mut tester,
        &did1.primary_key,
        3,
        2,
        false,
        Some(whole.into()),
    )
    .await?;

    // Create JoinIdentity auth for the MS to join DID2 with no-permissions.
    let mut res = tester
        .api
        .call()
        .identity()
        .add_authorization(
            Signatory::Account(ms.account.clone()),
            AuthorizationData::JoinIdentity(PermissionsBuilder::empty().build()),
            None,
        )?
        .execute(&mut did2)
        .await?;
    let auth_id = get_auth_id(&mut res)
        .await?
        .expect("Missing JoinIdentity auth id");

    // Prepare `system.remark` call.
    let remark_call = tester.api.call().system().remark(vec![])?;
    // Prepare `settlement.create_venue` call.
    let create_venue_call = tester.api.call().settlement().create_venue(
        VenueDetails(vec![]),
        vec![],
        VenueType::Other,
    )?;
    // Prepare `identity.leave_identity_as_key` call.
    let leave_did1_call = tester.api.call().identity().leave_identity_as_key()?;
    // Prepare `identity.join_identity_as_key` call.
    let join_did2_call = tester.api.call().identity().join_identity_as_key(auth_id)?;

    let expected = vec![
        true,  // remark.
        true,  // create venue.
        true,  // leave did1.
        true,  // remark.
        false, // create venue.
        true,  // join did2.
        true,  // remark.
        false, // create venue.
    ];
    let batch_call = tester.api.call().utility().force_batch(vec![
        remark_call.runtime_call().clone(),
        create_venue_call.runtime_call().clone(),
        leave_did1_call.into(), // The secondary key should have no identity here.
        remark_call.runtime_call().clone(),
        // The key shouldn't be allowed to create venues.  Has no identity.
        create_venue_call.runtime_call().clone(),
        // The key should be allowed to join DID2.
        join_did2_call.into(), // The key is now a secondar key of DID2 with no permissions.
        remark_call.runtime_call().clone(),
        // The secondary key shouldn't be allowed to create venues.  Has no call permissions.
        create_venue_call.runtime_call().clone(),
    ])?;

    let mut res = ms.run_proposal(batch_call).await?;
    res.ok().await?;
    let calls_ok = get_batch_results(&mut res).await?;
    assert_eq!(calls_ok, expected);

    Ok(())
}

#[tokio::test]
async fn secondary_key_ms_make_primary() -> Result<()> {
    let mut tester = PolymeshTester::new().await?;
    // Create a user with secondary keys to be the MS creator.
    let users = tester
        .users_with_secondary_keys(&[("MS_make_primary_with_sk", 1)])
        .await?;
    let mut did1 = users[0].clone();
    let sk = did1.get_sk(0)?.clone();

    // Create a MS as a secondary key with no permissions.
    let mut ms = MuliSigState::create_and_join_creator(&mut tester, &sk, 3, 2, false, None).await?;

    // Need to fund the MultiSig so that it can't pay
    // transaction fees.
    let mut res_transfer = tester
        .api
        .call()
        .balances()
        .transfer(ms.account.into(), 10 * ONE_POLYX)?
        .execute(&mut did1)
        .await?;

    // A secondary key can make the MS a primary.  If it
    // has permissions to create the authorization.
    let mut res_make_primary = ms.make_primary().await?;

    // Wait for transfer and make primary.
    res_transfer.ok().await?;
    res_make_primary.ok().await?;

    // Prepare `system.remark` call.
    let remark_call = tester.api.call().system().remark(vec![])?;
    // Prepare `settlement.create_venue` call.
    let create_venue_call = tester.api.call().settlement().create_venue(
        VenueDetails(vec![]),
        vec![],
        VenueType::Other,
    )?;

    let expected = vec![
        true, // remark.
        true, // create venue.  MS has full permissions as the primary key.
    ];
    let batch_call = tester.api.call().utility().force_batch(vec![
        remark_call.runtime_call().clone(),
        create_venue_call.runtime_call().clone(),
    ])?;

    let mut res = ms.run_proposal(batch_call).await?;
    res.ok().await?;
    let calls_ok = get_batch_results(&mut res).await?;
    assert_eq!(calls_ok, expected);

    Ok(())
}

#[tokio::test]
async fn secondary_key_creates_multisig() -> Result<()> {
    let mut tester = PolymeshTester::new().await?;
    // Create a user with secondary keys to be the MS creator.
    let users = tester
        .users_with_secondary_keys(&[("MS_Creator_DID_with_sk", 1)])
        .await?;
    let did1 = users[0].clone();
    let sk = did1.get_sk(0)?.clone();

    // Create a MS as a secondary key with no permissions.
    let mut ms = MuliSigState::create_and_join_creator(&mut tester, &sk, 3, 2, false, None).await?;

    // Prepare `system.remark` call.
    let remark_call = tester.api.call().system().remark(vec![])?;
    // Prepare `settlement.create_venue` call.
    let create_venue_call = tester.api.call().settlement().create_venue(
        VenueDetails(vec![]),
        vec![],
        VenueType::Other,
    )?;

    let expected = vec![
        true,  // remark.
        false, // create venue.  MS has no permissions.
    ];
    let batch_call = tester.api.call().utility().force_batch(vec![
        remark_call.runtime_call().clone(),
        create_venue_call.runtime_call().clone(),
    ])?;

    let mut res = ms.run_proposal(batch_call).await?;
    res.ok().await?;
    let calls_ok = get_batch_results(&mut res).await?;
    assert_eq!(calls_ok, expected);

    Ok(())
}

#[tokio::test]
async fn ms_make_secondary() -> Result<()> {
    let mut tester = PolymeshTester::new().await?;
    // Create a user with secondary keys to be the MS creator.
    let users = tester
        .users_with_secondary_keys(&[("MS_make_secondary_with_sk", 1)])
        .await?;
    let did1 = users[0].clone();
    let sk = did1.get_sk(0)?.clone();

    // Create a MS as a secondary key with no permissions.
    let mut ms = MuliSigState::create_and_join_creator(&mut tester, &sk, 3, 2, false, None).await?;

    let whole = PermissionsBuilder::whole();
    // The MS is already a secondary key, it can't join again.
    let res = ms.make_secondary(Some(whole.into())).await;
    assert!(res.is_err());

    Ok(())
}

#[tokio::test]
async fn ms_change_identity() -> Result<()> {
    let mut tester = PolymeshTester::new().await?;
    let users = tester
        .users(&["MultiSigChangeDID1", "MultiSigChangeDID2"])
        .await?;
    let did1 = users[0].clone();
    let mut did2 = users[1].clone();

    // Use the primary key of did1 to create a MS and join it do did1 as a secondary key.
    let whole = PermissionsBuilder::whole();
    let mut ms = MuliSigState::create_and_join_creator(
        &mut tester,
        &did1.primary_key,
        3,
        2,
        false,
        Some(whole.into()),
    )
    .await?;

    // Leave the identity did1.
    let mut res = ms.leave_did().await?;
    res.wait_in_block().await?;

    // Create JoinIdentity auth for the MS to join DID2 with no-permissions.
    let mut res = tester
        .api
        .call()
        .identity()
        .add_authorization(
            Signatory::Account(ms.account.clone()),
            AuthorizationData::JoinIdentity(PermissionsBuilder::empty().build()),
            None,
        )?
        .execute(&mut did2)
        .await?;
    let auth_id = get_auth_id(&mut res)
        .await?
        .expect("Missing JoinIdentity auth id");

    // Join did2.  The primary key of did2 pays the tx fees.
    let mut res = ms.join_identity(auth_id).await?;
    res.ok().await?;

    // Prepare `system.remark` call.
    let remark_call = tester.api.call().system().remark(vec![])?;
    // Should be able to run new proposals now.
    let mut res = ms.run_proposal(remark_call).await?;
    res.ok().await?;

    Ok(())
}

#[tokio::test]
async fn ms_needs_to_be_linked_to_an_identity() -> Result<()> {
    let mut tester = PolymeshTester::new().await?;
    let users = tester.users(&["MultiSigNeedsDID"]).await?;
    let did = users[0].clone();

    // Use the primary key of did1 to create a MS and join it do did1 as a secondary key.
    let whole = PermissionsBuilder::whole();
    let mut ms = MuliSigState::create_and_join_creator(
        &mut tester,
        &did.primary_key,
        3,
        2,
        false,
        Some(whole.into()),
    )
    .await?;

    // Leave the identity.
    let mut res = ms.leave_did().await?;
    res.wait_in_block().await?;

    // Prepare `system.remark` call.
    let remark_call = tester.api.call().system().remark(vec![])?;
    // Shouldn't be allowed, since the MS doesn't have a DID.
    let res = ms.run_proposal(remark_call).await;
    assert!(res.is_err());

    Ok(())
}

#[tokio::test]
async fn ms_remove_payer() -> Result<()> {
    let mut tester = PolymeshTester::new().await?;
    let users = tester.users(&["MultiSigRemovePayer"]).await?;
    let mut did = users[0].clone();

    // Use the primary key of did1 to create a MS and join it do did1 as a secondary key.
    let whole = PermissionsBuilder::whole();
    let mut ms = MuliSigState::create_and_join_creator(
        &mut tester,
        &did.primary_key,
        3,
        2,
        false,
        Some(whole.into()),
    )
    .await?;

    // Remove the paying DID.
    let mut res = ms.remove_payer().await?;
    res.wait_in_block().await?;

    // Shouldn't be allowed, since the MS doesn't have any POLYX
    // to pay the tx fees.
    let remark_call = tester.api.call().system().remark(vec![])?;
    let res = ms.run_proposal(remark_call).await;
    assert!(res.is_err());

    // Fund the MultiSig so that it can pay tx fees.
    let mut res_transfer = tester
        .api
        .call()
        .balances()
        .transfer(ms.account.into(), 10 * ONE_POLYX)?
        .execute(&mut did)
        .await?;
    // Wait for transfer.
    res_transfer.ok().await?;

    // Should be able to execute proposals now.
    let remark_call = tester.api.call().system().remark(vec![])?;
    let mut res = ms.run_proposal(remark_call).await?;
    res.ok().await?;

    Ok(())
}

#[tokio::test]
async fn ms_remove_payer_via_payer() -> Result<()> {
    let mut tester = PolymeshTester::new().await?;
    let users = tester.users(&["MultiSigRemovePayerViaPayer"]).await?;
    let mut did = users[0].clone();

    // Use the primary key of did1 to create a MS and join it do did1 as a secondary key.
    let whole = PermissionsBuilder::whole();
    let mut ms = MuliSigState::create_and_join_creator(
        &mut tester,
        &did.primary_key,
        3,
        2,
        false,
        Some(whole.into()),
    )
    .await?;

    // Remove the paying DID via the payer.
    let mut res = ms.remove_payer_via_payer().await?;
    res.wait_in_block().await?;

    // Shouldn't be allowed, since the MS doesn't have any POLYX
    // to pay the tx fees.
    let remark_call = tester.api.call().system().remark(vec![])?;
    let res = ms.run_proposal(remark_call).await;
    assert!(res.is_err());

    // Fund the MultiSig so that it can pay tx fees.
    let mut res_transfer = tester
        .api
        .call()
        .balances()
        .transfer(ms.account.into(), 10 * ONE_POLYX)?
        .execute(&mut did)
        .await?;
    // Wait for transfer.
    res_transfer.ok().await?;

    // Should be able to execute proposals now.
    let remark_call = tester.api.call().system().remark(vec![])?;
    let mut res = ms.run_proposal(remark_call).await?;
    res.ok().await?;

    Ok(())
}
