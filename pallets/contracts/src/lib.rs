// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymesh Association

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
//! The contracts module provides the Layer 2 solution for Polymesh.
//! These smart contracts are defined using WASM,
//! and facilitated by the `ink!` DSL provided by Parity.
//!
//! With this module, you or Alice can instantiate such a contract,
//! through `instantiate_with_code` or `instantiate_with_hash`,
//! attaching its key as a secondary key of the signer's identity.
//! Anyone can then call this smart contract, e.g., Bob,
//! which may call back into the runtime to e.g., `create_asset`.
//! However, during the execution of `create_asset`,
//! the current identity will be Alice, as opposed to `Bob`.
//!
//! Transaction fees for calling into the runtime are charged to the gas meter
//! of the contract.  Protocol fees are charged to the contract's address, the contract
//! can require the caller to pay these fees by making the call payable and require
//! enough POLYX is transferred to cover the protocol fees.
//!
//! ## Overview
//!
//! The PolymeshContracts module provides functions for:
//!
//! - Instantiating contracts with custom permissions.
//!
//! Use the standard Substrate `pallet_contracts` to instantiate, call, upload_code, or remove_code.
//! That interface is compatible with the Polkadot.js Contracts UI.
//!
//! ### Dispatchable Functions
//!
//! - `instantiate_with_code_perms` instantiates a contract with the code provided.
//!   The contract's address will be added as a secondary key with the provided permissions.
//! - `instantiate_with_hash_perms` instantiates a contract by hash,
//!   assuming that a contract with the same code already was uploaded.
//!   The contract's address will be added as a secondary key with the provided permissions.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
pub mod chain_extension;

use codec::{Compact, Decode, Encode};
use frame_support::dispatch::{
    DispatchErrorWithPostInfo, DispatchResult, DispatchResultWithPostInfo,
};
use frame_support::pallet_prelude::MaxEncodedLen;
use frame_support::traits::Get;
use frame_support::weights::Weight;
use frame_support::{decl_error, decl_event, decl_module, decl_storage, ensure};
use frame_system::ensure_root;
use frame_system::ensure_signed;
#[cfg(feature = "std")]
use pallet_contracts::Determinism;
use scale_info::TypeInfo;
use sp_core::crypto::UncheckedFrom;
use sp_runtime::traits::Hash;
use sp_std::{vec, vec::Vec};

pub use chain_extension::{ExtrinsicId, PolymeshExtension};
use pallet_contracts::weights::WeightInfo as FrameWeightInfo;
use pallet_contracts::Config as BConfig;
use pallet_contracts_primitives::Code;
use pallet_identity::ParentDid;
use polymesh_common_utilities::asset::AssetFnTrait;
use polymesh_common_utilities::traits::identity::{
    Config as IdentityConfig, WeightInfo as IdentityWeightInfo,
};
use polymesh_primitives::{storage_migrate_on, storage_migration_ver, Balance, Permissions};

type Identity<T> = pallet_identity::Module<T>;
type IdentityError<T> = pallet_identity::Error<T>;
type FrameContracts<T> = pallet_contracts::Pallet<T>;
type CodeHash<T> = <T as frame_system::Config>::Hash;

pub struct ContractPolymeshHooks;

#[derive(Clone, Debug, Decode, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct Api {
    desc: [u8; 4],
    major: u32,
}

impl Api {
    pub fn new(desc: [u8; 4], major: u32) -> Self {
        Self { desc, major }
    }
}

#[derive(Clone, Decode, Encode, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct ApiCodeHash<T: Config> {
    pub hash: CodeHash<T>,
}

impl<T: Config> Default for ApiCodeHash<T> {
    fn default() -> Self {
        Self {
            hash: CodeHash::<T>::default(),
        }
    }
}

impl<T: Config> ApiCodeHash<T> {
    pub fn new(hash: CodeHash<T>) -> Self {
        ApiCodeHash { hash }
    }
}

impl<T: Config> sp_std::fmt::Debug for ApiCodeHash<T> {
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(f, "hash: {:?}", self.hash)
    }
}

#[derive(Clone, Debug, Decode, Encode, Eq, Ord, PartialOrd, PartialEq, TypeInfo)]
pub struct ChainVersion {
    spec_version: u32,
    tx_version: u32,
}

impl ChainVersion {
    pub fn new(spec_version: u32, tx_version: u32) -> Self {
        ChainVersion {
            spec_version,
            tx_version,
        }
    }
}

#[derive(Clone, Decode, Encode, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct NextUpgrade<T: Config> {
    pub chain_version: ChainVersion,
    pub api_hash: ApiCodeHash<T>,
}

impl<T: Config> NextUpgrade<T> {
    pub fn new(chain_version: ChainVersion, api_hash: ApiCodeHash<T>) -> Self {
        Self {
            chain_version,
            api_hash,
        }
    }
}

impl<T: Config> sp_std::fmt::Debug for NextUpgrade<T> {
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(
            f,
            "chain_version: {:?} api_hash: {:?}",
            self.chain_version, self.api_hash
        )
    }
}

impl<T: Config> pallet_contracts::PolymeshHooks<T> for ContractPolymeshHooks
where
    T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
{
    fn check_call_permissions(caller: &T::AccountId) -> DispatchResult {
        pallet_permissions::Module::<T>::ensure_call_permissions(caller)?;
        Ok(())
    }

    fn on_instantiate_transfer(caller: &T::AccountId, contract: &T::AccountId) -> DispatchResult {
        // Get the caller's identity.
        let did =
            Identity::<T>::get_identity(&caller).ok_or(Error::<T>::InstantiatorWithNoIdentity)?;
        // Check if contact is already linked.
        match Identity::<T>::get_identity(&contract) {
            Some(contract_did) => {
                if contract_did != did && ParentDid::get(contract_did) != Some(did) {
                    // Contract address already linked to a different identity.
                    Err(IdentityError::<T>::AlreadyLinked.into())
                } else {
                    // Contract is already linked to caller's identity.
                    Ok(())
                }
            }
            None => {
                // Linked new contract address to caller's identity.  With empty permissions.
                Identity::<T>::unsafe_join_identity(did, Permissions::empty(), contract.clone());
                Ok(())
            }
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn register_did(_account_id: T::AccountId) -> DispatchResult {
        Ok(())
    }
}

pub const CHAIN_EXTENSION_BATCH_SIZE: u32 = 100;

macro_rules! cost {
    ($name:ident) => {
        (Self::$name(1).saturating_sub(Self::$name(0)))
    };
}

macro_rules! cost_batched {
    ($name:ident) => {
        cost!($name) / u64::from(CHAIN_EXTENSION_BATCH_SIZE)
    };
}

macro_rules! cost_byte_batched {
    ($name:ident) => {
        cost_batched!($name) / 1024
    };
}

pub trait WeightInfo {
    fn chain_extension_read_storage(k: u32, v: u32) -> Weight;
    fn chain_extension_get_version(r: u32) -> Weight;
    fn chain_extension_get_key_did(r: u32) -> Weight;
    fn chain_extension_hash_twox_64(r: u32) -> Weight;
    fn chain_extension_hash_twox_64_per_kb(n: u32) -> Weight;
    fn chain_extension_hash_twox_128(r: u32) -> Weight;
    fn chain_extension_hash_twox_128_per_kb(n: u32) -> Weight;
    fn chain_extension_hash_twox_256(r: u32) -> Weight;
    fn chain_extension_hash_twox_256_per_kb(n: u32) -> Weight;
    fn chain_extension_call_runtime(n: u32) -> Weight;
    fn dummy_contract() -> Weight;
    fn basic_runtime_call(n: u32) -> Weight;
    fn chain_extension_get_latest_api_upgrade(r: u32) -> Weight;
    fn chain_extension_get_next_asset_id(r: u32) -> Weight;
    fn upgrade_api() -> Weight;

    fn base_weight_with_code(code_len: u32, data_len: u32, salt_len: u32) -> Weight;
    fn base_weight_with_hash(data_len: u32, salt_len: u32) -> Weight;
    fn link_contract_as_primary_key() -> Weight;
    fn link_contract_as_secondary_key() -> Weight;

    fn update_call_runtime_whitelist(u: u32) -> Weight;

    // TODO: Needs improvement.
    fn read_storage(k: u32, v: u32) -> Weight {
        Self::chain_extension_read_storage(k, v).saturating_sub(Self::dummy_contract())
    }

    fn get_version() -> Weight {
        cost_batched!(chain_extension_get_version)
    }

    fn get_key_did() -> Weight {
        cost_batched!(chain_extension_get_key_did)
    }

    fn hash_twox_64(r: u32) -> Weight {
        let per_byte = cost_byte_batched!(chain_extension_hash_twox_64_per_kb);
        cost_batched!(chain_extension_hash_twox_64)
            .saturating_add(per_byte.saturating_mul(r as u64))
    }

    fn hash_twox_128(r: u32) -> Weight {
        let per_byte = cost_byte_batched!(chain_extension_hash_twox_128_per_kb);
        cost_batched!(chain_extension_hash_twox_128)
            .saturating_add(per_byte.saturating_mul(r as u64))
    }

    fn hash_twox_256(r: u32) -> Weight {
        let per_byte = cost_byte_batched!(chain_extension_hash_twox_256_per_kb);
        cost_batched!(chain_extension_hash_twox_256)
            .saturating_add(per_byte.saturating_mul(r as u64))
    }

    fn call_runtime(in_len: u32) -> Weight {
        Self::chain_extension_call_runtime(in_len)
            .saturating_sub(Self::dummy_contract())
            .saturating_sub(Self::basic_runtime_call(in_len))
    }

    fn get_latest_api_upgrade() -> Weight {
        cost_batched!(chain_extension_get_latest_api_upgrade)
    }

    fn get_next_asset_id() -> Weight {
        cost_batched!(chain_extension_get_next_asset_id)
    }
}

/// The `Config` trait for the smart contracts pallet.
pub trait Config:
    IdentityConfig + BConfig<Currency = Self::Balances> + frame_system::Config
{
    /// The overarching event type.
    type RuntimeEvent: From<Event<Self>> + Into<<Self as frame_system::Config>::RuntimeEvent>;

    /// Max value that `in_len` can take, that is,
    /// the length of the data sent from a contract when using the ChainExtension.
    type MaxInLen: Get<u32>;

    /// Max value that can be returned from the ChainExtension.
    type MaxOutLen: Get<u32>;

    /// Asset module trait
    type Asset: AssetFnTrait<Self::AccountId, Self::RuntimeOrigin>;

    /// The weight configuration for the pallet.
    type WeightInfo: WeightInfo;
}

decl_event! {
    pub enum Event<T>
    where
        Hash = CodeHash<T>,
        AccountId = <T as frame_system::Config>::AccountId,
    {
        /// Emitted when a contract starts supporting a new API upgrade.
        /// Contains the [`Api`], [`ChainVersion`], and the bytes for the code hash.
        ApiHashUpdated(Api, ChainVersion, Hash),
        /// Emitted when a contract calls into the runtime.
        /// Contains the account id set by the contract owner and the [`ExtrinsicId`].
        SCRuntimeCall(AccountId, ExtrinsicId)
    }
}

decl_error! {
    pub enum Error for Module<T: Config> where T::AccountId: UncheckedFrom<T::Hash>, T::AccountId: AsRef<[u8]> {
        /// Invalid `func_id` provided from contract.
        InvalidFuncId,
        /// Failed to decode a valid `RuntimeCall`.
        InvalidRuntimeCall,
        /// `ReadStorage` failed to write value into the contract's buffer.
        ReadStorageFailed,
        /// Data left in input when decoding arguments of a call.
        DataLeftAfterDecoding,
        /// Input data that a contract passed when using the ChainExtension was too large.
        InLenTooLarge,
        /// Output data returned from the ChainExtension was too large.
        OutLenTooLarge,
        /// A contract was attempted to be instantiated,
        /// but no identity was given to associate the new contract's key with.
        InstantiatorWithNoIdentity,
        /// Extrinsic is not allowed to be called by contracts.
        RuntimeCallDenied,
        /// The caller is not a primary key.
        CallerNotAPrimaryKey,
        /// Secondary key permissions are missing.
        MissingKeyPermissions,
        /// Only future chain versions are allowed.
        InvalidChainVersion,
        /// There are no api upgrades supported for the contract.
        NoUpgradesSupported
    }
}

storage_migration_ver!(1);

decl_storage! {
    trait Store for Module<T: Config> as PolymeshContracts where T::AccountId: UncheckedFrom<T::Hash>, T::AccountId: AsRef<[u8]> {
        /// Whitelist of extrinsics allowed to be called from contracts.
        pub CallRuntimeWhitelist get(fn call_runtime_whitelist):
            map hasher(identity) ExtrinsicId => bool;
        /// Storage version.
        StorageVersion get(fn storage_version) build(|_| Version::new(1)): Version;
        /// Stores the chain version and code hash for the next chain upgrade.
        pub ApiNextUpgrade get(fn api_next_upgrade):
            map hasher(twox_64_concat) Api => Option<NextUpgrade<T>>;
        /// Stores the code hash for the current api.
        pub CurrentApiHash get (fn api_tracker):
            map hasher(twox_64_concat) Api => Option<ApiCodeHash<T>>;
    }
    add_extra_genesis {
        config(call_whitelist): Vec<ExtrinsicId>;
        config(upgradable_code): Vec<u8>;
        config(upgradable_owner): Option<T::AccountId>;
        config(upgradable_description): [u8; 4];
        config(upgradable_major): u32;
        build(|config: &GenesisConfig<T>| {
            for ext_id in &config.call_whitelist {
                CallRuntimeWhitelist::insert(ext_id, true);
            }

            if let Some(ref owner) = config.upgradable_owner {
                let code_result = FrameContracts::<T>::bare_upload_code(
                    owner.clone(),
                    config.upgradable_code.clone(),
                    None,
                    Determinism::Deterministic,
                ).unwrap();
                log::info!("Uploaded upgradeable code {}", code_result.code_hash);
                let api_code_hash: ApiCodeHash<T> = ApiCodeHash::new(code_result.code_hash);
                let api: Api = Api::new(config.upgradable_description, config.upgradable_major);
                CurrentApiHash::insert(api, api_code_hash);
            }

        });
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin, T::AccountId: UncheckedFrom<T::Hash>, T::AccountId: AsRef<[u8]> {
        type Error = Error<T>;
        fn deposit_event() = default;

        fn on_runtime_upgrade() -> Weight {
            // We remove storage for CallRuntimeWhitelist under the previous pallet name (Contracts)
            // We do not copy the storage over as it will be re-initialised as part of the 6.0.0 post-release actions
            let old_pallet_name = "Contracts";
            storage_migrate_on!(StorageVersion, 1, {
                let prefixes = &[
                    "CallRuntimeWhitelist",
                ];
                for prefix in prefixes {
                    let res = frame_support::storage::migration::clear_storage_prefix(old_pallet_name.as_bytes(), prefix.as_bytes(), b"", None, None);
                    log::info!("Cleared storage prefix[{prefix}]: cursor={:?}, backend={}, unique={}, loops={}",
                        res.maybe_cursor, res.backend, res.unique, res.loops);
                }
            });
            Weight::zero()
        }

        /// Instantiates a smart contract defining it with the given `code` and `salt`.
        ///
        /// The contract will be attached as a secondary key,
        /// with `perms` as its permissions, to `origin`'s identity.
        ///
        /// The contract is transferred `endowment` amount of POLYX.
        /// This is distinct from the `gas_limit`,
        /// which controls how much gas the deployment code may at most consume.
        ///
        /// # Arguments
        /// - `endowment` amount of POLYX to transfer to the contract.
        /// - `gas_limit` for how much gas the `deploy` code in the contract may at most consume.
        /// - `storage_deposit_limit` The maximum amount of balance that can be charged/reserved
        ///   from the caller to pay for the storage consumed.
        /// - `code` with the WASM binary defining the smart contract.
        /// - `data` The input data to pass to the contract constructor.
        /// - `salt` used for contract address derivation.
        ///    By varying this, the same `code` can be used under the same identity.
        /// - `perms` that the new secondary key will have.
        ///
        /// # Errors
        /// - All the errors in `pallet_contracts::Call::instantiate_with_code` can also happen here.
        /// - CDD/Permissions are checked, unlike in `pallet_contracts`.
        /// - Errors that arise when adding a new secondary key can also occur here.
        #[weight = Module::<T>::weight_instantiate_with_code(code.len() as u32, data.len() as u32, salt.len() as u32, Some(perms)).saturating_add(*gas_limit)]
        pub fn instantiate_with_code_perms(
            origin,
            endowment: Balance,
            gas_limit: Weight,
            storage_deposit_limit: Option<Balance>,
            code: Vec<u8>,
            data: Vec<u8>,
            salt: Vec<u8>,
            perms: Permissions
        ) -> DispatchResultWithPostInfo {
            Self::general_instantiate(
                origin,
                endowment,
                gas_limit,
                storage_deposit_limit,
                Code::Upload(code),
                data,
                salt,
                Some(perms),
                false
            )
        }

        /// Instantiates a smart contract defining using the given `code_hash` and `salt`.
        ///
        /// Unlike `instantiate_with_code`,
        /// this assumes that at least one contract with the same WASM code has already been uploaded.
        ///
        /// The contract will be attached as a secondary key,
        /// with `perms` as its permissions, to `origin`'s identity.
        ///
        /// The contract is transferred `endowment` amount of POLYX.
        /// This is distinct from the `gas_limit`,
        /// which controls how much gas the deployment code may at most consume.
        ///
        /// # Arguments
        /// - `endowment` amount of POLYX to transfer to the contract.
        /// - `gas_limit` for how much gas the `deploy` code in the contract may at most consume.
        /// - `storage_deposit_limit` The maximum amount of balance that can be charged/reserved
        ///   from the caller to pay for the storage consumed.
        /// - `code_hash` of an already uploaded WASM binary.
        /// - `data` The input data to pass to the contract constructor.
        /// - `salt` used for contract address derivation.
        ///    By varying this, the same `code` can be used under the same identity.
        /// - `perms` that the new secondary key will have.
        ///
        /// # Errors
        /// - All the errors in `pallet_contracts::Call::instantiate` can also happen here.
        /// - CDD/Permissions are checked, unlike in `pallet_contracts`.
        /// - Errors that arise when adding a new secondary key can also occur here.
        #[weight = Module::<T>::weight_instantiate_with_hash(data.len() as u32, salt.len() as u32, Some(perms)).saturating_add(*gas_limit)]
        pub fn instantiate_with_hash_perms(
            origin,
            endowment: Balance,
            gas_limit: Weight,
            storage_deposit_limit: Option<Balance>,
            code_hash: CodeHash<T>,
            data: Vec<u8>,
            salt: Vec<u8>,
            perms: Permissions
        ) -> DispatchResultWithPostInfo {
            Self::general_instantiate(
                origin,
                endowment,
                gas_limit,
                storage_deposit_limit,
                Code::Existing(code_hash),
                data,
                salt,
                Some(perms),
                false
            )
        }

        /// Update CallRuntime whitelist.
        ///
        /// # Arguments
        ///
        /// # Errors
        #[weight = <T as Config>::WeightInfo::update_call_runtime_whitelist(updates.len() as u32)]
        pub fn update_call_runtime_whitelist(origin, updates: Vec<(ExtrinsicId, bool)>) -> DispatchResult {
            Self::base_update_call_runtime_whitelist(origin, updates)
        }

        /// Instantiates a smart contract defining it with the given `code` and `salt`.
        ///
        /// The contract will be attached as a primary key of a newly created child identity of the caller.
        ///
        /// # Arguments
        /// - `endowment`: Amount of POLYX to transfer to the contract.
        /// - `gas_limit`: For how much gas the `deploy` code in the contract may at most consume.
        /// - `storage_deposit_limit`: The maximum amount of balance that can be charged/reserved from the caller to pay for the storage consumed.
        /// - `code`: The WASM binary defining the smart contract.
        /// - `data`: The input data to pass to the contract constructor.
        /// - `salt`: Used for contract address derivation. By varying this, the same `code` can be used under the same identity.
        ///
        #[weight = Module::<T>::weight_instantiate_with_code(code.len() as u32, data.len() as u32, salt.len() as u32, None).saturating_add(*gas_limit)]
        pub fn instantiate_with_code_as_primary_key(
            origin,
            endowment: Balance,
            gas_limit: Weight,
            storage_deposit_limit: Option<Balance>,
            code: Vec<u8>,
            data: Vec<u8>,
            salt: Vec<u8>
        ) -> DispatchResultWithPostInfo {
            Self::general_instantiate(
                origin,
                endowment,
                gas_limit,
                storage_deposit_limit,
                Code::Upload(code),
                data,
                salt,
                None,
                true
            )
        }

        /// Instantiates a smart contract defining using the given `code_hash` and `salt`.
        ///
        /// Unlike `instantiate_with_code`, this assumes that at least one contract with the same WASM code has already been uploaded.
        ///
        /// The contract will be attached as a primary key of a newly created child identity of the caller.
        ///
        /// # Arguments
        /// - `endowment`: amount of POLYX to transfer to the contract.
        /// - `gas_limit`: for how much gas the `deploy` code in the contract may at most consume.
        /// - `storage_deposit_limit`: The maximum amount of balance that can be charged/reserved from the caller to pay for the storage consumed.
        /// - `code_hash`: of an already uploaded WASM binary.
        /// - `data`: The input data to pass to the contract constructor.
        /// - `salt`: used for contract address derivation. By varying this, the same `code` can be used under the same identity.
        ///
        #[weight = Module::<T>::weight_instantiate_with_hash(data.len() as u32, salt.len() as u32, None).saturating_add(*gas_limit)]
        pub fn instantiate_with_hash_as_primary_key(
            origin,
            endowment: Balance,
            gas_limit: Weight,
            storage_deposit_limit: Option<Balance>,
            code_hash: CodeHash<T>,
            data: Vec<u8>,
            salt: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            Self::general_instantiate(
                origin,
                endowment,
                gas_limit,
                storage_deposit_limit,
                Code::Existing(code_hash),
                data,
                salt,
                None,
                true
            )
        }

        #[weight =  <T as Config>::WeightInfo::upgrade_api()]
        pub fn upgrade_api(
            origin,
            api: Api,
            next_upgrade: NextUpgrade<T>
        ) -> DispatchResult {
            Self::base_upgrade_api(origin, api, next_upgrade)
        }
    }
}

impl<T: Config> Module<T>
where
    T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
{
    /// Instantiates a contract using `code` as the WASM code blob.
    fn base_update_call_runtime_whitelist(
        origin: T::RuntimeOrigin,
        updates: Vec<(ExtrinsicId, bool)>,
    ) -> DispatchResult {
        ensure_root(origin)?;

        for (ext_id, allow) in updates {
            if allow {
                CallRuntimeWhitelist::insert(ext_id, true);
            } else {
                CallRuntimeWhitelist::remove(ext_id);
            }
        }
        Ok(())
    }

    pub fn ensure_call_runtime(ext_id: ExtrinsicId) -> DispatchResult {
        ensure!(
            Self::call_runtime_whitelist(ext_id),
            Error::<T>::RuntimeCallDenied
        );
        Ok(())
    }

    /// Computes the weight of `instantiate_with_code(code, salt, perms)`.
    fn weight_link_contract_to_did(perms: Option<&Permissions>) -> Weight {
        match perms {
            Some(permissions) => <T as Config>::WeightInfo::link_contract_as_secondary_key()
                .saturating_add(<T as IdentityConfig>::WeightInfo::permissions_cost_perms(
                    permissions,
                )),
            None => <T as Config>::WeightInfo::link_contract_as_primary_key(),
        }
    }

    /// Computes the weight of `instantiate_with_code(code, salt, perms)`.
    fn weight_instantiate_with_code(
        code_len: u32,
        data_len: u32,
        salt_len: u32,
        perms: Option<&Permissions>,
    ) -> Weight {
        let instantiate_weight =
            <T as BConfig>::WeightInfo::instantiate_with_code(code_len, data_len, salt_len)
                .saturating_add(<T as Config>::WeightInfo::base_weight_with_code(
                    code_len, data_len, salt_len,
                ));
        instantiate_weight.saturating_add(Self::weight_link_contract_to_did(perms))
    }

    /// Computes weight of `instantiate_with_hash(salt, perms)`.
    fn weight_instantiate_with_hash(
        data_len: u32,
        salt_len: u32,
        perms: Option<&Permissions>,
    ) -> Weight {
        let instantiate_weight = <T as BConfig>::WeightInfo::instantiate(data_len, salt_len)
            .saturating_add(<T as Config>::WeightInfo::base_weight_with_hash(
                data_len, salt_len,
            ));
        instantiate_weight.saturating_add(Self::weight_link_contract_to_did(perms))
    }

    /// Get base weight and contract address.
    pub fn base_weight_and_contract_address(
        caller: &T::AccountId,
        code: &Code<CodeHash<T>>,
        data: &[u8],
        salt: &[u8],
        perms: Option<&Permissions>,
    ) -> (Weight, T::AccountId) {
        let data_len = data.len() as u32;
        let salt_len = salt.len() as u32;
        // The base weight of the Substrate call `instantiate*`.
        let (base_weight, code_hash) = match &code {
            Code::Existing(h) => (
                Self::weight_instantiate_with_hash(data_len, salt_len, perms),
                *h,
            ),
            Code::Upload(c) => {
                let code_len = c.len() as u32;
                (
                    Self::weight_instantiate_with_code(code_len, data_len, salt_len, perms),
                    T::Hashing::hash(c),
                )
            }
        };

        // Pre-compute what contract's address will be
        let contract = FrameContracts::<T>::contract_address(caller, &code_hash, &data, &salt);
        (base_weight, contract)
    }

    /// Link the contract to the caller's identity.
    pub fn link_contract_to_did(
        caller: &T::AccountId,
        contract: T::AccountId,
        perms: Option<Permissions>,
        deploy_as_child_identity: bool,
    ) -> Result<(), DispatchErrorWithPostInfo> {
        // Ensure we have perms + we'll need sender & DID.
        let caller = pallet_permissions::Module::<T>::ensure_call_permissions(caller)?;

        // Ensure contract is not linked to a DID
        Identity::<T>::ensure_key_did_unlinked(&contract)?;
        if deploy_as_child_identity {
            ensure!(
                caller.secondary_key.is_none(),
                Error::<T>::CallerNotAPrimaryKey
            );
            Identity::<T>::ensure_no_parent(caller.primary_did)?;
            Identity::<T>::unverified_create_child_identity(contract, caller.primary_did)?;
        } else {
            let perms = perms.ok_or(Error::<T>::MissingKeyPermissions)?;
            // Ensure that the key can be a secondary-key
            Identity::<T>::ensure_perms_length_limited(&perms)?;
            // Link contract's address to caller's identity as a secondary key with `perms`.
            Identity::<T>::unsafe_join_identity(caller.primary_did, perms, contract);
        }
        Ok(())
    }

    /// General logic for contract instantiation both when the code or code hash is given.
    fn general_instantiate(
        origin: T::RuntimeOrigin,
        endowment: Balance,
        gas_limit: Weight,
        storage_deposit_limit: Option<Balance>,
        code: Code<CodeHash<T>>,
        data: Vec<u8>,
        salt: Vec<u8>,
        perms: Option<Permissions>,
        deploy_as_child_identity: bool,
    ) -> DispatchResultWithPostInfo {
        let caller = ensure_signed(origin.clone())?;
        let (base_weight, contract) =
            Self::base_weight_and_contract_address(&caller, &code, &data, &salt, perms.as_ref());
        Self::link_contract_to_did(&caller, contract, perms, deploy_as_child_identity)?;

        // Instantiate the contract.
        let storage_deposit_limit = storage_deposit_limit.map(Compact);
        let mut result = match code {
            Code::Existing(code_hash) => FrameContracts::<T>::instantiate(
                origin,
                endowment,
                gas_limit,
                storage_deposit_limit,
                code_hash,
                data,
                salt,
            ),
            Code::Upload(code) => FrameContracts::<T>::instantiate_with_code(
                origin,
                endowment,
                gas_limit,
                storage_deposit_limit,
                code,
                data,
                salt,
            ),
        };
        // Add `base_weight` in returned actual weight.
        {
            // Get `post_info` from either the Ok/Err variant.
            let post_info = match &mut result {
                Ok(post_info) => post_info,
                Err(err) => &mut err.post_info,
            };
            // add the base weight to the actual weight.
            if let Some(actual_weight) = &mut post_info.actual_weight {
                *actual_weight = actual_weight.saturating_add(base_weight);
            }
            // Note: `actual_weight=None` is the worse case weight with already include the base weight.
        }
        // Return the result with the updated actual_weight.
        result
    }

    fn base_upgrade_api(
        origin: T::RuntimeOrigin,
        api: Api,
        next_upgrade: NextUpgrade<T>,
    ) -> DispatchResult {
        ensure_root(origin)?;

        let current_chain_version = ChainVersion::new(
            T::Version::get().spec_version,
            T::Version::get().transaction_version,
        );

        if next_upgrade.chain_version < current_chain_version {
            return Err(Error::<T>::InvalidChainVersion.into());
        }

        ApiNextUpgrade::<T>::insert(&api, &next_upgrade);

        Self::deposit_event(Event::<T>::ApiHashUpdated(
            api,
            next_upgrade.chain_version,
            next_upgrade.api_hash.hash,
        ));
        Ok(())
    }
}
