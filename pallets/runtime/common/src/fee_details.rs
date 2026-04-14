use codec::{Decode, Encode};
use core::convert::{TryFrom, TryInto};
use core::marker::PhantomData;
use sp_runtime::transaction_validity::InvalidTransaction;

use pallet_identity::{Config as IdentityConfig, Pallet as Identity};
use polymesh_primitives::traits::CurrentFeePayer;
use polymesh_primitives::transaction_payment::CallPaymentInfo;
use polymesh_primitives::{AccountId, AuthorizationData, IdentityId, Signatory, TransactionError};
use polymesh_transaction_payment::Pallet as PolymeshTransactionPallet;

use pallet_identity::Call as IdentityCall;
use pallet_multisig::Call as MultiSigCall;
use pallet_relayer::Call as RelayerCall;

type ValidPayerResult = Result<CallPaymentInfo<AccountId>, InvalidTransaction>;

#[derive(Encode, Decode)]
enum CallType {
    AcceptMultiSigSigner,
    AcceptIdentitySecondary,
    AcceptIdentityPrimary,
    RotatePrimaryToSecondary,
    RemoveAuthorization,
}

/// The set of `Call`s from pallets that `TxFeeHandler` recognizes specially.
pub enum Call<'a, R>
where
    R: IdentityConfig + pallet_multisig::Config + pallet_relayer::Config,
{
    MultiSig(&'a pallet_multisig::Call<R>),
    Identity(&'a pallet_identity::Call<R>),
    Relayer(&'a pallet_relayer::Call<R>),
}

/// The implementation of `CurrentFeePayer` for the chain.
#[derive(Default, Encode, Decode, Clone, Eq, PartialEq)]
pub struct TxFeeHandler<A>(PhantomData<A>);

impl<A> TxFeeHandler<A>
where
    A: IdentityConfig<AccountId = AccountId>
        + pallet_multisig::Config
        + pallet_relayer::Config
        + polymesh_transaction_payment::Config,
{
    /// Returns the account that will pay for the call.
    fn get_payers_account(
        account_id: AccountId,
        auth_id: &u64,
        call_type: CallType,
    ) -> Result<AccountId, InvalidTransaction> {
        if let Some(auth) =
            Identity::<A>::get_non_expired_auth(&Signatory::Account(account_id), auth_id)
        {
            match call_type {
                CallType::RemoveAuthorization => {
                    return Ok(TxFeeHandler::<A>::get_did_primary_key(auth.authorized_by)?);
                }
                CallType::AcceptMultiSigSigner => {
                    if let AuthorizationData::AddMultiSigSigner(_) = auth.authorization_data {
                        return Ok(TxFeeHandler::<A>::get_did_primary_key(auth.authorized_by)?);
                    }
                }
                CallType::AcceptIdentitySecondary => {
                    if let AuthorizationData::JoinIdentity(_) = auth.authorization_data {
                        return Ok(TxFeeHandler::<A>::get_did_primary_key(auth.authorized_by)?);
                    }
                }
                CallType::AcceptIdentityPrimary => {
                    if let AuthorizationData::RotatePrimaryKey = auth.authorization_data {
                        return Ok(TxFeeHandler::<A>::get_did_primary_key(auth.authorized_by)?);
                    }
                }
                CallType::RotatePrimaryToSecondary => {
                    if let AuthorizationData::RotatePrimaryKeyToSecondary(_) =
                        auth.authorization_data
                    {
                        return Ok(TxFeeHandler::<A>::get_did_primary_key(auth.authorized_by)?);
                    }
                }
            }
        }

        Err(InvalidTransaction::Custom(
            TransactionError::InvalidAuthorization as u8,
        ))
    }

    /// Returns the multisig payer account.
    fn get_multisig_payer(
        multisig_acc_id: AccountId,
        caller_acc_id: &AccountId,
        call_auth_id: Option<(CallType, &u64)>,
    ) -> Result<AccountId, InvalidTransaction> {
        if pallet_multisig::MultiSigSigners::<A>::contains_key(&multisig_acc_id, caller_acc_id) {
            if let Some((call_type, auth_id)) = call_auth_id {
                return TxFeeHandler::<A>::get_payers_account(multisig_acc_id, auth_id, call_type);
            }

            match pallet_multisig::Pallet::<A>::get_paying_did(&multisig_acc_id) {
                Some(did) => return Ok(TxFeeHandler::<A>::get_did_primary_key(did)?),
                None => return Ok(multisig_acc_id),
            }
        }

        Err(InvalidTransaction::Custom(
            TransactionError::MissingIdentity as u8,
        ))
    }

    /// Returns the primary key of the did.
    fn get_did_primary_key(did: IdentityId) -> Result<AccountId, InvalidTransaction> {
        Identity::<A>::get_primary_key(did).ok_or(InvalidTransaction::Custom(
            TransactionError::MissingIdentity as u8,
        ))
    }

    /// Returns the account that will pay for the call.
    fn handle_multisig_calls(call: &MultiSigCall<A>, caller_acc_id: AccountId) -> ValidPayerResult {
        // Returns true if the caller has already voted on the given multisig proposal.
        let already_voted = |multisig: &AccountId, proposal_id: &u64| {
            pallet_multisig::Votes::<A>::get((multisig, proposal_id), &caller_acc_id)
        };

        // Returns the proposal id for the given multisig and auth_id.
        let get_proposal_id = |multisig: &AccountId, auth_id: &u64| {
            pallet_multisig::AuthToProposalId::<A>::get(multisig, auth_id)
        };

        match call {
            MultiSigCall::accept_multisig_signer { auth_id } => {
                let paying_acc = TxFeeHandler::<A>::get_payers_account(
                    caller_acc_id,
                    auth_id,
                    CallType::AcceptMultiSigSigner,
                )?;
                Ok(CallPaymentInfo::new(paying_acc, Some(*auth_id), None))
            }
            MultiSigCall::approve_join_identity { multisig, auth_id } => {
                if let Some(proposal_id) = get_proposal_id(multisig, auth_id) {
                    if already_voted(&multisig, &proposal_id) {
                        return Err(InvalidTransaction::Custom(
                            TransactionError::AlreadyVoted as u8,
                        ));
                    }
                }
                let paying_acc = TxFeeHandler::<A>::get_multisig_payer(
                    multisig.clone(),
                    &caller_acc_id,
                    Some((CallType::AcceptIdentitySecondary, auth_id)),
                )?;
                Ok(CallPaymentInfo::new(
                    paying_acc,
                    Some(*auth_id),
                    Some(Signatory::Account(multisig.clone())),
                ))
            }
            MultiSigCall::approve {
                multisig,
                proposal_id,
                ..
            }
            | MultiSigCall::reject {
                multisig,
                proposal_id,
                ..
            } => {
                if already_voted(&multisig, &proposal_id) {
                    return Err(InvalidTransaction::Custom(
                        TransactionError::AlreadyVoted as u8,
                    ));
                }
                let paying_acc =
                    TxFeeHandler::<A>::get_multisig_payer(multisig.clone(), &caller_acc_id, None)?;
                Ok(CallPaymentInfo::new(paying_acc, None, None))
            }
            MultiSigCall::create_proposal { multisig, .. } => {
                let paying_acc =
                    TxFeeHandler::<A>::get_multisig_payer(multisig.clone(), &caller_acc_id, None)?;
                Ok(CallPaymentInfo::new(paying_acc, None, None))
            }
            _ => Ok(CallPaymentInfo::new(caller_acc_id, None, None)),
        }
    }

    /// Returns the account that will pay for the call.
    fn handle_identity_calls(call: &IdentityCall<A>, caller_acc_id: AccountId) -> ValidPayerResult {
        match call {
            IdentityCall::join_identity_as_key { auth_id } => {
                let paying_acc = TxFeeHandler::<A>::get_payers_account(
                    caller_acc_id,
                    auth_id,
                    CallType::AcceptIdentitySecondary,
                )?;
                Ok(CallPaymentInfo::new(paying_acc, Some(*auth_id), None))
            }
            IdentityCall::accept_primary_key {
                rotation_auth_id, ..
            } => {
                let paying_acc = TxFeeHandler::<A>::get_payers_account(
                    caller_acc_id,
                    rotation_auth_id,
                    CallType::AcceptIdentityPrimary,
                )?;
                Ok(CallPaymentInfo::new(
                    paying_acc,
                    Some(*rotation_auth_id),
                    None,
                ))
            }
            IdentityCall::rotate_primary_key_to_secondary { auth_id, .. } => {
                let paying_acc = TxFeeHandler::<A>::get_payers_account(
                    caller_acc_id,
                    auth_id,
                    CallType::RotatePrimaryToSecondary,
                )?;
                Ok(CallPaymentInfo::new(paying_acc, Some(*auth_id), None))
            }
            IdentityCall::remove_authorization {
                target,
                auth_id,
                auth_issuer_pays: true,
            } => {
                if target.as_account() != Some(&caller_acc_id) {
                    return Err(InvalidTransaction::Custom(
                        TransactionError::InvalidAuthorization as u8,
                    ));
                }
                let paying_acc = TxFeeHandler::<A>::get_payers_account(
                    caller_acc_id,
                    auth_id,
                    CallType::RemoveAuthorization,
                )?;
                Ok(CallPaymentInfo::new(
                    paying_acc,
                    Some(*auth_id),
                    Some(target.clone()),
                ))
            }
            _ => Ok(CallPaymentInfo::new(caller_acc_id, None, None)),
        }
    }

    /// Returns the account that will pay for the call.
    fn handle_relayer_calls(call: &RelayerCall<A>, caller_acc_id: AccountId) -> ValidPayerResult {
        if let RelayerCall::accept_subsidy { paying_key } = call {
            if pallet_relayer::Pallet::<A>::has_pending_subsidy(&caller_acc_id, paying_key) {
                return Ok(CallPaymentInfo::new(paying_key.clone(), None, None));
            }
        }

        Ok(CallPaymentInfo::new(caller_acc_id, None, None))
    }

    /// Decreases the authorization count for the given target and auth_id.
    fn decrease_auth_count(signatory: &Signatory<AccountId>, auth_id: &u64) {
        pallet_identity::Pallet::<A>::decrease_authorization_count(&signatory, auth_id);
    }
}

impl<C, A> CurrentFeePayer<AccountId, C> for TxFeeHandler<A>
where
    for<'a> Call<'a, A>: TryFrom<&'a C>,
    A: IdentityConfig<AccountId = AccountId>
        + pallet_multisig::Config
        + pallet_relayer::Config
        + polymesh_transaction_payment::Config,
{
    fn call_payment_info(
        call: &C,
        caller_acc_id: AccountId,
    ) -> Result<CallPaymentInfo<AccountId>, InvalidTransaction> {
        match call.try_into() {
            Ok(Call::MultiSig(multi_sig_call)) => {
                TxFeeHandler::<A>::handle_multisig_calls(multi_sig_call, caller_acc_id)
            }
            Ok(Call::Identity(identity_call)) => {
                TxFeeHandler::<A>::handle_identity_calls(identity_call, caller_acc_id)
            }
            Ok(Call::Relayer(relayer_call)) => {
                TxFeeHandler::<A>::handle_relayer_calls(relayer_call, caller_acc_id)
            }
            Err(_) => Ok(CallPaymentInfo::new(caller_acc_id, None, None)),
        }
    }

    fn set_payer_context(payer: Option<AccountId>) {
        PolymeshTransactionPallet::<A>::set_current_payer(payer);
    }

    fn get_payer_from_context() -> Option<AccountId> {
        PolymeshTransactionPallet::<A>::current_payer()
    }

    fn decrease_authorization_count(call_payment_info: &CallPaymentInfo<AccountId>) {
        if let Some(auth_id) = call_payment_info.auth_id() {
            if let Some(target) = call_payment_info.ms_signatory() {
                return TxFeeHandler::<A>::decrease_auth_count(target, &auth_id);
            }

            if let Some(payer_record) =
                pallet_identity::KeyRecords::<A>::get(call_payment_info.paying_account())
            {
                if let Some(payer_did) = payer_record.as_did() {
                    let signatory =
                        pallet_identity::AuthorizationsGiven::<A>::get(&payer_did, &auth_id);
                    pallet_identity::Pallet::<A>::decrease_authorization_count(
                        &signatory, &auth_id,
                    );
                }
            }
        }
    }
}
