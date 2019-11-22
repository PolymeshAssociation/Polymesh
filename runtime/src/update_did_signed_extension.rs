use crate::{identity, identity::LinkedKeyInfo, Runtime};

use primitives::{AccountId, IdentityId, Key, TransactionError};

use sr_primitives::{
    traits::SignedExtension,
    transaction_validity::{
        InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransaction,
    },
};

use codec::{Decode, Encode};
use core::convert::TryFrom;
use srml_support::dispatch::DispatchInfo;

type Identity = identity::Module<Runtime>;
type Call = crate::Call;

/// This signed extension double-checks and updates the current identifier extracted from
/// the caller account in each transaction.
///
/// # TODO
/// - After transaction, Do we need to clean `CurrentDid`?
/// - Move outside `lib.rs`. It needs a small refactor to extract `Runtime` from here.
#[derive(Default, Encode, Decode, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct UpdateDid;

impl UpdateDid {
    /// It extracts the `IdentityId` associated with `who` account.
    fn identity_from_key(who: &AccountId) -> Option<IdentityId> {
        if let Ok(who_key) = Key::try_from(who.encode()) {
            if let Some(linked_key_info) = Identity::key_to_identity_ids(&who_key) {
                if let LinkedKeyInfo::Unique(id) = linked_key_info {
                    return Some(id);
                }
            }
        }
        None
    }
}

impl SignedExtension for UpdateDid {
    type AccountId = AccountId;
    type Call = Call;
    type AdditionalSigned = ();
    type Pre = ();

    fn additional_signed(&self) -> rstd::result::Result<(), TransactionValidityError> {
        Ok(())
    }

    /// It ensures that transaction caller account has an associated identity and
    /// that identity has been validated by any KYC.
    /// The current identity will be accesible through `Identity::current_did`.
    ///
    /// Only the following methods can be called with no identity:
    ///     - `identity::register_did`
    fn validate(
        &self,
        who: &Self::AccountId,
        call: &Self::Call,
        _: DispatchInfo,
        _: usize,
    ) -> TransactionValidity {
        match call {
            // Add here any function from any module which does *not* need a current identity.
            Call::Identity(identity::Call::register_did(..)) => Ok(ValidTransaction::default()),
            // Other calls should be identified
            _ => {
                let id_opt = Self::identity_from_key(who);
                if let Some(id) = id_opt.clone() {
                    if Identity::has_valid_kyc(id) {
                        Identity::set_current_did(id_opt);
                        Ok(ValidTransaction::default())
                    } else {
                        Err(InvalidTransaction::Custom(TransactionError::RequiredKYC as u8).into())
                    }
                } else {
                    sr_primitives::print("ERROR: This transaction requires an Identity");
                    Err(InvalidTransaction::Custom(TransactionError::MissingIdentity as u8).into())
                }
            }
        }
    }
}
