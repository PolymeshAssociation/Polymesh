use codec::{Decode, Encode};
use core::convert::{TryFrom, TryInto};
use core::marker::PhantomData;
use pallet_identity::{Config as IdentityConfig, Context, Pallet as Identity};
use polymesh_primitives::{
    traits::CddAndFeeDetails, AccountId, AuthorizationData, IdentityId, Signatory, TransactionError,
};
use sp_runtime::transaction_validity::InvalidTransaction;

/// The set of `Call`s from pallets that `CddHandler` recognizes specially.
pub enum Call<'a, R>
where
    R: IdentityConfig + pallet_multisig::Config + pallet_relayer::Config,
{
    MultiSig(&'a pallet_multisig::Call<R>),
    Identity(&'a pallet_identity::Call<R>),
    Relayer(&'a pallet_relayer::Call<R>),
}

/// The implementation of `CddAndFeeDetails` for the chain.
#[derive(Default, Encode, Decode, Clone, Eq, PartialEq)]
pub struct CddHandler<A>(PhantomData<A>);

impl<C, A> CddAndFeeDetails<AccountId, C> for CddHandler<A>
where
    for<'a> Call<'a, A>: TryFrom<&'a C>,
    A: IdentityConfig<AccountId = AccountId> + pallet_multisig::Config + pallet_relayer::Config,
{
    /// Check if there's an eligible payer with valid CDD.
    /// Return the payer if found or else an error.
    /// Can also return Ok(none) to represent the case where
    /// CDD is valid but no payer should pay fee for this tx
    /// This also sets the identity in the context to the identity that was checked for CDD
    /// However, this does not set the payer context since that is meant to remain constant
    /// throughout the transaction. This function can also be used to simply check CDD and update identity context.
    fn get_valid_payer(call: &C, caller: &AccountId) -> ValidPayerResult {
        // Return the primary key as the payer.
        let did_primary_pays = |did: &IdentityId| Ok(Identity::<A>::get_primary_key(*did));

        let handle_multisig = |multisig: &AccountId, caller: &AccountId| {
            if pallet_multisig::MultiSigSigners::<A>::contains_key(multisig, caller) {
                // If the `multisig` has a paying DID, then it's primary key pays.
                match pallet_multisig::Pallet::<A>::get_paying_did(multisig) {
                    Some(did) => Ok(Identity::<A>::get_primary_key(did)),
                    None => Ok(Some(multisig.clone())),
                }
            } else {
                MISSING_ID
            }
        };

        // The primary key of the DID that created the authorization
        // pays the fee to accept the authorization.
        let is_auth_valid = |acc: &AccountId, auth_id: &u64, call_type: CallType| {
            // Fetch the auth if it exists and has not expired.
            match Identity::<A>::get_non_expired_auth(&Signatory::Account(acc.clone()), auth_id)
                .map(|auth| (auth.authorized_by, (auth.authorization_data, call_type)))
            {
                // Different auths have different authorization data requirements.
                // Hence we match call type to ensure proper authorization data is present.
                // We only need to check that there's a payer with a valid CDD.
                // Business logic for authorisations can be checked post-Signed Extension.
                Some((
                    by,
                    (AuthorizationData::AddMultiSigSigner(_), CallType::AcceptMultiSigSigner)
                    | (AuthorizationData::JoinIdentity(_), CallType::AcceptIdentitySecondary)
                    | (AuthorizationData::RotatePrimaryKey, CallType::AcceptIdentityPrimary)
                    | (
                        AuthorizationData::RotatePrimaryKeyToSecondary(_),
                        CallType::RotatePrimaryToSecondary,
                    )
                    | (AuthorizationData::AddRelayerPayingKey(..), CallType::AcceptRelayerPayingKey)
                    | (_, CallType::RemoveAuthorization),
                )) => did_primary_pays(&by),
                // None of the above apply, so error.
                _ => INVALID_AUTH,
            }
        };

        let handle_multisig_auth =
            |multisig: &AccountId, caller: &AccountId, auth_id: &u64, call_type: CallType| {
                if pallet_multisig::MultiSigSigners::<A>::contains_key(multisig, caller) {
                    is_auth_valid(multisig, auth_id, call_type)
                } else {
                    MISSING_ID
                }
            };

        // The CDD check and fee payer varies depending on the transaction.
        // This match covers all possible scenarios.
        match call.try_into() {
            // Call made by a key to accept invitation to become a signing key
            // of a multisig that has a valid CDD. The auth should be valid.
            Ok(Call::MultiSig(pallet_multisig::Call::accept_multisig_signer { auth_id })) => {
                is_auth_valid(caller, auth_id, CallType::AcceptMultiSigSigner)
            }
            // Call made by a multisig signing key to accept invitation to become a secondary key
            // of an existing identity that has a valid CDD. The auth should be valid.
            Ok(Call::MultiSig(pallet_multisig::Call::approve_join_identity {
                multisig,
                auth_id,
            })) => {
                handle_multisig_auth(multisig, caller, auth_id, CallType::AcceptIdentitySecondary)
            }
            // Call made by a new Account key to accept invitation to become a secondary key
            // of an existing identity that has a valid CDD. The auth should be valid.
            Ok(Call::Identity(pallet_identity::Call::join_identity_as_key { auth_id })) => {
                is_auth_valid(caller, auth_id, CallType::AcceptIdentitySecondary)
            }
            // Call made by a new Account key to accept invitation to become the primary key
            // of an existing identity that has a valid CDD. The auth should be valid.
            Ok(Call::Identity(pallet_identity::Call::accept_primary_key {
                rotation_auth_id,
                ..
            })) => is_auth_valid(caller, rotation_auth_id, CallType::AcceptIdentityPrimary),
            // Call made by a new Account key to accept invitation to become the primary key
            // of an existing identity that has a valid CDD. The auth should be valid.
            Ok(Call::Identity(pallet_identity::Call::rotate_primary_key_to_secondary {
                auth_id,
                ..
            })) => is_auth_valid(caller, auth_id, CallType::RotatePrimaryToSecondary),
            // Call made by a new Account key to remove invitation for certain authorizations
            // in an existing identity that has a valid CDD. The auth should be valid.
            Ok(Call::Identity(pallet_identity::Call::remove_authorization {
                auth_id,
                auth_issuer_pays: true,
                ..
            })) => is_auth_valid(caller, auth_id, CallType::RemoveAuthorization),
            // Call made by a user key to accept subsidy from a paying key. The auth should be valid.
            Ok(Call::Relayer(pallet_relayer::Call::accept_paying_key { auth_id })) => {
                is_auth_valid(caller, auth_id, CallType::AcceptRelayerPayingKey)
            }
            // Call made by an Account key to propose, reject or approve a multisig transaction.
            // The multisig must have valid CDD and the caller must be a signer of the multisig.
            Ok(Call::MultiSig(
                pallet_multisig::Call::create_proposal { multisig, .. }
                | pallet_multisig::Call::approve { multisig, .. }
                | pallet_multisig::Call::reject { multisig, .. },
            )) => handle_multisig(multisig, caller),
            // All other calls
            _ => Ok(Some(caller.clone())),
        }
    }

    /// Clears context. Should be called in post_dispatch
    fn clear_context() {
        Self::set_payer_context(None);
    }

    /// Sets payer in context. Should be called by the signed extension that first charges fee.
    fn set_payer_context(payer: Option<AccountId>) {
        Context::set_current_payer::<Identity<A>>(payer);
    }

    /// Fetches fee payer for further payments (forwarded calls)
    fn get_payer_from_context() -> Option<AccountId> {
        Context::current_payer::<Identity<A>>()
    }
}

#[derive(Encode, Decode)]
enum CallType {
    AcceptMultiSigSigner,
    AcceptRelayerPayingKey,
    AcceptIdentitySecondary,
    AcceptIdentityPrimary,
    RotatePrimaryToSecondary,
    /// Matches any call to `remove_authorization`,
    /// where the authorization is available for `auth.authorized_by` payer redirection.
    RemoveAuthorization,
}

type ValidPayerResult = Result<Option<AccountId>, InvalidTransaction>;

const MISSING_ID: ValidPayerResult = Err(InvalidTransaction::Custom(
    TransactionError::MissingIdentity as u8,
));

const INVALID_AUTH: ValidPayerResult = Err(InvalidTransaction::Custom(
    TransactionError::InvalidAuthorization as u8,
));
