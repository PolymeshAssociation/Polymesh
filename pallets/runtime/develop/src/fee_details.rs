// This file is part of the Polymesh distribution (https://github.com/PolymathNetwork/Polymesh).
// Copyright (c) 2020 Polymath

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use crate::{runtime, Runtime};

use pallet_balances as balances;
use pallet_identity as identity;
use pallet_multisig as multisig;
use pallet_transaction_payment::CddAndFeeDetails;
use polymesh_common_utilities::Context;
use polymesh_primitives::{
    traits::IdentityCurrency, AccountId, AccountKey, AuthorizationData, IdentityId, Signatory,
    TransactionError,
};
use sp_runtime::transaction_validity::InvalidTransaction;

use codec::{Decode, Encode};
use core::convert::TryFrom;
use frame_support::StorageMap;

type Identity = identity::Module<Runtime>;
type Balances = balances::Module<Runtime>;

type Call = runtime::Call;

enum CallType {
    AcceptMultiSigSigner,
    AcceptIdentitySigner,
    AcceptIdentityMaster,
}

#[derive(Default, Encode, Decode, Clone, Eq, PartialEq)]
pub struct CddHandler;

impl CddAndFeeDetails<Call> for CddHandler {
    /// Check if there's an eligible payer with valid CDD.
    /// Return the payer if found or else an error.
    /// Can also return Ok(none) to represent the case where
    /// CDD is valid but no payer should pay fee for this tx
    /// This also sets the identity in the context to the identity that was checked for CDD
    /// However, this does not set the payer context since that is meant to remain constant
    /// throughout the transaction. This function can also be used to simply check CDD and update identity context.
    fn get_valid_payer(
        call: &Call,
        caller: &Signatory,
    ) -> Result<Option<Signatory>, InvalidTransaction> {
        // The CDD check and fee payer varies depending on the transaction.
        // This match covers all possible scenarios.
        match call {
            // Register did call. This should be removed before mainnet launch and
            // all did registration should go through CDD
            Call::Identity(identity::Call::register_did(..)) => {
                sp_runtime::print("register_did, CDD check bypassed");
                Ok(Some(*caller))
            }
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
            // Call to set fee payer
            Call::Balances(balances::Call::change_charge_did_flag(charge_did)) => match caller {
                Signatory::AccountKey(key) => {
                    if let Some(did) = Identity::get_identity(key) {
                        if Identity::has_valid_cdd(did) {
                            Context::set_current_identity::<Identity>(Some(did));
                            if *charge_did {
                                return Ok(Some(Signatory::from(did)));
                            } else {
                                return Ok(Some(*caller));
                            }
                        }
                        return Err(InvalidTransaction::Custom(
                            TransactionError::CddRequired as u8,
                        )
                        .into());
                    }
                    // Return an error if any of the above checks fail
                    Err(InvalidTransaction::Custom(TransactionError::MissingIdentity as u8).into())
                }
                // A did was passed as the caller. The did should be charged the fee.
                // This will never happen during an external call.
                Signatory::Identity(did) => check_cdd(did),
            },
            // All other calls
            _ => match caller {
                // An external account was passed as the caller. This is the normal use case.
                // If the account has enabled charging fee to identity then the identity should be charged
                // otherwise, the account should be charged. In any case, the external account
                // must directly be linked to an identity with valid CDD.
                Signatory::AccountKey(key) => {
                    if let Some(did) = Identity::get_identity(key) {
                        if Identity::has_valid_cdd(did) {
                            Context::set_current_identity::<Identity>(Some(did));
                            if let Some(fee_did) = Balances::charge_fee_to_identity(&key) {
                                sp_runtime::print("charging identity");
                                return Ok(Some(Signatory::from(fee_did)));
                            } else {
                                sp_runtime::print("charging key");
                                return Ok(Some(*caller));
                            }
                        }
                        return Err(InvalidTransaction::Custom(
                            TransactionError::CddRequired as u8,
                        )
                        .into());
                    }
                    // Return an error if any of the above checks fail
                    Err(InvalidTransaction::Custom(TransactionError::MissingIdentity as u8).into())
                }
                // A did was passed as the caller. The did should be charged the fee.
                // This will never happen during an external call.
                Signatory::Identity(did) => check_cdd(did),
            },
        }
    }

    /// Clears context. Should be called in post_dispatch
    fn clear_context() {
        Context::set_current_identity::<Identity>(None);
        Context::set_current_payer::<Identity>(None);
    }

    /// Sets payer in context. Should be called by the signed extension that first charges fee.
    fn set_payer_context(payer: Option<Signatory>) {
        Context::set_current_payer::<Identity>(payer);
    }

    /// Fetches fee payer for further payements (forwareded calls)
    fn get_payer_from_context() -> Option<Signatory> {
        Context::current_payer::<Identity>()
    }

    fn set_current_identity(did: &IdentityId) {
        Context::set_current_identity::<Identity>(Some(*did));
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
                    // make sure that the auth was created by a valid multisig
                    if let Signatory::AccountKey(multisig) = auth.authorized_by {
                        let ms = AccountId::decode(&mut &multisig.as_slice()[..])
                            .map_err(|_| InvalidTransaction::Payment)?;
                        if <multisig::MultiSigCreator<Runtime>>::contains_key(&ms) {
                            // make sure that the multisig is attached to an identity with valid CDD
                            if let Some(did) = Identity::get_identity(
                                &AccountKey::try_from(ms.encode())
                                    .map_err(|_| InvalidTransaction::Payment)?,
                            ) {
                                return check_cdd(&did);
                            } else {
                                return Err(InvalidTransaction::Custom(
                                    TransactionError::MissingIdentity as u8,
                                )
                                .into());
                            }
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
    Err(InvalidTransaction::Custom(TransactionError::InvalidAuthorization as u8).into())
}

/// Returns signatory to charge fee if cdd is valid.
fn check_cdd(did: &IdentityId) -> Result<Option<Signatory>, InvalidTransaction> {
    if Identity::has_valid_cdd(*did) {
        Context::set_current_identity::<Identity>(Some(*did));
        return Ok(Some(Signatory::from(*did)));
    } else {
        sp_runtime::print("ERROR: This transaction requires an Identity");
        Err(InvalidTransaction::Custom(TransactionError::CddRequired as u8).into())
    }
}
