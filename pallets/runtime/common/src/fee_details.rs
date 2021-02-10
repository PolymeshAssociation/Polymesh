use codec::{Decode, Encode};
use polymesh_primitives::{AccountId, TransactionError};
use sp_runtime::transaction_validity::InvalidTransaction;

#[derive(Encode, Decode)]
pub enum CallType {
    AcceptMultiSigSigner,
    AcceptIdentitySecondary,
    AcceptIdentityPrimary,
    /// Matches any call to `remove_authorization`,
    /// where the authorization is available for `auth.authorized_by` payer redirection.
    RemoveAuthorization,
}

pub type ValidPayerResult = Result<Option<AccountId>, InvalidTransaction>;

pub fn cdd_required() -> ValidPayerResult {
    Err(InvalidTransaction::Custom(
        TransactionError::CddRequired as u8,
    ))
}

pub fn missing_id() -> ValidPayerResult {
    Err(InvalidTransaction::Custom(
        TransactionError::MissingIdentity as u8,
    ))
}
