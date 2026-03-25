use ark_ec::short_weierstrass::SWCurveConfig;
use polymesh_dart::{
    BatchedFeeAccountRegistrationProof, BatchedFeeAccountTopupProof, FeeAccountAssetState,
    FeeAccountRegistrationProof, FeeAccountTopupProof, FeePaymentWithBatchedProofs,
    InstantSettlementLegAffirmations, LegConfig, FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M,
    FEE_ASSET_ID,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use tokio::sync::RwLock;

use anyhow::Result;
use codec::{Decode, Encode};

use polymesh_dart::curve_tree::{
    AccountTreeConfig, CurveTreeWitnessPath, FeeAccountTreeConfig, LeafPathAndRoot,
};
use polymesh_dart::{
    curve_tree::get_account_curve_tree_parameters, AccountAssetRegistrationProof, AccountKeyPair,
    AccountPublicKey, AccountPublicKeys, AccountRegistrationProof, AssetMintingProof, AssetState,
    Balance as DartBalance, BatchedAccountAssetRegistrationProof, BatchedProof, BatchedProofs,
    EncryptionKeyRegistrationProof, EncryptionPublicKey, InstantReceiverAffirmationProof,
    InstantSenderAffirmationProof, InstantSettlementProof, LeafIndex, LegBuilder, LegEncrypted,
    LegRef, LegRole, MediatorAffirmationProof, MediatorId, ReceiverAffirmationProof,
    ReceiverClaimProof, SenderAffirmationProof, SenderCounterUpdateProof, SenderReversalProof,
    SettlementBuilder, SettlementProof, SettlementRef, ACCOUNT_TREE_L, ACCOUNT_TREE_M,
};
pub use polymesh_dart::{AccountAssetState, AccountKeys, AssetId as DartAssetId};

pub use polymesh_api::{Api, TransactionResults};
pub use polymesh_api_tester::{
    ConfidentialAssetsEvent, IdentityId, PolymeshTester, RuntimeEvent, User,
};

pub mod curve_tree;
pub use curve_tree::*;

use crate::wait_for_results;

pub fn print_curve_tree_path<const L: usize, P0: SWCurveConfig + Copy, P1: SWCurveConfig + Copy>(
    path: &CurveTreeWitnessPath<L, P0, P1>,
    action: &str,
) {
    eprintln!("{action}: Curve Tree Path:");
    eprintln!(
        "{action}:   Even inner nodes length: {}",
        path.even_internal_nodes.len()
    );
    for (i, node) in path.even_internal_nodes.iter().enumerate() {
        eprintln!(
            "{action}:     Even node[{}]: {:?}",
            i, node.child_node_to_randomize
        );
    }
    eprintln!(
        "{action}:   Odd inner nodes length: {}",
        path.odd_internal_nodes.len()
    );
    for (i, node) in path.odd_internal_nodes.iter().enumerate() {
        eprintln!(
            "{action}:     Odd node[{}]: {:?}",
            i, node.child_node_to_randomize
        );
    }
}

/// Search transaction events for DartAsset AssetId.
pub async fn get_asset_id(res: &mut TransactionResults) -> Result<Option<DartAssetId>> {
    // Ensure the transaction was successful.
    res.ok().await?;
    wait_for_results(res).await?;

    Ok(res.events().await?.and_then(|events| {
        for rec in &events.0 {
            match &rec.event {
                RuntimeEvent::ConfidentialAssets(ConfidentialAssetsEvent::AssetCreated {
                    asset_id,
                    ..
                }) => {
                    return Some(*asset_id);
                }
                _ => {
                    log::debug!("Skipping event: {:?}", rec.event);
                }
            }
        }
        None
    }))
}

/// Search for the new account asset state in the transaction results.
pub async fn get_account_leaf_inserted(
    res: &mut TransactionResults,
) -> Result<Option<(LeafIndex, AccountLeaf)>> {
    // Ensure the transaction was successful.
    res.ok().await?;
    wait_for_results(res).await?;

    Ok(res.events().await?.and_then(|events| {
        for rec in &events.0 {
            match &rec.event {
                RuntimeEvent::ConfidentialAssets(
                    ConfidentialAssetsEvent::AccountStateLeafInserted {
                        leaf_index,
                        account_commitment,
                    },
                ) => {
                    return Some((*leaf_index, to_scale(account_commitment)));
                }
                _ => {
                    log::debug!("Skipping event: {:?}", rec.event);
                }
            }
        }
        None
    }))
}

/// Search for the new fee account asset state in the transaction results.
pub async fn get_fee_account_leaf_inserted(
    res: &mut TransactionResults,
) -> Result<Option<(LeafIndex, FeeAccountLeaf)>> {
    // Ensure the transaction was successful.
    res.ok().await?;
    wait_for_results(res).await?;

    Ok(res.events().await?.and_then(|events| {
        for rec in &events.0 {
            match &rec.event {
                RuntimeEvent::ConfidentialAssets(
                    ConfidentialAssetsEvent::FeeAccountStateLeafInserted {
                        leaf_index,
                        fee_account_commitment,
                    },
                ) => {
                    return Some((*leaf_index, to_scale(fee_account_commitment)));
                }
                _ => {
                    log::debug!("Skipping event: {:?}", rec.event);
                }
            }
        }
        None
    }))
}

/// Search for the created settlement in the transaction results.
pub async fn get_settlement_ref(res: &mut TransactionResults) -> Result<Option<SettlementRef>> {
    // Ensure the transaction was successful.
    res.ok().await?;
    wait_for_results(res).await?;

    Ok(res.events().await?.and_then(|events| {
        for rec in &events.0 {
            match &rec.event {
                RuntimeEvent::ConfidentialAssets(ConfidentialAssetsEvent::SettlementCreated {
                    settlement_ref,
                    ..
                }) => {
                    return Some(to_scale(settlement_ref));
                }
                _ => {
                    log::debug!("Skipping event: {:?}", rec.event);
                }
            }
        }
        None
    }))
}

pub fn to_scale<T1: Encode, T2: Decode>(value: &T1) -> T2 {
    let encoded = value.encode();
    Decode::decode(&mut encoded.as_slice()).expect("Failed to decode")
}

pub fn create_keys() -> AccountKeys {
    let mut rng = rand::thread_rng();
    AccountKeys::rand(&mut rng).expect("Failed to create random keys")
}

pub struct DartUserAccountAssetState {
    keys: AccountKeys,
    pub asset_state: AccountAssetState,
    pub leaf_index: Option<LeafIndex>,
}

impl DartUserAccountAssetState {
    pub fn new(asset_state: AccountAssetState, keys: &AccountKeys) -> Self {
        Self {
            keys: keys.clone(),
            asset_state,
            leaf_index: None,
        }
    }

    pub fn commit_pending_state(&mut self) -> Result<()> {
        self.asset_state.commit_pending_state()?;
        Ok(())
    }

    pub async fn update_leaf_index(
        &mut self,
        res: &mut TransactionResults,
        action: &str,
    ) -> Result<()> {
        if let Some((leaf_index, account_leaf)) = get_account_leaf_inserted(res).await? {
            // Get the expect new account commitment.
            let expected_leaf = if let Some(state) = &self.asset_state.pending_state {
                let leaf = state.commitment(&self.keys)?.as_leaf_value()?;
                let nullifier = state.nullifier()?;

                log::debug!(
                    "{action}: pk={:?} Updating leaf index: {}, pending state balance: {}, leaf: {:?}, nullifier: {:?}",
                    self.keys.acct.public, leaf_index, state.balance, leaf, nullifier
                );

                leaf
            } else {
                let leaf = self
                    .asset_state
                    .current_state
                    .commitment(&self.keys)?
                    .as_leaf_value()?;
                let nullifier = self.asset_state.current_state.nullifier()?;

                log::debug!(
                    "{action}: pk={:?} No pending state in account asset when updating leaf index: {leaf_index}, leaf: {:?}, nullifier: {:?}",
                    self.keys.acct.public, leaf, nullifier,
                );

                leaf
            };

            // Check that the account leaf matches the expected state commitment.
            if account_leaf != expected_leaf {
                return Err(anyhow::anyhow!(
                    "Account leaf does not match the expected state commitment"
                ));
            }
            self.leaf_index = Some(leaf_index);

            // We can commit the pending state now that we have the leaf index.
            self.asset_state.commit_pending_state()?;
        } else {
            let tx_result = res.ok().await;
            return Err(anyhow::anyhow!(
                "Leaf index not found in {action} transaction results: {:?}",
                tx_result
            ));
        }
        Ok(())
    }

    pub async fn get_path_and_root(
        &self,
        account_tree: &AccountCurveTree,
    ) -> Result<LeafPathAndRoot<ACCOUNT_TREE_L, ACCOUNT_TREE_M, AccountTreeConfig>> {
        let leaf_index = self
            .leaf_index
            .ok_or_else(|| anyhow::anyhow!("Leaf index not set"))?;
        log::debug!(
            "Getting account path and root for leaf index: {}",
            leaf_index
        );
        let path_and_root = account_tree.get_path_and_root(leaf_index, None).await?;
        #[cfg(feature = "debug")]
        {
            let path = &path_and_root.get_path()?;
            print_curve_tree_path(
                path,
                &format!(
                    "Account leaf index {} at block: {}",
                    leaf_index, path_and_root.block_number
                ),
            );
        }
        Ok(path_and_root)
    }

    pub fn as_mut(&mut self) -> &mut AccountAssetState {
        &mut self.asset_state
    }
}

pub struct DartUserFeeAccountAssetState {
    pub account: AccountKeyPair,
    pub fee_tree: FeeAccountCurveTree,
    pub fee_account_state: FeeAccountAssetState,
    pub leaf_index: Option<LeafIndex>,
}

impl DartUserFeeAccountAssetState {
    pub async fn new(
        api: &Api,
        fee_account_state: FeeAccountAssetState,
        account: &AccountKeyPair,
    ) -> Result<Self> {
        Ok(Self {
            fee_tree: FeeAccountCurveTree::new(api).await?,
            account: account.clone(),
            fee_account_state,
            leaf_index: None,
        })
    }

    pub fn check_balance(&self, amount: DartBalance) -> bool {
        let current_balance = self.fee_account_state.current_state.balance;
        amount <= current_balance
    }

    pub fn commit_pending_state(&mut self) -> Result<()> {
        self.fee_account_state.commit_pending_state()?;
        Ok(())
    }

    pub async fn update_leaf_index(
        &mut self,
        res: &mut TransactionResults,
        action: &str,
    ) -> Result<()> {
        if let Some((leaf_index, account_leaf)) = get_fee_account_leaf_inserted(res).await? {
            // Get the expect new account commitment.
            let expected_leaf = if let Some(state) = &self.fee_account_state.pending_state {
                let leaf = state.commitment(&self.account)?.as_leaf_value()?;
                let nullifier = state.nullifier()?;
                log::debug!(
                    "{action}: pk={:?} Updating leaf index: {}, pending state balance: {}, leaf: {:?}, nullifier: {:?}",
                    self.account.public, leaf_index, state.balance, leaf, nullifier
                );
                leaf
            } else {
                let leaf = self
                    .fee_account_state
                    .current_state
                    .commitment(&self.account)?
                    .as_leaf_value()?;
                let nullifier = self.fee_account_state.current_state.nullifier()?;

                log::debug!(
                    "{action}: pk={:?} No pending state in fee account when updating leaf index: {leaf_index}, leaf: {:?}, nullifier: {:?}",
                    self.account.public, leaf, nullifier
                );
                leaf
            };

            // Check that the account leaf matches the expected state commitment.
            if account_leaf != expected_leaf {
                return Err(anyhow::anyhow!(
                    "Account leaf does not match the expected state commitment"
                ));
            }
            self.leaf_index = Some(leaf_index);

            // We can commit the pending state now that we have the leaf index.
            self.fee_account_state.commit_pending_state()?;
        } else {
            let tx_result = res.ok().await;
            return Err(anyhow::anyhow!(
                "Leaf index not found in {action} transaction results: {:?}",
                tx_result
            ));
        }
        Ok(())
    }

    pub async fn get_path_and_root(
        &self,
    ) -> Result<LeafPathAndRoot<FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M, FeeAccountTreeConfig>> {
        let leaf_index = self
            .leaf_index
            .ok_or_else(|| anyhow::anyhow!("Leaf index not set"))?;
        log::debug!(
            "Getting fee account path and root for leaf index: {}",
            leaf_index
        );
        let path_and_root = self.fee_tree.get_path_and_root(leaf_index, None).await?;
        #[cfg(feature = "debug")]
        {
            let path = &path_and_root.get_path()?;
            print_curve_tree_path(
                path,
                &format!(
                    "Fee account leaf index {} at block: {}",
                    leaf_index, path_and_root.block_number
                ),
            );
        }
        Ok(path_and_root)
    }

    pub fn as_mut(&mut self) -> &mut FeeAccountAssetState {
        &mut self.fee_account_state
    }
}

/// Dart private proof submission method.
pub enum DartProofSubmissionMethod {
    Direct,
    Relayer(DartUser),
}

impl DartProofSubmissionMethod {
    pub fn is_relayer(&self) -> bool {
        matches!(self, DartProofSubmissionMethod::Relayer(_))
    }
}

/// Dart Proof Submitter.  To support both direct and relayer submission.
pub struct DartProofSubmitter {
    pub api: Api,
    pub user: User,
    account: AccountKeyPair,
    method: DartProofSubmissionMethod,
    fee_state: Option<DartUserFeeAccountAssetState>,
}

impl DartProofSubmitter {
    pub fn new(user: User, account: AccountKeyPair) -> Self {
        let api = user.api.clone();
        Self {
            api,
            user,
            account,
            method: DartProofSubmissionMethod::Direct,
            fee_state: None,
        }
    }

    pub fn did(&self) -> IdentityId {
        self.user.did.unwrap_or_default()
    }

    pub async fn query_account_did(
        &self,
        account: &AccountPublicKey,
    ) -> Result<Option<IdentityId>> {
        let did = self
            .api
            .query()
            .confidential_assets()
            .account_did(to_scale(account))
            .await?;

        Ok(did)
    }

    pub async fn query_encryption_did(
        &self,
        enc: &EncryptionPublicKey,
    ) -> Result<Option<IdentityId>> {
        let did = self
            .api
            .query()
            .confidential_assets()
            .encryption_key_did(to_scale(enc))
            .await?;

        Ok(did)
    }

    pub async fn register_fee_account(&mut self, amount: DartBalance) -> Result<()> {
        if self.fee_state.is_some() {
            return Ok(());
        }
        // Generate fee account registration proof.
        let (proof, fee_state) = {
            let mut rng = rand::thread_rng();
            let did = self.did();
            let (proof, state) = FeeAccountRegistrationProof::new(
                &mut rng,
                &self.account,
                FEE_ASSET_ID,
                amount,
                &did.0[..],
            )?;
            (proof, state)
        };
        let mut fee_state =
            DartUserFeeAccountAssetState::new(&self.api, fee_state, &self.account).await?;

        let proof = BatchedFeeAccountRegistrationProof::<()> {
            proofs: vec![proof].try_into().expect("Only one proof"),
        };

        let mut res = self
            .api
            .call()
            .confidential_assets()
            .register_fee_accounts(to_scale(&proof))?
            .submit_and_watch(&mut self.user)
            .await?;
        res.ok().await?;
        wait_for_results(&mut res).await?;

        // Update the fee state with the new leaf index.
        fee_state
            .update_leaf_index(&mut res, "Register fee account")
            .await?;

        self.fee_state = Some(fee_state);
        Ok(())
    }

    pub async fn fee_account_topup(&mut self, amount: DartBalance) -> Result<()> {
        self.fee_account_topup_if_needed(amount, true).await
    }

    pub async fn fee_account_topup_if_needed(
        &mut self,
        amount: DartBalance,
        always_topup: bool,
    ) -> Result<()> {
        let did = self.did();
        if let Some(fee_state) = self.fee_state.as_mut() {
            if !always_topup && fee_state.check_balance(amount) {
                log::debug!(
                    "Skipping fee account topup, current balance sufficient: {}",
                    fee_state.fee_account_state.current_state.balance
                );
                return Ok(());
            }
            // Lookup our current account asset state in the on-chain account tree.
            let fee_account_lookup = fee_state.get_path_and_root().await?;
            let root_block = fee_account_lookup.block_number;

            // Generate fee account topup proof.
            let proof = {
                let mut rng = rand::thread_rng();
                FeeAccountTopupProof::new(
                    &mut rng,
                    &self.account,
                    fee_state.as_mut(),
                    amount,
                    &did.0[..],
                    &fee_account_lookup,
                )?
            };

            let proof = BatchedFeeAccountTopupProof::<()> {
                root_block,
                proofs: vec![proof].try_into().expect("Only one proof"),
            };

            let mut res = self
                .api
                .call()
                .confidential_assets()
                .topup_fee_accounts(to_scale(&proof))?
                .submit_and_watch(&mut self.user)
                .await?;
            res.ok().await?;
            wait_for_results(&mut res).await?;

            // Update the fee state with the new leaf index.
            fee_state
                .update_leaf_index(&mut res, "Topup fee account")
                .await?;

            Ok(())
        } else {
            self.register_fee_account(amount).await
        }
    }

    pub async fn fee_payment_batch(
        &mut self,
        amount: DartBalance,
        batched: BatchedProofs<()>,
    ) -> Result<FeePaymentWithBatchedProofs<()>> {
        // Topup our fee account if needed.
        self.fee_account_topup_if_needed(amount, false).await?;

        let fee_state = self
            .fee_state
            .as_mut()
            .expect("Shouldn't happen since we just topped up");

        // Lookup our current account asset state in the on-chain account tree.
        let fee_account_lookup = fee_state.get_path_and_root().await?;

        // Generate fee account topup proof.
        let mut rng = rand::thread_rng();
        Ok(FeePaymentWithBatchedProofs::new(
            &mut rng,
            &self.account,
            batched,
            fee_state.as_mut(),
            amount,
            &fee_account_lookup,
        )?)
    }

    pub async fn update_leaf_index(
        &mut self,
        res: &mut TransactionResults,
        action: &str,
    ) -> Result<()> {
        // Update the fee state with the new leaf index.
        let fee_state = self
            .fee_state
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Fee account hasn't been initialized"))?;
        fee_state.update_leaf_index(res, action).await?;
        Ok(())
    }

    pub async fn submit_batch_proof(
        &mut self,
        proof: BatchedProof<()>,
    ) -> Result<TransactionResults> {
        let proof = BatchedProofs::<()> {
            proofs: vec![proof].try_into().expect("Only one proof"),
        };

        //if let DartProofSubmissionMethod::Relayer(ref mut relayer) = self.method {
        if self.method.is_relayer() {
            // TODO: calculate tx fees based on batched proofs.
            let tx_fee = 3_000_000u64 * (proof.proofs.len() as u64);

            let fee_payment_batch = self.fee_payment_batch(tx_fee, proof).await?;

            if let DartProofSubmissionMethod::Relayer(ref mut relayer) = self.method {
                let mut res = relayer.relayer_submit_batch(fee_payment_batch).await?;

                // Update the fee state with the new leaf index.
                self.update_leaf_index(&mut res, "Fee payment batch")
                    .await?;

                Ok(res)
            } else {
                Err(anyhow::anyhow!(
                    "Proof submission method changed unexpectedly"
                ))
            }
        } else {
            let ts = self
                .api
                .call()
                .confidential_assets()
                .submit_batched_proofs(to_scale(&proof))?
                .submit_and_watch(&mut self.user)
                .await?;
            Ok(ts)
        }
    }

    pub async fn relayer_submit_batch(
        &mut self,
        batch: FeePaymentWithBatchedProofs<()>,
    ) -> Result<TransactionResults> {
        let ts = self
            .api
            .call()
            .confidential_assets()
            .relayer_submit_batched_proofs(to_scale(&batch))?
            .submit_and_watch(&mut self.user)
            .await?;
        Ok(ts)
    }

    pub async fn register_encryption_keys(
        &mut self,
        proof: EncryptionKeyRegistrationProof<()>,
    ) -> Result<TransactionResults> {
        let ts = self
            .api
            .call()
            .confidential_assets()
            .register_encryption_keys(to_scale(&proof))?
            .submit_and_watch(&mut self.user)
            .await?;
        Ok(ts)
    }

    pub async fn register_accounts(
        &mut self,
        proof: AccountRegistrationProof<()>,
    ) -> Result<TransactionResults> {
        let ts = self
            .api
            .call()
            .confidential_assets()
            .register_accounts(to_scale(&proof))?
            .submit_and_watch(&mut self.user)
            .await?;
        Ok(ts)
    }

    pub async fn register_account_assets(
        &mut self,
        proof: BatchedAccountAssetRegistrationProof<()>,
    ) -> Result<TransactionResults> {
        let ts = self
            .api
            .call()
            .confidential_assets()
            .register_account_assets(to_scale(&proof))?
            .submit_and_watch(&mut self.user)
            .await?;
        Ok(ts)
    }

    pub async fn create_asset(
        &mut self,
        name: &str,
        symbol: &str,
        decimals: u8,
        description: &str,
        mediators: BTreeMap<AccountPublicKey, EncryptionPublicKey>,
        auditors: BTreeSet<EncryptionPublicKey>,
    ) -> Result<DartAssetId> {
        let mut res = self
            .api
            .call()
            .confidential_assets()
            .create_asset(
                to_scale(&name.to_string()),
                to_scale(&symbol.to_string()),
                decimals,
                to_scale(&mediators),
                to_scale(&auditors),
                description.as_bytes().to_vec(),
            )?
            .submit_and_watch(&mut self.user)
            .await?;

        let asset_id = get_asset_id(&mut res)
            .await?
            .expect("Asset creation failed");

        Ok(asset_id)
    }

    pub async fn mint_asset(&mut self, proof: AssetMintingProof) -> Result<TransactionResults> {
        let res = self
            .api
            .call()
            .confidential_assets()
            .mint_asset(to_scale(&proof))?
            .submit_and_watch(&mut self.user)
            .await?;
        Ok(res)
    }

    pub async fn execute_instant_settlement(
        &mut self,
        proof: InstantSettlementProof,
    ) -> Result<TransactionResults> {
        let res = self
            .api
            .call()
            .confidential_assets()
            .execute_instant_settlement(to_scale(&proof))?
            .submit_and_watch(&mut self.user)
            .await?;
        Ok(res)
    }
}

/// Dart User
pub struct DartUserInner {
    pub keys: AccountKeys,
    pub registered: bool,
    pub enc_registered: bool,
    pub assets: BTreeMap<DartAssetId, DartUserAccountAssetState>,
    pub submitter: DartProofSubmitter,
}

impl DartUserInner {
    pub fn new(user: User) -> Self {
        let keys = create_keys();
        Self {
            submitter: DartProofSubmitter::new(user, keys.acct.clone()),
            keys,
            registered: false,
            enc_registered: false,
            assets: BTreeMap::new(),
        }
    }

    pub fn public_keys(&self) -> AccountPublicKeys {
        self.keys.public_keys()
    }

    pub fn did(&self) -> IdentityId {
        self.submitter.did()
    }

    pub fn set_relayer(&mut self, relayer: DartUser) {
        self.submitter.method = DartProofSubmissionMethod::Relayer(relayer);
    }

    pub fn is_account_asset_registered(&self, asset_id: DartAssetId) -> bool {
        self.assets.contains_key(&asset_id)
    }

    pub async fn register_encryption_key(&mut self) -> Result<()> {
        if self.enc_registered {
            return Ok(());
        }
        if self.registered {
            return Err(anyhow::anyhow!(
                "Can't register the encryption key stand alone and as part of account registration"
            ));
        }

        // Check if encryption key is already registered.
        let did = self.did();
        if let Some(existing_did) = self
            .submitter
            .query_encryption_did(&self.keys.public_keys().enc)
            .await?
        {
            if existing_did == did {
                log::debug!(
                    "Encryption key already registered for DID: {:?}",
                    existing_did
                );
                self.enc_registered = true;
                return Ok(());
            } else {
                return Err(anyhow::anyhow!(
                    "Encryption key already registered for different DID: {:?}",
                    existing_did
                ));
            }
        }

        // Generate encryption key registration proof.
        let proof = {
            let mut rng = rand::thread_rng();
            EncryptionKeyRegistrationProof::<()>::new(
                &mut rng,
                &[self.keys.enc.clone()],
                &did.0[..],
            )?
        };

        let mut res = self.submitter.register_encryption_keys(proof).await?;
        res.ok().await?;
        wait_for_results(&mut res).await?;
        self.enc_registered = true;
        Ok(())
    }

    pub async fn register_account(&mut self) -> Result<()> {
        if self.registered {
            return Ok(());
        }
        if self.enc_registered {
            return Err(anyhow::anyhow!(
                "Can't register the account keys if the encryption key is already registered"
            ));
        }

        // Check if account is already registered.
        let did = self.did();
        if let Some(existing_did) = self
            .submitter
            .query_account_did(&self.keys.public_keys().acct)
            .await?
        {
            if existing_did == did {
                log::debug!("Account already registered for DID: {:?}", existing_did);
                self.registered = true;
                return Ok(());
            } else {
                return Err(anyhow::anyhow!(
                    "Account already registered for different DID: {:?}",
                    existing_did
                ));
            }
        }

        // Generate account registration proof.
        let proof = {
            let mut rng = rand::thread_rng();
            AccountRegistrationProof::<()>::new(&mut rng, &[self.keys.clone()], &did.0[..])?
        };

        let mut res = self.submitter.register_accounts(proof).await?;
        res.ok().await?;
        wait_for_results(&mut res).await?;
        self.registered = true;
        Ok(())
    }

    pub async fn register_fee_account(&mut self, amount: DartBalance) -> Result<()> {
        self.submitter.register_fee_account(amount).await
    }

    pub async fn fee_account_topup(&mut self, amount: DartBalance) -> Result<()> {
        self.submitter.fee_account_topup(amount).await
    }

    pub async fn relayer_submit_batch(
        &mut self,
        batch: FeePaymentWithBatchedProofs<()>,
    ) -> Result<TransactionResults> {
        self.submitter.relayer_submit_batch(batch).await
    }

    pub async fn register_account_asset(&mut self, asset_id: DartAssetId) -> Result<()> {
        // Check if the account has already been registered for the asset.
        if self.is_account_asset_registered(asset_id) {
            return Ok(());
        }

        // Generate the account asset registration proof.
        let (proof, mut asset_state) = {
            let mut rng = rand::thread_rng();
            let did = self.did();
            let params = get_account_curve_tree_parameters();
            let (proof, asset_state) = AccountAssetRegistrationProof::new(
                &mut rng,
                &self.keys,
                asset_id,
                0,
                &did.0[..],
                params,
            )?;
            let asset_state = DartUserAccountAssetState::new(asset_state, &self.keys);
            (proof, asset_state)
        };

        // Submit the registration proof.
        log::debug!(
            "Registering account {:?} asset: {}",
            self.keys.public_keys().acct,
            asset_id
        );
        let proof = BatchedAccountAssetRegistrationProof::<()> {
            proofs: vec![proof].try_into().expect("Only one proof"),
        };
        let mut res = self.submitter.register_account_assets(proof).await?;
        res.ok().await?;
        wait_for_results(&mut res).await?;
        log::debug!(
            "Registered account asset: {}, res: {:?}",
            asset_id,
            res.ok().await
        );

        // Update the asset state with the new leaf index.
        asset_state
            .update_leaf_index(&mut res, "Register account asset")
            .await?;

        // Store the asset state in the user's assets.
        self.assets.insert(asset_id, asset_state);
        Ok(())
    }

    pub async fn create_asset(
        &mut self,
        name: &str,
        symbol: &str,
        decimals: u8,
        description: &str,
        mediators: BTreeMap<AccountPublicKey, EncryptionPublicKey>,
        auditors: BTreeSet<EncryptionPublicKey>,
    ) -> Result<DartAssetId> {
        self.submitter
            .create_asset(name, symbol, decimals, description, mediators, auditors)
            .await
    }

    pub async fn mint_asset(
        &mut self,
        account_tree: &AccountCurveTree,
        asset_id: DartAssetId,
        amount: u64,
    ) -> Result<()> {
        // Get our current account asset state.
        let asset_state = self
            .assets
            .get_mut(&asset_id)
            .ok_or_else(|| anyhow::anyhow!("Asset not registered: {}", asset_id))?;

        // Lookup our current account asset state in the on-chain account tree.
        let account_lookup = asset_state.get_path_and_root(account_tree).await?;

        // Generate minting proof.
        let proof = {
            let mut rng = rand::thread_rng();
            AssetMintingProof::new(
                &mut rng,
                &self.keys,
                asset_state.as_mut(),
                account_lookup,
                amount,
            )?
        };

        // Submit the minting proof.
        let mut res = self.submitter.mint_asset(proof).await?;
        res.ok().await?;
        wait_for_results(&mut res).await?;
        log::debug!("Mint assets: res={:?}", res.ok().await?);

        // Update the asset state with the new leaf index.
        asset_state.update_leaf_index(&mut res, "Minting").await?;

        Ok(())
    }

    pub async fn create_settlement(&mut self, proof: SettlementProof<()>) -> Result<SettlementRef> {
        // Submit the settlement proof.
        let mut res = self
            .submitter
            .submit_batch_proof(BatchedProof::CreateSettlement(proof))
            .await?;

        // Get the settlement reference from the transaction results.
        let settlement_ref = get_settlement_ref(&mut res)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Settlement creation failed"))?;

        Ok(settlement_ref)
    }

    pub async fn execute_instant_settlement(
        &mut self,
        proof: InstantSettlementProof<()>,
    ) -> Result<SettlementRef> {
        // Submit the instant settlement proof.
        let mut res = self
            .submitter
            .submit_batch_proof(BatchedProof::ExecuteInstantSettlement(proof))
            .await?;

        // Get the settlement reference from the transaction results.
        let settlement_ref = get_settlement_ref(&mut res)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Settlement creation failed"))?;

        Ok(settlement_ref)
    }

    pub async fn update_leaf_index(
        &mut self,
        asset_id: DartAssetId,
        mut res: &mut TransactionResults,
        action: &str,
    ) -> Result<()> {
        // Get our current account asset state.
        let asset_state = self
            .assets
            .get_mut(&asset_id)
            .ok_or_else(|| anyhow::anyhow!("Asset not registered: {}", asset_id))?;

        // Update the asset state with the new leaf index.
        asset_state.update_leaf_index(&mut res, action).await?;

        Ok(())
    }

    pub async fn sender_affirmation_proof(
        &mut self,
        tester: &DartAssetTester,
        leg_ref: LegRef,
        leg_enc: &LegEncrypted,
        asset_id: DartAssetId,
        amount: DartBalance,
    ) -> Result<SenderAffirmationProof> {
        // Try to decrypt the leg as the sender
        let leg = leg_enc.decrypt(LegRole::sender(), &self.keys)?;

        // Check the leg asset and amount.
        if leg.asset_id() != asset_id || leg.amount() != amount {
            return Err(anyhow::anyhow!(
                "Leg asset or amount does not match: {:?} != {:?}, {} != {}",
                leg.asset_id(),
                asset_id,
                leg.amount(),
                amount
            ));
        }

        // Get our current account asset state.
        let asset_state = self
            .assets
            .get_mut(&asset_id)
            .ok_or_else(|| anyhow::anyhow!("Asset not registered: {}", asset_id))?;

        // Lookup our current account asset state in the on-chain account tree.
        let account_tree = tester.account_tree().await;
        let account_lookup = asset_state.get_path_and_root(&account_tree).await?;

        // Generate sender affirmation proof.
        let proof = {
            let mut rng = rand::thread_rng();
            SenderAffirmationProof::new(
                &mut rng,
                &self.keys,
                &leg_ref,
                amount,
                &leg_enc,
                asset_state.as_mut(),
                account_lookup,
            )?
        };

        Ok(proof)
    }

    pub async fn receiver_affirmation_proof(
        &mut self,
        tester: &DartAssetTester,
        leg_ref: LegRef,
        leg_enc: &LegEncrypted,
        asset_id: DartAssetId,
        amount: DartBalance,
    ) -> Result<ReceiverAffirmationProof> {
        // Try to decrypt the leg as the receiver
        let leg = leg_enc.decrypt(LegRole::receiver(), &self.keys)?;

        // Check the leg asset and amount.
        if leg.asset_id() != asset_id || leg.amount() != amount {
            return Err(anyhow::anyhow!(
                "Leg asset or amount does not match: {:?} != {:?}, {} != {}",
                leg.asset_id(),
                asset_id,
                leg.amount(),
                amount
            ));
        }

        // Get our current account asset state.
        let asset_state = self
            .assets
            .get_mut(&asset_id)
            .ok_or_else(|| anyhow::anyhow!("Asset not registered: {}", asset_id))?;

        // Lookup our current account asset state in the on-chain account tree.
        let account_tree = tester.account_tree().await;
        let account_lookup = asset_state.get_path_and_root(&account_tree).await?;

        // Generate receiver affirmation proof.
        let proof = {
            let mut rng = rand::thread_rng();
            ReceiverAffirmationProof::new(
                &mut rng,
                &self.keys,
                &leg_ref,
                &leg_enc,
                asset_state.as_mut(),
                account_lookup,
            )?
        };

        Ok(proof)
    }

    pub async fn sender_affirmation(
        &mut self,
        tester: &DartAssetTester,
        leg_ref: LegRef,
        asset_id: DartAssetId,
        amount: DartBalance,
    ) -> Result<()> {
        // Get the encrypted settlement leg from the chain.
        let leg_enc = tester
            .get_settlement_leg(leg_ref)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Settlement leg not found"))?;

        // Generate sender affirmation proof.
        let proof = self
            .sender_affirmation_proof(tester, leg_ref, &leg_enc, asset_id, amount)
            .await?;

        // Submit the sender affirmation proof.
        let mut res = self
            .submitter
            .submit_batch_proof(BatchedProof::SenderAffirmation(proof))
            .await?;

        // Update the asset state with the new leaf index.
        self.update_leaf_index(asset_id, &mut res, "Sender affirmation")
            .await?;

        Ok(())
    }

    pub async fn receiver_affirmation(
        &mut self,
        tester: &DartAssetTester,
        leg_ref: LegRef,
        asset_id: DartAssetId,
        amount: DartBalance,
    ) -> Result<()> {
        // Get the encrypted settlement leg from the chain.
        let leg_enc = tester
            .get_settlement_leg(leg_ref)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Settlement leg not found"))?;

        // Generate receiver affirmation proof.
        let proof = self
            .receiver_affirmation_proof(tester, leg_ref, &leg_enc, asset_id, amount)
            .await?;

        // Submit the receiver affirmation proof.
        let mut res = self
            .submitter
            .submit_batch_proof(BatchedProof::ReceiverAffirmation(proof))
            .await?;

        // Update the asset state with the new leaf index.
        self.update_leaf_index(asset_id, &mut res, "Receiver affirmation")
            .await?;

        Ok(())
    }

    pub async fn instant_sender_affirmation_proof(
        &mut self,
        tester: &DartAssetTester,
        leg_ref: LegRef,
        leg_enc: &LegEncrypted,
        asset_id: DartAssetId,
        amount: DartBalance,
    ) -> Result<InstantSenderAffirmationProof> {
        // Try to decrypt the leg as the sender
        let leg = leg_enc.decrypt(LegRole::sender(), &self.keys)?;

        // Check the leg asset and amount.
        if leg.asset_id() != asset_id || leg.amount() != amount {
            return Err(anyhow::anyhow!(
                "Leg asset or amount does not match: {:?} != {:?}, {} != {}",
                leg.asset_id(),
                asset_id,
                leg.amount(),
                amount
            ));
        }

        // Get our current account asset state.
        let asset_state = self
            .assets
            .get_mut(&asset_id)
            .ok_or_else(|| anyhow::anyhow!("Asset not registered: {}", asset_id))?;

        // Lookup our current account asset state in the on-chain account tree.
        let account_tree = tester.account_tree().await;
        let account_lookup = asset_state.get_path_and_root(&account_tree).await?;

        // Generate sender affirmation proof.
        let proof = {
            let mut rng = rand::thread_rng();
            InstantSenderAffirmationProof::new(
                &mut rng,
                &self.keys,
                &leg_ref,
                amount,
                &leg_enc,
                asset_state.as_mut(),
                account_lookup,
            )?
        };

        Ok(proof)
    }

    pub async fn instant_receiver_affirmation_proof(
        &mut self,
        tester: &DartAssetTester,
        leg_ref: LegRef,
        leg_enc: &LegEncrypted,
        asset_id: DartAssetId,
        amount: DartBalance,
    ) -> Result<InstantReceiverAffirmationProof> {
        // Try to decrypt the leg as the receiver
        let leg = leg_enc.decrypt(LegRole::receiver(), &self.keys)?;

        // Check the leg asset and amount.
        if leg.asset_id() != asset_id || leg.amount() != amount {
            return Err(anyhow::anyhow!(
                "Leg asset or amount does not match: {:?} != {:?}, {} != {}",
                leg.asset_id(),
                asset_id,
                leg.amount(),
                amount
            ));
        }

        // Get our current account asset state.
        let asset_state = self
            .assets
            .get_mut(&asset_id)
            .ok_or_else(|| anyhow::anyhow!("Asset not registered: {}", asset_id))?;

        // Lookup our current account asset state in the on-chain account tree.
        let account_tree = tester.account_tree().await;
        let account_lookup = asset_state.get_path_and_root(&account_tree).await?;

        // Generate receiver affirmation proof.
        let proof = {
            let mut rng = rand::thread_rng();
            InstantReceiverAffirmationProof::new(
                &mut rng,
                &self.keys,
                &leg_ref,
                amount,
                &leg_enc,
                asset_state.as_mut(),
                account_lookup,
            )?
        };

        Ok(proof)
    }

    pub async fn instant_sender_affirmation(
        &mut self,
        tester: &DartAssetTester,
        leg_ref: LegRef,
        asset_id: DartAssetId,
        amount: DartBalance,
    ) -> Result<()> {
        // Get the encrypted settlement leg from the chain.
        let leg_enc = tester
            .get_settlement_leg(leg_ref)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Settlement leg not found"))?;

        // Generate instant sender affirmation proof.
        let proof = self
            .instant_sender_affirmation_proof(tester, leg_ref, &leg_enc, asset_id, amount)
            .await?;

        // Submit the instant sender affirmation proof.
        let mut res = self
            .submitter
            .submit_batch_proof(BatchedProof::InstantSenderAffirmation(proof))
            .await?;

        // Update the asset state with the new leaf index.
        self.update_leaf_index(asset_id, &mut res, "Instant Sender affirmation")
            .await?;

        Ok(())
    }

    pub async fn instant_receiver_affirmation(
        &mut self,
        tester: &DartAssetTester,
        leg_ref: LegRef,
        asset_id: DartAssetId,
        amount: DartBalance,
    ) -> Result<()> {
        // Get the encrypted settlement leg from the chain.
        let leg_enc = tester
            .get_settlement_leg(leg_ref)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Settlement leg not found"))?;

        // Generate instant receiver affirmation proof.
        let proof = self
            .instant_receiver_affirmation_proof(tester, leg_ref, &leg_enc, asset_id, amount)
            .await?;

        // Submit the instant receiver affirmation proof.
        let mut res = self
            .submitter
            .submit_batch_proof(BatchedProof::InstantReceiverAffirmation(proof))
            .await?;

        // Update the asset state with the new leaf index.
        self.update_leaf_index(asset_id, &mut res, "Instant Receiver affirmation")
            .await?;

        Ok(())
    }

    pub async fn mediator_affirmation_proof(
        &mut self,
        tester: &DartAssetTester,
        leg_ref: LegRef,
        mediator_id: MediatorId,
        accept: bool,
        asset_and_amount: Option<(DartAssetId, DartBalance)>,
    ) -> Result<MediatorAffirmationProof> {
        // Get the encrypted settlement leg from the chain.
        let leg_enc = tester
            .get_settlement_leg(leg_ref)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Settlement leg not found"))?;

        // Try to decrypt the leg as the mediator
        let leg = leg_enc
            .decrypt(LegRole::mediator(mediator_id), &self.keys)
            .unwrap();

        // Check the leg asset and amount if provided.
        if let Some((asset_id, amount)) = asset_and_amount {
            if leg.asset_id() != asset_id || leg.amount() != amount {
                return Err(anyhow::anyhow!(
                    "Leg asset or amount does not match: {:?} != {:?}, {} != {}",
                    leg.asset_id(),
                    asset_id,
                    leg.amount(),
                    amount
                ));
            }
        }

        // Generate mediator affirmation proof.
        let mut rng = rand::thread_rng();
        let med_enc = leg_enc.mediator_encryption(mediator_id)?;
        Ok(MediatorAffirmationProof::new(
            &mut rng, &leg_ref, &med_enc, &self.keys, 0, accept,
        )?)
    }

    pub async fn mediator_affirmation(
        &mut self,
        tester: &DartAssetTester,
        leg_ref: LegRef,
        mediator_id: MediatorId,
        accept: bool,
        asset_and_amount: Option<(DartAssetId, DartBalance)>,
    ) -> Result<()> {
        // Generate mediator affirmation proof.
        let proof = self
            .mediator_affirmation_proof(tester, leg_ref, mediator_id, accept, asset_and_amount)
            .await?;

        // Submit the mediator affirmation proof.
        let mut res = self
            .submitter
            .submit_batch_proof(BatchedProof::MediatorAffirmation(proof))
            .await?;
        res.ok().await?;
        wait_for_results(&mut res).await?;
        let res = res.ok().await;
        log::debug!(
            "Mediator affirmation submitted for leg_ref {:?}, accept={}, res={:?}",
            leg_ref,
            accept,
            res
        );

        Ok(())
    }

    pub async fn receiver_claim(
        &mut self,
        tester: &DartAssetTester,
        leg_ref: LegRef,
    ) -> Result<()> {
        // Get the encrypted settlement leg from the chain.
        let leg_enc = tester
            .get_settlement_leg(leg_ref)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Settlement leg not found"))?;

        // Try to decrypt the leg as the receiver
        let leg = leg_enc.decrypt(LegRole::receiver(), &self.keys)?;
        let asset_id = leg.asset_id();
        let amount = leg.amount();

        // Get our current account asset state.
        let asset_state = self
            .assets
            .get_mut(&asset_id)
            .ok_or_else(|| anyhow::anyhow!("Asset not registered: {}", leg.asset_id()))?;

        // Lookup our current account asset state in the on-chain account tree.
        let account_tree = tester.account_tree().await;
        let account_lookup = asset_state.get_path_and_root(&account_tree).await?;

        // Generate receiver claim proof.
        let proof = {
            let mut rng = rand::thread_rng();
            ReceiverClaimProof::new(
                &mut rng,
                &self.keys,
                &leg_ref,
                amount,
                &leg_enc,
                asset_state.as_mut(),
                account_lookup,
            )?
        };

        // Submit the receiver claim proof.
        let mut res = self
            .submitter
            .submit_batch_proof(BatchedProof::ReceiverClaim(proof))
            .await?;

        // Update the asset state with the new leaf index.
        asset_state
            .update_leaf_index(&mut res, "Receiver claim")
            .await?;

        Ok(())
    }

    pub async fn sender_counter_update(
        &mut self,
        tester: &DartAssetTester,
        leg_ref: LegRef,
    ) -> Result<()> {
        // Get the encrypted settlement leg from the chain.
        let leg_enc = tester
            .get_settlement_leg(leg_ref)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Settlement leg not found"))?;

        // Try to decrypt the leg as the sender
        let leg = leg_enc.decrypt(LegRole::sender(), &self.keys)?;

        // Get our current account asset state.
        let asset_state = self
            .assets
            .get_mut(&leg.asset_id())
            .ok_or_else(|| anyhow::anyhow!("Asset not registered: {}", leg.asset_id()))?;

        // Lookup our current account asset state in the on-chain account tree.
        let account_tree = tester.account_tree().await;
        let account_lookup = asset_state.get_path_and_root(&account_tree).await?;

        // Generate sender counter update proof.
        let proof = {
            let mut rng = rand::thread_rng();
            SenderCounterUpdateProof::new(
                &mut rng,
                &self.keys,
                &leg_ref,
                &leg_enc,
                asset_state.as_mut(),
                account_lookup,
            )?
        };

        // Submit the sender counter update proof.
        let mut res = self
            .submitter
            .submit_batch_proof(BatchedProof::SenderCounterUpdate(proof))
            .await?;

        // Update the asset state with the new leaf index.
        asset_state
            .update_leaf_index(&mut res, "Sender Update counter")
            .await?;

        Ok(())
    }

    pub async fn sender_revert(&mut self, tester: &DartAssetTester, leg_ref: LegRef) -> Result<()> {
        // Get the encrypted settlement leg from the chain.
        let leg_enc = tester
            .get_settlement_leg(leg_ref)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Settlement leg not found"))?;

        // Try to decrypt the leg as the sender
        let leg = leg_enc.decrypt(LegRole::sender(), &self.keys)?;
        let asset_id = leg.asset_id();
        let amount = leg.amount();

        // Get our current account asset state.
        let asset_state = self
            .assets
            .get_mut(&asset_id)
            .ok_or_else(|| anyhow::anyhow!("Asset not registered: {}", leg.asset_id()))?;

        // Lookup our current account asset state in the on-chain account tree.
        let account_tree = tester.account_tree().await;
        let account_lookup = asset_state.get_path_and_root(&account_tree).await?;

        // Generate sender revert proof.
        let proof = {
            let mut rng = rand::thread_rng();
            SenderReversalProof::new(
                &mut rng,
                &self.keys,
                &leg_ref,
                amount,
                &leg_enc,
                asset_state.as_mut(),
                account_lookup,
            )?
        };

        // Submit the sender revert proof.
        let mut res = self
            .submitter
            .submit_batch_proof(BatchedProof::SenderReversal(proof))
            .await?;

        // Update the asset state with the new leaf index.
        asset_state
            .update_leaf_index(&mut res, "Sender revert")
            .await?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct DartUser(Arc<RwLock<DartUserInner>>);

impl DartUser {
    pub fn new(user: User) -> Self {
        Self(Arc::new(RwLock::new(DartUserInner::new(user))))
    }

    pub async fn public_keys(&self) -> AccountPublicKeys {
        self.0.read().await.public_keys()
    }

    pub async fn did(&self) -> IdentityId {
        self.0.read().await.did()
    }

    pub async fn set_relayer(&self, relayer: DartUser) {
        self.0.write().await.set_relayer(relayer);
    }

    pub async fn register_encryption_key(&self) -> Result<()> {
        self.0.write().await.register_encryption_key().await
    }

    pub async fn register_account(&self) -> Result<()> {
        self.0.write().await.register_account().await
    }

    pub async fn register_fee_account(&self, amount: DartBalance) -> Result<()> {
        self.0.write().await.register_fee_account(amount).await
    }

    pub async fn fee_account_topup(&self, amount: DartBalance) -> Result<()> {
        self.0.write().await.fee_account_topup(amount).await
    }

    pub async fn relayer_submit_batch(
        &self,
        batch: FeePaymentWithBatchedProofs<()>,
    ) -> Result<TransactionResults> {
        self.0.write().await.relayer_submit_batch(batch).await
    }

    pub async fn register_account_asset(&self, asset_id: DartAssetId) -> Result<()> {
        self.0.write().await.register_account_asset(asset_id).await
    }

    pub async fn create_asset(
        &self,
        name: &str,
        symbol: &str,
        decimals: u8,
        description: &str,
        mediators: BTreeMap<AccountPublicKey, EncryptionPublicKey>,
        auditors: BTreeSet<EncryptionPublicKey>,
    ) -> Result<DartAssetId> {
        self.0
            .write()
            .await
            .create_asset(name, symbol, decimals, description, mediators, auditors)
            .await
    }

    pub async fn mint_asset(
        &self,
        account_tree: &AccountCurveTree,
        asset_id: DartAssetId,
        amount: u64,
    ) -> Result<()> {
        self.0
            .write()
            .await
            .mint_asset(account_tree, asset_id, amount)
            .await
    }

    pub async fn create_settlement(&self, proof: SettlementProof<()>) -> Result<SettlementRef> {
        self.0.write().await.create_settlement(proof).await
    }

    pub async fn execute_instant_settlement(
        &self,
        proof: InstantSettlementProof<()>,
    ) -> Result<SettlementRef> {
        self.0.write().await.execute_instant_settlement(proof).await
    }

    pub async fn update_leaf_index(
        &self,
        asset_id: DartAssetId,
        res: &mut TransactionResults,
        action: &str,
    ) -> Result<()> {
        self.0
            .write()
            .await
            .update_leaf_index(asset_id, res, action)
            .await
    }

    pub async fn sender_affirmation_proof(
        &self,
        tester: &DartAssetTester,
        leg_ref: LegRef,
        leg_enc: &LegEncrypted,
        asset_id: DartAssetId,
        amount: DartBalance,
    ) -> Result<SenderAffirmationProof> {
        self.0
            .write()
            .await
            .sender_affirmation_proof(tester, leg_ref, leg_enc, asset_id, amount)
            .await
    }

    pub async fn receiver_affirmation_proof(
        &self,
        tester: &DartAssetTester,
        leg_ref: LegRef,
        leg_enc: &LegEncrypted,
        asset_id: DartAssetId,
        amount: DartBalance,
    ) -> Result<ReceiverAffirmationProof> {
        self.0
            .write()
            .await
            .receiver_affirmation_proof(tester, leg_ref, leg_enc, asset_id, amount)
            .await
    }

    pub async fn instant_sender_affirmation_proof(
        &self,
        tester: &DartAssetTester,
        leg_ref: LegRef,
        leg_enc: &LegEncrypted,
        asset_id: DartAssetId,
        amount: DartBalance,
    ) -> Result<InstantSenderAffirmationProof> {
        self.0
            .write()
            .await
            .instant_sender_affirmation_proof(tester, leg_ref, leg_enc, asset_id, amount)
            .await
    }

    pub async fn instant_receiver_affirmation_proof(
        &self,
        tester: &DartAssetTester,
        leg_ref: LegRef,
        leg_enc: &LegEncrypted,
        asset_id: DartAssetId,
        amount: DartBalance,
    ) -> Result<InstantReceiverAffirmationProof> {
        self.0
            .write()
            .await
            .instant_receiver_affirmation_proof(tester, leg_ref, leg_enc, asset_id, amount)
            .await
    }

    pub async fn sender_affirmation(
        &self,
        tester: &DartAssetTester,
        leg_ref: LegRef,
        asset_id: DartAssetId,
        amount: DartBalance,
    ) -> Result<()> {
        self.0
            .write()
            .await
            .sender_affirmation(tester, leg_ref, asset_id, amount)
            .await
    }

    pub async fn receiver_affirmation(
        &self,
        tester: &DartAssetTester,
        leg_ref: LegRef,
        asset_id: DartAssetId,
        amount: DartBalance,
    ) -> Result<()> {
        self.0
            .write()
            .await
            .receiver_affirmation(tester, leg_ref, asset_id, amount)
            .await
    }

    pub async fn mediator_affirmation_proof(
        &self,
        tester: &DartAssetTester,
        leg_ref: LegRef,
        mediator_id: MediatorId,
        accept: bool,
        asset_and_amount: Option<(DartAssetId, DartBalance)>,
    ) -> Result<MediatorAffirmationProof> {
        self.0
            .write()
            .await
            .mediator_affirmation_proof(tester, leg_ref, mediator_id, accept, asset_and_amount)
            .await
    }

    pub async fn mediator_affirmation(
        &self,
        tester: &DartAssetTester,
        leg_ref: LegRef,
        mediator_id: MediatorId,
        accept: bool,
        asset_and_amount: Option<(DartAssetId, DartBalance)>,
    ) -> Result<()> {
        self.0
            .write()
            .await
            .mediator_affirmation(tester, leg_ref, mediator_id, accept, asset_and_amount)
            .await
    }

    pub async fn receiver_claim(&self, tester: &DartAssetTester, leg_ref: LegRef) -> Result<()> {
        self.0.write().await.receiver_claim(tester, leg_ref).await
    }

    pub async fn sender_revert(&self, tester: &DartAssetTester, leg_ref: LegRef) -> Result<()> {
        self.0.write().await.sender_revert(tester, leg_ref).await
    }
}

pub struct DartAssetTesterInner {
    pub tester: PolymeshTester,
    pub users: BTreeMap<String, DartUser>,
    pub account_tree: AccountCurveTree,
    pub asset_tree: AssetCurveTree,
    pub asset_lookup: BTreeMap<String, DartAssetId>,
    pub assets: BTreeMap<DartAssetId, DartTestAsset>,
}

impl DartAssetTesterInner {
    pub async fn init(user_names: &[&str], relayer: Option<&str>) -> Result<Self> {
        let mut tester = PolymeshTester::new().await?;

        let account_tree = AccountCurveTree::new(&tester.api).await?;
        let asset_tree = AssetCurveTree::new(&tester.api).await?;

        // Merge names to create all users in a batch.
        let mut names = user_names.to_vec();
        if let Some(relayer_name) = relayer {
            if !names.contains(&relayer_name) {
                names.push(relayer_name);
            }
        }
        let users = tester.users(&names).await?;

        let mut tester = Self {
            users: names
                .into_iter()
                .zip(users.into_iter())
                .map(|(n, u)| (n.to_string(), DartUser::new(u)))
                .collect(),
            tester,
            account_tree,
            asset_tree,
            asset_lookup: BTreeMap::new(),
            assets: BTreeMap::new(),
        };

        // Set relayer for all users if provided.
        if let Some(relayer_name) = relayer {
            let relayer = tester.user(relayer_name);
            for (name, user) in tester.users.iter_mut() {
                if name != relayer_name {
                    user.set_relayer(relayer.clone()).await;
                }
            }
        }

        Ok(tester)
    }

    pub fn user(&self, name: &str) -> DartUser {
        self.users.get(name).expect("Missing Investor").clone()
    }

    pub fn api(&self) -> Api {
        self.tester.api.clone()
    }

    pub async fn get_settlement_leg(&self, leg_ref: LegRef) -> Result<Option<LegEncrypted>> {
        let leg = self
            .api()
            .query()
            .confidential_assets()
            .settlement_legs(to_scale(&leg_ref.settlement), leg_ref.leg_id)
            .await?;

        Ok(leg.map(|l| to_scale(&l)))
    }

    pub fn get_named_asset(&self, name: &str) -> Option<DartTestAsset> {
        self.asset_lookup
            .get(name)
            .and_then(|id| self.assets.get(id))
            .cloned()
    }

    pub fn get_asset(&self, asset_id: DartAssetId) -> Option<DartTestAsset> {
        self.assets.get(&asset_id).cloned()
    }

    pub fn register_asset(&mut self, name: String, asset_id: DartAssetId, asset: DartTestAsset) {
        // Store the asset in the lookup map.
        self.asset_lookup.insert(name, asset_id);
        self.assets.insert(asset_id, asset);
    }
}

#[derive(Clone)]
pub struct DartAssetTester(Arc<RwLock<DartAssetTesterInner>>);

impl DartAssetTester {
    pub async fn init(user_names: &[&str]) -> Result<Self> {
        let inner = DartAssetTesterInner::init(user_names, None).await?;
        Ok(Self(Arc::new(RwLock::new(inner))))
    }

    pub async fn init_with_relayer(user_names: &[&str], relayer: &str) -> Result<Self> {
        let inner = DartAssetTesterInner::init(user_names, Some(relayer)).await?;
        Ok(Self(Arc::new(RwLock::new(inner))))
    }

    pub fn spawn<F, Fut>(&self, f: F) -> tokio::task::JoinHandle<Fut::Output>
    where
        F: FnOnce(DartAssetTester) -> Fut,
        Fut: core::future::Future + Send + 'static,
        Fut::Output: Send + 'static,
    {
        let tester = self.clone();
        tokio::spawn(f(tester))
    }

    pub async fn user(&self, name: &str) -> DartUser {
        self.0.read().await.user(name)
    }

    pub async fn api(&self) -> Api {
        self.0.read().await.api()
    }

    pub async fn account_tree(&self) -> AccountCurveTree {
        self.0.read().await.account_tree.clone()
    }

    pub async fn asset_tree(&self) -> AssetCurveTree {
        self.0.read().await.asset_tree.clone()
    }

    pub async fn get_settlement_leg(&self, leg_ref: LegRef) -> Result<Option<LegEncrypted>> {
        self.0.read().await.get_settlement_leg(leg_ref).await
    }

    pub async fn get_named_asset(&self, name: &str) -> Option<DartTestAsset> {
        self.0.read().await.get_named_asset(name)
    }

    pub async fn get_asset(&self, asset_id: DartAssetId) -> Option<DartTestAsset> {
        self.0.read().await.get_asset(asset_id)
    }

    pub async fn register_asset(&self, asset: DartTestAsset) {
        let name = asset.name().await;
        let asset_id = asset.id;
        self.0.write().await.register_asset(name, asset_id, asset);
    }

    pub async fn create_asset(
        &self,
        asset_issuer: &DartUser,
        name: &str,
        mediators: &[&DartUser],
        auditors: &[&DartUser],
        mint_amount: Option<DartBalance>,
    ) -> Result<DartTestAsset> {
        // Check if the asset already exists.
        if let Some(asset) = self.0.read().await.get_named_asset(name) {
            // mint more if needed.
            if let Some(amount) = mint_amount {
                asset.mint_more(self, amount).await?;
            }
            return Ok(asset);
        }

        let account_tree = self.account_tree().await;
        // Create a new asset.
        let asset = DartTestAsset::new(
            &account_tree,
            asset_issuer,
            name,
            mediators,
            auditors,
            mint_amount,
        )
        .await?;

        // Register the asset in the tester.
        self.register_asset(asset.clone()).await;

        Ok(asset)
    }

    pub async fn create_asset_and_fund_investors(
        &self,
        asset_issuer: &DartUser,
        name: &str,
        mediators: &[&DartUser],
        auditors: &[&DartUser],
        mint_extra_amount: Option<DartBalance>,
        investors: &[&DartUser],
        amount: DartBalance,
        use_instant: bool,
    ) -> Result<DartTestAsset> {
        let total_mint = mint_extra_amount.unwrap_or(0) + (investors.len() as DartBalance * amount);

        // Register all accounts (issuer, mediators, auditors, investors).
        let mut tasks = Vec::new();
        {
            // Register asset issuer's account.
            let asset_issuer = asset_issuer.clone();
            tasks.push(tokio::spawn(async move {
                asset_issuer.register_account().await
            }));
            // Register mediators' encryption key.
            for &mediator in mediators {
                let mediator = mediator.clone();
                tasks.push(tokio::spawn(
                    async move { mediator.register_account().await },
                ));
            }
            // Register auditors' encryption key.
            for &auditor in auditors {
                let auditor = auditor.clone();
                tasks.push(tokio::spawn(async move {
                    auditor.register_encryption_key().await
                }));
            }
            // Register investors' accounts.
            for &investor in investors {
                let investor = investor.clone();
                tasks.push(tokio::spawn(
                    async move { investor.register_account().await },
                ));
            }
        }
        // Wait for all registrations to complete.
        for task in tasks {
            task.await??; // Propagate any errors.
        }

        // Create the asset with the total mint amount.
        let asset = self
            .create_asset(asset_issuer, name, mediators, auditors, Some(total_mint))
            .await?;

        // Register and fund each investor.
        asset
            .register_and_fund_investors(self, investors, amount, use_instant)
            .await?;

        Ok(asset)
    }
}

pub struct DartTestAssetInner {
    pub id: DartAssetId,
    pub issuer: DartUser,
    pub name: String,
    issuer_balance: DartBalance,
    pub auditors: Vec<DartUser>,
    pub mediators: Vec<DartUser>,
    pub total_supply: DartBalance,
}

impl DartTestAssetInner {
    pub async fn new(
        account_tree: &AccountCurveTree,
        asset_issuer: &DartUser,
        name: &str,
        mediators: &[&DartUser],
        auditors: &[&DartUser],
        mint_amount: Option<DartBalance>,
    ) -> Result<Self> {
        assert!(
            (auditors.len() + mediators.len()) >= 1,
            "At least one auditor or mediator is required"
        );

        // Asset issuer register their account.
        asset_issuer.register_account().await?;

        // Create mediator user and keys.
        let mut track_enc_keys = BTreeSet::new();
        let mut auditor_keys = BTreeSet::new();
        for &auditor in auditors {
            auditor.register_encryption_key().await?;
            let enc_key = auditor.public_keys().await.enc;
            track_enc_keys.insert(enc_key);
            auditor_keys.insert(enc_key);
        }
        let mut mediator_keys = BTreeMap::new();
        for &mediator in mediators {
            mediator.register_account().await?;
            let med_keys = mediator.public_keys().await;
            track_enc_keys.insert(med_keys.enc);
            mediator_keys.insert(med_keys.acct, med_keys.enc);
        }

        // Create the asset.
        let asset_id = asset_issuer
            .create_asset(name, "TST", 0u8, "Test asset", mediator_keys, auditor_keys)
            .await?;

        let mint_amount = mint_amount.unwrap_or(0);
        if mint_amount > 0 {
            // Asset issuer registers the asset.
            asset_issuer.register_account_asset(asset_id).await?;

            // Mint asset proof.
            asset_issuer
                .mint_asset(account_tree, asset_id, mint_amount)
                .await?;
        }

        Ok(Self {
            id: asset_id,
            name: name.to_string(),
            issuer: asset_issuer.clone(),
            issuer_balance: mint_amount,
            mediators: mediators.into_iter().copied().cloned().collect(),
            auditors: auditors.into_iter().copied().cloned().collect(),
            total_supply: mint_amount,
        })
    }

    pub fn mediator_count(&self) -> usize {
        self.mediators.len()
    }

    pub fn mediators(&self) -> Vec<DartUser> {
        self.mediators.clone()
    }

    pub async fn asset_state(&self) -> Result<AssetState> {
        let mut auditors = Vec::new();
        for auditor in &self.auditors {
            auditors.push(auditor.public_keys().await.enc);
        }
        let mut mediators = Vec::new();
        for mediator in &self.mediators {
            let med_keys = mediator.public_keys().await;
            mediators.push((med_keys.acct, med_keys.enc));
        }
        Ok(AssetState::new::<()>(self.id, &mediators, &auditors)?)
    }

    pub async fn mint_more(&mut self, tester: &DartAssetTester, amount: DartBalance) -> Result<()> {
        // Mint asset proof.
        self.issuer
            .mint_asset(&tester.account_tree().await, self.id, amount)
            .await?;

        // Update the total supply and issuer balance.
        self.total_supply += amount;
        self.issuer_balance += amount;

        Ok(())
    }

    pub async fn take_issuer_funds(
        &mut self,
        tester: &DartAssetTester,
        amount: DartBalance,
    ) -> Result<()> {
        if amount > self.issuer_balance {
            self.mint_more(tester, amount).await?;
        }

        self.issuer_balance -= amount;

        Ok(())
    }

    pub async fn register_investor(&mut self, user: &DartUser) -> Result<()> {
        // Register the account.
        user.register_account().await?;

        // Account asset registration proof.
        user.register_account_asset(self.id).await?;

        Ok(())
    }

    pub async fn register_investors(&mut self, investors: &[&DartUser]) -> Result<()> {
        let asset_id = self.id;
        // Register each investor.
        let mut tasks = Vec::with_capacity(investors.len());
        for &user in investors {
            let user = user.clone();
            tasks.push(tokio::spawn(async move {
                // Register the account.
                user.register_account().await?;

                // Account asset registration proof.
                user.register_account_asset(asset_id).await
            }));
        }
        // Wait for all registrations to complete.
        for task in tasks {
            task.await??; // Propagate any errors.
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct DartTestAsset {
    pub id: DartAssetId,
    inner: Arc<RwLock<DartTestAssetInner>>,
}

impl DartTestAsset {
    pub async fn new(
        account_tree: &AccountCurveTree,
        asset_issuer: &DartUser,
        name: &str,
        mediators: &[&DartUser],
        auditors: &[&DartUser],
        mint_amount: Option<DartBalance>,
    ) -> Result<Self> {
        let inner = DartTestAssetInner::new(
            account_tree,
            asset_issuer,
            name,
            mediators,
            auditors,
            mint_amount,
        )
        .await?;

        Ok(Self {
            id: inner.id,
            inner: Arc::new(RwLock::new(inner)),
        })
    }

    pub fn from_inner(inner: DartTestAssetInner) -> Self {
        Self {
            id: inner.id,
            inner: Arc::new(RwLock::new(inner)),
        }
    }

    pub async fn name(&self) -> String {
        let inner = self.inner.read().await;
        inner.name.clone()
    }

    pub async fn issuer(&self) -> DartUser {
        let inner = self.inner.read().await;
        inner.issuer.clone()
    }

    pub async fn mediators(&self) -> Vec<DartUser> {
        let inner = self.inner.read().await;
        inner.mediators()
    }

    pub async fn asset_state(&self) -> Result<AssetState> {
        let inner = self.inner.read().await;
        inner.asset_state().await
    }

    pub async fn register_investor(&self, user: &DartUser) -> Result<()> {
        let mut inner = self.inner.write().await;
        inner.register_investor(user).await
    }

    pub async fn register_investors(&self, investors: &[&DartUser]) -> Result<()> {
        let mut inner = self.inner.write().await;
        inner.register_investors(investors).await
    }

    pub async fn mint_more(&self, tester: &DartAssetTester, amount: DartBalance) -> Result<()> {
        let mut inner = self.inner.write().await;
        inner.mint_more(tester, amount).await
    }

    pub async fn register_and_fund_investors(
        &self,
        tester: &DartAssetTester,
        investors: &[&DartUser],
        amount: DartBalance,
        use_instant: bool,
    ) -> Result<()> {
        // Get funds from the issuer.
        {
            let mut inner = self.inner.write().await;

            let total_amount = amount * investors.len() as DartBalance;
            inner.take_issuer_funds(tester, total_amount).await?;
        }

        let asset_id = self.id;
        let _settlement = DartSettlementState::register_and_fund_investors(
            tester,
            investors,
            asset_id,
            amount,
            true,
            use_instant,
        )
        .await?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct DartSettlementLegState {
    pub leg_ref: LegRef,
    pub sender: DartUser,
    pub receiver: DartUser,
    pub asset_id: DartAssetId,
    pub amount: DartBalance,
    pub mediators: Vec<DartUser>,
}

impl DartSettlementLegState {
    pub async fn sender_affirms(&self, tester: &DartAssetTester) -> Result<()> {
        self.sender
            .sender_affirmation(tester, self.leg_ref, self.asset_id, self.amount)
            .await?;
        Ok(())
    }

    pub async fn receiver_affirms(&self, tester: &DartAssetTester) -> Result<()> {
        self.receiver
            .receiver_affirmation(tester, self.leg_ref, self.asset_id, self.amount)
            .await?;
        Ok(())
    }

    pub async fn sender_revert(&self, tester: &DartAssetTester) -> Result<()> {
        self.sender.sender_revert(tester, self.leg_ref).await?;
        Ok(())
    }

    pub async fn mediators_affirm(&self, tester: &DartAssetTester, accept: bool) -> Result<()> {
        for (id, mediator) in self.mediators.iter().enumerate() {
            log::debug!(
                "Leg {:?}: Mediator {:?} affirming with keys {:?}, accept={}",
                self.leg_ref,
                id,
                mediator.public_keys().await,
                accept
            );
            mediator
                .mediator_affirmation(
                    tester,
                    self.leg_ref,
                    id as _,
                    accept,
                    Some((self.asset_id, self.amount)),
                )
                .await?;
        }
        Ok(())
    }

    pub async fn affirm_leg(&self, tester: &DartAssetTester) -> Result<()> {
        self.sender_affirms(tester).await?;
        self.receiver_affirms(tester).await?;
        self.mediators_affirm(tester, true).await?;
        Ok(())
    }

    pub async fn receiver_claim(&self, tester: &DartAssetTester) -> Result<()> {
        self.receiver.receiver_claim(tester, self.leg_ref).await?;
        Ok(())
    }
}

pub struct DartLeg {
    pub sender: DartUser,
    pub receiver: DartUser,
    pub asset_id: DartAssetId,
    pub amount: DartBalance,
}

#[derive(Clone)]
pub struct DartSettlementState {
    pub settlement_ref: SettlementRef,
    pub legs: Vec<DartSettlementLegState>,
}

impl DartSettlementState {
    pub async fn new(
        tester: &DartAssetTester,
        venue: &DartUser,
        legs: &[DartLeg],
        memo: Option<&[u8]>,
    ) -> Result<Self> {
        Self::new_full(tester, venue, legs, memo, false, false).await
    }

    pub async fn new_full(
        tester: &DartAssetTester,
        venue: &DartUser,
        legs: &[DartLeg],
        memo: Option<&[u8]>,
        affirm_and_claim: bool,
        use_instant: bool,
    ) -> Result<Self> {
        let asset_tree = tester.asset_tree().await;
        let block_number = asset_tree.get_block_number().await?;
        let root = asset_tree.fetch_root(Some(block_number)).await?;

        // Create the investors and setup settlement legs.
        let mut leg_states = Vec::with_capacity(legs.len());
        let mut settlement =
            SettlementBuilder::<()>::new_root(memo.unwrap_or(b"NewSettlement"), block_number, root);
        for (leg_idx, leg) in legs.into_iter().enumerate() {
            let asset_id = leg.asset_id;
            let amount = leg.amount;

            let leg_ref = LegRef {
                leg_id: leg_idx as _,
                ..Default::default()
            };
            // Get the asset details.
            let asset = tester
                .get_asset(asset_id)
                .await
                .ok_or_else(|| anyhow::anyhow!("Asset not registered: {}", asset_id))?;
            let asset_state = asset.asset_state().await?;

            // Add the asset path to the settlement.
            let path = asset_tree
                .get_path_to_leaf(asset_id as _, 0, Some(block_number))
                .await?;
            settlement.add_path(asset_id, path)?;

            settlement.add_leg(LegBuilder {
                sender: leg.sender.public_keys().await,
                receiver: leg.receiver.public_keys().await,
                asset: asset_state,
                amount,
                config: LegConfig::default(),
                public_enc_keys: vec![],
            });
            leg_states.push(DartSettlementLegState {
                leg_ref,
                sender: leg.sender.clone(),
                receiver: leg.receiver.clone(),
                asset_id,
                amount,
                mediators: asset.mediators().await,
            });
        }

        // Create the settlement to transfer assets to all investors.
        let proof = {
            let mut rng = rand::thread_rng();
            settlement.build(&mut rng)?
        };
        let settlement_ref = proof.settlement_ref();

        // Correct the settlement reference in the legs.
        for leg in &mut leg_states {
            leg.leg_ref.settlement = settlement_ref;
        }

        if use_instant {
            let mut leg_affirmations = Vec::with_capacity(legs.len());

            // Generate the instant leg affirmations.
            for (leg_state, leg) in leg_states.iter().zip(proof.legs.iter()) {
                let leg_ref = leg_state.leg_ref;
                let asset_id = leg_state.asset_id;
                let amount = leg_state.amount;
                let leg_enc = leg.leg_enc();
                // Sender instant affirmation proof.
                let sender = leg_state
                    .sender
                    .instant_sender_affirmation_proof(tester, leg_ref, leg_enc, asset_id, amount)
                    .await?;
                // Receiver instant affirmation proof.
                let receiver = leg_state
                    .receiver
                    .instant_receiver_affirmation_proof(tester, leg_ref, leg_enc, asset_id, amount)
                    .await?;

                let mut mediators = Vec::new();
                for (id, mediator) in leg_state.mediators.iter().enumerate() {
                    let mediator_proof = mediator
                        .mediator_affirmation_proof(tester, leg_ref, id as _, true, None)
                        .await?;
                    mediators.push(mediator_proof);
                }

                leg_affirmations.push(InstantSettlementLegAffirmations {
                    sender,
                    receiver,
                    mediators: mediators.try_into().expect("Should have correct length"),
                });
            }

            let instant_settlement = InstantSettlementProof {
                settlement: proof,
                leg_affirmations: leg_affirmations
                    .try_into()
                    .expect("Should have the correct length"),
            };
            // Submit the instant settlement proof.
            venue.execute_instant_settlement(instant_settlement).await?;
        } else {
            // Submit the settlement proof.
            venue.create_settlement(proof).await?;
        }

        let settlement = Self {
            settlement_ref,
            legs: leg_states,
        };

        // Affirm and claim assets if requested.  Instant settlements included affirmations.
        if affirm_and_claim && !use_instant {
            // Affirm all the legs in the settlement.
            settlement.affirm_legs(tester).await?;

            // All investors claim their assets.
            settlement.receivers_claim_assets(tester).await?;
        }

        Ok(settlement)
    }

    pub async fn register_and_fund_investors(
        tester: &DartAssetTester,
        investors: &[&DartUser],
        asset_id: DartAssetId,
        amount: DartBalance,
        affirm_and_claim: bool,
        use_instant: bool,
    ) -> Result<Self> {
        let asset = tester
            .get_asset(asset_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("Asset not registered: {}", asset_id))?;
        let issuer = asset.issuer().await;

        // Register all investors for the asset.
        asset.register_investors(investors).await?;

        let legs = investors
            .into_iter()
            .map(|&investor| DartLeg {
                sender: issuer.clone(),
                receiver: investor.clone(),
                asset_id,
                amount,
            })
            .collect::<Vec<_>>();

        Ok(Self::new_full(
            tester,
            &issuer,
            &legs,
            Some(b"FundInvestors"),
            affirm_and_claim,
            use_instant,
        )
        .await?)
    }

    pub async fn senders_affirm_legs(&self, tester: &DartAssetTester) -> Result<()> {
        // All senders affirm their legs.
        for leg in &self.legs {
            leg.sender_affirms(tester).await?;
        }
        Ok(())
    }

    pub async fn receivers_affirm_legs(&self, tester: &DartAssetTester) -> Result<()> {
        // All receivers affirm their legs.
        for leg in &self.legs {
            leg.receiver_affirms(tester).await?;
        }
        Ok(())
    }

    pub async fn senders_revert_legs(&self, tester: &DartAssetTester) -> Result<()> {
        // All senders revert their legs.
        for leg in &self.legs {
            leg.sender_revert(tester).await?;
        }
        Ok(())
    }

    pub async fn mediators_affirm_legs(
        &self,
        tester: &DartAssetTester,
        accept: bool,
    ) -> Result<()> {
        // All mediators affirm their legs.
        for leg in &self.legs {
            leg.mediators_affirm(tester, accept).await?;
        }
        Ok(())
    }

    pub async fn affirm_legs(&self, tester: &DartAssetTester) -> Result<()> {
        // Affirm all the legs in the settlement.
        let mut tasks = Vec::with_capacity(self.legs.len());
        for leg in &self.legs {
            // Sender affirm task.
            {
                let tester = tester.clone();
                let leg = leg.clone();
                tasks.push(tokio::spawn(
                    async move { leg.sender_affirms(&tester).await },
                ));
            }
            // Receiver affirm task.
            {
                let tester = tester.clone();
                let leg = leg.clone();
                tasks.push(tokio::spawn(
                    async move { leg.receiver_affirms(&tester).await },
                ));
            }
            // Mediator affirm task.
            if leg.mediators.len() > 0 {
                let tester = tester.clone();
                let leg = leg.clone();
                tasks.push(tokio::spawn(async move {
                    leg.mediators_affirm(&tester, true).await
                }));
            }
        }
        // Wait for all tasks to complete.
        for task in tasks {
            task.await??;
        }
        Ok(())
    }

    pub async fn receivers_claim_assets(&self, tester: &DartAssetTester) -> Result<()> {
        // All investors claim their assets.
        let mut tasks = Vec::with_capacity(self.legs.len());
        for leg in &self.legs {
            // Investor claims the assets.
            {
                let tester = tester.clone();
                let leg = leg.clone();
                tasks.push(tokio::spawn(
                    async move { leg.receiver_claim(&tester).await },
                ));
            }
        }
        // Wait for all tasks to complete.
        for task in tasks {
            task.await??;
        }
        Ok(())
    }
}

// ============================================================================
// Test Helpers for Negative Test Cases
// ============================================================================

/// Helper to create a settlement with a specific configuration for negative tests.
/// This allows testing various failure scenarios.
pub async fn create_test_settlement(
    tester: &DartAssetTester,
    venue: &DartUser,
    sender: &DartUser,
    receiver: &DartUser,
    asset_id: DartAssetId,
    amount: DartBalance,
) -> Result<DartSettlementState> {
    let legs = vec![DartLeg {
        sender: sender.clone(),
        receiver: receiver.clone(),
        asset_id,
        amount,
    }];

    DartSettlementState::new(tester, venue, &legs, Some(b"TestSettlement")).await
}

/// Helper to test that an operation fails as expected, logging the error for debugging.
pub fn assert_operation_fails<T>(result: Result<T>, context: &str) {
    match result {
        Ok(_) => panic!("Expected {} to fail, but it succeeded", context),
        Err(e) => {
            log::info!("Operation '{}' failed as expected: {:?}", context, e);
        }
    }
}
