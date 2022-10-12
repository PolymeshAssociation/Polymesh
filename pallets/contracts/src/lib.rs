// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
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
//! The Contracts module provides functions for:
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

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{
        DispatchError, DispatchErrorWithPostInfo, DispatchResult, DispatchResultWithPostInfo,
        Dispatchable, GetDispatchInfo,
    },
    ensure,
    log::trace,
    storage::unhashed,
    traits::{Get, GetCallMetadata},
    weights::Weight,
};
use frame_system::RawOrigin;
use pallet_contracts::chain_extension as ce;
use pallet_contracts::Config as BConfig;
use pallet_contracts_primitives::{Code, ContractResult};
use pallet_identity::PermissionedCallOriginData;
use pallet_identity::WeightInfo as _;
use pallet_permissions::with_call_metadata;
use polymesh_common_utilities::traits::identity::Config as IdentityConfig;
use polymesh_common_utilities::{with_transaction, Context};
use polymesh_primitives::{Balance, IdentityId, Permissions};
use sp_core::crypto::UncheckedFrom;
use sp_core::Bytes;
use sp_runtime::traits::Hash;
use sp_std::borrow::Cow;
use sp_std::vec::Vec;

type Identity<T> = pallet_identity::Module<T>;
type IdentityError<T> = pallet_identity::Error<T>;
type FrameContracts<T> = pallet_contracts::Pallet<T>;
type CodeHash<T> = <T as frame_system::Config>::Hash;

/// Maximum decoding depth.
const MAX_DECODE_DEPTH: u32 = 10;

pub struct ContractPolymeshHooks;

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
                if contract_did != did {
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
}

pub trait WeightInfo {
    /// Computes the cost of instantiating where `code_len`
    /// and `salt_len` are specified in kilobytes.
    fn instantiate_with_code_perms(code_len: u32, salt_len: u32) -> Weight;

    /// Computes the cost of instantiating for `code` and `salt`.
    ///
    /// Permissions are not accounted for here.
    fn instantiate_with_code_bytes(code: &[u8], salt: &[u8]) -> Weight {
        Self::instantiate_with_code_perms(code.len() as u32, salt.len() as u32)
    }

    /// Computes the cost of instantiating where `salt_len` is specified in kilobytes.
    fn instantiate_with_hash_perms(salt_len: u32) -> Weight;

    /// Computes the cost of instantiating for `salt`.
    ///
    /// Permissions are not accounted for here.
    fn instantiate_with_hash_bytes(salt: &[u8]) -> Weight {
        Self::instantiate_with_hash_perms((salt.len()) as u32)
    }

    /// Computes the cost just for executing the chain extension,
    /// subtracting costs for `call` itself and runtime callbacks.
    fn chain_extension(in_len: u32) -> Weight {
        Self::chain_extension_full(in_len)
            .saturating_sub(Self::chain_extension_early_exit())
            .saturating_sub(Self::basic_runtime_call(in_len))
    }

    /// Returns the weight for a full execution of a smart contract `call` that
    /// calls  `register_custom_asset_type` in the runtime via a chain extension.
    /// The asset type is `in_len` characters long.
    fn chain_extension_full(in_len: u32) -> Weight;

    /// Returns the weight for a smart contract `call` that enters the chain extension
    /// but then immediately returns.
    fn chain_extension_early_exit() -> Weight;

    /// Returns the weight of executing `Asset::register_custom_asset_type`
    /// with an asset type that is `in_len` characters long.
    fn basic_runtime_call(in_len: u32) -> Weight;
}

/// The `Config` trait for the smart contracts pallet.
pub trait Config:
    IdentityConfig + BConfig<Currency = Self::Balances> + frame_system::Config
{
    /// The overarching event type.
    type Event: From<Event> + Into<<Self as frame_system::Config>::Event>;

    /// Max value that `in_len` can take, that is,
    /// the length of the data sent from a contract when making a runtime call.
    type MaxInLen: Get<u32>;

    /// The weight configuration for the pallet.
    type WeightInfo: WeightInfo;
}

decl_event! {
    pub enum Event {
        // This pallet does not directly define custom events.
        // See `pallet_contracts` and `pallet_identity`
        // for events currently emitted by extrinsics of this pallet.
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
        /// Input data that a contract passed when making a runtime call was too large.
        InLenTooLarge,
        /// A contract was attempted to be instantiated,
        /// but no identity was given to associate the new contract's key with.
        InstantiatorWithNoIdentity,
    }
}

decl_storage! {
    trait Store for Module<T: Config> as Contracts where T::AccountId: UncheckedFrom<T::Hash>, T::AccountId: AsRef<[u8]> {
        // Storage items defined in `pallet_contracts` and `pallet_identity`.
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin, T::AccountId: UncheckedFrom<T::Hash>, T::AccountId: AsRef<[u8]> {
        type Error = Error<T>;
        fn deposit_event() = default;

        fn on_initialize(block: T::BlockNumber) -> Weight {
            // Does he know what I do and... 🎶
            pallet_contracts::Pallet::<T>::on_initialize(block)
        }

        fn on_runtime_upgrade() -> Weight {
            // 🎶 ...You'll pass this on, won't you and?
            pallet_contracts::Pallet::<T>::on_runtime_upgrade()
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
        #[weight = Module::<T>::weight_instantiate_with_code(&code, &salt, &perms).saturating_add(*gas_limit)]
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
            Self::base_instantiate_with_code(origin, endowment, gas_limit, storage_deposit_limit, code, data, salt, perms)
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
        #[weight = Module::<T>::weight_instantiate_with_hash(&salt, &perms).saturating_add(*gas_limit)]
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
            Self::base_instantiate_with_hash(origin, endowment, gas_limit, storage_deposit_limit, code_hash, data, salt, perms)
        }
    }
}

impl<T: Config> Module<T>
where
    T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
{
    /// Instantiates a contract using `code` as the WASM code blob.
    fn base_instantiate_with_code(
        origin: T::Origin,
        endowment: Balance,
        gas_limit: Weight,
        storage_deposit_limit: Option<Balance>,
        code: Vec<u8>,
        inst_data: Vec<u8>,
        salt: Vec<u8>,
        perms: Permissions,
    ) -> DispatchResultWithPostInfo {
        Self::general_instantiate(
            origin,
            endowment,
            // Compute the base weight of roughly `base_instantiate`.
            Self::weight_instantiate_with_code(&code, &salt, &perms),
            gas_limit,
            storage_deposit_limit,
            Code::Upload(Bytes(code)),
            inst_data,
            salt,
            perms,
        )
    }

    /// Computes weight of `instantiate_with_code(code, salt, perms)`.
    fn weight_instantiate_with_code(code: &[u8], salt: &[u8], perms: &Permissions) -> Weight {
        <T as Config>::WeightInfo::instantiate_with_code_bytes(&code, &salt).saturating_add(
            <T as IdentityConfig>::WeightInfo::permissions_cost_perms(perms),
        )
    }

    /// Instantiates a contract using an existing WASM code blob with `code_hash` as its code.
    fn base_instantiate_with_hash(
        origin: T::Origin,
        endowment: Balance,
        gas_limit: Weight,
        storage_deposit_limit: Option<Balance>,
        code_hash: CodeHash<T>,
        inst_data: Vec<u8>,
        salt: Vec<u8>,
        perms: Permissions,
    ) -> DispatchResultWithPostInfo {
        Self::general_instantiate(
            origin,
            endowment,
            // Compute the base weight of roughly `base_instantiate`.
            Self::weight_instantiate_with_hash(&salt, &perms),
            gas_limit,
            storage_deposit_limit,
            Code::Existing(code_hash),
            inst_data,
            salt,
            perms,
        )
    }

    /// Computes weight of `instantiate_with_hash(code, salt, perms)`.
    fn weight_instantiate_with_hash(salt: &[u8], perms: &Permissions) -> Weight {
        <T as Config>::WeightInfo::instantiate_with_hash_bytes(&salt).saturating_add(
            <T as IdentityConfig>::WeightInfo::permissions_cost_perms(perms),
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
        storage_deposit_limit: Option<Balance>,
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
        let contract_key =
            FrameContracts::<T>::contract_address(&sender, &Self::code_hash(&code), &salt);

        // ...and ensure that key can be a secondary-key of DID...
        Identity::<T>::ensure_perms_length_limited(&perms)?;
        Identity::<T>::ensure_key_did_unlinked(&contract_key)?;

        with_transaction(|| {
            // Link contract's address to caller's identity as a secondary key with `perms`.
            Identity::<T>::unsafe_join_identity(did, perms, contract_key);

            // Now we can finally instantiate the contract.
            Self::handle_error(
                base_weight,
                FrameContracts::<T>::bare_instantiate(
                    sender,
                    endowment,
                    gas_limit,
                    storage_deposit_limit,
                    code,
                    inst_data,
                    salt,
                    false,
                ),
            )
        })
    }

    /// Computes the code hash of `code`.
    fn code_hash(code: &Code<CodeHash<T>>) -> Cow<'_, CodeHash<T>> {
        match &code {
            Code::Existing(h) => Cow::Borrowed(h),
            Code::Upload(c) => Cow::Owned(T::Hashing::hash(c)),
        }
    }

    /// Enriches `result` of executing a smart contract with actual weight,
    /// accounting for the consumed gas.
    fn handle_error<A>(
        base_weight: Weight,
        result: ContractResult<Result<A, DispatchError>, Balance>,
    ) -> DispatchResultWithPostInfo {
        let post_info = Some(result.gas_consumed.saturating_add(base_weight)).into();
        match result.result {
            Ok(_) => Ok(post_info),
            Err(error) => Err(DispatchErrorWithPostInfo { post_info, error }),
        }
    }
}

/// Polymesh ChainExtension callable.
#[derive(Clone, Copy, Debug)]
enum FuncId {
    /// No operation -- Used for benchmarking the ChainExtension.
    #[cfg(feature = "runtime-benchmarks")]
    NOP,
    CallRuntime,
    ReadStorage,
    GetSpecVersion,
    GetTransactionVersion,

    /// Deprecated Polymesh v5.0.x chain extensions.
    OldCallRuntime(u8, u8),
}

impl FuncId {
    fn try_from(id: u32) -> Option<Self> {
        let ext_id = (id >> 16) as u16;
        let func_id = (id & 0x0000FFFF) as u16;
        match ext_id {
            0x00 => match func_id {
                #[cfg(feature = "runtime-benchmarks")]
                0x00 => Some(Self::NOP),
                0x01 => Some(Self::CallRuntime),
                0x02 => Some(Self::ReadStorage),
                0x03 => Some(Self::GetSpecVersion),
                0x04 => Some(Self::GetTransactionVersion),
                _ => None,
            },
            0x1A => match func_id {
                0x00_00 | 0x01_00 | 0x02_00 | 0x03_00 | 0x11_00 => {
                    Some(Self::OldCallRuntime(0x1A, (func_id >> 8) as u8))
                }
                _ => None,
            },
            0x2F => match func_id {
                0x01_00 => Some(Self::OldCallRuntime(0x2F, 0x01)),
                _ => None,
            },
            _ => None,
        }
    }
}

/// Returns the `contract`'s DID or errors.
fn contract_did<T: Config>(contract: &T::AccountId) -> Result<IdentityId, DispatchError>
where
    T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
{
    // N.B. it might be the case that the contract is a primary key due to rotation.
    Ok(Identity::<T>::get_identity(contract).ok_or(Error::<T>::InstantiatorWithNoIdentity)?)
}

/// Run `with` while the current DID and Payer is temporarily set to the given one.
fn with_did_and_payer<T: Config, W: FnOnce() -> R, R>(
    did: IdentityId,
    payer: T::AccountId,
    with: W,
) -> R {
    let old_payer = Context::current_payer::<Identity<T>>();
    let old_did = Context::current_identity::<Identity<T>>();
    Context::set_current_payer::<Identity<T>>(Some(payer));
    Context::set_current_identity::<Identity<T>>(Some(did));
    let result = with();
    Context::set_current_payer::<Identity<T>>(old_payer);
    Context::set_current_identity::<Identity<T>>(old_did);
    result
}

// TODO: Move into a different file.
struct ChainInput<'a, 'b> {
    input1: &'a [u8],
    input2: &'b [u8],
}

impl<'a, 'b> ChainInput<'a, 'b> {
    pub fn new(input1: &'a [u8], input2: &'b [u8]) -> Self {
        Self { input1, input2 }
    }

    pub fn len(&self) -> usize {
        self.input1.len() + self.input2.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a, 'b> codec::Input for ChainInput<'a, 'b> {
    fn remaining_len(&mut self) -> Result<Option<usize>, codec::Error> {
        Ok(Some(self.len()))
    }

    fn read(&mut self, into: &mut [u8]) -> Result<(), codec::Error> {
        let len = into.len();
        let in1_len = self.input1.len();
        let in2_len = self.input2.len();
        if len > (in1_len + in2_len) {
            return Err("Not enough data to fill buffer".into());
        }
        // `input1` still has bytes, read from it first.
        if in1_len > 0 {
            let off = in1_len.min(len);
            // Split `into` buffer into two parts.
            let (into1, into2) = into.split_at_mut(off);
            // Read from `input1`.
            let len = into1.len();
            into1.copy_from_slice(&self.input1[..len]);
            self.input1 = &self.input1[len..];
            // Read from `input2`.
            let len = into2.len();
            into2.copy_from_slice(&self.input2[..len]);
            self.input2 = &self.input2[len..];
        } else {
            // `input1` is empty, only read from `input2`.
            let len = into.len();
            into.copy_from_slice(&self.input2[..len]);
            self.input2 = &self.input2[len..];
        }
        Ok(())
    }
}

fn read_storage<T, E>(env: ce::Environment<E, ce::InitState>) -> ce::Result<ce::RetVal>
where
    T: Config,
    T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
    E: ce::Ext<T = T>,
{
    use codec::Encode;
    let mut env = env.buf_in_buf_out();
    let in_len = env.in_len();

    // TODO: benchmarks and weights.
    let key = env.read(in_len)?;
    trace!(
        target: "runtime",
        "PolymeshExtension contract ReadStorage: key={:?}",
        key
    );
    let value = unhashed::get_raw(key.as_slice());
    trace!(
        target: "runtime",
        "PolymeshExtension contract ReadStorage: value={:?}",
        value
    );
    let encoded = value.encode();
    env.write(&encoded, false, None).map_err(|err| {
        trace!(
            target: "runtime",
            "PolymeshExtension failed to write storage value into contract memory:{:?}",
            err
        );
        Error::<T>::ReadStorageFailed
    })?;

    Ok(ce::RetVal::Converging(0))
}

fn get_version<T, E>(
    env: ce::Environment<E, ce::InitState>,
    get_spec: bool,
) -> ce::Result<ce::RetVal>
where
    T: Config,
    T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
    E: ce::Ext<T = T>,
{
    use codec::Encode;
    let mut env = env.prim_in_buf_out();

    let runtime_version = <T as frame_system::Config>::Version::get();
    let version = if get_spec {
        runtime_version.spec_version
    } else {
        runtime_version.transaction_version
    }
    .encode();
    env.write(&version, false, None).map_err(|err| {
        trace!(
            target: "runtime",
            "PolymeshExtension failed to write value into contract memory:{:?}",
            err
        );
        Error::<T>::ReadStorageFailed
    })?;

    Ok(ce::RetVal::Converging(0))
}

fn call_runtime<T, E>(
    env: ce::Environment<E, ce::InitState>,
    old_call: Option<(u8, u8)>,
) -> ce::Result<ce::RetVal>
where
    <T as BConfig>::Call: GetDispatchInfo + GetCallMetadata,
    T: Config,
    T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
    E: ce::Ext<T = T>,
{
    let mut env = env.buf_in_buf_out();
    let in_len = env.in_len();

    // Limit `in_len` to a maximum.
    ensure!(
        in_len <= <T as Config>::MaxInLen::get(),
        Error::<T>::InLenTooLarge
    );

    // Charge weight as a linear function of `in_len`.
    env.charge_weight(<T as Config>::WeightInfo::chain_extension(in_len))?;

    // Decide what to call in the runtime.
    use codec::DecodeLimit;
    let call = match old_call {
        None => {
            // TODO: Decode pallet_id, extrinsic_id and do call filtering.
            let input = env.read(in_len)?;
            <<T as BConfig>::Call>::decode_all_with_depth_limit(
                MAX_DECODE_DEPTH,
                &mut input.as_slice(),
            )
            .map_err(|_| Error::<T>::InvalidRuntimeCall)?
        }
        Some((pallet_id, extrinsic_id)) => {
            // Convert old ChainExtension runtime calls into `Call` format.
            let extrinsic = [pallet_id, extrinsic_id];
            let params = env.read(in_len)?;
            let mut input = ChainInput::new(&extrinsic, params.as_slice());
            let call =
                <<T as BConfig>::Call>::decode_with_depth_limit(MAX_DECODE_DEPTH, &mut input)
                    .map_err(|_| Error::<T>::InvalidRuntimeCall)?;
            ensure!(input.is_empty(), Error::<T>::DataLeftAfterDecoding);
            call
        }
    };

    // Charge weight for the call.
    let di = call.get_dispatch_info();
    let charged_amount = env.charge_weight(di.weight)?;

    // Execute call requested by contract, with current DID set to the contract owner.
    let addr = env.ext().address().clone();
    let result = with_did_and_payer::<T, _, _>(contract_did::<T>(&addr)?, addr.clone(), || {
        with_call_metadata(call.get_call_metadata(), || {
            // Dispatch the call, avoiding use of `ext.call_runtime()`,
            // as that uses `CallFilter = Nothing`, which would case a problem for us.
            call.dispatch(RawOrigin::Signed(addr).into())
        })
    });

    // Refund unspent weight.
    let post_di = result.unwrap_or_else(|e| e.post_info);
    // This check isn't necessary but avoids some work.
    if post_di.actual_weight.is_some() {
        let actual_weight = post_di.calc_actual_weight(&di);
        env.adjust_weight(charged_amount, actual_weight);
    }

    // Ensure the call was successful.
    result.map_err(|e| e.error)?;

    // Done; continue with smart contract execution when returning.
    Ok(ce::RetVal::Converging(0))
}

/// A chain extension allowing calls to polymesh pallets
/// and using the contract's DID instead of the caller's DID.
impl<T> ce::ChainExtension<T> for Module<T>
where
    <T as BConfig>::Call: GetDispatchInfo + GetCallMetadata,
    T: Config,
    T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
{
    fn enabled() -> bool {
        true
    }

    fn call<E: ce::Ext<T = T>>(
        func_id: u32,
        env: ce::Environment<E, ce::InitState>,
    ) -> ce::Result<ce::RetVal> {
        // Decode chain extension id.
        let func_id = FuncId::try_from(func_id)
            .ok_or(Error::<T>::InvalidFuncId)
            .map_err(|e| {
                trace!(
                    target: "runtime",
                    "PolymeshExtension contract calling invalid func_id={:?}",
                    func_id
                );
                e
            })?;

        trace!(
            target: "runtime",
            "PolymeshExtension contract calling: func_id={:?}",
            func_id
        );
        match func_id {
            // Used for benchmarking `chain_extension_early_exit`,
            // so we can remove the cost of the call + overhead of using any chain extension.
            // That is, we want to subtract costs not directly arising from *this* function body.
            // Caveat: This `if` imposes a minor cost during benchmarking but we'll live with that.
            #[cfg(feature = "runtime-benchmarks")]
            FuncId::NOP => {
                let mut env = env.buf_in_buf_out();
                let in_len = env.in_len();

                // No input data is allowed for this chain extension.
                ensure!(in_len == 0, Error::<T>::DataLeftAfterDecoding);
                return Ok(ce::RetVal::Converging(0));
            }
            FuncId::ReadStorage => read_storage(env),
            FuncId::CallRuntime => call_runtime(env, None),
            FuncId::GetSpecVersion => get_version(env, true),
            FuncId::GetTransactionVersion => get_version(env, false),
            FuncId::OldCallRuntime(p, e) => call_runtime(env, Some((p, e))),
        }
    }
}
