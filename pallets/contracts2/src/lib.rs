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

//! # Contracts Module
//!
//! TODO
//!
//! ## Overview
//!
//! The Contracts module provides functions for:
//!
//! TODO
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `todo` - TODO
//!
//! ### Public Functions
//!
//! - `todo` - TODO

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(associated_type_bounds)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchErrorWithPostInfo, DispatchResultWithPostInfo},
    weights::Weight,
};
use pallet_contracts_primitives::{Code, ContractResult};
use pallet_identity::PermissionedCallOriginData;
use polymesh_common_utilities::traits::identity::Config as IdentityConfig;
use polymesh_common_utilities::with_transaction;
use polymesh_primitives::{Balance, Permissions};
use sp_core::crypto::UncheckedFrom;
use sp_core::Bytes;
use sp_runtime::traits::Hash;

type Identity<T> = pallet_identity::Module<T>;
type BaseContracts<T> = pallet_contracts::Pallet<T>;
type CodeHash<T> = <T as frame_system::Config>::Hash;

pub trait WeightInfo {
    fn call() -> Weight;
    fn instantiate_with_code(code_len: u32, salt_len: u32) -> Weight;
    fn instantiate_with_hash(salt_len: u32) -> Weight;
}

/// The `Config` trait for the smart contracts pallet.
pub trait Config:
    IdentityConfig
    + pallet_contracts::Config<Currency = Self::Balances>
    + frame_system::Config<
        AccountId: AsRef<[u8]> + UncheckedFrom<<Self as frame_system::Config>::Hash>,
    >
{
    /// The overarching event type.
    type Event: From<Event> + Into<<Self as frame_system::Config>::Event>;

    /// The weight configuration for the pallet.
    type WeightInfo: WeightInfo;
}

decl_event! {
    pub enum Event {}
}

decl_error! {
    pub enum Error for Module<T: Config> {}
}

decl_storage! {
    trait Store for Module<T: Config> as Contracts {}
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;

        /// TODO
        #[weight = <T as Config>::WeightInfo::call().saturating_add(*gas_limit)]
        pub fn call(
            origin,
            contract: T::AccountId,
            value: Balance,
            gas_limit: Weight,
            data: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            Self::base_call(origin, contract, value, gas_limit, data)
        }

        /// TODO
        #[weight = <T as Config>::WeightInfo::instantiate_with_code(
            code.len() as u32,
            salt.len() as u32,
        )]
        pub fn instantiate_with_code(
            origin,
            endowment: Balance,
            gas_limit: Weight,
            code: Vec<u8>,
            data: Vec<u8>,
            salt: Vec<u8>,
            perms: Permissions
        ) -> DispatchResultWithPostInfo {
            Self::base_instantiate_with_code(origin, endowment, gas_limit, code, data, salt, perms)
        }

        /// TODO
        #[weight = <T as Config>::WeightInfo::instantiate_with_hash(salt.len() as u32)]
        pub fn instantiate_with_hash(
            origin,
            endowment: Balance,
            gas_limit: Weight,
            code_hash: CodeHash<T>,
            data: Vec<u8>,
            salt: Vec<u8>,
            perms: Permissions
        ) -> DispatchResultWithPostInfo {
            Self::base_instantiate_with_hash(origin, endowment, gas_limit, code_hash, data, salt, perms)
        }
    }
}

impl<T: Config> Module<T> {
    fn base_call(
        origin: T::Origin,
        contract: T::AccountId,
        value: Balance,
        gas_limit: Weight,
        data: Vec<u8>,
    ) -> DispatchResultWithPostInfo {
        // Ensure contract caller has perms.
        let sender = Identity::<T>::ensure_origin_call_permissions(origin)?.sender;

        // Execute contract.
        Self::handle_error(
            <T as Config>::WeightInfo::call(),
            BaseContracts::<T>::bare_call(sender, contract, value, gas_limit, data, false),
        )
    }

    /// Instantiates a contract using `code` as the WASM code blob.
    fn base_instantiate_with_code(
        origin: T::Origin,
        endowment: Balance,
        gas_limit: Weight,
        code: Vec<u8>,
        inst_data: Vec<u8>,
        salt: Vec<u8>,
        perms: Permissions,
    ) -> DispatchResultWithPostInfo {
        Self::general_instantiate(
            origin,
            endowment,
            // Compute the base weight of roughly `base_instantiate`.
            <T as Config>::WeightInfo::instantiate_with_code(
                (code.len() / 1024) as u32,
                (salt.len() / 1024) as u32,
            ),
            gas_limit,
            T::Hashing::hash(&code),
            Code::Upload(Bytes(code)),
            inst_data,
            salt,
            perms,
        )
    }

    /// Instantiates a contract using an existing WASM code blob with `code_hash` as its code.
    fn base_instantiate_with_hash(
        origin: T::Origin,
        endowment: Balance,
        gas_limit: Weight,
        code_hash: CodeHash<T>,
        inst_data: Vec<u8>,
        salt: Vec<u8>,
        perms: Permissions,
    ) -> DispatchResultWithPostInfo {
        Self::general_instantiate(
            origin,
            endowment,
            // Compute the base weight of roughly `base_instantiate`.
            <T as Config>::WeightInfo::instantiate_with_hash((salt.len() / 1024) as u32),
            gas_limit,
            code_hash,
            Code::Existing(code_hash),
            inst_data,
            salt,
            perms,
        )
    }

    /// General logic for contract instantiation both when the code or code hash is given.
    ///
    /// The interesting parameters here are:
    /// - `base_weight` assigns a base non-variable weight to the full extrinsic.
    /// - `code_hash` specifies the hash to use to derive the contract's key.
    /// - `code` specifies the code for the contract, either as a code blob or an existing hash.
    ///   The hash of `code` in either case is assumed to correspond to `code_hash`.
    fn general_instantiate(
        origin: T::Origin,
        endowment: Balance,
        base_weight: Weight,
        gas_limit: Weight,
        code_hash: CodeHash<T>,
        code: Code<CodeHash<T>>,
        inst_data: Vec<u8>,
        salt: Vec<u8>,
        perms: Permissions,
    ) -> DispatchResultWithPostInfo {
        // Ensure we have perms + we'll need sender & DID.
        let PermissionedCallOriginData {
            primary_did: did,
            sender,
            ..
        } = Identity::<T>::ensure_origin_call_permissions(origin)?;

        // Pre-compute what contract's key will be...
        let contract_key = BaseContracts::<T>::contract_address(&sender, &code_hash, &salt);

        // ...and ensure that key can be a secondary-key of DID...
        Identity::<T>::ensure_secondary_key_can_be_added(&did, &contract_key, &perms)?;

        with_transaction(|| {
            // Roll back `join_identity` if contract was not instantiated.
            // ...so that the CDD due to `endowment` passes.
            Identity::<T>::unsafe_join_identity(did, perms, contract_key);

            // Now we can finally instantiate the contract.
            Self::handle_error(
                base_weight,
                BaseContracts::<T>::bare_instantiate(
                    sender.clone(),
                    endowment,
                    gas_limit,
                    code,
                    inst_data,
                    salt,
                    false,
                ),
            )
        })
    }

    /// Enriches `result` of executing a smart contract with actual weight,
    /// accounting for the consumed gas.
    fn handle_error<A>(
        base_weight: Weight,
        result: ContractResult<Result<A, DispatchError>>,
    ) -> DispatchResultWithPostInfo {
        let post_info = Some(result.gas_consumed.saturating_add(base_weight)).into();
        match result.result {
            Ok(_) => Ok(post_info),
            Err(error) => Err(DispatchErrorWithPostInfo { post_info, error }),
        }
    }
}
