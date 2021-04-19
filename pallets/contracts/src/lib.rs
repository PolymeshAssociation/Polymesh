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

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use core::mem;
use frame_support::{
    decl_error, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult, DispatchResultWithPostInfo},
    ensure,
    traits::Get,
    weights::DispatchClass::Operational,
};
use frame_system::ensure_root;
use pallet_base::{ensure_opt_string_limited, ensure_string_limited};
use pallet_contracts::{weights::WeightInfo as _, BalanceOf, CodeHash, Schedule};
use pallet_identity as identity;
pub use polymesh_common_utilities::traits::contracts::{Event, Trait, WeightInfo};
use polymesh_common_utilities::{
    protocol_fee::{ChargeProtocolFee, ProtocolOp},
    traits::contracts::{RawEvent, ContractsFn},
    with_transaction,
};
use polymesh_primitives::{
    ExtensionAttributes, Gas, IdentityId, MetaUrl, SmartExtensionType, TemplateDetails,
    TemplateMetadata,
};
use sp_core::crypto::UncheckedFrom;
use sp_runtime::{
    traits::{Hash, StaticLookup},
    Perbill, SaturatedConversion,
};
use sp_std::prelude::*;

type Identity<T> = identity::Module<T>;
type Contracts<T> = pallet_contracts::Module<T>;

const INSTANTIATE_WITH_CODE_EXTRA: u64 = 50_000_000;
const INSTANTIATE_EXTRA: u64 = 500_000_000;

decl_storage! {
    trait Store for Module<T: Trait> as ContractsWrapper
    where
       T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>
    {
        /// Store the meta details of the smart extension template.
        pub MetadataOfTemplate get(fn get_metadata_of): map hasher(identity) CodeHash<T> => TemplateMetadata<BalanceOf<T>>;
        /// Store the details of the template (Ex- owner, frozen etc).
        pub TemplateInfo get(fn get_template_details): map hasher(identity) CodeHash<T> => TemplateDetails<BalanceOf<T>>;
        /// Details of extension get updated.
        pub ExtensionInfo get(fn extension_info): map hasher(identity) T::AccountId => ExtensionAttributes<BalanceOf<T>>;
        /// Nonce for the smart extension account id generation.
        /// Using explicit nonce as in batch transaction accounts nonce doesn't get incremented.
        pub ExtensionNonce get(fn extension_nonce): u64;
        /// Store if `put_code` extrinsic is enabled or disabled.
        pub EnablePutCode get(fn is_put_code_enabled) config(enable_put_code): bool;
    }
}

decl_error! {
    pub enum Error for Module<T: Trait>
    where
        T::AccountId: UncheckedFrom<T::Hash>,
        T::AccountId: AsRef<[u8]>,
    {
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
        /// Given identityId is not CDD.
        NewOwnerIsNotCDD,
        /// Insufficient max_fee provided by the user to instantiate the SE.
        InsufficientMaxFee,
        /// `put_code` extrinsic is disabled. See `set_put_code_flag` extrinsic.
        PutCodeIsNotAllowed,
    }
}

decl_module! {
    // Wrap dispatchable functions for contracts so that we can add additional gating logic.
    pub struct Module<T: Trait> for enum Call
    where
        origin: T::Origin,
        T::AccountId: UncheckedFrom<T::Hash>,
        T::AccountId: AsRef<[u8]>,
    {

        /// Initialize the default event for this module.
        fn deposit_event() = default;

        /// Error type.
        type Error = Error<T>;

        /// The minimum amount required to generate a tombstone.
        const NetworkShareInInstantiationFee: Perbill = T::NetworkShareInFee::get();

        // Simply forwards to the `update_schedule` function in the Contract module.
        #[weight = <T as pallet_contracts::Config>::WeightInfo::update_schedule()]
        pub fn update_schedule(origin, schedule: Schedule<T>) -> DispatchResult {
            Contracts::<T>::update_schedule(origin, schedule)
        }

        /// Enable or disable the extrinsic `put_code` in this module.
        ///
        /// ## Arguments
        /// - `origin` which must be root.
        /// - `is_enabled` is the new value for this flag.
        ///
        /// ## Errors
        /// - `BadOrigin` if caller is not root.
        ///
        /// ## Permissions
        /// None
        #[weight = (<T as Trait>::WeightInfo::set_put_code_flag(), Operational)]
        pub fn set_put_code_flag(origin, is_enabled: bool) -> DispatchResult {
            Self::base_set_put_code_flag(origin, is_enabled)
        }

        /// Simply forwards to the `put_code` function in the Contract module.
        ///
        /// # Additional functionality
        /// 1. Allow origin to pass some meta-details related to template code.
        /// 2. Charge protocol fee for deploying the template.
        ///
        /// # Errors
        /// - `PutCodeIsNotAllowed` if the `put_code` flag is false. See `set_put_code_flag()`.
        /// - `frame_system::BadOrigin` if `origin` is not signed.
        /// - `pallet_permission::Error::<T>::UnAutorizedCaller` if `origin` does not have a valid
        /// IdentityId.
        /// - `TooLong` if the strings embedded in `meta_info` are too long.
        /// - `pallet_contrats::Error::<T>::CodeTooLarge` if `code` length is grater than the chain
        /// setting for `pallet_contrats::max_code_size`.
        /// - Before `code` is inserted, some checks are performed on it, and them could raise up
        /// some errors. Please see `pallet_contracts::wasm::prepare_contract` for details.
        #[weight =
        <T as pallet_contracts::Config>::WeightInfo::instantiate_with_code(
            code.len() as u32 / 1024,
            salt.len() as u32 / 1024,
            )
            .saturating_add(*gas_limit)
            .saturating_add(INSTANTIATE_WITH_CODE_EXTRA)
        ]
        pub fn instantiate_with_code(
            origin,
            #[compact] endowment: BalanceOf<T>,
            #[compact] gas_limit: Gas,
            code: Vec<u8>,
            data: Vec<u8>,
            salt: Vec<u8>,
            meta_info: TemplateMetadata<BalanceOf<T>>,
            instantiation_fee: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            ensure!(Self::is_put_code_enabled(), Error::<T>::PutCodeIsNotAllowed);
            let did = Identity::<T>::ensure_perms(origin.clone())?;

            // Ensure strings are limited in length.
            ensure_string_limited::<T>(&meta_info.description)?;
            ensure_opt_string_limited::<T>(meta_info.url.as_deref())?;
            if let SmartExtensionType::Custom(ty) = &meta_info.se_type {
                ensure_string_limited::<T>(ty)?;
            }

            // Save metadata related to the SE template
            // Generate the code_hash here as well because there is no way
            // to read it directly from the upstream `pallet-contracts` module.
            let code_hash = T::Hashing::hash(&code);

            // Rollback the `put_code()` if user is not able to pay the protocol-fee.
            let post_info = with_transaction(|| -> DispatchResultWithPostInfo {
                // Charge the protocol fee
                T::ProtocolFee::charge_fee(ProtocolOp::ContractsPutCode)?;

                // Call underlying function
                Contracts::<T>::instantiate_with_code(origin, endowment, gas_limit, code, data, salt)
            })?;

            // Update the storage.
            <TemplateInfo<T>>::insert(code_hash, TemplateDetails {
                instantiation_fee,
                owner: did,
                frozen: false
            });
            <MetadataOfTemplate<T>>::insert(code_hash, meta_info);

            Ok(post_info)
        }

        // Simply forwards to the `call` function in the Contract module.
        #[weight = <T as pallet_contracts::Config>::WeightInfo::call().saturating_add(*gas_limit)]
        pub fn call(
            origin,
            dest: <T::Lookup as StaticLookup>::Source,
            #[compact] value: BalanceOf<T>,
            #[compact] gas_limit: Gas,
            data: Vec<u8>
        ) -> DispatchResultWithPostInfo {
            Contracts::<T>::call(origin, dest, value, gas_limit, data)
        }

        /// Simply forwards to the `instantiate` function in the Contract module.
        ///
        /// # Additional functionality
        /// 1. Check whether instantiation of given code_hash is allowed or not.
        /// 2. Charge instantiation fee.
        ///
        /// # Errors
        /// InstantiationIsNotAllowed - It occurred when instantiation of the template is frozen.
        /// InsufficientMaxFee - Provided max_fee is less than required.
        #[weight = <T as Trait>::WeightInfo::instantiate().saturating_add(*gas_limit)]
        pub fn instantiate(
            origin,
            #[compact] endowment: BalanceOf<T>,
            #[compact] gas_limit: Gas,
            code_hash: CodeHash<T>,
            data: Vec<u8>,
            salt: Vec<u8>,
            max_fee: BalanceOf<T>
        ) -> DispatchResultWithPostInfo {
            let sender = Identity::<T>::ensure_origin_call_permissions(origin.clone())?.sender;
            // Access the meta details of SE template
            let template_details = Self::get_template_details(code_hash);

            // Check whether instantiation is allowed or not.
            ensure!(!template_details.is_instantiation_frozen(), Error::<T>::InstantiationIsNotAllowed);

            // Check instantiation fee should be <= max_fee.
            let instantiation_fee = template_details.get_instantiation_fee();
            ensure!(instantiation_fee <= max_fee, Error::<T>::InsufficientMaxFee);

            let contract_address = Contracts::<T>::contract_address(&sender, &code_hash, &salt);
            let mut post_info = with_transaction(|| {
                // Update the extension nonce.
                ExtensionNonce::mutate(|n| *n = *n + 1u64);
                // Charge instantiation fee
                let fee = (instantiation_fee.saturated_into::<u128>()).into();
                let owner_pk = Self::get_primary_key(&template_details.owner);
                T::ProtocolFee::charge_extension_instantiation_fee(fee, owner_pk, T::NetworkShareInFee::get())?;

                // Generate the contract address. Generating here to avoid cloning of the vec.
                // transmit the call to the base `pallet-contracts` module.
                Contracts::<T>::instantiate(origin, endowment, gas_limit, code_hash, data, salt)
            })?;
            if let Some(ref mut actual_weight) = post_info.actual_weight {
                *actual_weight = actual_weight.saturating_add(INSTANTIATE_EXTRA);
            }

            // Update the usage fee for the extension instance.
            <ExtensionInfo<T>>::insert(contract_address, Self::ext_details(&code_hash));

            // Update the actual weight of the extrinsic.
            Ok(post_info)
        }

        /// Allows a smart extension template owner to freeze the instantiation.
        ///
        /// # Arguments
        /// * origin - Only owner of the template is allowed to execute the dispatchable.
        /// * code_hash - Unique hash of the smart extension template.
        #[weight = <T as Trait>::WeightInfo::freeze_instantiation()]
        pub fn freeze_instantiation(origin, code_hash: CodeHash<T>) -> DispatchResult {
            // Ensure whether the extrinsic is signed & validate the `code_hash`.
            let (did, template_details) = Self::ensure_signed_and_template_exists(origin, code_hash)?;

            // If instantiation is already frozen then there is no point of changing the storage value.
            ensure!(!template_details.is_instantiation_frozen(), Error::<T>::InstantiationAlreadyFrozen);
            // Change the `frozen` variable to `true`.
            <TemplateInfo<T>>::mutate(&code_hash, |template_details| template_details.frozen = true);

            // Emit event.
            Self::deposit_event(RawEvent::InstantiationFreezed(did, code_hash));
            Ok(())
        }

        /// Allows a smart extension template owner to un freeze the instantiation.
        ///
        /// # Arguments
        /// * origin - Only owner of the template is allowed to execute the dispatchable.
        /// * code_hash - Unique hash of the smart extension template.
        #[weight = <T as Trait>::WeightInfo::unfreeze_instantiation()]
        pub fn unfreeze_instantiation(origin, code_hash: CodeHash<T>) -> DispatchResult {
            // Ensure whether the extrinsic is signed & validate the `code_hash`.
            let (did, template_details) = Self::ensure_signed_and_template_exists(origin, code_hash)?;

            // If instantiation is already un-frozen then there is no point of changing the storage value.
            ensure!(template_details.is_instantiation_frozen(), Error::<T>::InstantiationAlreadyUnFrozen);
            // Change the `frozen` variable to `false`.
            <TemplateInfo<T>>::mutate(&code_hash, |template_details| template_details.frozen = false);

            // Emit event.
            Self::deposit_event(RawEvent::InstantiationUnFreezed(did, code_hash));
            Ok(())
        }

        /// Transfer ownership of the template, Can only be called by the owner of template.
        /// `new_owner` should posses the valid CDD claim.
        ///
        /// # Arguments
        /// * origin Owner of the provided code_hash.
        /// * code_hash Unique identifer of the template.
        /// * new_owner Identity that will be the new owner of the provided code_hash.
        #[weight = <T as Trait>::WeightInfo::transfer_template_ownership()]
        pub fn transfer_template_ownership(origin, code_hash: CodeHash<T>, new_owner: IdentityId) -> DispatchResult {
            // Ensure whether the extrinsic is signed & validate the `code_hash`.
            let (did, _) = Self::ensure_signed_and_template_exists(origin, code_hash)?;

            // Ensuring the `new_owner` is CDD or not.
            ensure!(Identity::<T>::has_valid_cdd(new_owner), Error::<T>::NewOwnerIsNotCDD);

            // Change the `owner` variable value to the given did.
            <TemplateInfo<T>>::mutate(&code_hash, |template_details| template_details.owner = new_owner);

            // Emit event.
            Self::deposit_event(RawEvent::TemplateOwnershipTransferred(did, code_hash, new_owner));
            Ok(())
        }

        /// Change the usage fee & the instantiation fee of the smart extension template
        ///
        /// # Arguments
        /// * origin - Only owner of template is allowed to execute the dispatchable.
        /// * code_hash - Unique hash of the smart extension template.
        /// * new_instantiation_fee - New value of instantiation fee for the smart extension template.
        /// * new_usage_fee - New value of usage fee for the smart extension template.
        #[weight = <T as Trait>::WeightInfo::change_template_fees()]
        pub fn change_template_fees(origin, code_hash: CodeHash<T>, new_instantiation_fee: Option<BalanceOf<T>>, new_usage_fee: Option<BalanceOf<T>>) -> DispatchResult {
            // Ensure whether the extrinsic is signed & validate the `code_hash`.
            let (did, _) = Self::ensure_signed_and_template_exists(origin, code_hash)?;

            // Update the fees
            if let Some(usage_fee) = new_usage_fee {
                // Update the usage fee for a given code hash.
                let old_usage_fee = <MetadataOfTemplate<T>>::mutate(&code_hash, |metadata| mem::replace(&mut metadata.usage_fee, usage_fee));
                // Emit event with the old & new usage fee.
                Self::deposit_event(RawEvent::TemplateUsageFeeChanged(did, code_hash, old_usage_fee, usage_fee));
            }
            if let Some(instantiation_fee) = new_instantiation_fee {
                // Update the instantiation fee for a given code_hash.
                let old_instantiation_fee = <TemplateInfo<T>>::mutate(&code_hash, |template_details| mem::replace(&mut template_details.instantiation_fee, instantiation_fee));
                // Emit event with the old & new instantiation fee.
                Self::deposit_event(RawEvent::TemplateInstantiationFeeChanged(did, code_hash, old_instantiation_fee, instantiation_fee));
            }
            Ok(())
        }

        /// Change the template meta url.
        ///
        /// # Arguments
        /// * origin - Only owner of template is allowed to execute the dispatchable.
        /// * code_hash - Unique hash of the smart extension template.
        /// * new_url - New meta url that need to replace with old url.
        #[weight = <T as Trait>::WeightInfo::change_template_meta_url(new_url.as_ref().map_or(0, |u| u.0.len()) as u32 )]
        pub fn change_template_meta_url(origin, code_hash: CodeHash<T>, new_url: Option<MetaUrl>) -> DispatchResult {
            // Ensure whether the extrinsic is signed & validate the `code_hash`.
            let (did, _) = Self::ensure_signed_and_template_exists(origin, code_hash)?;
            // Ensure URL is limited in length.
            ensure_opt_string_limited::<T>(new_url.as_deref())?;
            // Update the usage fee for a given code hash.
            let old_url = <MetadataOfTemplate<T>>::mutate(&code_hash, |metadata| mem::replace(&mut metadata.url, new_url.clone()));
            // Emit event with old and new url.
            Self::deposit_event(RawEvent::TemplateMetaUrlChanged(did, code_hash, old_url, new_url));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T>
where
    T::AccountId: UncheckedFrom<T::Hash>,
    T::AccountId: AsRef<[u8]>,
{
    // Internal function
    // Perform some basic sanity checks
    fn ensure_signed_and_template_exists(
        origin: T::Origin,
        code_hash: CodeHash<T>,
    ) -> Result<(IdentityId, TemplateDetails<BalanceOf<T>>), DispatchError> {
        // Ensure the transaction is signed and ensure `origin` has the required permission to
        // execute the dispatchable.
        let did = Identity::<T>::ensure_perms(origin)?;
        // Validate whether the template exists or not for a given code_hash.
        ensure!(
            <TemplateInfo<T>>::contains_key(code_hash),
            Error::<T>::TemplateNotExists
        );

        // Access the meta details of SE template
        let template_details = Self::get_template_details(code_hash);
        // Ensure sender's DID is the owner of the template.
        ensure!(
            did == template_details.owner,
            Error::<T>::UnAuthorizedOrigin
        );
        // Return the DID and the template details.
        Ok((did, template_details))
    }

    fn get_primary_key(id: &IdentityId) -> T::AccountId {
        Identity::<T>::did_records(id).primary_key
    }

    fn ext_details(code_hash: &CodeHash<T>) -> ExtensionAttributes<BalanceOf<T>> {
        let meta_info = Self::get_metadata_of(code_hash);
        ExtensionAttributes {
            usage_fee: meta_info.usage_fee,
            version: meta_info.version,
        }
    }

    fn base_set_put_code_flag(origin: T::Origin, is_enabled: bool) -> DispatchResult {
        ensure_root(origin)?;
        EnablePutCode::put(is_enabled);
        Self::deposit_event(RawEvent::PutCodeFlagChanged(is_enabled));
        Ok(())
    }
}

impl<T: Trait> ContractsFn<T::AccountId, BalanceOf<T>> for Module<T>
where
    T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
{
    fn extension_info(acc: T::AccountId) -> ExtensionAttributes<BalanceOf<T>> {
        Self::extension_info(acc)
    }
}
