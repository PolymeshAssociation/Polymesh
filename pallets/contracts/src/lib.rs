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

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult, DispatchResultWithPostInfo},
    ensure,
    traits::Get
};
use frame_system::{self as system, ensure_signed};
use pallet_contracts::{BalanceOf, CodeHash, ContractAddressFor, Gas, Schedule};
use pallet_identity as identity;
use polymesh_common_utilities::{
    identity::Trait as IdentityTrait,
    protocol_fee::{ChargeProtocolFee, ProtocolOp},
    Context,
};
use polymesh_primitives::{IdentityId, SmartExtensionMetadata, TemplateMetadata};
use sp_core::crypto::UncheckedFrom;
use sp_runtime::{
    traits::{Hash, Saturating, StaticLookup},
    Perbill, SaturatedConversion,
};
use polymesh_common_utilities::{ with_transaction };
use sp_std::{marker::PhantomData, prelude::*};

type Identity<T> = identity::Module<T>;

/// Nonce based contract address determiner.
///
/// Address calculated from the code (of the constructor), input data to the constructor,
/// the account id that requested the account creation and the latest nonce of the account id.
///
/// Formula: `blake2_256(blake2_256(code) + blake2_256(data) + origin + blake2_256(nonce))`
pub struct NonceBasedAddressDeterminer<T: Trait>(PhantomData<T>);
impl<T: Trait> ContractAddressFor<CodeHash<T>, T::AccountId> for NonceBasedAddressDeterminer<T>
where
    T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
{
    fn contract_address_for(
        code_hash: &CodeHash<T>,
        data: &[u8],
        origin: &T::AccountId,
    ) -> T::AccountId {
        let data_hash = T::Hashing::hash(data);
        let nonce = <frame_system::Module<T>>::account(origin).nonce;
        let nonce_hash = T::Hashing::hash(&(nonce.encode()));
        let mut buf = Vec::new();
        buf.extend_from_slice(code_hash.as_ref());
        buf.extend_from_slice(data_hash.as_ref());
        buf.extend_from_slice(origin.as_ref());
        buf.extend_from_slice(nonce_hash.as_ref());

        UncheckedFrom::unchecked_from(T::Hashing::hash(&buf[..]))
    }
}

pub trait Trait: pallet_contracts::Trait + IdentityTrait {
    /// Event type
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    /// Percentage distribution of instantiation fee to the validators and treasury.
    type NetworkShareInFee: Get<Perbill>;
}

decl_storage! {
    trait Store for Module<T: Trait> as ContractsWrapper {
        /// Store the meta details of the smart extension template.
        pub TemplateMetaDetails get(fn get_template_meta_details): map hasher(twox_64_concat) CodeHash<T> => TemplateMetadata<BalanceOf<T>, T::AccountId>;
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
        /// When instantiation of the template is already frozen.
        InstantiationAlreadyFrozen,
        /// When instantiation of the template is already un-frozen.
        InstantiationAlreadyUnFrozen,
        /// When un-authorized personnel try to access the un-authorized extrinsic.
        UnAuthorizedOrigin,
        /// User is not able to pay the protocol fee because of insufficient funds or because of something else.
        FailedToPayProtocolFee,
        /// Failed To charge the instantiation fee for the smart extension.
        FailedToPayInstantiationFee
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
        /// Emitted when the instantiation of the template get frozen.
        /// IdentityId of the owner, Code hash of the template.
        InstantiationFreezed(IdentityId, CodeHash),
        /// Emitted when the instantiation of the template gets un-frozen.
        /// IdentityId of the owner, Code hash of the template.
        InstantiationUnFreezed(IdentityId, CodeHash),
    }
}

decl_module! {
    // Wrap dispatchable functions for contracts so that we can add additional gating logic.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        /// Initialize the default event for this module.
        fn deposit_event() = default;

        /// Error type.
        type Error = Error<T>;

        /// The minimum amount required to generate a tombstone.
        const NetworkShareInInstantiationFee: Perbill = T::NetworkShareInFee::get();

        // Simply forwards to the `update_schedule` function in the Contract module.
        #[weight = pallet_contracts::Call::<T>::update_schedule(schedule.clone()).get_dispatch_info().weight]
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

            // Rollback the `put_code()` if user is not able to pay the protocol-fee.
            with_transaction(|| {
                // Call underlying function
                <pallet_contracts::Module<T>>::put_code(origin, code)?;
                // Update the storage.
                <TemplateMetaDetails<T>>::insert(code_hash, TemplateMetadata {
                    meta_info: meta_info,
                    owner: sender,
                    frozen: false
                });
                // Charge the protocol fee
                T::ProtocolFee::charge_fee(ProtocolOp::ContractsPutCode)
            })?;
            Ok(())
        }

        // Simply forwards to the `call` function in the Contract module.
        #[weight = pallet_contracts::Call::<T>::call(dest.clone(), *value, *gas_limit, data.clone()).get_dispatch_info().weight]
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
        ///
        /// # Errors
        /// InstantiationIsNotAllowed - It occurred when instantiation of the template is frozen.
        #[weight = 500_000_000 + *gas_limit]
        pub fn instantiate(
            origin,
            #[compact] endowment: BalanceOf<T>,
            #[compact] gas_limit: Gas,
            code_hash: CodeHash<T>,
            data: Vec<u8>
        ) -> DispatchResultWithPostInfo {
            ensure_signed(origin.clone())?;
            // Access the meta details of SE template
            let meta_details = Self::get_template_meta_details(code_hash);

            // Check whether instantiation is allowed or not.
            ensure!(!meta_details.is_instantiation_frozen(), Error::<T>::InstantiationIsNotAllowed);

            let mut actual_weight = None;

            with_transaction(|| {
                // transmit the call to the base `pallet-contracts` module.
                actual_weight = <pallet_contracts::Module<T>>::instantiate(origin, endowment, gas_limit, code_hash, data).map(|post_info| post_info.actual_weight).map_err(|err| err.error)?;
                // Charge instantiation fee
                T::ProtocolFee::charge_extension_instantiation_fee((meta_details.get_instantiation_fee().saturated_into::<u128>()).into(), meta_details.owner, T::NetworkShareInFee::get())
            })?;

            // Update the actual weight of the extrinsic.
            Ok(actual_weight.map(|w| w + 500_000_000).into())
        }

        /// Change the instantiation fee of the smart extension template
        ///
        /// # Arguments
        /// * origin - Only owner of template is allowed to execute the dispatchable.
        /// * code_hash - Unique hash of the smart extension template.
        /// * new_instantiation_fee - New value of instantiation fee to the smart extension template.
        #[weight = 1000_000_000]
        pub fn change_instantiation_fee(origin, code_hash: CodeHash<T>, new_instantiation_fee: BalanceOf<T>) -> DispatchResult {
            // Ensure whether the extrinsic is signed & validate the `code_hash`.
            let (did, meta_details) = Self::ensure_signed_and_template_exists(origin, code_hash)?;
            // Emit event with the old fee and the new instantiation fee.
            Self::deposit_event(RawEvent::InstantiationFeeChanged(did, code_hash, meta_details.meta_info.instantiation_fee, new_instantiation_fee));

            // Update the instantiation fee for a given code_hash.
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
            // Ensure whether the extrinsic is signed & validate the `code_hash`.
            let (did, meta_details) = Self::ensure_signed_and_template_exists(origin, code_hash)?;

            // If instantiation is already frozen then there is no point of changing the storage value.
            ensure!(!meta_details.is_instantiation_frozen(), Error::<T>::InstantiationAlreadyFrozen);
            // Change the `frozen` variable to `true`.
            <TemplateMetaDetails<T>>::mutate(&code_hash, |meta_details| meta_details.frozen = true);

            // Emit event.
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
            // Ensure whether the extrinsic is signed & validate the `code_hash`.
            let (did, meta_details) = Self::ensure_signed_and_template_exists(origin, code_hash)?;

            // If instantiation is already un-frozen then there is no point of changing the storage value.
            ensure!(meta_details.is_instantiation_frozen(), Error::<T>::InstantiationAlreadyUnFrozen);
            // Change the `frozen` variable to `false`.
            <TemplateMetaDetails<T>>::mutate(&code_hash, |meta_details| meta_details.frozen = false);

            // Emit event.
            Self::deposit_event(RawEvent::InstantiationUnFreezed(did, code_hash));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    // Internal function
    // Perform some basic sanity checks
    fn ensure_signed_and_template_exists(
        origin: T::Origin,
        code_hash: CodeHash<T>,
    ) -> Result<(IdentityId, TemplateMetadata<BalanceOf<T>, T::AccountId>), DispatchError> {
        // Ensure the transaction is signed.
        let sender = ensure_signed(origin.clone())?;
        // Get the DID of the sender.
        let did = Context::current_identity_or::<Identity<T>>(&sender)?;
        // Validate whether the template exists or not for a given code_hash.
        ensure!(
            <TemplateMetaDetails<T>>::contains_key(code_hash),
            Error::<T>::TemplateNotExists
        );

        // Access the meta details of SE template
        let meta_details = Self::get_template_meta_details(code_hash);
        // Ensure sender is the owner of the template.
        ensure!(
            sender == meta_details.owner.clone(),
            Error::<T>::UnAuthorizedOrigin
        );
        // Return the DID
        Ok((did, meta_details))
    }
}
