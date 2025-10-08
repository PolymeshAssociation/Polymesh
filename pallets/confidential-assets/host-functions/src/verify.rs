use codec::{Decode, Encode};
use sp_core::bounded::BoundedVec;
#[cfg(feature = "std")]
use sp_io::hashing::blake2_256;
use sp_std::vec::Vec;

use rand_chacha::ChaCha20Rng as Rng;
use rand_core::SeedableRng;

use polymesh_dart::curve_tree::get_account_curve_tree_parameters;
use polymesh_dart::{
    AccountPublicKey, AccountPublicKeys, AccountRegistrationProof, AccountStateCommitment,
    AccountStateNullifier, AccountStateUpdate, AssetId, AssetMintingProof, Balance,
    BatchedAccountAssetRegistrationProof, BatchedFeeAccountRegistrationProof,
    BatchedFeeAccountTopupProof, DartLimits, EncryptionKeyRegistrationProof, EncryptionPublicKey,
    FeeAccountPaymentProof, FeeAccountRegistrationProof, FeeAccountStateCommitment,
    FeeAccountStateNullifier, FeeAccountTopupProof, InstantReceiverAffirmationProof,
    InstantSenderAffirmationProof, LegEncrypted, LegRef, MediatorAffirmationProof,
    PolymeshPrivateLimits, ProofHash, ReceiverAffirmationProof, ReceiverClaimProof,
    SenderAffirmationProof, SenderCounterUpdateProof, SenderReversalProof, SettlementProof,
    SettlementRef,
};
use polymesh_dart_common::NullifierSkGenCounter;
use polymesh_primitives::IdentityId;

use crate::{
    AccountTreeRoot, AssetTreeRoot, BatchId, BatchSeed, Error, FeeAccountTreeRoot, WorkRequest,
    WorkRequestExecution, WorkRequestKind,
};

/// Verify DART asset proof request.
#[derive(Encode, Decode, Clone)]
pub enum VerifyDartAssetRequest {
    AccountRegistration {
        did: IdentityId,
        proof: AccountRegistrationProof<PolymeshPrivateLimits>,
    },
    EncryptionKeyRegistration {
        did: IdentityId,
        proof: EncryptionKeyRegistrationProof<PolymeshPrivateLimits>,
    },
    BatchedAccountAssetRegistration {
        did: IdentityId,
        proof: BatchedAccountAssetRegistrationProof<PolymeshPrivateLimits>,
    },
    MintAsset {
        did: IdentityId,
        root: AccountTreeRoot,
        proof: AssetMintingProof,
    },
    CreateSettlement {
        root: AssetTreeRoot,
        proof: SettlementProof<PolymeshPrivateLimits>,
    },
    SenderAffirmation {
        leg_enc: LegEncrypted,
        root: AccountTreeRoot,
        proof: SenderAffirmationProof,
    },
    ReceiverAffirmation {
        leg_enc: LegEncrypted,
        root: AccountTreeRoot,
        proof: ReceiverAffirmationProof,
    },
    MediatorAffirmation {
        leg_enc: LegEncrypted,
        proof: MediatorAffirmationProof,
    },
    SenderCounterUpdate {
        leg_enc: LegEncrypted,
        root: AccountTreeRoot,
        proof: SenderCounterUpdateProof,
    },
    SenderReversal {
        leg_enc: LegEncrypted,
        root: AccountTreeRoot,
        proof: SenderReversalProof,
    },
    ReceiverClaim {
        leg_enc: LegEncrypted,
        root: AccountTreeRoot,
        proof: ReceiverClaimProof,
    },
    FeeAccountRegistration {
        did: IdentityId,
        proof: FeeAccountRegistrationProof,
    },
    BatchedFeeAccountRegistration {
        did: IdentityId,
        proof: BatchedFeeAccountRegistrationProof<PolymeshPrivateLimits>,
    },
    FeeAccountTopup {
        did: IdentityId,
        root: FeeAccountTreeRoot,
        proof: FeeAccountTopupProof,
    },
    BatchedFeeAccountTopup {
        did: IdentityId,
        root: FeeAccountTreeRoot,
        proof: BatchedFeeAccountTopupProof<PolymeshPrivateLimits>,
    },
    FeeAccountPayment {
        ctx: ProofHash,
        root: FeeAccountTreeRoot,
        proof: FeeAccountPaymentProof,
    },
    InstantSenderAffirmation {
        leg_enc: LegEncrypted,
        root: AccountTreeRoot,
        proof: InstantSenderAffirmationProof,
    },
    InstantReceiverAffirmation {
        leg_enc: LegEncrypted,
        root: AccountTreeRoot,
        proof: InstantReceiverAffirmationProof,
    },
}

impl VerifyDartAssetRequest {
    pub fn verify_with_seed(&self, seed: BatchSeed) -> Result<VerifyDartProofResponse, Error> {
        match self {
            Self::AccountRegistration { did, proof } => {
                proof.verify(&did.0[..]).map_err(|_| Error::VerifyFailed)?;
            }
            Self::EncryptionKeyRegistration { did, proof } => {
                proof.verify(&did.0[..]).map_err(|_| Error::VerifyFailed)?;
            }
            Self::BatchedAccountAssetRegistration { did, proof } => {
                let mut rng = Rng::from_seed(seed);
                let params = get_account_curve_tree_parameters();
                proof
                    .batched_verify(&did.0[..], &params, &mut rng)
                    .map_err(|_| Error::VerifyFailed)?;
            }
            Self::MintAsset { did, root, proof } => {
                let mut rng = Rng::from_seed(seed);
                proof
                    .verify(&did.0[..], root, &mut rng)
                    .map_err(|_| Error::VerifyFailed)?;
            }
            Self::CreateSettlement { root, proof } => {
                let mut rng = Rng::from_seed(seed);
                proof
                    .batched_verify(root, &mut rng)
                    .map_err(|_| Error::VerifyFailed)?;
            }
            Self::SenderAffirmation {
                leg_enc,
                root,
                proof,
            } => {
                let mut rng = Rng::from_seed(seed);
                proof
                    .verify(&leg_enc, root, &mut rng)
                    .map_err(|_| Error::VerifyFailed)?;
            }
            Self::ReceiverAffirmation {
                leg_enc,
                root,
                proof,
            } => {
                let mut rng = Rng::from_seed(seed);
                proof
                    .verify(&leg_enc, root, &mut rng)
                    .map_err(|_| Error::VerifyFailed)?;
            }
            Self::InstantSenderAffirmation {
                leg_enc,
                root,
                proof,
            } => {
                let mut rng = Rng::from_seed(seed);
                proof
                    .verify(&leg_enc, root, &mut rng)
                    .map_err(|_| Error::VerifyFailed)?;
            }
            Self::InstantReceiverAffirmation {
                leg_enc,
                root,
                proof,
            } => {
                let mut rng = Rng::from_seed(seed);
                proof
                    .verify(&leg_enc, root, &mut rng)
                    .map_err(|_| Error::VerifyFailed)?;
            }
            Self::MediatorAffirmation { leg_enc, proof } => {
                proof.verify(&leg_enc).map_err(|_| Error::VerifyFailed)?;
            }
            Self::SenderCounterUpdate {
                leg_enc,
                root,
                proof,
            } => {
                let mut rng = Rng::from_seed(seed);
                proof
                    .verify(&leg_enc, root, &mut rng)
                    .map_err(|_| Error::VerifyFailed)?;
            }
            Self::SenderReversal {
                leg_enc,
                root,
                proof,
            } => {
                let mut rng = Rng::from_seed(seed);
                proof
                    .verify(&leg_enc, root, &mut rng)
                    .map_err(|_| Error::VerifyFailed)?;
            }
            Self::ReceiverClaim {
                leg_enc,
                root,
                proof,
            } => {
                let mut rng = Rng::from_seed(seed);
                proof
                    .verify(&leg_enc, root, &mut rng)
                    .map_err(|_| Error::VerifyFailed)?;
            }
            Self::FeeAccountRegistration { did, proof } => {
                proof.verify(&did.0[..]).map_err(|_| Error::VerifyFailed)?;
            }
            Self::BatchedFeeAccountRegistration { did, proof } => {
                proof.verify(&did.0[..]).map_err(|_| Error::VerifyFailed)?;
            }
            Self::FeeAccountTopup { did, root, proof } => {
                let mut rng = Rng::from_seed(seed);
                let root = root.root_node()?;
                proof
                    .verify(&mut rng, &did.0[..], &root)
                    .map_err(|_| Error::VerifyFailed)?;
            }
            Self::BatchedFeeAccountTopup { did, root, proof } => {
                let mut rng = Rng::from_seed(seed);
                proof
                    .batched_verify(&mut rng, &did.0[..], root)
                    .map_err(|_| Error::VerifyFailed)?;
            }
            Self::FeeAccountPayment { ctx, root, proof } => {
                let mut rng = Rng::from_seed(seed);
                proof
                    .verify(&mut rng, &ctx.0, root)
                    .map_err(|_| Error::VerifyFailed)?;
            }
        }
        self.get_response()
    }

    pub fn get_response(&self) -> Result<VerifyDartProofResponse, Error> {
        match self {
            Self::AccountRegistration { did, proof } => {
                return Ok(VerifyDartProofResponse::AccountRegistration {
                    did: *did,
                    accounts: proof.accounts.clone(),
                });
            }
            Self::EncryptionKeyRegistration { did, proof } => {
                return Ok(VerifyDartProofResponse::EncryptionKeyRegistration {
                    did: *did,
                    keys: proof.keys.clone(),
                });
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
                        nullifier: state.nullifier,
                    })
                    .collect();
                return Ok(VerifyDartProofResponse::BatchedAccountAssetRegistration {
                    did: *did,
                    registrations,
                });
            }
            Self::MintAsset { did, proof, .. } => {
                return Ok(VerifyDartProofResponse::MintAsset {
                    did: *did,
                    account: proof.pk,
                    asset_id: proof.asset_id,
                    amount: proof.amount,
                    account_state_commitment: proof.account_state_commitment(),
                    nullifier: proof.nullifier,
                });
            }
            Self::CreateSettlement { proof, .. } => {
                let legs = proof.legs.iter().map(|leg| leg.leg_enc.clone()).collect();
                return Ok(VerifyDartProofResponse::CreateSettlement {
                    id: proof.settlement_ref(),
                    memo: proof.memo.clone(),
                    legs,
                });
            }
            Self::SenderAffirmation { proof, .. } => {
                return Ok(VerifyDartProofResponse::SenderAffirmation {
                    leg_ref: proof.leg_ref,
                    account_state_commitment: proof.account_state_commitment(),
                    nullifier: proof.nullifier,
                });
            }
            Self::ReceiverAffirmation { proof, .. } => {
                return Ok(VerifyDartProofResponse::ReceiverAffirmation {
                    leg_ref: proof.leg_ref,
                    account_state_commitment: proof.account_state_commitment(),
                    nullifier: proof.nullifier,
                });
            }
            Self::InstantSenderAffirmation { proof, .. } => {
                return Ok(VerifyDartProofResponse::InstantSenderAffirmation {
                    leg_ref: proof.leg_ref,
                    account_state_commitment: proof.account_state_commitment(),
                    nullifier: proof.nullifier,
                });
            }
            Self::InstantReceiverAffirmation { proof, .. } => {
                return Ok(VerifyDartProofResponse::InstantReceiverAffirmation {
                    leg_ref: proof.leg_ref,
                    account_state_commitment: proof.account_state_commitment(),
                    nullifier: proof.nullifier,
                });
            }
            Self::MediatorAffirmation { proof, .. } => {
                return Ok(VerifyDartProofResponse::MediatorAffirmation {
                    leg_ref: proof.leg_ref,
                    accept: proof.accept,
                });
            }
            Self::SenderCounterUpdate { proof, .. } => {
                return Ok(VerifyDartProofResponse::SenderCounterUpdate {
                    leg_ref: proof.leg_ref,
                    account_state_commitment: proof.account_state_commitment(),
                    nullifier: proof.nullifier,
                });
            }
            Self::SenderReversal { proof, .. } => {
                return Ok(VerifyDartProofResponse::SenderReversal {
                    leg_ref: proof.leg_ref,
                    account_state_commitment: proof.account_state_commitment(),
                    nullifier: proof.nullifier,
                });
            }
            Self::ReceiverClaim { proof, .. } => {
                return Ok(VerifyDartProofResponse::ReceiverClaim {
                    leg_ref: proof.leg_ref,
                    account_state_commitment: proof.account_state_commitment(),
                    nullifier: proof.nullifier,
                });
            }
            Self::FeeAccountRegistration { did, proof } => {
                return Ok(VerifyDartProofResponse::FeeAccountRegistration {
                    did: *did,
                    registration: FeeAccountUpdate {
                        account: proof.account,
                        asset_id: proof.asset_id,
                        is_topup: false,
                        amount: proof.amount,
                        account_state_commitment: proof.account_state_commitment,
                        nullifier: None,
                    },
                });
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
                return Ok(VerifyDartProofResponse::BatchedFeeAccountRegistration {
                    did: *did,
                    registrations,
                });
            }
            Self::FeeAccountTopup { did, proof, .. } => {
                return Ok(VerifyDartProofResponse::FeeAccountTopup {
                    did: *did,
                    topup: FeeAccountUpdate {
                        account: proof.account,
                        asset_id: proof.asset_id,
                        is_topup: true,
                        amount: proof.amount,
                        account_state_commitment: proof.updated_account_state_commitment,
                        nullifier: Some(proof.nullifier),
                    },
                });
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
                return Ok(VerifyDartProofResponse::BatchedFeeAccountTopup { did: *did, topups });
            }
            Self::FeeAccountPayment { proof, .. } => {
                return Ok(VerifyDartProofResponse::FeeAccountPayment {
                    asset_id: proof.asset_id,
                    amount: proof.amount,
                    account_state_commitment: proof.updated_account_state_commitment,
                    nullifier: proof.nullifier,
                });
            }
        }
    }

    pub fn submit(&self, batch_id: BatchId) -> Result<WorkRequestExecution, Error> {
        let req = WorkRequest::new_verify(self);
        req.submit(batch_id)
    }

    pub fn submit_and_wait(&self, batch_id: BatchId) -> Result<VerifyDartProofResponse, Error> {
        let req = WorkRequest::new_verify(self);
        let res = req.submit(batch_id)?;
        let resp = res.wait_for_results(batch_id)?;
        match resp.kind {
            WorkRequestKind::VerifyProof(_) => {
                let resp =
                    Decode::decode(&mut &resp.resp[..]).map_err(|_| Error::InvalidWorkResult)?;
                Ok(resp)
            }
            _ => Err(Error::InvalidWorkResult),
        }
    }
}

#[cfg(feature = "std")]
impl VerifyDartAssetRequest {
    pub fn verify(&self) -> Result<VerifyDartProofResponse, Error> {
        let seed = self.using_encoded(blake2_256);
        self.verify_with_seed(seed)
    }
}

#[cfg(not(feature = "std"))]
impl VerifyDartAssetRequest {
    pub fn verify(self) -> Result<VerifyDartProofResponse, Error> {
        crate::native_dart_assets::verify_proof(self)
    }
}

#[derive(Encode, Decode, Clone)]
pub struct RegisterAccountAsset {
    pub account: AccountPublicKey,
    pub asset_id: AssetId,
    pub counter: NullifierSkGenCounter,
    pub account_state_commitment: AccountStateCommitment,
    pub nullifier: AccountStateNullifier,
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
pub enum VerifyDartProofResponse<T: DartLimits = PolymeshPrivateLimits> {
    AccountRegistration {
        did: IdentityId,
        accounts: BoundedVec<AccountPublicKeys, T::MaxKeysPerRegProof>,
    },
    EncryptionKeyRegistration {
        did: IdentityId,
        keys: BoundedVec<EncryptionPublicKey, T::MaxKeysPerRegProof>,
    },
    BatchedAccountAssetRegistration {
        did: IdentityId,
        registrations: Vec<RegisterAccountAsset>,
    },
    MintAsset {
        did: IdentityId,
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
    SenderReversal {
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
        did: IdentityId,
        registration: FeeAccountUpdate,
    },
    BatchedFeeAccountRegistration {
        did: IdentityId,
        registrations: Vec<FeeAccountUpdate>,
    },
    FeeAccountTopup {
        did: IdentityId,
        topup: FeeAccountUpdate,
    },
    BatchedFeeAccountTopup {
        did: IdentityId,
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
