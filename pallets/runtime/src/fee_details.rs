use crate::{multisig, runtime, Runtime};

use pallet_transaction_payment::FeeDetails;
use polymesh_primitives::{
    traits::IdentityCurrency, AccountKey, Authorization, IdentityId, Signatory, TransactionError,
};
use polymesh_runtime_balances as balances;
use polymesh_runtime_common::{identity::LinkedKeyInfo, Context};
use polymesh_runtime_identity as identity;

use sp_runtime::transaction_validity::{
    InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransaction,
};

use codec::{Decode, Encode};
use core::convert::TryFrom;
use frame_support::dispatch::DispatchInfo;

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
                if is_auth_valid(caller, auth_id, CallType::AcceptMultiSigSigner) {
                    Ok(Some(*caller))
                } else {
                    Err(InvalidTransaction::Payment)
                }
            }
            Call::Identity(identity::Call::accept_master_key(rotation_auth_id, ..)) => {
                if is_auth_valid(caller, rotation_auth_id, CallType::AcceptMultiSigSigner) {
                    Ok(Some(*caller))
                } else {
                    Err(InvalidTransaction::Payment)
                }
            }
            Call::Identity(identity::Call::join_identity_as_key(auth_id, ..)) => {
                if is_auth_valid(caller, auth_id, CallType::AcceptMultiSigSigner) {
                    Ok(Some(*caller))
                } else {
                    Err(InvalidTransaction::Payment)
                }
            }
            _ => match caller {
                Signatory::AccountKey(key) => {
                    if let Some(did) = Balances::charge_fee_to_identity(&key) {
                        Ok(Some(Signatory::from(did)))
                    } else {
                        Ok(Some(*caller))
                    }
                }
                _ => Ok(Some(*caller)),
            },
        }
    }
}

fn is_auth_valid(singer: &Signatory, auth_id: &u64, call_type: CallType) -> bool {
    true
}
