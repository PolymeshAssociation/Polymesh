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

use codec::Encode;
use frame_benchmarking::{account, benchmarks};
use frame_support::{storage::unhashed, traits::tokens::currency::Currency};
use frame_system::{Config as SysTrait, Pallet as System, RawOrigin};
use pallet_contracts::benchmarking::code::body::DynInstr::{Counter, Regular};
use pallet_contracts::benchmarking::code::{
    body, max_pages, DataSegment, ImportedFunction, ImportedMemory, Location, ModuleDefinition,
    WasmModule,
};
use pallet_contracts::Pallet as FrameContracts;
use sp_runtime::traits::StaticLookup;
use sp_runtime::Perbill;
use sp_std::prelude::*;
use wasm_instrument::parity_wasm::elements::{Instruction, ValueType};

use pallet_identity::ParentDid;
use polymesh_common_utilities::benchs::{cdd_provider, user, AccountIdOf, User, UserBuilder};
use polymesh_common_utilities::constants::currency::POLY;
use polymesh_common_utilities::group::GroupTrait;
use polymesh_common_utilities::TestUtilsFn;
use polymesh_primitives::asset::AssetId;
use polymesh_primitives::identity::limits::{
    MAX_ASSETS, MAX_EXTRINSICS, MAX_PALLETS, MAX_PORTFOLIOS,
};
use polymesh_primitives::secondary_key::ExtrinsicNames;
use polymesh_primitives::{
    AssetPermissions, Balance, ExtrinsicName, ExtrinsicPermissions, PalletName, PalletPermissions,
    Permissions, PortfolioId, PortfolioNumber, PortfolioPermissions,
};

use crate::chain_extension::*;
use crate::*;

pub(crate) const SEED: u32 = 0;

pub const CHAIN_EXTENSION_BATCHES: u32 = 20;

const ENDOWMENT: Balance = 1_000 * POLY;

const SALT_BYTE: u8 = 0xFF;

pub struct BenchmarkContractPolymeshHooks;

impl<T: Config + TestUtilsFn<<T as SysTrait>::AccountId>> pallet_contracts::PolymeshHooks<T>
    for BenchmarkContractPolymeshHooks
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

    fn register_did(account_id: T::AccountId) -> DispatchResult {
        let cdd_provider_origin = {
            match T::CddServiceProviders::get_members().first() {
                Some(cdd_did) => {
                    let cdd_acc = pallet_identity::Module::<T>::get_primary_key(*cdd_did).unwrap();
                    RawOrigin::Signed(cdd_acc).into()
                }
                None => cdd_provider::<T>("cdd", 0).origin.into(),
            }
        };

        pallet_identity::Module::<T>::cdd_register_did_with_cdd(
            cdd_provider_origin,
            account_id.into(),
            Vec::new(),
            None,
        )
    }
}

/// Construct the default salt used for most benchmarks.
fn salt() -> Vec<u8> {
    vec![SALT_BYTE]
}

/// Create a funded user used by all benchmarks.
fn funded_user<T: Config + TestUtilsFn<AccountIdOf<T>>>(seed: u32) -> User<T> {
    let user = user::<T>("actor", seed);
    T::Currency::make_free_balance_be(&user.account(), 1_000_000 * POLY);
    user
}

/// The `user` instantiates `wasm.code` as the contract with `salt`.
/// Returns the address of the new contract.
fn instantiate<T: Config>(user: &User<T>, wasm: WasmModule<T>, salt: Vec<u8>) -> T::AccountId
where
    T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
{
    let callee = FrameContracts::<T>::contract_address(&user.account(), &wasm.hash, &[], &salt);
    Pallet::<T>::instantiate_with_code_perms(
        user.origin().into(),
        ENDOWMENT,   // endowment
        Weight::MAX, // gas limit
        None,
        wasm.code,
        vec![], // data
        salt,
        Permissions::default(), // Full perms necessary for calling into the runtime.
    )
    .expect("could not create contract");
    callee
}

fn put_storage_value(key: &[u8], len: u32) -> u32 {
    let value = vec![0x00; len as usize];
    unhashed::put_raw(&key, &value);
    // Calculate Encoded lenght: `Option<Vec<u8>>`
    Some(value).encoded_size() as u32
}

fn secondary_key_permission(
    n_assets: u64,
    n_portfolios: u128,
    n_extrinsics: u64,
    n_pallets: u64,
) -> Permissions {
    let asset = AssetPermissions::elems((0..n_assets).map(|i| AssetId::new([i as u8; 16])));
    let portfolio = PortfolioPermissions::elems(
        (0..n_portfolios).map(|did| PortfolioId::user_portfolio(did.into(), PortfolioNumber(0))),
    );
    let extrinsics = ExtrinsicNames::elems((0..n_extrinsics).map(|e| ExtrinsicName::generate(e)));
    let extrinsic = ExtrinsicPermissions::these((0..n_pallets).map(|p| {
        (
            PalletName::generate(p),
            PalletPermissions {
                extrinsics: extrinsics.clone(),
            },
        )
    }));
    Permissions {
        asset,
        extrinsic,
        portfolio,
    }
}

struct Contract<T: Config> {
    caller: User<T>,
    addr: <T::Lookup as StaticLookup>::Source,
}

impl<T> Contract<T>
where
    T: Config + TestUtilsFn<AccountIdOf<T>>,
    T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
{
    pub fn new(wasm: WasmModule<T>) -> Self {
        // Construct a user
        let caller = UserBuilder::<T>::default()
            .seed(SEED)
            .generate_did()
            .become_cdd_provider()
            .build("Caller");
        T::Currency::make_free_balance_be(&caller.account(), 1_000_000 * POLY);

        // Instantiate the contract.
        let account_id = instantiate::<T>(&caller, wasm, salt());

        Self {
            caller,
            addr: T::Lookup::unlookup(account_id),
        }
    }

    /// Creates a contract that will call `seal_call_chain_extension' with `FuncId::GetKeyDid`.
    fn new_seal_chain_extension(
        repetitions: u32,
        input: Vec<u8>,
        key_len: u32,
        output_len: usize,
    ) -> Self {
        let code = WasmModule::<T>::from(ModuleDefinition {
            memory: Some(ImportedMemory::max::<T>()),
            imported_functions: vec![ImportedFunction {
                module: "seal0",
                name: "seal_call_chain_extension",
                params: vec![ValueType::I32; 5],
                return_type: Some(ValueType::I32),
            }],
            data_segments: vec![
                DataSegment {
                    offset: 0,
                    value: input.clone(),
                },
                DataSegment {
                    offset: input.len() as u32,
                    value: output_len.to_le_bytes().into(),
                },
            ],
            call_body: Some(body::repeated_dyn(
                repetitions,
                vec![
                    Regular(Instruction::I32Const(FuncId::GetKeyDid.into())),
                    Counter(0, key_len),
                    Regular(Instruction::I32Const(key_len as i32)),
                    Regular(Instruction::I32Const(input.len() as i32 + 4)),
                    Regular(Instruction::I32Const(input.len() as i32)),
                    Regular(Instruction::Call(0)),
                    Regular(Instruction::Drop),
                ],
            )),
            ..Default::default()
        });
        Self::new(code)
    }

    /// Create and setup a contract to call the ChainExtension.
    fn chain_extension(repeat: u32, func_id: FuncId, input: Vec<u8>, out_len: u32) -> Self {
        let in_len = input.len() as u32;
        let out_len_ptr = in_len;
        let out_len_vec = out_len.to_le_bytes().to_vec();
        let out_ptr = out_len_ptr + out_len_vec.len() as u32;
        let wasm = WasmModule::<T>::from(ModuleDefinition {
            memory: Some(ImportedMemory::max::<T>()),
            data_segments: vec![
                // Input
                DataSegment {
                    offset: 0,
                    value: input,
                },
                // Output Length
                DataSegment {
                    offset: out_len_ptr,
                    value: out_len_vec,
                },
            ],
            // Import `seal_call_chain_extension`.
            imported_functions: vec![ImportedFunction {
                module: "seal0",
                name: "seal_call_chain_extension",
                params: vec![ValueType::I32; 5],
                return_type: Some(ValueType::I32),
            }],
            // Call `seal_call_chain_extension` with the given `func_id`, and `input`.
            call_body: Some(body::repeated(
                repeat,
                &[
                    Instruction::I32Const(func_id.into()),
                    Instruction::I32Const(0), // in_ptr
                    Instruction::I32Const(in_len as i32),
                    Instruction::I32Const(out_ptr as i32),
                    Instruction::I32Const(out_len_ptr as i32),
                    Instruction::Call(0), // Call `seal_call_chain_extension`.
                    Instruction::Drop,
                ],
            )),
            ..Default::default()
        });
        Self::new(wasm)
    }

    /// Create and setup a contract to call the ChainExtension KeyHasher.
    fn key_hasher(repeat: u32, hasher: KeyHasher, size: HashSize, in_len: u32) -> Self {
        let out_len = match size {
            HashSize::B64 => 8,
            HashSize::B128 => 16,
            HashSize::B256 => 32,
        };
        let func_id = FuncId::KeyHasher(hasher, size);
        let input = vec![0x00; in_len as usize];
        Self::chain_extension(repeat, func_id, input, out_len)
    }

    #[track_caller]
    pub fn call(&self) {
        FrameContracts::<T>::call(
            self.caller.origin().into(),
            self.addr.clone(),
            0,
            Weight::MAX,
            None,
            vec![],
        )
        .unwrap();
    }
}

benchmarks! {
    where_clause { where
        T: frame_system::Config,
        T: TestUtilsFn<AccountIdOf<T>>,
        T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
    }

    chain_extension_read_storage {
        let k in 1 .. T::MaxInLen::get() as u32;
        let v in 1 .. T::MaxOutLen::get() as u32;

        // Generate a raw storage key and put a value in it.
        let key = (0..k).map(|k| k as u8).collect::<Vec<u8>>();
        let out_len = put_storage_value(&key, v);
        // Setup ChainExtension.
        let contract = Contract::<T>::chain_extension(1, FuncId::ReadStorage, key, out_len);
    }: {
        contract.call();
    }

    // Benchmark ChainExtension GetSpecVersion and GetTransactionVersion.
    chain_extension_get_version {
        let r in 0 .. CHAIN_EXTENSION_BATCHES;

        // Setup ChainExtension.
        let contract = Contract::<T>::chain_extension(r * CHAIN_EXTENSION_BATCH_SIZE, FuncId::GetSpecVersion, vec![], 4);
    }: {
        contract.call();
    }

    // Benchmark ChainExtension GetKeyDid.
    chain_extension_get_key_did {
        let r in 1..CHAIN_EXTENSION_BATCHES;

        let secondary_key_permission = secondary_key_permission(
            MAX_ASSETS as u64,
            MAX_PORTFOLIOS as u128,
            MAX_EXTRINSICS as u64,
            MAX_PALLETS as u64
        );

        let encoded_accounts = (0..r * CHAIN_EXTENSION_BATCH_SIZE)
            .map(|i| {
                let primary_user = funded_user::<T>(SEED + i);
                let secondary_key: T::AccountId = account("key", i, SEED);
                Identity::<T>::unsafe_join_identity(
                    primary_user.did(),
                    secondary_key_permission.clone(),
                    secondary_key.clone(),
                );
                secondary_key.encode()
            })
            .collect::<Vec<_>>();
        let account_len = encoded_accounts.get(0).map(|acc| acc.len()).unwrap_or(0) as u32;
        let accounts_bytes = encoded_accounts.iter().flat_map(|a| a.clone()).collect::<Vec<_>>();

        let contract = Contract::<T>::new_seal_chain_extension(
            r * CHAIN_EXTENSION_BATCH_SIZE,
            accounts_bytes,
            account_len,
            33
        );
    }: {
        contract.call();
    }

    chain_extension_hash_twox_64 {
        let r in 0 .. CHAIN_EXTENSION_BATCHES;

        // Setup ChainExtension.
        let contract = Contract::<T>::key_hasher(r * CHAIN_EXTENSION_BATCH_SIZE, KeyHasher::Twox, HashSize::B64, 0);
    }: {
        contract.call();
    }

    chain_extension_hash_twox_64_per_kb {
        let n in 0 .. max_pages::<T>() * 4;

        // Setup ChainExtension.
        let contract = Contract::<T>::key_hasher(CHAIN_EXTENSION_BATCH_SIZE, KeyHasher::Twox, HashSize::B64, n * 1024);
    }: {
        contract.call();
    }

    chain_extension_hash_twox_128 {
        let r in 0 .. CHAIN_EXTENSION_BATCHES;

        // Setup ChainExtension.
        let contract = Contract::<T>::key_hasher(r * CHAIN_EXTENSION_BATCH_SIZE, KeyHasher::Twox, HashSize::B128, 0);
    }: {
        contract.call();
    }

    chain_extension_hash_twox_128_per_kb {
        let n in 0 .. max_pages::<T>() * 4;

        // Setup ChainExtension.
        let contract = Contract::<T>::key_hasher(CHAIN_EXTENSION_BATCH_SIZE, KeyHasher::Twox, HashSize::B128, n * 1024);
    }: {
        contract.call();
    }

    chain_extension_hash_twox_256 {
        let r in 0 .. CHAIN_EXTENSION_BATCHES;

        // Setup ChainExtension.
        let contract = Contract::<T>::key_hasher(r * CHAIN_EXTENSION_BATCH_SIZE, KeyHasher::Twox, HashSize::B256, 0);
    }: {
        contract.call();
    }

    chain_extension_hash_twox_256_per_kb {
        let n in 0 .. max_pages::<T>() * 4;

        // Setup ChainExtension.
        let contract = Contract::<T>::key_hasher(CHAIN_EXTENSION_BATCH_SIZE, KeyHasher::Twox, HashSize::B256, n * 1024);
    }: {
        contract.call();
    }

    chain_extension_call_runtime {
        let n in 1 .. (T::MaxInLen::get() as u32 - 4);

        // Encode `System::remark(remark: Vec<u8>)` call.
        let input = (0u8 /* System */, 0u8 /* remark */, vec![b'A'; n as usize]).encode();
        // Setup ChainExtension.
        let contract = Contract::<T>::chain_extension(1, FuncId::CallRuntime, input, 0);
    }: {
        contract.call();
    }

    // Measure overhead of calling a contract.
    dummy_contract {
        // Setup dummy contract
        let wasm = WasmModule::<T>::dummy();
        let contract = Contract::<T>::new(wasm);
    }: {
        contract.call();
    }

    basic_runtime_call {
        let n in 1 .. (T::MaxInLen::get() as u32 - 4);

        let user = funded_user::<T>(SEED);
        let remark = vec![b'A'; n as usize];
        let origin = user.origin().into();
    }: {
        System::<T>::remark(origin, remark).unwrap();
    }

    base_weight_with_hash {
            let i in 0 .. max_pages::<T>() * 64 * 1024;
            let s in 0 .. max_pages::<T>() * 64 * 1024;
            let input = vec![42u8; i as usize];
            let salt = vec![42u8; s as usize];

        // Have the user instantiate a dummy contract.
        let wasm = WasmModule::<T>::dummy();
        let code = Code::Existing(wasm.hash.clone());

        let user = funded_user::<T>(SEED);
        let caller = user.account();
        let perms = Some(Permissions::default());
    }: {
        Module::<T>::base_weight_and_contract_address(&caller, &code, &input, &salt, perms.as_ref())
    }

    base_weight_with_code {
            let c in 0 .. Perbill::from_percent(49).mul_ceil(T::MaxCodeLen::get());
            let i in 0 .. max_pages::<T>() * 64 * 1024;
            let s in 0 .. max_pages::<T>() * 64 * 1024;

            let input = vec![42u8; i as usize];
            let salt = vec![42u8; s as usize];

        // Create a dummy contract.
        let wasm = WasmModule::<T>::sized(c, Location::Deploy);
        let code = Code::Upload(wasm.code.clone());

        let user = funded_user::<T>(SEED);
        let caller = user.account();
        let perms = Some(Permissions::default());
    }: {
        Module::<T>::base_weight_and_contract_address(&caller, &code, &input, &salt, perms.as_ref())
    }

    update_call_runtime_whitelist {
        let u in 0 .. 2000;

        let updates = (0..u)
            .map(|id| ([(id & 0xFF) as u8, (id >> 8) as u8].into(), true))
            .collect();
    }: _(RawOrigin::Root, updates)

    link_contract_as_secondary_key {
        let alice = UserBuilder::<T>::default()
            .generate_did()
            .become_cdd_provider()
            .build("Alice");
        T::Currency::make_free_balance_be(&alice.account(), 1_000_000 * POLY);
        let caller = alice.account();

        let wasm = WasmModule::<T>::dummy();
        let addr = FrameContracts::<T>::contract_address(&caller, &wasm.hash, &[], &[]);
        let perms = Some(Permissions::default());
    }: {
        Module::<T>::link_contract_to_did(&caller, addr, perms, false)?;
    }

    link_contract_as_primary_key {
        let alice = UserBuilder::<T>::default()
            .generate_did()
            .become_cdd_provider()
            .build("Alice");
        T::Currency::make_free_balance_be(&alice.account(), 1_000_000 * POLY);
        let caller = alice.account();

        let wasm = WasmModule::<T>::dummy();
        let addr = FrameContracts::<T>::contract_address(&caller, &wasm.hash, &[], &[]);
    }: {
        Module::<T>::link_contract_to_did(&caller, addr, None, true)?;
    }

    upgrade_api {
        let current_spec_version = T::Version::get().spec_version;
        let current_tx_version = T::Version::get().transaction_version;
        let api_code_hash: ApiCodeHash<T> = ApiCodeHash { hash: CodeHash::<T>::default() };
        let chain_version = ChainVersion::new(current_spec_version, current_tx_version);
        let api = Api::new(*b"POLY", current_spec_version);
        let next_upgrade = NextUpgrade::new(chain_version, api_code_hash);
    }: _(RawOrigin::Root, api.clone(), next_upgrade.clone())
    verify {
        assert_eq!(ApiNextUpgrade::<T>::get(&api).unwrap(), next_upgrade);
        assert_eq!(CurrentApiHash::<T>::get(&api),None);
    }

    chain_extension_get_latest_api_upgrade {
        let r in 0 .. CHAIN_EXTENSION_BATCHES;

        let current_spec_version = T::Version::get().spec_version;
        let current_tx_version = T::Version::get().transaction_version;

        let api_code_hash: ApiCodeHash<T> = ApiCodeHash { hash: CodeHash::<T>::default() };
        let next_upgrade = NextUpgrade::new(ChainVersion::new(current_spec_version, current_tx_version), api_code_hash.clone());
        let output_len: u32 = api_code_hash.hash.as_ref().len() as u32;
        let api = Api::new(*b"POLY", current_spec_version);

        Module::<T>::upgrade_api(
            RawOrigin::Root.into(),
            api.clone(),
            next_upgrade.clone(),
        ).unwrap();

        let encoded_input = (0..r * CHAIN_EXTENSION_BATCH_SIZE)
            .map(|_| {
                api.encode()
            })
            .collect::<Vec<_>>();
        let input_bytes =  encoded_input.iter().flat_map(|a| a.clone()).collect::<Vec<_>>();

        let contract = Contract::<T>::chain_extension(
            r * CHAIN_EXTENSION_BATCH_SIZE,
            FuncId::GetLatestApiUpgrade,
            input_bytes,
            output_len
        );
    }: { contract.call(); }

    chain_extension_get_next_asset_id {
        let r in 0..CHAIN_EXTENSION_BATCHES;

        let encoded_input = (0..r * CHAIN_EXTENSION_BATCH_SIZE)
            .map(|i| funded_user::<T>(SEED + i).account().encode())
            .collect::<Vec<_>>();
        let input_bytes =  encoded_input.iter().flat_map(|a| a.clone()).collect::<Vec<_>>();

        let contract = Contract::<T>::chain_extension(r * CHAIN_EXTENSION_BATCH_SIZE, FuncId::GetNextAssetId, input_bytes, 16);
    }: {
        contract.call();
    }
}
