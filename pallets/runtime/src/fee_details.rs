use crate::{multisig, runtime, Runtime};

use pallet_transaction_payment::FeeDetails;
use polymesh_primitives::{AccountKey, IdentityId, Signatory, TransactionError};
use polymesh_runtime_common::{identity::LinkedKeyInfo, Context};
use polymesh_runtime_identity as identity;

use sp_runtime::transaction_validity::{
    InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransaction,
};

use codec::{Decode, Encode};
use core::convert::TryFrom;
use frame_support::dispatch::DispatchInfo;

type Identity = identity::Module<Runtime>;
type Call = runtime::Call;

/// This signed extension double-checks and updates the current identifier extracted from
/// the caller account in each transaction.
#[derive(Default, Encode, Decode, Clone, Eq, PartialEq)]
pub struct FeeHandler;

// impl<T: frame_system::Trait + Send + Sync> UpdateDid<T> {
//     pub fn new() -> Self {
//         UpdateDid(PhantomData)
//     }

//     /// It extracts the `IdentityId` associated with `who` account.
//     fn identity_from_key(who: &T::AccountId) -> Option<IdentityId> {
//         if let Ok(who_key) = AccountKey::try_from(who.encode()) {
//             if let Some(linked_key_info) = Identity::key_to_identity_ids(&who_key) {
//                 if let LinkedKeyInfo::Unique(id) = linked_key_info {
//                     return Some(id);
//                 }
//             }
//         }
//         None
//     }
// }

impl FeeDetails<Call> for FeeHandler {
    fn whom_to_charge(
        call: &Call,
        caller: &Signatory,
    ) -> Result<Option<Signatory>, InvalidTransaction> {
        Ok(None)
    }
}
