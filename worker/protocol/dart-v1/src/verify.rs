use bounded_collections::BoundedVec;
use codec::{Decode, Encode};
use sp_std::vec::Vec;

use rand_chacha::ChaCha20Rng as Rng;
use rand_core::SeedableRng;

use polymesh_dart::curve_tree::get_account_curve_tree_parameters;
use polymesh_dart::{
    AccountPublicKey, AccountPublicKeys, AccountRegistrationProof, AccountStateCommitment,
    AccountStateNullifier, AccountStateUpdate, AssetId, AssetKeysLookup, AssetMintingProof,
    Balance, BatchedAccountAssetRegistrationProof, BatchedFeeAccountRegistrationProof,
    BatchedFeeAccountTopupProof, DartLimits, EncryptionKeyRegistrationProof, EncryptionPublicKey,
    FeeAccountPaymentProof, FeeAccountRegistrationProof, FeeAccountStateCommitment,
    FeeAccountStateNullifier, FeeAccountTopupProof, InstantReceiverAffirmationProof,
    InstantSenderAffirmationProof, LegEncrypted, LegRef, MediatorAffirmationProof, PolymeshLimits,
    ProofHash, ReceiverAffirmationProof, ReceiverClaimProof, ReceiverRevertAffirmationProof,
    SenderAffirmationProof, SenderCounterUpdateProof, SenderRevertAffirmationProof,
    SettlementProof, SettlementRef, blake2_256,
    Error as DartError,
};
use polymesh_dart_common::NullifierSkGenCounter;
use polymesh_worker_common::{ProtocolError, WorkSeed, WorkerSessionId};

use crate::{AccountTreeRoot, AssetTreeRoot, Did, Error, FeeAccountTreeRoot};

/// Verify DART asset proof request.
#[derive(Encode, Decode, Clone)]
pub enum VerifyDartAssetRequest {
    AccountRegistration {
        did: Did,
        proof: AccountRegistrationProof<PolymeshLimits>,
    },
    EncryptionKeyRegistration {
        did: Did,
        proof: EncryptionKeyRegistrationProof<PolymeshLimits>,
    },
    BatchedAccountAssetRegistration {
        did: Did,
        proof: BatchedAccountAssetRegistrationProof<PolymeshLimits>,
    },
    MintAsset {
        did: Did,
        root: AccountTreeRoot,
        proof: AssetMintingProof<PolymeshLimits>,
    },
    CreateSettlement {
        root: AssetTreeRoot,
        asset_lookup: AssetKeysLookup,
        proof: SettlementProof<PolymeshLimits>,
    },
    SenderAffirmation {
        leg_enc: LegEncrypted,
        root: AccountTreeRoot,
        proof: SenderAffirmationProof<PolymeshLimits>,
    },
    ReceiverAffirmation {
        leg_enc: LegEncrypted,
        root: AccountTreeRoot,
        proof: ReceiverAffirmationProof<PolymeshLimits>,
    },
    MediatorAffirmation {
        leg_enc: LegEncrypted,
        proof: MediatorAffirmationProof<PolymeshLimits>,
    },
    SenderCounterUpdate {
        leg_enc: LegEncrypted,
        root: AccountTreeRoot,
        proof: SenderCounterUpdateProof<PolymeshLimits>,
    },
    SenderRevertAffirmation {
        leg_enc: LegEncrypted,
        root: AccountTreeRoot,
        proof: SenderRevertAffirmationProof<PolymeshLimits>,
    },
    ReceiverRevertAffirmation {
        leg_enc: LegEncrypted,
        root: AccountTreeRoot,
        proof: ReceiverRevertAffirmationProof<PolymeshLimits>,
    },
    ReceiverClaim {
        leg_enc: LegEncrypted,
        root: AccountTreeRoot,
        proof: ReceiverClaimProof<PolymeshLimits>,
    },
    FeeAccountRegistration {
        did: Did,
        proof: FeeAccountRegistrationProof<PolymeshLimits>,
    },
    BatchedFeeAccountRegistration {
        did: Did,
        proof: BatchedFeeAccountRegistrationProof<PolymeshLimits>,
    },
    FeeAccountTopup {
        did: Did,
        root: FeeAccountTreeRoot,
        proof: FeeAccountTopupProof<PolymeshLimits>,
    },
    BatchedFeeAccountTopup {
        did: Did,
        root: FeeAccountTreeRoot,
        proof: BatchedFeeAccountTopupProof<PolymeshLimits>,
    },
    FeeAccountPayment {
        ctx: ProofHash,
        root: FeeAccountTreeRoot,
        proof: FeeAccountPaymentProof<PolymeshLimits>,
    },
    InstantSenderAffirmation {
        leg_enc: LegEncrypted,
        root: AccountTreeRoot,
        proof: InstantSenderAffirmationProof<PolymeshLimits>,
    },
    InstantReceiverAffirmation {
        leg_enc: LegEncrypted,
        root: AccountTreeRoot,
        proof: InstantReceiverAffirmationProof<PolymeshLimits>,
    },
}

impl VerifyDartAssetRequest {
    pub fn verify_with_seed(&self, seed: WorkSeed) -> Result<VerifyDartProofResponse, Error> {
        self._verify(seed).map_err(|err| {
            #[cfg(feature = "std")]
            {
                log::warn!("Proof verification failed: {err}");
            }
            Error::VerifyFailed
        })?;
        self.get_response()
    }

    fn _verify(&self, seed: WorkSeed) -> Result<(), DartError> {
        match self {
            Self::AccountRegistration { did, proof } => {
                proof.verify(&did[..])?;
            }
            Self::EncryptionKeyRegistration { did, proof } => {
                proof.verify(&did[..])?;
            }
            Self::BatchedAccountAssetRegistration { did, proof } => {
                let mut rng = Rng::from_seed(seed);
                let params = get_account_curve_tree_parameters();
                proof
                    .batched_verify(&did[..], &params, &mut rng)?;
            }
            Self::MintAsset { did, root, proof } => {
                let mut rng = Rng::from_seed(seed);
                proof
                    .verify(&did[..], root, &mut rng)?;
            }
            Self::CreateSettlement {
                root,
                asset_lookup,
                proof,
            } => {
                let mut rng = Rng::from_seed(seed);
                proof
                    //.batched_verify(root, &asset_lookup, &mut rng)?;
                    .verify(root, &asset_lookup, &mut rng)?;
            }
            Self::SenderAffirmation {
                leg_enc,
                root,
                proof,
            } => {
                let mut rng = Rng::from_seed(seed);
                proof
                    .verify(&leg_enc, root, &mut rng)?;
            }
            Self::ReceiverAffirmation {
                leg_enc,
                root,
                proof,
            } => {
                let mut rng = Rng::from_seed(seed);
                proof
                    .verify(&leg_enc, root, &mut rng)?;
            }
            Self::InstantSenderAffirmation {
                leg_enc,
                root,
                proof,
            } => {
                let mut rng = Rng::from_seed(seed);
                proof
                    .verify(&leg_enc, root, &mut rng)?;
            }
            Self::InstantReceiverAffirmation {
                leg_enc,
                root,
                proof,
            } => {
                let mut rng = Rng::from_seed(seed);
                proof
                    .verify(&leg_enc, root, &mut rng)?;
            }
            Self::MediatorAffirmation { leg_enc, proof } => {
                let med_enc = leg_enc.mediator_encryption(proof.key_index)?;
                proof.verify(&med_enc)?;
            }
            Self::SenderCounterUpdate {
                leg_enc,
                root,
                proof,
            } => {
                let mut rng = Rng::from_seed(seed);
                proof
                    .verify(&leg_enc, root, &mut rng)?;
            }
            Self::SenderRevertAffirmation {
                leg_enc,
                root,
                proof,
            } => {
                let mut rng = Rng::from_seed(seed);
                proof
                    .verify(&leg_enc, root, &mut rng)?;
            }
            Self::ReceiverRevertAffirmation {
                leg_enc,
                root,
                proof,
            } => {
                let mut rng = Rng::from_seed(seed);
                proof
                    .verify(&leg_enc, root, &mut rng)?;
            }
            Self::ReceiverClaim {
                leg_enc,
                root,
                proof,
            } => {
                let mut rng = Rng::from_seed(seed);
                proof
                    .verify(&leg_enc, root, &mut rng)?;
            }
            Self::FeeAccountRegistration { did, proof } => {
                proof.verify(&did[..])?;
            }
            Self::BatchedFeeAccountRegistration { did, proof } => {
                proof.verify(&did[..])?;
            }
            Self::FeeAccountTopup { did, root, proof } => {
                let mut rng = Rng::from_seed(seed);
                let root = root.root_node()?;
                proof
                    .verify(&mut rng, &did[..], &root)?;
            }
            Self::BatchedFeeAccountTopup { did, root, proof } => {
                let mut rng = Rng::from_seed(seed);
                proof
                    .batched_verify(&mut rng, &did[..], root)?;
            }
            Self::FeeAccountPayment { ctx, root, proof } => {
                let mut rng = Rng::from_seed(seed);
                proof
                    .verify(&mut rng, &ctx.0, root)?;
            }
        }

        Ok(())
    }

    pub fn get_response(&self) -> Result<VerifyDartProofResponse, Error> {
        match self {
            Self::AccountRegistration { did, proof } => {
                Ok(VerifyDartProofResponse::AccountRegistration {
                    did: *did,
                    accounts: proof.accounts.clone(),
                })
            }
            Self::EncryptionKeyRegistration { did, proof } => {
                Ok(VerifyDartProofResponse::EncryptionKeyRegistration {
                    did: *did,
                    keys: proof.keys.clone(),
                })
            }
            Self::BatchedAccountAssetRegistration { did, proof } => {
                let registrations = proof
                    .proofs
                    .iter()
                    .map(|state| RegisterAccountAsset {
                        account: state.account,
                        asset_id: state.asset_id,
                        counter: state.counter,
                        account_state_commitment: state.account_state_commitment,
                    })
                    .collect();
                Ok(VerifyDartProofResponse::BatchedAccountAssetRegistration {
                    did: *did,
                    registrations,
                })
            }
            Self::MintAsset { did, proof, .. } => Ok(VerifyDartProofResponse::MintAsset {
                did: *did,
                account: proof.pk,
                asset_id: proof.asset_id,
                amount: proof.amount,
                account_state_commitment: proof.account_state_commitment(),
                nullifier: proof.nullifier,
            }),
            Self::CreateSettlement { proof, .. } => {
                let legs = proof.legs.iter().map(|leg| leg.leg_enc().clone()).collect();
                Ok(VerifyDartProofResponse::CreateSettlement {
                    id: proof.settlement_ref(),
                    memo: proof.memo.clone(),
                    legs,
                })
            }
            Self::SenderAffirmation { proof, .. } => {
                Ok(VerifyDartProofResponse::SenderAffirmation {
                    leg_ref: proof.leg_ref,
                    account_state_commitment: proof.account_state_commitment(),
                    nullifier: proof.nullifier,
                })
            }
            Self::ReceiverAffirmation { proof, .. } => {
                Ok(VerifyDartProofResponse::ReceiverAffirmation {
                    leg_ref: proof.leg_ref,
                    account_state_commitment: proof.account_state_commitment(),
                    nullifier: proof.nullifier,
                })
            }
            Self::InstantSenderAffirmation { proof, .. } => {
                Ok(VerifyDartProofResponse::InstantSenderAffirmation {
                    leg_ref: proof.leg_ref,
                    account_state_commitment: proof.account_state_commitment(),
                    nullifier: proof.nullifier,
                })
            }
            Self::InstantReceiverAffirmation { proof, .. } => {
                Ok(VerifyDartProofResponse::InstantReceiverAffirmation {
                    leg_ref: proof.leg_ref,
                    account_state_commitment: proof.account_state_commitment(),
                    nullifier: proof.nullifier,
                })
            }
            Self::MediatorAffirmation { proof, .. } => {
                Ok(VerifyDartProofResponse::MediatorAffirmation {
                    leg_ref: proof.leg_ref,
                    accept: proof.accept,
                })
            }
            Self::SenderCounterUpdate { proof, .. } => {
                Ok(VerifyDartProofResponse::SenderCounterUpdate {
                    leg_ref: proof.leg_ref,
                    account_state_commitment: proof.account_state_commitment(),
                    nullifier: proof.nullifier,
                })
            }
            Self::SenderRevertAffirmation { proof, .. } => {
                Ok(VerifyDartProofResponse::SenderRevertAffirmation {
                    leg_ref: proof.leg_ref,
                    account_state_commitment: proof.account_state_commitment(),
                    nullifier: proof.nullifier,
                })
            }
            Self::ReceiverRevertAffirmation { proof, .. } => {
                Ok(VerifyDartProofResponse::ReceiverRevertAffirmation {
                    leg_ref: proof.leg_ref,
                    account_state_commitment: proof.account_state_commitment(),
                    nullifier: proof.nullifier,
                })
            }
            Self::ReceiverClaim { proof, .. } => Ok(VerifyDartProofResponse::ReceiverClaim {
                leg_ref: proof.leg_ref,
                account_state_commitment: proof.account_state_commitment(),
                nullifier: proof.nullifier,
            }),
            Self::FeeAccountRegistration { did, proof } => {
                Ok(VerifyDartProofResponse::FeeAccountRegistration {
                    did: *did,
                    registration: FeeAccountUpdate {
                        account: proof.account,
                        asset_id: proof.asset_id,
                        is_topup: false,
                        amount: proof.amount,
                        account_state_commitment: proof.account_state_commitment,
                        nullifier: None,
                    },
                })
            }
            Self::BatchedFeeAccountRegistration { did, proof } => {
                let registrations = proof
                    .proofs
                    .iter()
                    .map(|proof| FeeAccountUpdate {
                        account: proof.account,
                        asset_id: proof.asset_id,
                        is_topup: false,
                        amount: proof.amount,
                        account_state_commitment: proof.account_state_commitment,
                        nullifier: None,
                    })
                    .collect();
                Ok(VerifyDartProofResponse::BatchedFeeAccountRegistration {
                    did: *did,
                    registrations,
                })
            }
            Self::FeeAccountTopup { did, proof, .. } => {
                Ok(VerifyDartProofResponse::FeeAccountTopup {
                    did: *did,
                    topup: FeeAccountUpdate {
                        account: proof.account,
                        asset_id: proof.asset_id,
                        is_topup: true,
                        amount: proof.amount,
                        account_state_commitment: proof.updated_account_state_commitment,
                        nullifier: Some(proof.nullifier),
                    },
                })
            }
            Self::BatchedFeeAccountTopup { did, proof, .. } => {
                let topups = proof
                    .proofs
                    .iter()
                    .map(|proof| FeeAccountUpdate {
                        account: proof.account,
                        asset_id: proof.asset_id,
                        is_topup: true,
                        amount: proof.amount,
                        account_state_commitment: proof.updated_account_state_commitment,
                        nullifier: Some(proof.nullifier),
                    })
                    .collect();
                Ok(VerifyDartProofResponse::BatchedFeeAccountTopup { did: *did, topups })
            }
            Self::FeeAccountPayment { proof, .. } => {
                Ok(VerifyDartProofResponse::FeeAccountPayment {
                    asset_id: proof.asset_id,
                    amount: proof.amount,
                    account_state_commitment: proof.updated_account_state_commitment,
                    nullifier: proof.nullifier,
                })
            }
        }
    }
}

impl VerifyDartAssetRequest {
    pub fn do_verify(self) -> Result<VerifyDartProofResponse, ProtocolError> {
        let seed = blake2_256(&self);
        Ok(self.verify_with_seed(seed)?)
    }

    pub fn submit_and_wait(
        self,
        session_id: WorkerSessionId,
    ) -> Result<VerifyDartProofResponse, ProtocolError> {
        self.verify(session_id)
    }
}

#[cfg(feature = "impl_protocol")]
impl VerifyDartAssetRequest {
    pub fn verify(
        self,
        _session_id: WorkerSessionId,
    ) -> Result<VerifyDartProofResponse, ProtocolError> {
        self.do_verify()
    }
}

#[cfg(not(feature = "impl_protocol"))]
impl VerifyDartAssetRequest {
    pub fn verify(
        self,
        session_id: WorkerSessionId,
    ) -> Result<VerifyDartProofResponse, ProtocolError> {
        let req = crate::DartWorkRequest::VerifyProof(self);
        match req.session_execute_and_wait(session_id)? {
            crate::DartWorkResponse::VerifyProof(res) => Ok(res),
            _ => Err(ProtocolError::UnexpectedResponse),
        }
    }
}

#[derive(Encode, Decode, Clone)]
pub struct RegisterAccountAsset {
    pub account: AccountPublicKeys,
    pub asset_id: AssetId,
    pub counter: NullifierSkGenCounter,
    pub account_state_commitment: AccountStateCommitment,
}

#[derive(Encode, Decode, Clone)]
pub struct FeeAccountUpdate {
    pub account: AccountPublicKey,
    pub asset_id: AssetId,
    pub is_topup: bool,
    pub amount: Balance,
    pub account_state_commitment: FeeAccountStateCommitment,
    pub nullifier: Option<FeeAccountStateNullifier>,
}

#[derive(Encode, Decode, Clone)]
pub enum VerifyDartProofResponse<T: DartLimits = PolymeshLimits> {
    AccountRegistration {
        did: Did,
        accounts: BoundedVec<AccountPublicKeys, T::MaxKeysPerRegProof>,
    },
    EncryptionKeyRegistration {
        did: Did,
        keys: BoundedVec<EncryptionPublicKey, T::MaxKeysPerRegProof>,
    },
    BatchedAccountAssetRegistration {
        did: Did,
        registrations: Vec<RegisterAccountAsset>,
    },
    MintAsset {
        did: Did,
        account: AccountPublicKey,
        asset_id: AssetId,
        amount: Balance,
        account_state_commitment: AccountStateCommitment,
        nullifier: AccountStateNullifier,
    },
    CreateSettlement {
        id: SettlementRef,
        memo: BoundedVec<u8, T::MaxSettlementMemoLength>,
        legs: Vec<LegEncrypted>,
    },
    SenderAffirmation {
        leg_ref: LegRef,
        account_state_commitment: AccountStateCommitment,
        nullifier: AccountStateNullifier,
    },
    ReceiverAffirmation {
        leg_ref: LegRef,
        account_state_commitment: AccountStateCommitment,
        nullifier: AccountStateNullifier,
    },
    MediatorAffirmation {
        leg_ref: LegRef,
        accept: bool,
    },
    SenderCounterUpdate {
        leg_ref: LegRef,
        account_state_commitment: AccountStateCommitment,
        nullifier: AccountStateNullifier,
    },
    SenderRevertAffirmation {
        leg_ref: LegRef,
        account_state_commitment: AccountStateCommitment,
        nullifier: AccountStateNullifier,
    },
    ReceiverRevertAffirmation {
        leg_ref: LegRef,
        account_state_commitment: AccountStateCommitment,
        nullifier: AccountStateNullifier,
    },
    ReceiverClaim {
        leg_ref: LegRef,
        account_state_commitment: AccountStateCommitment,
        nullifier: AccountStateNullifier,
    },
    FeeAccountRegistration {
        did: Did,
        registration: FeeAccountUpdate,
    },
    BatchedFeeAccountRegistration {
        did: Did,
        registrations: Vec<FeeAccountUpdate>,
    },
    FeeAccountTopup {
        did: Did,
        topup: FeeAccountUpdate,
    },
    BatchedFeeAccountTopup {
        did: Did,
        topups: Vec<FeeAccountUpdate>,
    },
    FeeAccountPayment {
        asset_id: AssetId,
        amount: Balance,
        account_state_commitment: FeeAccountStateCommitment,
        nullifier: FeeAccountStateNullifier,
    },
    InstantSenderAffirmation {
        leg_ref: LegRef,
        account_state_commitment: AccountStateCommitment,
        nullifier: AccountStateNullifier,
    },
    InstantReceiverAffirmation {
        leg_ref: LegRef,
        account_state_commitment: AccountStateCommitment,
        nullifier: AccountStateNullifier,
    },
}
