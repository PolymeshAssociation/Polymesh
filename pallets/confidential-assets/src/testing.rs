use frame_support::assert_ok;
use polymesh_dart::{AccountRegistrationProof, BatchedAccountAssetRegistrationProof};
use scale_info::prelude::format;
use sp_std::cell::RefCell;
use sp_std::collections::btree_map::BTreeMap;
use sp_std::rc::Rc;
use sp_std::{vec, vec::Vec};

use pallet_identity::benchmarking::{user, User};

use polymesh_dart::{
    curve_tree::{MultiLeafPathAndRoot, ProverCurveTree},
    AccountAssetRegistrationProof, AccountAssetState, AccountKeyPair, AccountKeys,
    AccountPublicKeys, AssetMintingProof, AssetState, Balance as DartBalance, EncryptionKeyPair,
    FeeAccountAssetState, LegBuilder, LegRole, MediatorAffirmationProof, ReceiverAffirmationProof,
    ReceiverClaimProof, SenderAffirmationProof, SenderCounterUpdateProof, SenderReversalProof,
    SettlementBuilder, ACCOUNT_TREE_L, ACCOUNT_TREE_M, ASSET_TREE_L, ASSET_TREE_M,
    FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M,
};
use polymesh_primitives::erc20::{Name, Symbol};
use polymesh_worker_protocol_dart_v0::{GenerateDartProofRequest, GenerateDartProofResponse};

use crate::*;

pub(crate) const SEED: u32 = 42;
pub(crate) const TICKER_SEED: u32 = 10_000_000;
pub(crate) const AUDITOR_SEED: u32 = 10_000;

pub fn generate<T: Config>(req: GenerateDartProofRequest) -> GenerateDartProofResponse {
    let session_id = Pallet::<T>::session_id().expect("Session ID not set");
    req.submit_and_wait(session_id)
        .expect("Failed to generate proof")
}

// Make sure the pallet is initialized.
pub fn init_block<T: Config>() {
    if Pallet::<T>::session_id().is_err() {
        Pallet::<T>::init_block();
    }
}

pub fn set_skip_verify<T: Config>(_skip: bool) {
    // TODO: Implement skip verify for worker extension.
}

pub type AccountProverCurveTree<T> = ProverCurveTree<
    ACCOUNT_TREE_L,
    ACCOUNT_TREE_M,
    AccountTreeConfig,
    AccountCurveTreeStorage<T>,
    Error<T>,
>;

pub fn get_prover_account_curve_tree<T: Config>() -> AccountProverCurveTree<T> {
    AccountProverCurveTree::new(ACCOUNT_TREE_HEIGHT).expect("Failed to create prover curve tree")
}

pub type FeeAccountProverCurveTree<T> = ProverCurveTree<
    FEE_ACCOUNT_TREE_L,
    FEE_ACCOUNT_TREE_M,
    FeeAccountTreeConfig,
    FeeAccountCurveTreeStorage<T>,
    Error<T>,
>;

pub fn get_prover_fee_account_curve_tree<T: Config>() -> FeeAccountProverCurveTree<T> {
    FeeAccountProverCurveTree::new(FEE_ACCOUNT_TREE_HEIGHT)
        .expect("Failed to create prover curve tree")
}
pub type AssetProverCurveTree<T> = ProverCurveTree<
    ASSET_TREE_L,
    ASSET_TREE_M,
    AssetTreeConfig,
    AssetCurveTreeStorage<T>,
    Error<T>,
>;

pub fn get_prover_asset_curve_tree<T: Config>() -> AssetProverCurveTree<T> {
    AssetProverCurveTree::new(ASSET_TREE_HEIGHT).expect("Failed to create prover curve tree")
}

pub struct OffchainProverState<T: Config> {
    pub account_tree: AccountProverCurveTree<T>,
    pub fee_account_tree: FeeAccountProverCurveTree<T>,
    pub asset_tree: AssetProverCurveTree<T>,
}

impl<T: Config> OffchainProverState<T> {
    pub fn new() -> Self {
        init_block::<T>();

        Self {
            account_tree: get_prover_account_curve_tree(),
            fee_account_tree: get_prover_fee_account_curve_tree(),
            asset_tree: get_prover_asset_curve_tree(),
        }
    }

    pub fn apply_new_leaves(&mut self) {
        // Update the curve tree roots.
        Pallet::<T>::update_account_curve_tree_root();
        Pallet::<T>::update_fee_account_curve_tree_root();

        self.account_tree.apply_new_leaves();
        self.fee_account_tree.apply_new_leaves();
        self.asset_tree.apply_new_leaves();
    }

    pub fn get_settlement_leg(&self, leg_ref: LegRef) -> LegEncrypted {
        SettlementLegs::<T>::get((leg_ref.settlement, leg_ref.leg_id))
            .expect("Failed to get settlement leg")
    }
}

pub struct DartUserInner<T: Config> {
    pub user: User<T>,
    pub keys: AccountKeys,
    pub assets: BTreeMap<ConfidentialAssetId, AccountAssetState>,
}

impl<T: Config> DartUserInner<T> {
    pub fn new(name: &'static str) -> Self {
        Self::new_from_seed(name, SEED)
    }

    pub fn asset_user(name: &'static str, asset: u32, user_idx: u32) -> Self {
        Self::new_from_seed(name, (asset + 1) * TICKER_SEED + user_idx)
    }

    pub fn auditor_user(name: &'static str, asset: u32, auditor_idx: u32) -> Self {
        Self::new_from_seed(
            name,
            (asset + 1) * TICKER_SEED + (auditor_idx + 1) * AUDITOR_SEED,
        )
    }

    fn new_from_seed(name: &'static str, seed: u32) -> Self {
        let user = user::<T>(name, seed);
        let key_seed = format!("{}-{}", name, seed);
        let keys = AccountKeys::from_seed(&key_seed).expect("Failed to generate random keys");
        Self {
            user,
            keys,
            assets: BTreeMap::new(),
        }
    }

    pub fn new_account(&self, name: &'static str, seed: u32) -> Self {
        let key_seed = format!("{}-{}", name, seed);
        let keys = AccountKeys::from_seed(&key_seed).expect("Failed to generate random keys");
        Self {
            user: self.user.clone(),
            keys,
            assets: BTreeMap::new(),
        }
    }

    pub fn public_keys(&self) -> AccountPublicKeys {
        self.keys.public_keys()
    }

    pub fn did(&self) -> IdentityId {
        self.user.did()
    }

    pub fn account(&self) -> T::AccountId {
        self.user.account()
    }

    pub fn origin(&self) -> <T as frame_system::Config>::RuntimeOrigin {
        self.user.origin().into()
    }

    pub fn raw_origin(&self) -> frame_system::RawOrigin<T::AccountId> {
        self.user.origin()
    }

    pub fn register_encryption_keys_proof(
        &self,
        keys: Vec<EncryptionKeyPair>,
    ) -> EncryptionKeyRegistrationProof<PolymeshPrivateLimits> {
        // Generate the encryption key registration proof.
        let req = GenerateDartProofRequest::EncryptionKeyRegistration {
            keys,
            did: self.did().into(),
        };
        if let GenerateDartProofResponse::EncryptionKeyRegistration { proof } = generate::<T>(req) {
            return proof;
        } else {
            panic!("Failed to generate encryption key registration proof");
        }
    }

    pub fn register_encryption_key(&self) {
        self.register_encryption_keys(vec![self.keys.enc.clone()]);
    }

    pub fn register_encryption_keys(&self, keys: Vec<EncryptionKeyPair>) {
        let proof = self.register_encryption_keys_proof(keys);

        assert_ok!(Pallet::<T>::register_encryption_keys(self.origin(), proof));
    }

    pub fn register_accounts_proof(
        &self,
        accounts: Vec<AccountKeys>,
    ) -> AccountRegistrationProof<PolymeshPrivateLimits> {
        // Generate the account registration proof.
        let req = GenerateDartProofRequest::AccountRegistration {
            accounts,
            did: self.did().into(),
        };
        if let GenerateDartProofResponse::AccountRegistration { proof } = generate::<T>(req) {
            return proof;
        } else {
            panic!("Failed to generate account registration proof");
        }
    }

    pub fn register_account(&self) {
        self.register_accounts(vec![self.keys.clone()]);
    }

    pub fn register_accounts(&self, accounts: Vec<AccountKeys>) {
        let proof = self.register_accounts_proof(accounts);

        assert_ok!(Pallet::<T>::register_accounts(self.origin(), proof));
    }

    pub fn register_fee_accounts_proof(
        &self,
        accounts: Vec<(AccountKeyPair, ConfidentialAssetId, DartBalance)>,
    ) -> (
        BatchedFeeAccountRegistrationProof<PolymeshPrivateLimits>,
        Vec<FeeAccountAssetState>,
    ) {
        // Generate the account registration proof.
        let req = GenerateDartProofRequest::BatchedFeeAccountRegistration {
            accounts,
            did: self.did().into(),
        };
        if let GenerateDartProofResponse::BatchedFeeAccountRegistration {
            proof,
            account_states,
        } = generate::<T>(req)
        {
            return (proof, account_states);
        } else {
            panic!("Failed to generate fee account registration proof");
        }
    }

    pub fn register_fee_account(&self, amount: DartBalance) -> FeeAccountAssetState {
        self.register_fee_accounts(vec![(self.keys.acct.clone(), FEE_ASSET_ID, amount)])[0].clone()
    }

    pub fn register_fee_accounts(
        &self,
        accounts: Vec<(AccountKeyPair, ConfidentialAssetId, DartBalance)>,
    ) -> Vec<FeeAccountAssetState> {
        let (proof, mut states) = self.register_fee_accounts_proof(accounts);

        for state in &mut states {
            // Commit the pending state to the fee account state.
            state
                .commit_pending_state()
                .expect("Failed to commit pending state");
        }
        assert_ok!(Pallet::<T>::register_fee_accounts(self.origin(), proof));

        states
    }

    pub fn register_account_asset_proof(
        &mut self,
        asset_id: ConfidentialAssetId,
    ) -> (AccountAssetRegistrationProof, AccountAssetState) {
        assert!(
            !self.assets.contains_key(&asset_id),
            "Asset already registered"
        );

        // Generate the account asset registration proof.
        let req = GenerateDartProofRequest::AccountAssetRegistration {
            keys: self.keys.clone(),
            did: self.did().into(),
            asset_id,
            counter: 0,
        };
        if let GenerateDartProofResponse::AccountAssetRegistration {
            proof,
            account_state,
        } = generate::<T>(req)
        {
            return (proof, account_state);
        } else {
            panic!("Failed to generate account asset registration proof");
        }
    }

    pub fn register_account_asset(&mut self, asset_id: ConfidentialAssetId) {
        let (proof, mut account_state) = self.register_account_asset_proof(asset_id);

        let proof = BatchedAccountAssetRegistrationProof {
            proofs: vec![proof].try_into().expect("Failed to create vec"),
        };
        assert_ok!(Pallet::<T>::register_account_assets(self.origin(), proof));

        // Commit the pending state to the account state.
        account_state
            .commit_pending_state()
            .expect("Failed to commit pending state");

        self.assets.insert(asset_id, account_state);
    }

    pub fn mint_asset_proof(
        &mut self,
        off_chain: &OffchainProverState<T>,
        asset_id: ConfidentialAssetId,
        amount: DartBalance,
    ) -> (AssetMintingProof<AccountTreeConfig>, &mut AccountAssetState) {
        let account_state = self
            .assets
            .get_mut(&asset_id)
            .expect("Asset not registered for user");

        // Generate the mint asset proof.
        let current_state_commitment = account_state
            .current_commitment(&self.keys)
            .expect("current commitment");
        let req = GenerateDartProofRequest::MintAsset {
            keys: self.keys.clone(),
            amount,
            path: off_chain
                .account_tree
                .get_path_and_root(current_state_commitment.as_leaf_value().expect("leaf path"))
                .expect("Failed to get path to leaf"),
            account_state: account_state.clone(),
        };
        if let GenerateDartProofResponse::MintAsset {
            proof,
            account_state: new_state,
        } = generate::<T>(req)
        {
            *account_state = new_state;
            return (proof, account_state);
        } else {
            panic!("Failed to generate mint asset proof");
        }
    }

    pub fn mint_asset(
        &mut self,
        off_chain: &OffchainProverState<T>,
        asset_id: ConfidentialAssetId,
        amount: DartBalance,
    ) {
        let origin = self.origin();
        let (mint_proof, account_state) = self.mint_asset_proof(off_chain, asset_id, amount);

        assert_ok!(Pallet::<T>::mint_asset(origin, mint_proof));

        // Update pending state in the account state.
        account_state
            .commit_pending_state()
            .expect("Failed to commit pending state");
    }

    pub fn sender_affirmation_proof(
        &mut self,
        off_chain: &OffchainProverState<T>,
        leg_ref: LegRef,
        asset_id: ConfidentialAssetId,
        amount: DartBalance,
    ) -> (
        SenderAffirmationProof<AccountTreeConfig>,
        &mut AccountAssetState,
    ) {
        let leg_enc = off_chain.get_settlement_leg(leg_ref);

        // Get the account state for the asset.
        let account_state = self
            .assets
            .get_mut(&asset_id)
            .expect("Asset not registered for user");

        // Generate the sender affirmation proof.
        let current_state_commitment = account_state
            .current_commitment(&self.keys)
            .expect("current commitment");
        let req = GenerateDartProofRequest::SenderAffirmation {
            keys: self.keys.clone(),
            leg_ref,
            amount,
            leg_enc: leg_enc.clone(),
            path: off_chain
                .account_tree
                .get_path_and_root(current_state_commitment.as_leaf_value().expect("leaf path"))
                .expect("Failed to get path to leaf"),
            account_state: account_state.clone(),
        };
        if let GenerateDartProofResponse::SenderAffirmation {
            proof,
            account_state: new_state,
        } = generate::<T>(req)
        {
            *account_state = new_state;
            return (proof, account_state);
        } else {
            panic!("Failed to generate sender affirmation proof");
        }
    }

    pub fn instant_sender_affirmation_proof(
        &mut self,
        off_chain: &OffchainProverState<T>,
        leg_ref: LegRef,
        asset_id: ConfidentialAssetId,
        amount: DartBalance,
    ) -> (
        InstantSenderAffirmationProof<AccountTreeConfig>,
        &mut AccountAssetState,
    ) {
        let leg_enc = off_chain.get_settlement_leg(leg_ref);

        // Get the account state for the asset.
        let account_state = self
            .assets
            .get_mut(&asset_id)
            .expect("Asset not registered for user");

        // Generate the instant sender affirmation proof.
        let current_state_commitment = account_state
            .current_commitment(&self.keys)
            .expect("current commitment");
        let req = GenerateDartProofRequest::InstantSenderAffirmation {
            keys: self.keys.clone(),
            leg_ref,
            amount,
            leg_enc: leg_enc.clone(),
            path: off_chain
                .account_tree
                .get_path_and_root(current_state_commitment.as_leaf_value().expect("leaf path"))
                .expect("Failed to get path to leaf"),
            account_state: account_state.clone(),
        };
        if let GenerateDartProofResponse::InstantSenderAffirmation {
            proof,
            account_state: new_state,
        } = generate::<T>(req)
        {
            *account_state = new_state;
            return (proof, account_state);
        } else {
            panic!("Failed to generate instant sender affirmation proof");
        }
    }

    pub fn sender_affirmation(
        &mut self,
        off_chain: &OffchainProverState<T>,
        leg_ref: LegRef,
        asset_id: ConfidentialAssetId,
        amount: DartBalance,
    ) {
        let origin = self.origin();
        let (proof, account_state) =
            self.sender_affirmation_proof(off_chain, leg_ref, asset_id, amount);

        assert_ok!(Pallet::<T>::sender_affirmation(origin, proof));

        // Update pending state in the account state.
        account_state
            .commit_pending_state()
            .expect("Failed to commit pending state");
    }

    pub fn receiver_affirmation_proof(
        &mut self,
        off_chain: &OffchainProverState<T>,
        leg_ref: LegRef,
        asset_id: ConfidentialAssetId,
        _amount: DartBalance,
    ) -> (
        ReceiverAffirmationProof<AccountTreeConfig>,
        &mut AccountAssetState,
    ) {
        let leg_enc = off_chain.get_settlement_leg(leg_ref);

        // Get the account state for the asset.
        let account_state = self
            .assets
            .get_mut(&asset_id)
            .expect("Asset not registered for user");

        // Generate the receiver affirmation proof.
        let current_state_commitment = account_state
            .current_commitment(&self.keys)
            .expect("current commitment");
        let req = GenerateDartProofRequest::ReceiverAffirmation {
            keys: self.keys.clone(),
            leg_ref,
            leg_enc: leg_enc.clone(),
            path: off_chain
                .account_tree
                .get_path_and_root(current_state_commitment.as_leaf_value().expect("leaf path"))
                .expect("Failed to get path to leaf"),
            account_state: account_state.clone(),
        };
        if let GenerateDartProofResponse::ReceiverAffirmation {
            proof,
            account_state: new_state,
        } = generate::<T>(req)
        {
            *account_state = new_state;
            return (proof, account_state);
        } else {
            panic!("Failed to generate receiver affirmation proof");
        }
    }

    pub fn instant_receiver_affirmation_proof(
        &mut self,
        off_chain: &OffchainProverState<T>,
        leg_ref: LegRef,
        asset_id: ConfidentialAssetId,
        amount: DartBalance,
    ) -> (
        InstantReceiverAffirmationProof<AccountTreeConfig>,
        &mut AccountAssetState,
    ) {
        let leg_enc = off_chain.get_settlement_leg(leg_ref);

        // Get the account state for the asset.
        let account_state = self
            .assets
            .get_mut(&asset_id)
            .expect("Asset not registered for user");

        // Generate the instant receiver affirmation proof.
        let current_state_commitment = account_state
            .current_commitment(&self.keys)
            .expect("current commitment");
        let req = GenerateDartProofRequest::InstantReceiverAffirmation {
            keys: self.keys.clone(),
            leg_ref,
            amount,
            leg_enc: leg_enc.clone(),
            path: off_chain
                .account_tree
                .get_path_and_root(current_state_commitment.as_leaf_value().expect("leaf path"))
                .expect("Failed to get path to leaf"),
            account_state: account_state.clone(),
        };
        if let GenerateDartProofResponse::InstantReceiverAffirmation {
            proof,
            account_state: new_state,
        } = generate::<T>(req)
        {
            *account_state = new_state;
            return (proof, account_state);
        } else {
            panic!("Failed to generate instant receiver affirmation proof");
        }
    }

    pub fn receiver_affirmation(
        &mut self,
        off_chain: &OffchainProverState<T>,
        leg_ref: LegRef,
        asset_id: ConfidentialAssetId,
        amount: DartBalance,
    ) {
        let origin = self.origin();
        let (proof, account_state) =
            self.receiver_affirmation_proof(off_chain, leg_ref, asset_id, amount);

        assert_ok!(Pallet::<T>::receiver_affirmation(origin, proof));

        // Update pending state in the account state.
        account_state
            .commit_pending_state()
            .expect("Failed to commit pending state");
    }

    pub fn receiver_claim_proof(
        &mut self,
        off_chain: &OffchainProverState<T>,
        leg_ref: LegRef,
        asset_id: ConfidentialAssetId,
        amount: DartBalance,
    ) -> (
        ReceiverClaimProof<AccountTreeConfig>,
        &mut AccountAssetState,
    ) {
        let leg_enc = off_chain.get_settlement_leg(leg_ref);

        // Get the account state for the asset.
        let account_state = self
            .assets
            .get_mut(&asset_id)
            .expect("Asset not registered for user");

        // Generate the receiver claim proof.
        let current_state_commitment = account_state
            .current_commitment(&self.keys)
            .expect("current commitment");
        let req = GenerateDartProofRequest::ReceiverClaim {
            keys: self.keys.clone(),
            leg_ref,
            amount,
            leg_enc: leg_enc.clone(),
            path: off_chain
                .account_tree
                .get_path_and_root(current_state_commitment.as_leaf_value().expect("leaf path"))
                .expect("Failed to get path to leaf"),
            account_state: account_state.clone(),
        };
        if let GenerateDartProofResponse::ReceiverClaim {
            proof,
            account_state: new_state,
        } = generate::<T>(req)
        {
            *account_state = new_state;
            return (proof, account_state);
        } else {
            panic!("Failed to generate receiver claim proof");
        }
    }

    pub fn receiver_claim(
        &mut self,
        off_chain: &OffchainProverState<T>,
        leg_ref: LegRef,
        asset_id: ConfidentialAssetId,
        amount: DartBalance,
    ) {
        let origin = self.origin();
        let (proof, account_state) =
            self.receiver_claim_proof(off_chain, leg_ref, asset_id, amount);

        assert_ok!(Pallet::<T>::receiver_claim(origin, proof));

        // Update pending state in the account state.
        account_state
            .commit_pending_state()
            .expect("Failed to commit pending state");
    }

    pub fn sender_revert_proof(
        &mut self,
        off_chain: &OffchainProverState<T>,
        leg_ref: LegRef,
        asset_id: ConfidentialAssetId,
        amount: DartBalance,
    ) -> (
        SenderReversalProof<AccountTreeConfig>,
        &mut AccountAssetState,
    ) {
        let leg_enc = off_chain.get_settlement_leg(leg_ref);

        // Get the account state for the asset.
        let account_state = self
            .assets
            .get_mut(&asset_id)
            .expect("Asset not registered for user");

        // Generate the sender revert proof.
        let current_state_commitment = account_state
            .current_commitment(&self.keys)
            .expect("current commitment");
        let req = GenerateDartProofRequest::SenderRevert {
            keys: self.keys.clone(),
            leg_ref,
            amount,
            leg_enc: leg_enc.clone(),
            path: off_chain
                .account_tree
                .get_path_and_root(current_state_commitment.as_leaf_value().expect("leaf path"))
                .expect("Failed to get path to leaf"),
            account_state: account_state.clone(),
        };
        if let GenerateDartProofResponse::SenderRevert {
            proof,
            account_state: new_state,
        } = generate::<T>(req)
        {
            *account_state = new_state;
            return (proof, account_state);
        } else {
            panic!("Failed to generate sender revert proof");
        }
    }

    pub fn sender_revert(
        &mut self,
        off_chain: &OffchainProverState<T>,
        leg_ref: LegRef,
        asset_id: ConfidentialAssetId,
        amount: DartBalance,
    ) {
        let origin = self.origin();
        let (proof, account_state) = self.sender_revert_proof(off_chain, leg_ref, asset_id, amount);

        assert_ok!(Pallet::<T>::sender_revert(origin, proof));

        // Update pending state in the account state.
        account_state
            .commit_pending_state()
            .expect("Failed to commit pending state");
    }

    pub fn sender_counter_update_proof(
        &mut self,
        off_chain: &OffchainProverState<T>,
        leg_ref: LegRef,
        asset_id: ConfidentialAssetId,
    ) -> (
        SenderCounterUpdateProof<AccountTreeConfig>,
        &mut AccountAssetState,
    ) {
        let leg_enc = off_chain.get_settlement_leg(leg_ref);

        // Get the account state for the asset.
        let account_state = self
            .assets
            .get_mut(&asset_id)
            .expect("Asset not registered for user");

        // Generate the sender counter update proof.
        let current_state_commitment = account_state
            .current_commitment(&self.keys)
            .expect("current commitment");
        let req = GenerateDartProofRequest::SenderCounterUpdate {
            keys: self.keys.clone(),
            leg_ref,
            leg_enc: leg_enc.clone(),
            path: off_chain
                .account_tree
                .get_path_and_root(current_state_commitment.as_leaf_value().expect("leaf path"))
                .expect("Failed to get path to leaf"),
            account_state: account_state.clone(),
        };
        if let GenerateDartProofResponse::SenderCounterUpdate {
            proof,
            account_state: new_state,
        } = generate::<T>(req)
        {
            *account_state = new_state;
            return (proof, account_state);
        } else {
            panic!("Failed to generate sender counter update proof");
        }
    }

    pub fn sender_counter_update(
        &mut self,
        off_chain: &OffchainProverState<T>,
        leg_ref: LegRef,
        asset_id: ConfidentialAssetId,
    ) {
        let origin = self.origin();
        let (proof, account_state) = self.sender_counter_update_proof(off_chain, leg_ref, asset_id);

        assert_ok!(Pallet::<T>::sender_update_counter(origin, proof));

        // Update pending state in the account state.
        account_state
            .commit_pending_state()
            .expect("Failed to commit pending state");
    }

    pub fn mediator_affirmation_proof(
        &self,
        off_chain: &OffchainProverState<T>,
        leg_ref: LegRef,
        accept: bool,
        verify: Option<(ConfidentialAssetId, DartBalance)>,
    ) -> MediatorAffirmationProof {
        let leg_enc = off_chain.get_settlement_leg(leg_ref);
        let leg = leg_enc
            .decrypt(LegRole::mediator(0), &self.keys)
            .expect("Failed to decrypt leg");

        if let Some((asset_id, amount)) = verify {
            assert!(
                asset_id == leg.asset_id,
                "Asset ID mismatch in encrypted leg"
            );
            assert!(amount == leg.amount(), "Amount mismatch in encrypted leg");
        }

        // Generate the mediator affirmation proof.
        let req = GenerateDartProofRequest::MediatorAffirmation {
            leg_ref,
            leg_enc: leg_enc.clone(),
            keys: self.keys.clone(),
            key_index: 0,
            accept,
        };
        if let GenerateDartProofResponse::MediatorAffirmation { proof } = generate::<T>(req) {
            return proof;
        } else {
            panic!("Failed to generate mediator affirmation proof");
        }
    }

    pub fn mediator_affirmation(
        &self,
        off_chain: &OffchainProverState<T>,
        leg_ref: LegRef,
        accept: bool,
        verify: Option<(ConfidentialAssetId, DartBalance)>,
    ) {
        let origin = self.origin();
        let proof = self.mediator_affirmation_proof(off_chain, leg_ref, accept, verify);

        assert_ok!(Pallet::<T>::mediator_affirmation(origin, proof));
    }
}

#[derive(Clone)]
pub struct DartUser<T: Config>(Rc<RefCell<DartUserInner<T>>>);

impl<T: Config> DartUser<T> {
    pub fn new(name: &'static str) -> Self {
        Self(Rc::new(RefCell::new(DartUserInner::new(name))))
    }

    pub fn asset_user(name: &'static str, asset: u32, user_idx: u32) -> Self {
        Self(Rc::new(RefCell::new(DartUserInner::asset_user(
            name, asset, user_idx,
        ))))
    }

    pub fn auditor_user(name: &'static str, asset: u32, auditor_idx: u32) -> Self {
        Self(Rc::new(RefCell::new(DartUserInner::auditor_user(
            name,
            asset,
            auditor_idx,
        ))))
    }

    pub fn public_keys(&self) -> AccountPublicKeys {
        let inner = self.0.borrow();
        inner.keys.public_keys()
    }

    pub fn keys(&self) -> AccountKeys {
        let inner = self.0.borrow();
        inner.keys.clone()
    }

    pub fn did(&self) -> IdentityId {
        let inner = self.0.borrow();
        inner.user.did()
    }

    pub fn account(&self) -> T::AccountId {
        let inner = self.0.borrow();
        inner.user.account()
    }

    pub fn origin(&self) -> <T as frame_system::Config>::RuntimeOrigin {
        let inner = self.0.borrow();
        inner.user.origin().into()
    }

    pub fn raw_origin(&self) -> frame_system::RawOrigin<T::AccountId> {
        let inner = self.0.borrow();
        inner.user.origin()
    }

    pub fn new_account(&self, name: &'static str, seed: u32) -> Self {
        let inner = self.0.borrow();
        let new_inner = inner.new_account(name, seed);
        Self(Rc::new(RefCell::new(new_inner)))
    }

    pub fn register_encryption_keys_proof(
        &self,
        keys: Vec<EncryptionKeyPair>,
    ) -> EncryptionKeyRegistrationProof<PolymeshPrivateLimits> {
        let inner = self.0.borrow();
        inner.register_encryption_keys_proof(keys)
    }

    pub fn register_encryption_key(&self) {
        let inner = self.0.borrow();
        inner.register_encryption_key();
    }

    pub fn register_encryption_keys(&self, keys: Vec<EncryptionKeyPair>) {
        let inner = self.0.borrow();
        inner.register_encryption_keys(keys);
    }

    pub fn register_account(&self) {
        let inner = self.0.borrow();
        inner.register_account();
    }

    pub fn register_accounts(&self, accounts: Vec<AccountKeys>) {
        let inner = self.0.borrow();
        inner.register_accounts(accounts);
    }

    pub fn register_fee_account(&self, amount: DartBalance) -> FeeAccountAssetState {
        let inner = self.0.borrow();
        inner.register_fee_account(amount)
    }

    pub fn register_fee_accounts(
        &self,
        accounts: Vec<(AccountKeyPair, ConfidentialAssetId, DartBalance)>,
    ) -> Vec<FeeAccountAssetState> {
        let inner = self.0.borrow();
        inner.register_fee_accounts(accounts)
    }

    pub fn register_account_asset_proof(
        &self,
        asset_id: ConfidentialAssetId,
    ) -> (AccountAssetRegistrationProof, AccountAssetState) {
        let mut inner = self.0.borrow_mut();
        inner.register_account_asset_proof(asset_id)
    }

    pub fn register_account_asset(&self, asset_id: ConfidentialAssetId) {
        let mut inner = self.0.borrow_mut();
        inner.register_account_asset(asset_id);
    }

    pub fn mint_asset_proof(
        &self,
        off_chain: &OffchainProverState<T>,
        asset_id: ConfidentialAssetId,
        amount: DartBalance,
    ) -> (AssetMintingProof<AccountTreeConfig>, AccountAssetState) {
        let mut inner = self.0.borrow_mut();
        let (proof, account_state) = inner.mint_asset_proof(off_chain, asset_id, amount);

        (proof, account_state.clone())
    }

    pub fn mint_asset(
        &self,
        off_chain: &OffchainProverState<T>,
        asset_id: ConfidentialAssetId,
        amount: DartBalance,
    ) {
        let mut inner = self.0.borrow_mut();
        inner.mint_asset(off_chain, asset_id, amount);
    }

    pub fn sender_affirmation_proof(
        &self,
        off_chain: &OffchainProverState<T>,
        leg_ref: LegRef,
        asset_id: ConfidentialAssetId,
        amount: DartBalance,
    ) -> (SenderAffirmationProof<AccountTreeConfig>, AccountAssetState) {
        let mut inner = self.0.borrow_mut();
        let (proof, account_state) =
            inner.sender_affirmation_proof(off_chain, leg_ref, asset_id, amount);
        (proof, account_state.clone())
    }

    pub fn instant_sender_affirmation_proof(
        &self,
        off_chain: &OffchainProverState<T>,
        leg_ref: LegRef,
        asset_id: ConfidentialAssetId,
        amount: DartBalance,
    ) -> (
        InstantSenderAffirmationProof<AccountTreeConfig>,
        AccountAssetState,
    ) {
        let mut inner = self.0.borrow_mut();
        let (proof, account_state) =
            inner.instant_sender_affirmation_proof(off_chain, leg_ref, asset_id, amount);
        (proof, account_state.clone())
    }

    pub fn sender_affirmation(
        &self,
        off_chain: &OffchainProverState<T>,
        leg_ref: LegRef,
        asset_id: ConfidentialAssetId,
        amount: DartBalance,
    ) {
        let mut inner = self.0.borrow_mut();
        inner.sender_affirmation(off_chain, leg_ref, asset_id, amount);
    }

    pub fn receiver_affirmation_proof(
        &self,
        off_chain: &OffchainProverState<T>,
        leg_ref: LegRef,
        asset_id: ConfidentialAssetId,
        amount: DartBalance,
    ) -> (
        ReceiverAffirmationProof<AccountTreeConfig>,
        AccountAssetState,
    ) {
        let mut inner = self.0.borrow_mut();
        let (proof, account_state) =
            inner.receiver_affirmation_proof(off_chain, leg_ref, asset_id, amount);
        (proof, account_state.clone())
    }

    pub fn instant_receiver_affirmation_proof(
        &self,
        off_chain: &OffchainProverState<T>,
        leg_ref: LegRef,
        asset_id: ConfidentialAssetId,
        amount: DartBalance,
    ) -> (
        InstantReceiverAffirmationProof<AccountTreeConfig>,
        AccountAssetState,
    ) {
        let mut inner = self.0.borrow_mut();
        let (proof, account_state) =
            inner.instant_receiver_affirmation_proof(off_chain, leg_ref, asset_id, amount);
        (proof, account_state.clone())
    }

    pub fn receiver_affirmation(
        &self,
        off_chain: &OffchainProverState<T>,
        leg_ref: LegRef,
        asset_id: ConfidentialAssetId,
        amount: DartBalance,
    ) {
        let mut inner = self.0.borrow_mut();
        inner.receiver_affirmation(off_chain, leg_ref, asset_id, amount);
    }

    pub fn receiver_claim_proof(
        &self,
        off_chain: &OffchainProverState<T>,
        leg_ref: LegRef,
        asset_id: ConfidentialAssetId,
        amount: DartBalance,
    ) -> (ReceiverClaimProof<AccountTreeConfig>, AccountAssetState) {
        let mut inner = self.0.borrow_mut();
        let (proof, account_state) =
            inner.receiver_claim_proof(off_chain, leg_ref, asset_id, amount);

        (proof, account_state.clone())
    }

    pub fn receiver_claim(
        &self,
        off_chain: &OffchainProverState<T>,
        leg_ref: LegRef,
        asset_id: ConfidentialAssetId,
        amount: DartBalance,
    ) {
        let mut inner = self.0.borrow_mut();
        inner.receiver_claim(off_chain, leg_ref, asset_id, amount);
    }

    pub fn sender_revert_proof(
        &self,
        off_chain: &OffchainProverState<T>,
        leg_ref: LegRef,
        asset_id: ConfidentialAssetId,
        amount: DartBalance,
    ) -> (SenderReversalProof<AccountTreeConfig>, AccountAssetState) {
        let mut inner = self.0.borrow_mut();
        let (proof, account_state) =
            inner.sender_revert_proof(off_chain, leg_ref, asset_id, amount);

        (proof, account_state.clone())
    }

    pub fn sender_revert(
        &self,
        off_chain: &OffchainProverState<T>,
        leg_ref: LegRef,
        asset_id: ConfidentialAssetId,
        amount: DartBalance,
    ) {
        let mut inner = self.0.borrow_mut();
        inner.sender_revert(off_chain, leg_ref, asset_id, amount);
    }

    pub fn sender_counter_update_proof(
        &self,
        off_chain: &OffchainProverState<T>,
        leg_ref: LegRef,
        asset_id: ConfidentialAssetId,
    ) -> (
        SenderCounterUpdateProof<AccountTreeConfig>,
        AccountAssetState,
    ) {
        let mut inner = self.0.borrow_mut();
        let (proof, account_state) =
            inner.sender_counter_update_proof(off_chain, leg_ref, asset_id);

        (proof, account_state.clone())
    }

    pub fn sender_counter_update(
        &self,
        off_chain: &OffchainProverState<T>,
        leg_ref: LegRef,
        asset_id: ConfidentialAssetId,
    ) {
        let mut inner = self.0.borrow_mut();
        inner.sender_counter_update(off_chain, leg_ref, asset_id);
    }

    pub fn mediator_affirmation_proof(
        &self,
        off_chain: &OffchainProverState<T>,
        leg_ref: LegRef,
        accept: bool,
        verify: Option<(ConfidentialAssetId, DartBalance)>,
    ) -> MediatorAffirmationProof {
        let inner = self.0.borrow();
        inner.mediator_affirmation_proof(off_chain, leg_ref, accept, verify)
    }

    pub fn mediator_affirmation(
        &self,
        off_chain: &OffchainProverState<T>,
        leg_ref: LegRef,
        accept: bool,
        verify: Option<(ConfidentialAssetId, DartBalance)>,
    ) {
        let inner = self.0.borrow();
        inner.mediator_affirmation(off_chain, leg_ref, accept, verify);
    }
}

pub struct DartTestAsset<T: Config> {
    pub id: ConfidentialAssetId,
    asset_idx: u32,
    idx: u32,
    pub issuer: DartUser<T>,
    issuer_balance: DartBalance,
    pub mediators: Vec<DartUser<T>>,
    pub auditors: Vec<DartUser<T>>,
    pub total_supply: DartBalance,
}

impl<T: Config> DartTestAsset<T> {
    pub fn new(
        off_chain: &mut OffchainProverState<T>,
        name: &'static str,
        asset_idx: u32,
        mediator_count: u32,
        auditor_count: u32,
        mint_amount: Option<DartBalance>,
    ) -> Self {
        assert!(
            (auditor_count + mediator_count) >= 1,
            "At least one auditor or mediator is required"
        );

        // Create an asset issuer and create an asset.
        let asset_issuer = DartUser::<T>::asset_user(name, asset_idx, 0);

        // Register the asset issuer's account.
        asset_issuer.register_account();

        // Create auditors and mediators.
        let mut auditors = Vec::new();
        let mut auditor_keys = BoundedBTreeSet::new();
        for i in 0..auditor_count {
            let auditor = DartUser::<T>::auditor_user("Auditor", asset_idx, i);
            // Register the auditor's keys.
            auditor.register_encryption_key();
            auditor_keys
                .try_insert(auditor.public_keys().enc)
                .expect("Failed to push auditor keys");
            auditors.push(auditor);
        }
        let mut mediators = Vec::new();
        let mut mediator_keys = BoundedBTreeMap::new();
        for i in 0..mediator_count {
            let mediator = DartUser::<T>::auditor_user("Mediator", asset_idx, i);
            // Register the mediator's keys.
            mediator.register_account();
            let med_keys = mediator.public_keys();
            mediator_keys
                .try_insert(med_keys.acct, med_keys.enc)
                .expect("Failed to push mediator keys");
            mediators.push(mediator);
        }

        // Asset Data.
        let data = b"Test Asset"
            .to_vec()
            .try_into()
            .expect("Failed to convert asset data to AssetData");
        let name = Name::test_asset(asset_idx);
        let symbol = Symbol::test_asset(asset_idx);
        let decimals = 0u8;

        // Create the asset.
        let asset_id = Pallet::<T>::base_create_asset(
            asset_issuer.did(),
            name,
            symbol,
            decimals,
            mediator_keys,
            auditor_keys,
            data,
        )
        .expect("Failed to create asset");

        // Need to update the asset curve tree.
        off_chain.apply_new_leaves();

        let mint_amount = mint_amount.unwrap_or(0);
        if mint_amount > 0 {
            // Asset issuer registers the asset.
            asset_issuer.register_account_asset(asset_id);

            // Update the curve tree roots.
            off_chain.apply_new_leaves();

            // Mint asset proof.
            asset_issuer.mint_asset(&off_chain, asset_id, mint_amount);

            // Update the curve tree roots.
            off_chain.apply_new_leaves();
        }

        Self {
            id: asset_id,
            asset_idx,
            idx: 0,
            issuer: asset_issuer,
            issuer_balance: mint_amount,
            mediators,
            auditors,
            total_supply: mint_amount,
        }
    }

    pub fn has_mediator(&self) -> bool {
        self.mediators.len() > 0
    }

    pub fn mediator(&self) -> Option<DartUser<T>> {
        self.mediators.get(0).cloned()
    }

    pub fn asset_state(&self) -> AssetState {
        let auditors = self
            .auditors
            .iter()
            .map(|a| a.public_keys().enc)
            .collect::<Vec<_>>();
        let mediators = self
            .mediators
            .iter()
            .map(|m| {
                let keys = m.public_keys();
                (keys.acct, keys.enc)
            })
            .collect::<Vec<_>>();
        AssetState::new::<PolymeshPrivateLimits>(self.id, &mediators, &auditors)
            .expect("Failed to create AssetState")
    }

    pub fn mint_more(&mut self, off_chain: &mut OffchainProverState<T>, amount: DartBalance) {
        // Mint asset proof.
        self.issuer.mint_asset(off_chain, self.id, amount);

        // Update the curve tree roots.
        off_chain.apply_new_leaves();

        // Update the total supply and issuer balance.
        self.total_supply += amount;
        self.issuer_balance += amount;
    }

    pub fn create_investor(&mut self) -> DartUser<T> {
        let idx = self.idx + 1;
        self.idx = idx;
        // Create a user to invest in the asset.
        let user = DartUser::<T>::asset_user("Investor", self.asset_idx, idx);

        // Register the account.
        user.register_account();

        // Account asset registration proof.
        user.register_account_asset(self.id);

        user
    }

    pub fn create_and_fund_investors(
        &mut self,
        count: usize,
        amount: DartBalance,
        off_chain: &mut OffchainProverState<T>,
    ) -> Vec<DartUser<T>> {
        let total_amount = amount * count as DartBalance;
        if total_amount > self.issuer_balance {
            self.mint_more(off_chain, total_amount);
        }

        let (settlement, investors) =
            DartSettlementState::create_investors_and_fund(self, count, amount, off_chain);

        // Update the curve tree roots.
        off_chain.apply_new_leaves();

        // Affirm all the legs in the settlement.
        settlement.affirm_legs(off_chain);

        // All investors claim their assets.
        settlement.claim_assets(off_chain);

        investors
    }
}

#[derive(Clone)]
pub struct DartSettlementLegState<T: Config> {
    pub leg_ref: LegRef,
    pub sender: DartUser<T>,
    pub receiver: DartUser<T>,
    pub asset_id: ConfidentialAssetId,
    pub amount: DartBalance,
    pub mediators: Vec<DartUser<T>>,
}

impl<T: Config> DartSettlementLegState<T> {
    pub fn sender_affirmation(&self, off_chain: &mut OffchainProverState<T>) {
        self.sender
            .sender_affirmation(off_chain, self.leg_ref, self.asset_id, self.amount);
    }

    pub fn receiver_affirmation(&self, off_chain: &mut OffchainProverState<T>) {
        self.receiver
            .receiver_affirmation(off_chain, self.leg_ref, self.asset_id, self.amount);
    }

    pub fn mediator_affirmation(&self, off_chain: &OffchainProverState<T>, accept: bool) {
        for mediator in &self.mediators {
            mediator.mediator_affirmation(
                off_chain,
                self.leg_ref,
                accept,
                Some((self.asset_id, self.amount)),
            );
        }
    }

    pub fn affirm_leg(&self, off_chain: &mut OffchainProverState<T>) {
        self.sender_affirmation(off_chain);
        off_chain.apply_new_leaves();
        self.receiver_affirmation(off_chain);
        off_chain.apply_new_leaves();
        self.mediator_affirmation(off_chain, true);
    }

    pub fn receiver_claim(&self, off_chain: &mut OffchainProverState<T>) {
        self.receiver
            .receiver_claim(off_chain, self.leg_ref, self.asset_id, self.amount);
    }
}

#[derive(Clone)]
pub struct DartSettlementState<T: Config> {
    pub settlement_ref: SettlementRef,
    pub legs: Vec<DartSettlementLegState<T>>,
}

impl<T: Config> DartSettlementState<T> {
    pub fn create_investors_and_fund(
        asset: &mut DartTestAsset<T>,
        count: usize,
        amount: DartBalance,
        off_chain: &mut OffchainProverState<T>,
    ) -> (Self, Vec<DartUser<T>>) {
        let asset_id = asset.id;
        let issuer = asset.issuer.clone();
        let asset_data = asset.asset_state();

        // Create the investors.
        let mut investors = Vec::with_capacity(count);
        let mut settlement = SettlementBuilder::<PolymeshPrivateLimits>::new(b"FundInvestors");
        for _ in 0..count {
            let user = asset.create_investor();
            settlement.add_leg(LegBuilder {
                sender: issuer.public_keys(),
                receiver: user.public_keys(),
                asset: asset_data.clone(),
                amount,
                config: Default::default(),
                public_enc_keys: vec![],
            });
            investors.push(user);
        }

        // Create the settlement to transfer assets to all investors.
        let paths = MultiLeafPathAndRoot::from_paths(vec![off_chain
            .asset_tree
            .get_path_and_root_by_index(asset_id as LeafIndex)
            .expect("Failed to get path to leaf")])
        .expect("Failed to create MultiLeafPathAndRoot");
        let req = GenerateDartProofRequest::CreateSettlement {
            paths,
            settlement: settlement,
        };
        let settlement_ref =
            if let GenerateDartProofResponse::CreateSettlement { proof } = generate::<T>(req) {
                let settlement_ref = proof.settlement_ref();
                assert_ok!(Pallet::<T>::create_settlement(issuer.origin(), proof));
                settlement_ref
            } else {
                panic!("Failed to generate create settlement proof");
            };

        // After generating the settlement, save the leg details.
        let mut legs = Vec::with_capacity(count);
        for (idx, user) in investors.iter().enumerate() {
            let leg_ref = LegRef {
                settlement: settlement_ref,
                leg_id: idx as _,
            };
            legs.push(DartSettlementLegState {
                leg_ref,
                sender: issuer.clone(),
                receiver: user.clone(),
                asset_id,
                amount,
                mediators: asset.mediators.clone(),
            });
        }

        let settlement = Self {
            settlement_ref,
            legs,
        };

        (settlement, investors)
    }

    pub fn senders_affirm_legs(&self, off_chain: &mut OffchainProverState<T>) {
        // All senders affirm their legs.
        for leg in &self.legs {
            leg.sender_affirmation(off_chain);
            off_chain.apply_new_leaves();
        }
    }

    pub fn receivers_affirm_legs(&self, off_chain: &mut OffchainProverState<T>) {
        // All receivers affirm their legs.
        for leg in &self.legs {
            leg.receiver_affirmation(off_chain);
            off_chain.apply_new_leaves();
        }
    }

    pub fn mediators_affirm_legs(&self, off_chain: &mut OffchainProverState<T>, accept: bool) {
        // All mediators affirm their legs.
        for leg in &self.legs {
            leg.mediator_affirmation(off_chain, accept);
            off_chain.apply_new_leaves();
        }
    }

    pub fn affirm_legs(&self, off_chain: &mut OffchainProverState<T>) {
        // Affirm all the legs in the settlement.
        for leg in &self.legs {
            leg.affirm_leg(off_chain);

            off_chain.apply_new_leaves();
        }
    }

    pub fn claim_assets(&self, off_chain: &mut OffchainProverState<T>) {
        // All investors claim their assets.
        for leg in &self.legs {
            // Investor claims the assets.
            leg.receiver_claim(off_chain);
        }
        off_chain.apply_new_leaves();
    }
}
