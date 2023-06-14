// This file is part of Substrate.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
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

// Modified by Polymath Inc - 2nd February 2021
//  - Priority of a transaction is always zero.

use codec::{Decode, Encode};
use frame_support::dispatch::{DispatchClass, DispatchInfo, PostDispatchInfo};
use frame_system::{CheckWeight as CW, Config};
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{DispatchInfoOf, Dispatchable, PostDispatchInfoOf, SignedExtension},
    transaction_validity::{InvalidTransaction, TransactionValidity, TransactionValidityError},
    DispatchResult,
};

/// Block resource (weight) limit check.
#[derive(Encode, Decode, Clone, Eq, PartialEq, Default, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct CheckWeight<T: Config + Send + Sync>(CW<T>);

impl<T: Config + Send + Sync> CheckWeight<T>
where
    T::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
{
    /// Creates new `SignedExtension` to check weight of the extrinsic.
    pub fn new() -> Self {
        Self(CW::new())
    }

    /// Do the validate checks. This can be applied to both signed and unsigned.
    ///
    /// It only checks that the block weight and length limit will not exceed.
    fn do_validate(info: &DispatchInfoOf<T::RuntimeCall>, len: usize) -> TransactionValidity {
        let tv = CW::<T>::do_validate(info, len)?;
        Ok(tv)
    }
}

impl<T: Config + Send + Sync> SignedExtension for CheckWeight<T>
where
    T::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
{
    type AccountId = T::AccountId;
    type Call = T::RuntimeCall;
    type AdditionalSigned = ();
    type Pre = ();
    const IDENTIFIER: &'static str = "CheckWeight";

    fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
        Ok(())
    }

    fn pre_dispatch(
        self,
        who: &Self::AccountId,
        call: &Self::Call,
        info: &DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> Result<(), TransactionValidityError> {
        self.0.pre_dispatch(who, call, info, len)
    }

    fn validate(
        &self,
        _who: &Self::AccountId,
        _call: &Self::Call,
        info: &DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> TransactionValidity {
        if info.class == DispatchClass::Mandatory {
            Err(InvalidTransaction::MandatoryValidation)?
        }
        Self::do_validate(info, len)
    }

    fn pre_dispatch_unsigned(
        call: &Self::Call,
        info: &DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> Result<(), TransactionValidityError> {
        CW::<T>::pre_dispatch_unsigned(call, info, len)
    }

    fn validate_unsigned(
        _call: &Self::Call,
        info: &DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> TransactionValidity {
        Self::do_validate(info, len)
    }

    fn post_dispatch(
        pre: Option<Self::Pre>,
        info: &DispatchInfoOf<Self::Call>,
        post_info: &PostDispatchInfoOf<Self::Call>,
        len: usize,
        result: &DispatchResult,
    ) -> Result<(), TransactionValidityError> {
        CW::<T>::post_dispatch(pre, info, post_info, len, result)
    }
}

impl<T: Config + Send + Sync> sp_std::fmt::Debug for CheckWeight<T> {
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(f, "CheckWeight")
    }

    #[cfg(not(feature = "std"))]
    fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{new_test_ext, Test, CALL};
    use frame_support::weights::Pays;

    #[test]
    fn signed_ext_check_weight_works() {
        new_test_ext().execute_with(|| {
            let normal = DispatchInfo {
                weight: 100,
                class: DispatchClass::Normal,
                pays_fee: Pays::Yes,
            };
            let op = DispatchInfo {
                weight: 100,
                class: DispatchClass::Operational,
                pays_fee: Pays::Yes,
            };
            let len = 0_usize;

            let priority = CheckWeight::<Test>::new()
                .validate(&1, CALL, &normal, len)
                .unwrap()
                .priority;
            assert_eq!(priority, 0);

            let priority = CheckWeight::<Test>::new()
                .validate(&1, CALL, &op, len)
                .unwrap()
                .priority;
            assert_eq!(priority, 0);
        })
    }
}
