use codec::{Decode, Encode};
use core::convert::{TryFrom, TryInto};
use core::marker::PhantomData;
use sp_runtime::transaction_validity::InvalidTransaction;

use pallet_identity::{Config as IdentityConfig, Context, Pallet as Identity};
use polymesh_primitives::traits::CddAndFeeDetails;
use polymesh_primitives::{AccountId, AuthorizationData, IdentityId, Signatory, TransactionError};

use pallet_identity::Call as IdentityCall;
use pallet_multisig::Call as MultiSigCall;
use pallet_relayer::Call as RelayerCall;

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

impl<A> CddHandler<A>
where
    A: IdentityConfig<AccountId = AccountId> + pallet_multisig::Config + pallet_relayer::Config,
{
    /// If the authorization is valid for the call type, returns the account that will pay for the call.
    fn get_payers_account(
        signatory_acc: &Signatory<AccountId>,
        auth_id: &u64,
        call_type: CallType,
    ) -> ValidPayerResult {
        if let Some(auth) = Identity::<A>::get_non_expired_auth(signatory_acc, auth_id) {
            match call_type {
                CallType::RemoveAuthorization => {
                    return Ok(CddHandler::<A>::get_did_primary_key(auth.authorized_by));
                }
                CallType::AcceptMultiSigSigner => {
                    if let AuthorizationData::AddMultiSigSigner(_) = auth.authorization_data {
                        return Ok(CddHandler::<A>::get_did_primary_key(auth.authorized_by));
                    }
                }
                CallType::AcceptIdentitySecondary => {
                    if let AuthorizationData::JoinIdentity(_) = auth.authorization_data {
                        return Ok(CddHandler::<A>::get_did_primary_key(auth.authorized_by));
                    }
                }
                CallType::AcceptIdentityPrimary => {
                    if let AuthorizationData::RotatePrimaryKey = auth.authorization_data {
                        return Ok(CddHandler::<A>::get_did_primary_key(auth.authorized_by));
                    }
                }
                CallType::AcceptRelayerPayingKey => {
                    if let AuthorizationData::AddRelayerPayingKey(_, _, _) = auth.authorization_data
                    {
                        return Ok(CddHandler::<A>::get_did_primary_key(auth.authorized_by));
                    }
                }
                CallType::RotatePrimaryToSecondary => {
                    if let AuthorizationData::RotatePrimaryKeyToSecondary(_) =
                        auth.authorization_data
                    {
                        return Ok(CddHandler::<A>::get_did_primary_key(auth.authorized_by));
                    }
                }
            }
        }

        INVALID_AUTH
    }

    /// If the multisig and the authorization are valid, returns the multisig payer account.
    fn get_multisig_payer(
        multisig_acc_id: AccountId,
        caller_acc_id: &AccountId,
        auth_id: Option<&u64>,
        call_type: Option<CallType>,
    ) -> ValidPayerResult {
        if pallet_multisig::MultiSigSigners::<A>::contains_key(&multisig_acc_id, caller_acc_id) {
            if let Some(auth_id) = auth_id {
                let call_type = call_type.ok_or(InvalidTransaction::Custom(1))?;
                let signatory_acc = Signatory::Account(multisig_acc_id);
                return CddHandler::<A>::get_payers_account(&signatory_acc, auth_id, call_type);
            }

            match pallet_multisig::Pallet::<A>::get_paying_did(&multisig_acc_id) {
                Some(did) => return Ok(CddHandler::<A>::get_did_primary_key(did)),
                None => return Ok(Some(multisig_acc_id.clone())),
            }
        }

        MISSING_ID
    }

    /// Returns the primary key of the did.
    fn get_did_primary_key(did: IdentityId) -> Option<AccountId> {
        Identity::<A>::get_primary_key(did)
    }

    /// Returns the account that will pay for the call.
    fn handle_multisig_calls(
        call: &MultiSigCall<A>,
        caller_acc_id: &AccountId,
    ) -> ValidPayerResult {
        match call {
            MultiSigCall::accept_multisig_signer { auth_id } => {
                CddHandler::<A>::get_payers_account(
                    &Signatory::Account(caller_acc_id.clone()),
                    auth_id,
                    CallType::AcceptMultiSigSigner,
                )
            }
            MultiSigCall::approve_join_identity { multisig, auth_id } => {
                CddHandler::<A>::get_multisig_payer(
                    multisig.clone(),
                    caller_acc_id,
                    Some(auth_id),
                    Some(CallType::AcceptIdentitySecondary),
                )
            }
            MultiSigCall::create_proposal { multisig, .. }
            | MultiSigCall::approve { multisig, .. }
            | MultiSigCall::reject { multisig, .. } => {
                CddHandler::<A>::get_multisig_payer(multisig.clone(), caller_acc_id, None, None)
            }
            _ => Ok(Some(caller_acc_id.clone())),
        }
    }

    /// Returns the account that will pay for the call.
    fn handle_identity_calls(
        call: &IdentityCall<A>,
        caller_acc_id: &AccountId,
    ) -> ValidPayerResult {
        match call {
            IdentityCall::join_identity_as_key { auth_id } => CddHandler::<A>::get_payers_account(
                &Signatory::Account(caller_acc_id.clone()),
                auth_id,
                CallType::AcceptIdentitySecondary,
            ),
            IdentityCall::accept_primary_key {
                rotation_auth_id, ..
            } => CddHandler::<A>::get_payers_account(
                &Signatory::Account(caller_acc_id.clone()),
                rotation_auth_id,
                CallType::AcceptIdentityPrimary,
            ),
            IdentityCall::rotate_primary_key_to_secondary { auth_id, .. } => {
                CddHandler::<A>::get_payers_account(
                    &Signatory::Account(caller_acc_id.clone()),
                    auth_id,
                    CallType::RotatePrimaryToSecondary,
                )
            }
            IdentityCall::remove_authorization {
                target,
                auth_id,
                auth_issuer_pays: true,
            } => {
                CddHandler::<A>::get_payers_account(&target, auth_id, CallType::RemoveAuthorization)
            }
            _ => Ok(Some(caller_acc_id.clone())),
        }
    }

    /// Returns the account that will pay for the call.
    fn handle_relayer_calls(call: &RelayerCall<A>, caller_acc_id: &AccountId) -> ValidPayerResult {
        if let RelayerCall::accept_paying_key { auth_id } = call {
            return CddHandler::<A>::get_payers_account(
                &Signatory::Account(caller_acc_id.clone()),
                auth_id,
                CallType::AcceptRelayerPayingKey,
            );
        }

        Ok(Some(caller_acc_id.clone()))
    }
}

impl<C, A> CddAndFeeDetails<AccountId, C> for CddHandler<A>
where
    for<'a> Call<'a, A>: TryFrom<&'a C>,
    A: IdentityConfig<AccountId = AccountId> + pallet_multisig::Config + pallet_relayer::Config,
{
    fn get_valid_payer(call: &C, caller_acc_id: &AccountId) -> ValidPayerResult {
        match call.try_into() {
            Ok(Call::MultiSig(multi_sig_call)) => {
                CddHandler::<A>::handle_multisig_calls(multi_sig_call, caller_acc_id)
            }
            Ok(Call::Identity(identity_call)) => {
                CddHandler::<A>::handle_identity_calls(identity_call, caller_acc_id)
            }
            Ok(Call::Relayer(relayer_call)) => {
                CddHandler::<A>::handle_relayer_calls(relayer_call, caller_acc_id)
            }
            Err(_) => Ok(Some(caller_acc_id.clone())),
        }
    }

    fn clear_context() {
        Self::set_payer_context(None);
    }

    fn set_payer_context(payer: Option<AccountId>) {
        Context::set_current_payer::<Identity<A>>(payer);
    }

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
