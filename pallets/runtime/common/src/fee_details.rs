use codec::{Decode, Encode};
use core::convert::{TryFrom, TryInto};
use core::marker::PhantomData;
use pallet_identity::Module;
use polymesh_common_utilities::traits::identity::Config;
use polymesh_common_utilities::{traits::transaction_payment::CddAndFeeDetails, Context};
use polymesh_primitives::{AccountId, AuthorizationData, IdentityId, Signatory, TransactionError};
use sp_runtime::transaction_validity::InvalidTransaction;

/// A `CddHandler` that considers `TestUtils`.
pub type DevCddHandler<A> = CddHandler<A, WithTestUtils<A>>;

/// Hook for `CddHandler`'s `get_valid_payer`.
pub trait GetValidPayerHook<C> {
    /// Gets called by `CddHandler::get_valid_payer` as a pre-processing step.
    fn get_valid_payer(call: &C, caller: &AccountId) -> Option<ValidPayerResult>;
}

/// Provides a hook to deal with `TestUtils::register_did`.
pub struct WithTestUtils<A>(PhantomData<A>);

impl<C, A> GetValidPayerHook<C> for WithTestUtils<A>
where
    for<'a> &'a pallet_test_utils::Call<A>: TryFrom<&'a C>,
    A: pallet_test_utils::Config,
{
    fn get_valid_payer(call: &C, caller: &AccountId) -> Option<ValidPayerResult> {
        match <&pallet_test_utils::Call<A>>::try_from(call) {
            // Register did call.
            // all did registration should go through CDD
            Ok(pallet_test_utils::Call::register_did { .. }) => Some(Ok(Some(caller.clone()))),
            _ => None,
        }
    }
}

/// Provides a hook that does nothing.
pub struct Noop;

impl<C> GetValidPayerHook<C> for Noop {
    fn get_valid_payer(_: &C, _: &AccountId) -> Option<ValidPayerResult> {
        None
    }
}

/// The set of `Call`s from pallets that `CddHandler` recognizes specially.
pub enum Call<'a, R>
where
    R: Config + pallet_multisig::Config + pallet_relayer::Config,
{
    MultiSig(&'a pallet_multisig::Call<R>),
    Identity(&'a pallet_identity::Call<R>),
    Relayer(&'a pallet_relayer::Call<R>),
}

/// The implementation of `CddAndFeeDetails` for the chain.
#[derive(Default, Encode, Decode, Clone, Eq, PartialEq)]
pub struct CddHandler<A, H>(PhantomData<(A, H)>);

impl<C, A, H> CddAndFeeDetails<AccountId, C> for CddHandler<A, H>
where
    H: GetValidPayerHook<C>,
    for<'a> Call<'a, A>: TryFrom<&'a C>,
    A: Config<AccountId = AccountId> + pallet_multisig::Config + pallet_relayer::Config,
{
    /// Check if there's an eligible payer with valid CDD.
    /// Return the payer if found or else an error.
    /// Can also return Ok(none) to represent the case where
    /// CDD is valid but no payer should pay fee for this tx
    /// This also sets the identity in the context to the identity that was checked for CDD
    /// However, this does not set the payer context since that is meant to remain constant
    /// throughout the transaction. This function can also be used to simply check CDD and update identity context.
    fn get_valid_payer(call: &C, caller: &AccountId) -> ValidPayerResult {
        match H::get_valid_payer(call, caller) {
            Some(r) => return r,
            None => {}
        }

        // Check if the `did` has a valid CDD claim.
        let check_did_cdd = |did: &IdentityId| {
            if Module::<A>::has_valid_cdd(*did) {
                Ok(None)
            } else {
                CDD_REQUIRED
            }
        };

        // Check if the `did` has a valid CDD claim
        // and return the primary key as the payer.
        let did_primary_pays = |did: &IdentityId| {
            check_did_cdd(did)?;
            Ok(Module::<A>::get_primary_key(*did))
        };

        // Check if the `caller` key has a DID and a valid CDD claim.
        // The caller is also the payer.
        let caller_pays = |caller: &AccountId| {
            match pallet_identity::Module::<A>::get_identity(caller) {
                Some(did) => {
                    check_did_cdd(&did)?;
                    Ok(Some(caller.clone()))
                }
                // Return if there's no DID.
                None => MISSING_ID,
            }
        };

        let handle_multisig = |multisig: &AccountId, caller: &AccountId| {
            if pallet_multisig::MultiSigSigners::<A>::contains_key(multisig, caller) {
                let ms_pays = caller_pays(multisig)?;
                // If the `multisig` has a paying DID, then it's primary key pays.
                match pallet_multisig::Pallet::<A>::get_paying_did(multisig) {
                    Some(did) => Ok(Module::<A>::get_primary_key(did)),
                    None => Ok(ms_pays),
                }
            } else {
                MISSING_ID
            }
        };

        // The primary key of the DID that created the authorization
        // pays the fee to accept the authorization.
        let is_auth_valid = |acc: &AccountId, auth_id: &u64, call_type: CallType| {
            // Fetch the auth if it exists and has not expired.
            match Module::<A>::get_non_expired_auth(&Signatory::Account(acc.clone()), auth_id)
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
                _auth_issuer_pays: true,
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
            // All other calls.
            //
            // The external account must directly be linked to an identity with valid CDD.
            _ => caller_pays(caller),
        }
    }

    /// Clears context. Should be called in post_dispatch
    fn clear_context() {
        Self::set_payer_context(None);
    }

    /// Sets payer in context. Should be called by the signed extension that first charges fee.
    fn set_payer_context(payer: Option<AccountId>) {
        Context::set_current_payer::<pallet_identity::Module<A>>(payer);
    }

    /// Fetches fee payer for further payments (forwarded calls)
    fn get_payer_from_context() -> Option<AccountId> {
        Context::current_payer::<pallet_identity::Module<A>>()
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

const CDD_REQUIRED: ValidPayerResult = Err(InvalidTransaction::Custom(
    TransactionError::CddRequired as u8,
));

const MISSING_ID: ValidPayerResult = Err(InvalidTransaction::Custom(
    TransactionError::MissingIdentity as u8,
));

const INVALID_AUTH: ValidPayerResult = Err(InvalidTransaction::Custom(
    TransactionError::InvalidAuthorization as u8,
));
