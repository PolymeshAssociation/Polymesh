// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use codec::{Decode, DecodeWithMemTracking, Encode};
use frame_support::dispatch::DispatchInfo;
use frame_support::ensure;
use frame_support::pallet_prelude::Weight;
use scale_info::TypeInfo;
use sp_runtime::traits::{AsSystemOriginSigner, DispatchInfoOf};
use sp_runtime::traits::{Dispatchable, TransactionExtension};
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionValidityError};
use sp_runtime::transaction_validity::{TransactionPriority, TransactionSource, ValidTransaction};
use sp_std::{fmt, marker::PhantomData};

use crate::{Config, Key};

/// Ensure that signed transactions are only valid if they are signed by sudo account.
///
/// In the initial phase of a chain without any tokens you can not prevent accounts from sending
/// transactions.
/// These transactions would enter the transaction pool as the succeed the validation, but would
/// fail on applying them as they are not allowed/disabled/whatever. This would be some huge dos
/// vector to any kind of chain. This extension solves the dos vector by preventing any kind of
/// transaction entering the pool as long as it is not signed by the sudo account.
#[derive(Clone, Eq, PartialEq, Encode, Decode, DecodeWithMemTracking, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct CheckOnlySudoAccount<T: Config + Send + Sync>(PhantomData<T>);

impl<T: Config + Send + Sync> Default for CheckOnlySudoAccount<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Config + Send + Sync> fmt::Debug for CheckOnlySudoAccount<T> {
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CheckOnlySudoAccount")
    }

    #[cfg(not(feature = "std"))]
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

impl<T: Config + Send + Sync> CheckOnlySudoAccount<T> {
    /// Creates new `SignedExtension` to check sudo key.
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T: Config + Send + Sync> TransactionExtension<<T as Config>::RuntimeCall>
    for CheckOnlySudoAccount<T>
where
    <T as Config>::RuntimeCall: Dispatchable<Info = DispatchInfo>,
    <<T as Config>::RuntimeCall as Dispatchable>::RuntimeOrigin: AsSystemOriginSigner<T::AccountId>,
{
    const IDENTIFIER: &'static str = "CheckOnlySudoAccount";
    type Implicit = ();
    type Val = ();
    type Pre = ();

    fn weight(&self, _: &<T as Config>::RuntimeCall) -> Weight {
        Weight::zero()
    }

    fn validate(
        &self,
        origin: <<T as Config>::RuntimeCall as Dispatchable>::RuntimeOrigin,
        _call: &<T as Config>::RuntimeCall,
        info: &DispatchInfoOf<<T as Config>::RuntimeCall>,
        _len: usize,
        _: (),
        _implication: &impl Encode,
        _source: TransactionSource,
    ) -> Result<
        (
            ValidTransaction,
            Self::Val,
            <<T as Config>::RuntimeCall as Dispatchable>::RuntimeOrigin,
        ),
        TransactionValidityError,
    > {
        let sudo_key: T::AccountId = Key::<T>::get().ok_or(InvalidTransaction::Custom(0))?;

        let caller_acc = origin
            .as_system_origin_signer()
            .ok_or(InvalidTransaction::BadSigner)?;

        ensure!(
            sudo_key == *caller_acc,
            TransactionValidityError::Invalid(InvalidTransaction::BadSigner)
        );

        let valid_transaction = ValidTransaction {
            priority: info.call_weight.ref_time() as TransactionPriority,
            ..Default::default()
        };

        Ok((valid_transaction, (), origin))
    }

    fn prepare(
        self,
        _val: Self::Val,
        origin: &<<T as Config>::RuntimeCall as Dispatchable>::RuntimeOrigin,
        _call: &<T as Config>::RuntimeCall,
        _info: &DispatchInfoOf<<T as Config>::RuntimeCall>,
        _len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        let sudo_key: T::AccountId = Key::<T>::get().ok_or(InvalidTransaction::Custom(0))?;

        let caller_acc = origin
            .as_system_origin_signer()
            .ok_or(InvalidTransaction::BadSigner)?;

        ensure!(
            sudo_key == *caller_acc,
            TransactionValidityError::Invalid(InvalidTransaction::BadSigner)
        );

        Ok(())
    }
}
