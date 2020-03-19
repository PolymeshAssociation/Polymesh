use crate::{multisig, runtime, Runtime};

use pallet_transaction_payment::FeeDetails;
use polymesh_primitives::{traits::IdentityCurrency, AuthorizationData, Signatory};
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
        match call {
            Call::MultiSig(multisig::Call::accept_multisig_signer_as_key(auth_id)) => {
                is_auth_valid(caller, auth_id, CallType::AcceptMultiSigSigner)
            }
            Call::Identity(identity::Call::join_identity_as_key(auth_id, ..)) => {
                is_auth_valid(caller, auth_id, CallType::AcceptIdentitySigner)
            }
            Call::Identity(identity::Call::accept_master_key(rotation_auth_id, ..)) => {
                is_auth_valid(caller, rotation_auth_id, CallType::AcceptIdentityMaster)
            }
            Call::MultiSig(multisig::Call::create_or_approve_proposal_as_key(multisig, ..))
            | Call::MultiSig(multisig::Call::create_proposal_as_key(multisig, ..))
            | Call::MultiSig(multisig::Call::approve_as_key(multisig, ..)) => {
                if <multisig::MultiSigSigners<Runtime>>::exists(multisig, caller) {
                    return Ok(Some(Signatory::from(
                        <multisig::MultiSigCreator<Runtime>>::get(multisig),
                    )));
                }
                Err(InvalidTransaction::Payment)
            }
            // All other calls
            _ => match caller {
                // An external account(or multisig) was passed as the caller.
                // If the account has enabled charging fee to identity then the identity should be charged
                // otherwise, the account should be charged.
                Signatory::AccountKey(key) => {
                    if let Some(did) = Balances::charge_fee_to_identity(&key) {
                        Ok(Some(Signatory::from(did)))
                    } else {
                        Ok(Some(*caller))
                    }
                }
                // A did was passed as the caller. The did should be charged the fee.
                _ => Ok(Some(*caller)),
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
    if let Some(auth) = Identity::get_non_expired_auth(singer, auth_id) {
        match call_type {
            CallType::AcceptMultiSigSigner => {
                if auth.authorization_data == AuthorizationData::AddMultiSigSigner {
                    return Ok(Some(auth.authorized_by));
                }
            }
            CallType::AcceptIdentitySigner => {
                if let AuthorizationData::JoinIdentity(did) = auth.authorization_data {
                    let master = Identity::did_records(&did).master_key;
                    if auth.authorized_by == Signatory::from(master) {
                        return Ok(Some(Signatory::from(did)));
                    }
                }
            }
            CallType::AcceptIdentityMaster => {
                if let AuthorizationData::RotateMasterKey(did) = auth.authorization_data {
                    let master = Identity::did_records(&did).master_key;
                    if auth.authorized_by == Signatory::from(master) {
                        return Ok(Some(Signatory::from(did)));
                    }
                }
            }
        }
    }
    Err(InvalidTransaction::Payment)
}
