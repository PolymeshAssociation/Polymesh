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
//!   - Integrate DID into all calls, and validate signing_key
//!   - Track ownership of code and instances via DIDs
//!
//! ## Possible Tokenomics
//!
//!   - Initially restrict list of accounts that can put_code
//!   - When code is instantiated enforce a POLYX fee to the DID owning the code (i.e. that executed put_code)

use frame_support::{
    decl_error, decl_module, decl_storage, decl_event,
    dispatch::{DispatchResult, DispatchResultWithPostInfo},
    ensure,
    traits::Get,
};
use frame_system::{ self as system, ensure_signed };
use pallet_contracts::{BalanceOf, CodeHash, Gas, Schedule};
use pallet_identity as identity;
use polymesh_common_utilities::{
    identity::Trait as IdentityTrait,
    protocol_fee::{ChargeProtocolFee, ProtocolOp},
    Context, traits::CommonTrait
};
use polymesh_primitives::{
    IdentityId, Signatory, SmartExtensionMetadata, SmartExtensionType, TemplateMetaData,
};
use sp_runtime::{
    traits::{Hash, Saturating, StaticLookup},
    Perbill,
};
use sp_std::convert::TryFrom;
use sp_std::prelude::Vec;

type Identity<T> = identity::Module<T>;
pub trait Trait: pallet_contracts::Trait + IdentityTrait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    /// Percentage distribution of instantiation fee to the validators and treasury.
    type NetworkShareInFee: Get<Perbill>;
}

decl_storage! {
    trait Store for Module<T: Trait> as ContractsWrapper {
        /// Store the meta details of the smart extension template.
        pub TemplateMetaDetails get(fn get_template_meta_details): map hasher(identity) CodeHash<T> => TemplateMetaData<BalanceOf<T>, T::AccountId>;
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// The sender must be a signing key for the DID.
        SenderMustBeSigningKeyForDid,
        /// Instantiation is not allowed.
        InstantiationIsNotAllowed,
        /// Smart extension template not exist in the storage.
        TemplateNotExists,
        /// When instantiation of the template is already freezed.
        InstantiationAlreadyFreezed,
        /// When instantiation of the template is already un-freezed.
        InstantiationAlreadyUnFreezed
    }
}

decl_event! {
    pub enum Event<T>
        where
        Balance = BalanceOf<T>,
        CodeHash = <T as frame_system::Trait>::Hash,
    {   
        /// Emitted when instantiation fee of a template get changed.
        /// IdentityId of the owner, Code hash of the template, old instantiation fee, new instantiation fee. 
        InstantiationFeeChanged(IdentityId, CodeHash, Balance, Balance),
        /// Emitted when the instantiation of the template get freezed.
        /// IdentityId of the owner, Code hash of the template.
        InstantiationFreezed(IdentityId, CodeHash),
        /// Emitted when the instantiation of the template gets un-freezed.
        /// IdentityId of the owner, Code hash of the template.
        InstantiationUnFreezed(IdentityId, CodeHash),
    }
}

decl_module! {
    // Wrap dispatchable functions for contracts so that we can add additional gating logic
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        /// initialize the default event for this module.
        fn deposit_event() = default;

        /// Error type.
        type Error = Error<T>;

        /// The minimum amount required to generate a tombstone.
        const NetworkShareInInstantiationFee: Perbill = T::NetworkShareInFee::get();

        // Simply forwards to the `update_schedule` function in the Contract module.
        #[weight = 500_000]
        pub fn update_schedule(origin, schedule: Schedule) -> DispatchResult {
            <pallet_contracts::Module<T>>::update_schedule(origin, schedule)
        }

        /// Simply forwards to the `put_code` function in the Contract module.
        ///
        /// # Additional functionality
        /// 1. Allow origin to pass some meta-details related to template code.
        /// 2. Charge protocol fee for deploying the template.
        #[weight = 50_000_000.saturating_add(pallet_contracts::Call::<T>::put_code(code.clone()).get_dispatch_info().weight)]
        pub fn put_code(
            origin,
            meta_info: SmartExtensionMetadata<BalanceOf<T>>,
            code: Vec<u8>
        ) -> DispatchResult {
            let sender = ensure_signed(origin.clone())?;

            // Save metadata related to the SE template
            // Generate the code_hash here as well because there is no way
            // to read it directly from the upstream `pallet-contracts` module.
            let code_hash = T::Hashing::hash(&code);

            // Call underlying function
            <pallet_contracts::Module<T>>::put_code(origin, code)?;

            // Charge the protocol fee
            // TODO: Introduce the new fee function that will allow to distribute the
            // protocol fee to different participants instead of only treasury.
            T::ProtocolFee::charge_fee(ProtocolOp::ContractsPutCode)?;
            <TemplateMetaDetails<T>>::insert(code_hash, TemplateMetaData {
                meta_info: meta_info,
                owner: sender,
                is_freeze: false
            });
            Ok(())
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

        /// Simply forwards to the `instantiate` function in the Contract module.
        ///
        /// # Additional functionality
        /// 1. Check whether instantiation of given code_hash is allowed or not.
        /// 2. Charge instantiation fee.
        #[weight = 500_000_000 + *gas_limit]
        pub fn instantiate(
            origin,
            #[compact] endowment: BalanceOf<T>,
            #[compact] gas_limit: Gas,
            code_hash: CodeHash<T>,
            data: Vec<u8>
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_signed(origin.clone())?;
            // Access the meta details of SE template
            let meta_details = Self::get_template_meta_details(code_hash);
            // Check whether instantiation is allowed or not.
            ensure!(!meta_details.is_instantiation_freezed(), Error::<T>::InstantiationIsNotAllowed);
            <pallet_contracts::Module<T>>::instantiate(origin, endowment, gas_limit, code_hash, data)
                .map(|mut info| {
                    // Charge instantiation fee
                    //T::ProtocolFee::charge_extension_instantiation_fee(meta_details.get_instantiation_fee(), meta_details.owner, T::NetworkShareInFee::get());
                    if let Some(weight) = info.actual_weight {
                        info.actual_weight = Some(weight + 500_000_000);
                    }
                    info
                }).map_err(|mut err_info| {
                    if let Some(weight) = err_info.post_info.actual_weight {
                        err_info.post_info.actual_weight = Some(weight + 500_000_000);
                    }
                    err_info
                })
        }

        /// Change the instantiation fee of the smart extension template
        ///
        /// # Arguments
        /// * origin - Only owner of template is allowed to execute the dispatchable.
        /// * code_hash - Unique hash of the smart extension template.
        /// * new_instantiation_fee - New value of instantiation fee to the smart extension template.
        #[weight = 1000_000_000]
        pub fn change_instantiation_fee(origin, code_hash: CodeHash<T>, new_instantiation_fee: BalanceOf<T>) -> DispatchResult {
            let sender = ensure_signed(origin.clone())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            ensure!(<TemplateMetaDetails<T>>::contains_key(code_hash), Error::<T>::TemplateNotExists);
            Self::deposit_event(RawEvent::InstantiationFeeChanged(did, code_hash, Self::get_template_meta_details(code_hash).meta_info.instantiation_fee, new_instantiation_fee));
            <TemplateMetaDetails<T>>::mutate(&code_hash, |meta_details| meta_details.meta_info.instantiation_fee = new_instantiation_fee);
            Ok(())
        }

        /// Allows a smart extension template owner to freeze the instantiation.
        ///
        /// # Arguments
        /// * origin - Only owner of the template is allowed to execute the dispatchable.
        /// * code_hash - Unique hash of the smart extension template.
        #[weight = 1_000_000_000]
        pub fn freeze_instantiation(origin, code_hash: CodeHash<T>) -> DispatchResult {
            let sender = ensure_signed(origin.clone())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            ensure!(<TemplateMetaDetails<T>>::contains_key(code_hash), Error::<T>::TemplateNotExists);
             // Access the meta details of SE template
            let meta_details = Self::get_template_meta_details(code_hash);
            ensure!(!meta_details.is_instantiation_freezed(), Error::<T>::InstantiationAlreadyFreezed);
            <TemplateMetaDetails<T>>::mutate(&code_hash, |meta_details| meta_details.is_freeze = true);
            Self::deposit_event(RawEvent::InstantiationFreezed(did, code_hash));
            Ok(())
        }

        /// Allows a smart extension template owner to un freeze the instantiation.
        ///
        /// # Arguments
        /// * origin - Only owner of the template is allowed to execute the dispatchable.
        /// * code_hash - Unique hash of the smart extension template.
        #[weight = 1_000_000_000]
        pub fn unfreeze_instantiation(origin, code_hash: CodeHash<T>) -> DispatchResult {
            let sender = ensure_signed(origin.clone())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            ensure!(<TemplateMetaDetails<T>>::contains_key(code_hash), Error::<T>::TemplateNotExists);
             // Access the meta details of SE template
            let meta_details = Self::get_template_meta_details(code_hash);
            ensure!(meta_details.is_instantiation_freezed(), Error::<T>::InstantiationAlreadyUnFreezed);
            <TemplateMetaDetails<T>>::mutate(&code_hash, |meta_details| meta_details.is_freeze = false);
            Self::deposit_event(RawEvent::InstantiationUnFreezed(did, code_hash));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {}
