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
            RuntimeEvent::MultiSig(MultiSigEvent::ProposalAdded(_, _, id)) => {
                return Ok(*id);
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
            RuntimeEvent::MultiSig(MultiSigEvent::ProposalExecuted(..)) => {
                return Ok(Some(true));
            }
            RuntimeEvent::MultiSig(MultiSigEvent::ProposalFailedToExecute(..)) => {
                return Ok(Some(false));
            }
            _ => (),
        }
    }
    Ok(None)
}

pub struct MuliSigState {
    creator: User,
    account: AccountId,
    signers: Vec<AccountSigner>,
    sigs_required: u64,
}

impl MuliSigState {
    async fn create(
        tester: &mut PolymeshTester,
        create_name: &str,
        n_signers: usize,
        sigs_required: u64,
        join_did_as_primary: Option<bool>,
    ) -> Result<MuliSigState> {
        let mut signers = Vec::with_capacity(n_signers);
        let signer_name_prefix = format!("{create_name}Signer");
        for idx in 0..n_signers {
            signers.push(tester.new_signer_idx(&signer_name_prefix, idx)?);
        }

        let mut creator = tester.user(create_name).await?;

        let mut res = tester
            .api
            .call()
            .multi_sig()
            .create_multisig(signers.iter().map(|s| s.account()).collect(), sigs_required)?
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
                RuntimeEvent::MultiSig(MultiSigEvent::MultiSigCreated(_, ms_account, ..)) => {
                    account = Some(*ms_account);
                }
                _ => (),
            }
        }

        let mut ms = MuliSigState {
            creator,
            account: account.expect("Failed to get MS address"),
            signers,
            sigs_required,
        };
        match join_did_as_primary {
            Some(true) => {
                results.push(ms.make_primary().await?);
            }
            Some(false) => {
                results.push(ms.make_secondary().await?);
            }
            None => (),
        }

        Ok(ms)
    }

    async fn make_primary(&mut self) -> Result<TransactionResults> {
        let res = self
            .creator
            .api
            .call()
            .multi_sig()
            .make_multisig_primary(self.account.clone(), None)?
            .submit_and_watch(&mut self.creator)
            .await?;
        Ok(res)
    }

    async fn make_secondary(&mut self) -> Result<TransactionResults> {
        let res = self
            .creator
            .api
            .call()
            .multi_sig()
            .make_multisig_secondary(self.account.clone())?
            .submit_and_watch(&mut self.creator)
            .await?;
        Ok(res)
    }

    async fn create_proposal(&mut self, proposal: WrappedCall) -> Result<TransactionResults> {
        let res = self
            .creator
            .api
            .call()
            .multi_sig()
            .create_proposal(
                self.account.clone(),
                proposal.runtime_call().clone(),
                None,
            )?
            .submit_and_watch(&mut self.signers[0])
            .await?;
        Ok(res)
    }

    async fn run_proposal(&mut self, proposal: WrappedCall) -> Result<TransactionResults> {
        // Create proposal with the first signer.
        let mut res = self.create_proposal(proposal).await?;
        res.wait_finalized().await?;
        if self.sigs_required > 1 {
            let id = get_ms_proposal_id(&mut res).await?;
            let weight = Weight::from_parts(10_000_000_000, 0);
            let approve_call =
                self.creator
                    .api
                    .call()
                    .multi_sig()
                    .approve(self.account.clone(), id, weight)?;
            let mut results = Vec::new();
            for signer in &mut self.signers[1..self.sigs_required as usize] {
                let res = approve_call.submit_and_watch(signer).await?;
                results.push(res);
            }
            // Find which approval call executed the proposal.
            for mut res in results {
                res.wait_finalized().await?;
                if let Some(result) = ms_proposal_executed(&mut res).await? {
                    assert!(result, "Failed to execute proposal.");
                    return Ok(res);
                }
            }
            Err(anyhow!("Failed to execute MS proposal"))
        } else {
            Ok(res)
        }
    }

    async fn set_ms_key_permissions(
        &mut self,
        permissions: impl Into<Permissions> + Send,
    ) -> Result<TransactionResults> {
        let permissions = permissions.into();
        let res = self
            .creator
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
        let record = self
            .creator
            .api
            .query()
            .identity()
            .key_records(self.account)
            .await?
            .ok_or_else(|| anyhow!("Missing KeyRecords"))?;
        let key_permissions = match record {
            KeyRecord::SecondaryKey(_, perms) => Some(perms),
            _ => None,
        };
        assert_eq!(Some(permissions), key_permissions);
        Ok(())
    }
}

#[tokio::test]
async fn multisig_as_secondary_key_change_identity() -> Result<()> {
    let mut tester = PolymeshTester::new().await?;
    let users = tester
        .users(&["MultiSigSecondaryDID1", "MultiSigSecondaryDID2"])
        .await?;
    let mut did2 = users[1].clone();

    let mut ms =
        MuliSigState::create(&mut tester, "MultiSigSecondaryDID1", 3, 2, Some(false)).await?;

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

    // Change MS account's secondary key permissions.
    let whole = PermissionsBuilder::whole();
    ms.set_ms_key_permissions(&whole).await?;

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
