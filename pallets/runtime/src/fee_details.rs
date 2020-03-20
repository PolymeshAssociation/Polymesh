use crate::{multisig, runtime, Runtime};

use pallet_transaction_payment::FeeDetails;
use polymesh_primitives::{
    traits::IdentityCurrency, AccountId, AuthorizationData, IdentityId, Signatory, TransactionError,
};
use polymesh_runtime_balances as balances;
use polymesh_runtime_identity as identity;

use sp_runtime::transaction_validity::InvalidTransaction;

use codec::{Decode, Encode};
use frame_support::{StorageDoubleMap, StorageMap};

type Identity = identity::Module<Runtime>;
type Balances = balances::Module<Runtime>;
type Call = runtime::Call;

enum CallType {
    AcceptMultiSigSigner,
    AcceptIdentitySigner,
    AcceptIdentityMaster,
}

#[derive(Default, Encode, Decode, Clone, Eq, PartialEq)]
pub struct FeeHandler;

impl FeeDetails<Call> for FeeHandler {
    fn whom_to_charge(
        call: &Call,
        caller: &Signatory,
    ) -> Result<Option<Signatory>, InvalidTransaction> {
        // The CDD check and fee payer varies depending on the transaction.
        // This match covers all possible scenarios.
        match call {
            // Call made by a new Account key to accept invitation to become a signing key
            // of an existing multisig that has a valid CDD. The auth should be valid.
            Call::MultiSig(multisig::Call::accept_multisig_signer_as_key(auth_id)) => {
                sp_runtime::print("accept_multisig_signer_as_key");
                is_auth_valid(caller, auth_id, CallType::AcceptMultiSigSigner)
            }
            // Call made by a new Account key to accept invitation to become a signing key
            // of an existing identity that has a valid CDD. The auth should be valid.
            Call::Identity(identity::Call::join_identity_as_key(auth_id, ..)) => {
                sp_runtime::print("join_identity_as_key");
                is_auth_valid(caller, auth_id, CallType::AcceptIdentitySigner)
            }
            // Call made by a new Account key to accept invitation to become the master key
            // of an existing identity that has a valid CDD. The auth should be valid.
            Call::Identity(identity::Call::accept_master_key(rotation_auth_id, ..)) => {
                sp_runtime::print("accept_master_key");
                is_auth_valid(caller, rotation_auth_id, CallType::AcceptIdentityMaster)
            }
            // Call made by an Account key to propose or approve a multisig transaction.
            // The multisig must have valid CDD and the caller must be a signer of the multisig.
            Call::MultiSig(multisig::Call::create_or_approve_proposal_as_key(multisig, ..))
            | Call::MultiSig(multisig::Call::create_proposal_as_key(multisig, ..))
            | Call::MultiSig(multisig::Call::approve_as_key(multisig, ..)) => {
                sp_runtime::print("multisig stuff");
                if <multisig::MultiSigSigners<Runtime>>::exists(multisig, caller) {
                    return check_cdd(&<multisig::MultiSigCreator<Runtime>>::get(multisig));
                }
                Err(InvalidTransaction::Payment)
            }
            // All other calls
            _ => match caller {
                // An external account was passed as the caller. This is the normal use case.
                // If the account has enabled charging fee to identity then the identity should be charged
                // otherwise, the account should be charged. In any case, the external account
                // must directly be linked to an identity with valid CDD.
                Signatory::AccountKey(key) => {
                    if let Some(did) = Identity::get_identity(key) {
                        if Identity::has_valid_cdd(did) {
                            if let Some(fee_did) = Balances::charge_fee_to_identity(&key) {
                                sp_runtime::print("charging identity");
                                return Ok(Some(Signatory::from(fee_did)));
                            } else {
                                sp_runtime::print("charging key");
                                return Ok(Some(*caller));
                            }
                        }
                    }
                    // Return an error if any of the above checks fail
                    // TODO: Make errors more specific
                    Err(InvalidTransaction::Payment)
                }
                // A did was passed as the caller. The did should be charged the fee.
                // This will never happen during an external call.
                Signatory::Identity(did) => check_cdd(did),
            },
        }
    }
}

/// Returns signatory to charge fee if auth is valid.
fn is_auth_valid(
    singer: &Signatory,
    auth_id: &u64,
    call_type: CallType,
) -> Result<Option<Signatory>, InvalidTransaction> {
    // Fetches the auth if it exists and has not expired
    if let Some(auth) = Identity::get_non_expired_auth(singer, auth_id) {
        // Different auths have different authorization data requirements and hence we match call type
        // to make sure proper authorization data is present.
        match call_type {
            CallType::AcceptMultiSigSigner => {
                if auth.authorization_data == AuthorizationData::AddMultiSigSigner {
                    // make sure that the auth was created by a multisig
                    if let Signatory::AccountKey(multisig) = auth.authorized_by {
                        let ms = AccountId::decode(&mut &multisig.as_slice()[..])
                            .map_err(|_| InvalidTransaction::Payment)?;
                        if <multisig::MultiSigCreator<Runtime>>::exists(&ms) {
                            // make sure that the multisig creator has valid CDD
                            return check_cdd(&<multisig::MultiSigCreator<Runtime>>::get(ms));
                        }
                    }
                }
            }
            CallType::AcceptIdentitySigner => {
                if let AuthorizationData::JoinIdentity(did) = auth.authorization_data {
                    // make sure that the auth was created by the master key of an identity with valid CDD
                    let master = Identity::did_records(&did).master_key;
                    if auth.authorized_by == Signatory::from(master) {
                        return check_cdd(&did);
                    }
                }
            }
            CallType::AcceptIdentityMaster => {
                if let AuthorizationData::RotateMasterKey(did) = auth.authorization_data {
                    // make sure that the auth was created by the master key of an identity with valid CDD
                    let master = Identity::did_records(&did).master_key;
                    if auth.authorized_by == Signatory::from(master) {
                        return check_cdd(&did);
                    }
                }
            }
        }
    }
    // Return an error if any of the above checks fail
    // TODO: Make errors more specific
    Err(InvalidTransaction::Payment)
}

/// Returns signatory to charge fee if cdd is valid.
fn check_cdd(did: &IdentityId) -> Result<Option<Signatory>, InvalidTransaction> {
    if Identity::has_valid_cdd(*did) {
        return Ok(Some(Signatory::from(*did)));
    } else {
        sp_runtime::print("ERROR: This transaction requires an Identity");
        Err(InvalidTransaction::Custom(TransactionError::MissingIdentity as u8).into())
    }
}
