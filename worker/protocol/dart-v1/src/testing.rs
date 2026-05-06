use codec::{Decode, Encode};
use polymesh_dart::EncryptionPublicKey;
use polymesh_dart::key_distribution_proof::KeyDistributionProof;
use sp_std::vec::Vec;

use rand_chacha::ChaCha20Rng as Rng;
use rand_core::SeedableRng;

use polymesh_dart::curve_tree::get_account_curve_tree_parameters;
use polymesh_dart::{
    ACCOUNT_TREE_L, ACCOUNT_TREE_M, ASSET_TREE_L, ASSET_TREE_M, AccountAssetRegistrationProof,
    AccountAssetState, AccountKeyPair, AccountKeys, AccountRegistrationProof, AssetId,
    AssetMintingProof, Balance, BatchedAccountAssetRegistrationProof,
    BatchedFeeAccountRegistrationProof, BatchedFeeAccountTopupProof, EncryptionKeyPair,
    EncryptionKeyRegistrationProof, Error as DartError, FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M,
    FeeAccountAssetState, FeeAccountPaymentProof, FeeAccountRegistrationProof,
    FeeAccountTopupProof, InstantReceiverAffirmationProof, InstantSenderAffirmationProof,
    LegEncrypted, LegRef, MediatorAffirmationProof, PolymeshLimits, ProofHash,
    ReceiverAffirmationProof, ReceiverClaimProof, ReceiverRevertAffirmationProof,
    SenderAffirmationProof, SenderCounterUpdateProof, SenderRevertAffirmationProof,
    SettlementBuilder, SettlementProof, blake2_256,
    curve_tree::{
        AccountTreeConfig, AssetTreeConfig, FeeAccountTreeConfig, LeafPathAndRoot,
        MultiLeafPathAndRoot,
    },
};
use polymesh_worker_common::{ProtocolError, WorkSeed, WorkerSessionId};

use crate::{Did, Error};

pub type AccountLeafPathAndRoot =
    LeafPathAndRoot<ACCOUNT_TREE_L, ACCOUNT_TREE_M, AccountTreeConfig>;
pub type AssetLeafPathAndRoot = LeafPathAndRoot<ASSET_TREE_L, ASSET_TREE_M, AssetTreeConfig>;
pub type FeeAccountLeafPathAndRoot =
    LeafPathAndRoot<FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M, FeeAccountTreeConfig>;

#[derive(Encode, Decode, Clone)]
pub enum GenerateDartProofRequest {
    AccountRegistration {
        accounts: Vec<AccountKeys>,
        did: Did,
    },
    EncryptionKeyRegistration {
        keys: Vec<EncryptionKeyPair>,
        did: Did,
    },
    AccountAssetRegistration {
        keys: AccountKeys,
        did: Did,
        asset_id: AssetId,
        counter: u16,
    },
    BatchedAccountAssetRegistration {
        did: Did,
        account_assets: Vec<(AccountKeys, AssetId, u16)>,
    },
    MintAsset {
        keys: AccountKeys,
        did: Did,
        amount: Balance,
        path: AccountLeafPathAndRoot,
        account_state: AccountAssetState,
    },
    CreateSettlement {
        paths: MultiLeafPathAndRoot<ASSET_TREE_L, ASSET_TREE_M, AssetTreeConfig>,
        settlement: SettlementBuilder<PolymeshLimits>,
    },
    SenderAffirmation {
        keys: AccountKeys,
        leg_ref: LegRef,
        leg_enc: LegEncrypted,
        amount: Balance,
        path: AccountLeafPathAndRoot,
        account_state: AccountAssetState,
    },
    ReceiverAffirmation {
        keys: AccountKeys,
        leg_ref: LegRef,
        leg_enc: LegEncrypted,
        path: AccountLeafPathAndRoot,
        account_state: AccountAssetState,
    },
    MediatorAffirmation {
        keys: AccountKeys,
        leg_ref: LegRef,
        leg_enc: LegEncrypted,
        key_index: u8,
        accept: bool,
    },
    ReceiverClaim {
        keys: AccountKeys,
        leg_ref: LegRef,
        leg_enc: LegEncrypted,
        amount: Balance,
        path: AccountLeafPathAndRoot,
        account_state: AccountAssetState,
    },
    SenderCounterUpdate {
        keys: AccountKeys,
        leg_ref: LegRef,
        leg_enc: LegEncrypted,
        path: AccountLeafPathAndRoot,
        account_state: AccountAssetState,
    },
    SenderRevertAffirmation {
        keys: AccountKeys,
        leg_ref: LegRef,
        leg_enc: LegEncrypted,
        amount: Balance,
        path: AccountLeafPathAndRoot,
        account_state: AccountAssetState,
    },
    ReceiverRevertAffirmation {
        keys: AccountKeys,
        leg_ref: LegRef,
        leg_enc: LegEncrypted,
        path: AccountLeafPathAndRoot,
        account_state: AccountAssetState,
    },
    FeeAccountRegistration {
        did: Did,
        account: AccountKeyPair,
        asset_id: AssetId,
        amount: Balance,
    },
    BatchedFeeAccountRegistration {
        did: Did,
        accounts: Vec<(AccountKeyPair, AssetId, Balance)>,
    },
    FeeAccountTopup {
        did: Did,
        account: AccountKeyPair,
        amount: Balance,
        path: FeeAccountLeafPathAndRoot,
        account_state: FeeAccountAssetState,
    },
    BatchedFeeAccountTopup {
        did: Did,
        paths: MultiLeafPathAndRoot<FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M, FeeAccountTreeConfig>,
        topups: Vec<(AccountKeyPair, Balance, FeeAccountAssetState)>,
    },
    FeeAccountPayment {
        ctx: ProofHash,
        account: AccountKeyPair,
        amount: Balance,
        path: FeeAccountLeafPathAndRoot,
        account_state: FeeAccountAssetState,
    },
    InstantSenderAffirmation {
        keys: AccountKeys,
        leg_ref: LegRef,
        leg_enc: LegEncrypted,
        amount: Balance,
        path: AccountLeafPathAndRoot,
        account_state: AccountAssetState,
    },
    InstantReceiverAffirmation {
        keys: AccountKeys,
        leg_ref: LegRef,
        leg_enc: LegEncrypted,
        amount: Balance,
        path: AccountLeafPathAndRoot,
        account_state: AccountAssetState,
    },
    KeyDistribution {
        did: Did,
        key: EncryptionKeyPair,
        recipients: Vec<EncryptionPublicKey>,
    },
}

impl GenerateDartProofRequest {
    pub fn generate_with_seed(self, seed: WorkSeed) -> Result<GenerateDartProofResponse, Error> {
        let resp = self._generate(seed).map_err(|err| {
            #[cfg(feature = "std")]
            {
                log::warn!("Proof generation failed: {err}");
            }
            Error::GenerateProofFailed
        })?;

        Ok(resp)
    }

    fn _generate(self, seed: WorkSeed) -> Result<GenerateDartProofResponse, DartError> {
        let mut rng = Rng::from_seed(seed);
        match self {
            Self::AccountRegistration { accounts, did } => {
                let proof = AccountRegistrationProof::new(&mut rng, accounts.as_slice(), &did)?;
                Ok(GenerateDartProofResponse::AccountRegistration { proof })
            }
            Self::EncryptionKeyRegistration { keys, did } => {
                let proof = EncryptionKeyRegistrationProof::new(&mut rng, keys.as_slice(), &did)?;
                Ok(GenerateDartProofResponse::EncryptionKeyRegistration { proof })
            }
            Self::AccountAssetRegistration {
                keys,
                did,
                asset_id,
                counter,
            } => {
                let params = get_account_curve_tree_parameters();
                let (proof, account_state) = AccountAssetRegistrationProof::new(
                    &mut rng, &keys, asset_id, counter, &did, params,
                )?;
                Ok(GenerateDartProofResponse::AccountAssetRegistration {
                    proof,
                    account_state,
                })
            }
            Self::BatchedAccountAssetRegistration {
                did,
                account_assets,
            } => {
                let params = get_account_curve_tree_parameters();
                let (proof, account_states) = BatchedAccountAssetRegistrationProof::new(
                    &mut rng,
                    &account_assets,
                    &did,
                    params,
                )?;
                Ok(GenerateDartProofResponse::BatchedAccountAssetRegistration {
                    proof,
                    account_states,
                })
            }
            Self::MintAsset {
                keys,
                did,
                amount,
                path,
                mut account_state,
            } => {
                let proof = AssetMintingProof::new(
                    &mut rng,
                    &keys,
                    &did,
                    &mut account_state,
                    path,
                    amount,
                )?;
                Ok(GenerateDartProofResponse::MintAsset {
                    proof,
                    account_state,
                })
            }
            Self::CreateSettlement {
                paths,
                settlement: builder,
            } => {
                let proof = builder.encrypt_and_prove(&mut rng, paths)?;
                Ok(GenerateDartProofResponse::CreateSettlement { proof })
            }
            Self::SenderAffirmation {
                keys,
                leg_ref,
                leg_enc,
                amount,
                path,
                mut account_state,
            } => {
                let proof = SenderAffirmationProof::new(
                    &mut rng,
                    &keys,
                    &leg_ref,
                    amount,
                    &leg_enc,
                    &mut account_state,
                    &path,
                )?;

                Ok(GenerateDartProofResponse::SenderAffirmation {
                    proof,
                    account_state,
                })
            }
            Self::ReceiverAffirmation {
                keys,
                leg_ref,
                leg_enc,
                path,
                mut account_state,
            } => {
                let proof = ReceiverAffirmationProof::new(
                    &mut rng,
                    &keys,
                    &leg_ref,
                    &leg_enc,
                    &mut account_state,
                    path,
                )?;

                Ok(GenerateDartProofResponse::ReceiverAffirmation {
                    proof,
                    account_state,
                })
            }
            Self::InstantSenderAffirmation {
                keys,
                leg_ref,
                leg_enc,
                amount,
                path,
                mut account_state,
            } => {
                let proof = InstantSenderAffirmationProof::new(
                    &mut rng,
                    &keys,
                    &leg_ref,
                    amount,
                    &leg_enc,
                    &mut account_state,
                    path,
                )?;

                Ok(GenerateDartProofResponse::InstantSenderAffirmation {
                    proof,
                    account_state,
                })
            }
            Self::InstantReceiverAffirmation {
                keys,
                leg_ref,
                leg_enc,
                amount,
                path,
                mut account_state,
            } => {
                let proof = InstantReceiverAffirmationProof::new(
                    &mut rng,
                    &keys,
                    &leg_ref,
                    amount,
                    &leg_enc,
                    &mut account_state,
                    &path,
                )?;

                Ok(GenerateDartProofResponse::InstantReceiverAffirmation {
                    proof,
                    account_state,
                })
            }
            Self::MediatorAffirmation {
                keys,
                leg_ref,
                leg_enc,
                key_index,
                accept,
            } => {
                let med_enc = leg_enc.mediator_encryption(key_index)?;
                let proof = MediatorAffirmationProof::new(
                    &mut rng, &leg_ref, &med_enc, &keys, key_index, accept,
                )?;
                Ok(GenerateDartProofResponse::MediatorAffirmation { proof })
            }
            Self::ReceiverClaim {
                keys,
                leg_ref,
                leg_enc,
                amount,
                path,
                mut account_state,
            } => {
                let proof = ReceiverClaimProof::new(
                    &mut rng,
                    &keys,
                    &leg_ref,
                    amount,
                    &leg_enc,
                    &mut account_state,
                    path,
                )?;
                Ok(GenerateDartProofResponse::ReceiverClaim {
                    proof,
                    account_state,
                })
            }
            Self::SenderCounterUpdate {
                keys,
                leg_ref,
                leg_enc,
                path,
                mut account_state,
            } => {
                let proof = SenderCounterUpdateProof::new(
                    &mut rng,
                    &keys,
                    &leg_ref,
                    &leg_enc,
                    &mut account_state,
                    path,
                )?;
                Ok(GenerateDartProofResponse::SenderCounterUpdate {
                    proof,
                    account_state,
                })
            }
            Self::SenderRevertAffirmation {
                keys,
                leg_ref,
                leg_enc,
                amount,
                path,
                mut account_state,
            } => {
                let proof = SenderRevertAffirmationProof::new(
                    &mut rng,
                    &keys,
                    &leg_ref,
                    amount,
                    &leg_enc,
                    &mut account_state,
                    path,
                )?;
                Ok(GenerateDartProofResponse::SenderRevertAffirmation {
                    proof,
                    account_state,
                })
            }
            Self::ReceiverRevertAffirmation {
                keys,
                leg_ref,
                leg_enc,
                path,
                mut account_state,
            } => {
                let proof = ReceiverRevertAffirmationProof::new(
                    &mut rng,
                    &keys,
                    &leg_ref,
                    &leg_enc,
                    &mut account_state,
                    path,
                )?;
                Ok(GenerateDartProofResponse::ReceiverRevertAffirmation {
                    proof,
                    account_state,
                })
            }
            Self::FeeAccountRegistration {
                did,
                account,
                asset_id,
                amount,
            } => {
                let (proof, account_state) =
                    FeeAccountRegistrationProof::new(&mut rng, &account, asset_id, amount, &did)?;
                Ok(GenerateDartProofResponse::FeeAccountRegistration {
                    proof,
                    account_state,
                })
            }
            Self::BatchedFeeAccountRegistration { did, accounts } => {
                let registrations = accounts
                    .iter()
                    .map(|(acct, asset_id, amount)| (acct, *asset_id, *amount))
                    .collect::<Vec<_>>();
                let (proof, account_states) = BatchedFeeAccountRegistrationProof::new(
                    &mut rng,
                    registrations.as_slice(),
                    &did,
                )?;
                Ok(GenerateDartProofResponse::BatchedFeeAccountRegistration {
                    proof,
                    account_states,
                })
            }
            Self::FeeAccountTopup {
                did,
                account,
                amount,
                path,
                mut account_state,
            } => {
                let proof = FeeAccountTopupProof::new(
                    &mut rng,
                    &account,
                    &mut account_state,
                    amount,
                    &did,
                    &path,
                )?;
                Ok(GenerateDartProofResponse::FeeAccountTopup {
                    proof,
                    account_state,
                })
            }
            Self::BatchedFeeAccountTopup { did, paths, topups } => {
                let mut topups = topups
                    .iter()
                    .map(|(acct, amount, state)| (acct, *amount, state.clone()))
                    .collect::<Vec<_>>();
                let proof = BatchedFeeAccountTopupProof::new(
                    &mut rng,
                    topups.as_mut_slice(),
                    &did,
                    &paths,
                )?;
                Ok(GenerateDartProofResponse::BatchedFeeAccountTopup {
                    proof,
                    account_states: topups.into_iter().map(|(_, _, s)| s).collect(),
                })
            }
            Self::FeeAccountPayment {
                ctx,
                account,
                amount,
                path,
                mut account_state,
            } => {
                let proof = FeeAccountPaymentProof::new(
                    &mut rng,
                    &account,
                    &ctx.0,
                    &mut account_state,
                    amount,
                    &path,
                )?;
                Ok(GenerateDartProofResponse::FeeAccountPayment {
                    proof,
                    account_state,
                })
            }
            Self::KeyDistribution {
                did,
                key,
                recipients,
            } => {
                let params = get_account_curve_tree_parameters();
                let proof = KeyDistributionProof::new(&mut rng, &key, &recipients, &did, params)?;
                Ok(GenerateDartProofResponse::KeyDistribution { proof })
            }
        }
    }
}

impl GenerateDartProofRequest {
    pub fn do_generate(self) -> Result<GenerateDartProofResponse, ProtocolError> {
        let seed = blake2_256(&self);
        Ok(self.generate_with_seed(seed)?)
    }

    pub fn submit_and_wait(
        self,
        session_id: WorkerSessionId,
    ) -> Result<GenerateDartProofResponse, ProtocolError> {
        self.generate(session_id)
    }
}

#[cfg(feature = "impl_protocol")]
impl GenerateDartProofRequest {
    pub fn generate(
        self,
        _session_id: WorkerSessionId,
    ) -> Result<GenerateDartProofResponse, ProtocolError> {
        self.do_generate()
    }
}

#[cfg(not(feature = "impl_protocol"))]
impl GenerateDartProofRequest {
    pub fn generate(
        self,
        session_id: WorkerSessionId,
    ) -> Result<GenerateDartProofResponse, ProtocolError> {
        let req = crate::DartWorkRequest::GenerateProof(self);
        match req.session_execute_and_wait(session_id)? {
            crate::DartWorkResponse::GenerateProof(res) => Ok(res),
            _ => Err(ProtocolError::UnexpectedResponse),
        }
    }
}

#[derive(Encode, Decode, Clone)]
pub enum GenerateDartProofResponse {
    AccountRegistration {
        proof: AccountRegistrationProof<PolymeshLimits>,
    },
    EncryptionKeyRegistration {
        proof: EncryptionKeyRegistrationProof<PolymeshLimits>,
    },
    AccountAssetRegistration {
        proof: AccountAssetRegistrationProof<PolymeshLimits>,
        account_state: AccountAssetState,
    },
    BatchedAccountAssetRegistration {
        proof: BatchedAccountAssetRegistrationProof<PolymeshLimits>,
        account_states: Vec<AccountAssetState>,
    },
    MintAsset {
        proof: AssetMintingProof<PolymeshLimits>,
        account_state: AccountAssetState,
    },
    CreateSettlement {
        proof: SettlementProof<PolymeshLimits>,
    },
    SenderAffirmation {
        proof: SenderAffirmationProof<PolymeshLimits>,
        account_state: AccountAssetState,
    },
    ReceiverAffirmation {
        proof: ReceiverAffirmationProof<PolymeshLimits>,
        account_state: AccountAssetState,
    },
    MediatorAffirmation {
        proof: MediatorAffirmationProof<PolymeshLimits>,
    },
    ReceiverClaim {
        proof: ReceiverClaimProof<PolymeshLimits>,
        account_state: AccountAssetState,
    },
    SenderCounterUpdate {
        proof: SenderCounterUpdateProof<PolymeshLimits>,
        account_state: AccountAssetState,
    },
    SenderRevertAffirmation {
        proof: SenderRevertAffirmationProof<PolymeshLimits>,
        account_state: AccountAssetState,
    },
    ReceiverRevertAffirmation {
        proof: ReceiverRevertAffirmationProof<PolymeshLimits>,
        account_state: AccountAssetState,
    },
    FeeAccountRegistration {
        proof: FeeAccountRegistrationProof<PolymeshLimits>,
        account_state: FeeAccountAssetState,
    },
    BatchedFeeAccountRegistration {
        proof: BatchedFeeAccountRegistrationProof<PolymeshLimits>,
        account_states: Vec<FeeAccountAssetState>,
    },
    FeeAccountTopup {
        proof: FeeAccountTopupProof<PolymeshLimits>,
        account_state: FeeAccountAssetState,
    },
    BatchedFeeAccountTopup {
        proof: BatchedFeeAccountTopupProof<PolymeshLimits>,
        account_states: Vec<FeeAccountAssetState>,
    },
    FeeAccountPayment {
        proof: FeeAccountPaymentProof<PolymeshLimits>,
        account_state: FeeAccountAssetState,
    },
    InstantSenderAffirmation {
        proof: InstantSenderAffirmationProof<PolymeshLimits>,
        account_state: AccountAssetState,
    },
    InstantReceiverAffirmation {
        proof: InstantReceiverAffirmationProof<PolymeshLimits>,
        account_state: AccountAssetState,
    },
    KeyDistribution {
        proof: KeyDistributionProof<PolymeshLimits>,
    },
}
