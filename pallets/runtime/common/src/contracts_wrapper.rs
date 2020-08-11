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

//! # Contracts Wrapper Module
//!
//! The Contracts Wrapper module wraps Contracts, allowing for DID integration and permissioning
//!
//! ## To Do
//!
//!   - Remove the ability to call the Contracts module, bypassing Contracts Wrapper
//!   - Integrate DID into all calls, and validate secondary_key
//!   - Track ownership of code and instances via DIDs
//!
//! ## Possible Tokenomics
//!
//!   - Initially restrict list of accounts that can put_code
//!   - When code is instantiated enforce a POLYX fee to the DID owning the code (i.e. that executed put_code)

use frame_support::{
    decl_error, decl_module, decl_storage,
    dispatch::{DispatchResult, DispatchResultWithPostInfo},
    ensure,
};
use frame_system::ensure_signed;
use pallet_contracts::{BalanceOf, CodeHash, Gas, Schedule};
use pallet_identity as identity;
use polymesh_common_utilities::{identity::Trait as IdentityTrait, Context};
use polymesh_primitives::{IdentityId, Signatory};
use sp_runtime::traits::StaticLookup;
use sp_std::convert::TryFrom;
use sp_std::prelude::Vec;

pub trait Trait: pallet_contracts::Trait + IdentityTrait {}

decl_storage! {
    trait Store for Module<T: Trait> as ContractsWrapper {
        pub CodeHashDid: map hasher(twox_64_concat) CodeHash<T> => Option<IdentityId>;
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// The sender must be a secondary key for the DID.
        SenderMustBeSecondaryKeyForDid,
    }
}

type Identity<T> = identity::Module<T>;

decl_module! {
    // Wrap dispatchable functions for contracts so that we can add additional gating logic
    // TODO: Figure out how to remove dispatchable calls from the underlying contracts module
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        // Simply forwards to the `update_schedule` function in the Contract module.
        #[weight = 500_000]
        pub fn update_schedule(origin, schedule: Schedule) -> DispatchResult {
            <pallet_contracts::Module<T>>::update_schedule(origin, schedule)
        }

        // Simply forwards to the `put_code` function in the Contract module.
        #[weight = 500_000 + 10_000 * u64::try_from(code.len()).unwrap_or_default()]
        pub fn put_code(
            origin,
            code: Vec<u8>
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            let signer = Signatory::Account(sender.clone());

            // Check that sender is allowed to act on behalf of `did`
            ensure!(
                <identity::Module<T>>::is_signer_authorized(did, &signer),
                Error::<T>::SenderMustBeSecondaryKeyForDid
            );

            // Call underlying function
            let new_origin = frame_system::RawOrigin::Signed(sender).into();
            <pallet_contracts::Module<T>>::put_code(new_origin, code)
        }

        // Simply forwards to the `call` function in the Contract module.
        #[weight = 700_000]
        pub fn call(
            origin,
            dest: <T::Lookup as StaticLookup>::Source,
            #[compact] value: BalanceOf<T>,
            #[compact] gas_limit: Gas,
            data: Vec<u8>
        ) -> DispatchResultWithPostInfo {
            <pallet_contracts::Module<T>>::call(origin, dest, value, gas_limit, data)
        }

        // Simply forwards to the `instantiate` function in the Contract module.
        #[weight = 500_000 + 10_000 * u64::try_from(data.len()).unwrap_or_default()]
        pub fn instantiate(
            origin,
            #[compact] endowment: BalanceOf<T>,
            #[compact] gas_limit: Gas,
            code_hash: CodeHash<T>,
            data: Vec<u8>
        ) -> DispatchResultWithPostInfo {
            <pallet_contracts::Module<T>>::instantiate(origin, endowment, gas_limit, code_hash, data)
        }
    }
}

impl<T: Trait> Module<T> {}
