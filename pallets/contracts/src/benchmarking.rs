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

use crate::*;

use codec::Encode;
use frame_benchmarking::benchmarks;
use frame_support::traits::{tokens::currency::Currency, ReservableCurrency};
use pallet_asset::Pallet as Asset;
use pallet_contracts::benchmarking::code::{
    body, max_pages, DataSegment, ImportedFunction, ImportedMemory, Location, ModuleDefinition,
    WasmModule,
};
use pallet_contracts::Pallet as FrameContracts;
use polymesh_common_utilities::{
    benchs::{user, AccountIdOf, User},
    constants::currency::POLY,
    TestUtilsFn,
};
use polymesh_primitives::{Balance, Permissions};
use pwasm_utils::parity_wasm::elements::{Instruction, ValueType};
use sp_runtime::Perbill;
use sp_std::prelude::*;

pub(crate) const SEED: u32 = 0;

const ENDOWMENT: Balance = 1_000 * POLY;

const SALT_BYTE: u8 = 0xFF;

/// Construct the default salt used for most benchmarks.
fn salt() -> Vec<u8> {
    vec![SALT_BYTE]
}

/// Create a funded user used by all benchmarks.
fn funded_user<T: Config + TestUtilsFn<AccountIdOf<T>>>() -> User<T> {
    let user = user::<T>("actor", SEED);
    T::Currency::make_free_balance_be(&user.account(), 1_000_000 * POLY);
    user
}

/// Returns the free balance of `acc`.
fn free_balance<T: Config>(acc: &T::AccountId) -> Balance {
    T::Currency::free_balance(&acc)
}

/// Returns the reserved balance of `acc`.
fn reserved_balance<T: Config>(acc: &T::AccountId) -> Balance
where
    T::Currency: ReservableCurrency<T::AccountId>,
{
    T::Currency::reserved_balance(&acc)
}

/// The `user` instantiates `wasm.code` as the contract with `salt`.
/// Returns the address of the new contract.
fn instantiate<T: Config>(user: &User<T>, wasm: WasmModule<T>, salt: Vec<u8>) -> T::AccountId {
    let callee = FrameContracts::<T>::contract_address(&user.account(), &wasm.hash, &salt);
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

/// Returns a module definition that will import and call `seal_call_chain_extension`.
fn chain_extension_module_def(func_id: i32, in_ptr: i32, in_len: i32) -> ModuleDefinition {
    ModuleDefinition {
        // Import `seal_call_chain_extension`.
        imported_functions: vec![ImportedFunction {
            module: "seal0",
            name: "seal_call_chain_extension",
            params: vec![ValueType::I32; 5],
            return_type: Some(ValueType::I32),
        }],
        // Call `seal_call_chain_extension` with the given `func_id`, `in_ptr`, and `in_len`.
        call_body: Some(body::plain(vec![
            Instruction::I32Const(func_id),
            Instruction::I32Const(in_ptr),
            Instruction::I32Const(in_len),
            Instruction::I32Const(0), // `output_ptr`
            Instruction::I32Const(0), // `output_len`
            Instruction::Call(0),     // Call `seal_call_chain_extension`, assumed to be at `0`.
            Instruction::Drop,
            Instruction::End,
        ])),
        ..Default::default()
    }
}

/// Make the `prepare_instantiate` input.
fn prepare_input(n: u32) -> Vec<u8> {
    // For simplicity, we assume the salt is `n` long and the rest is 0 long.
    let hash = Vec::<u8>::new();
    let salt = vec![b'A'; n as usize];
    let perms = Permissions::default();
    (hash, salt, perms).encode()
}

benchmarks! {
    where_clause { where
        T: pallet_asset::Config,
        <T as pallet_contracts::Config>::Currency: ReservableCurrency<T::AccountId>,
        T: TestUtilsFn<AccountIdOf<T>>,
    }

    prepare_instantiate_full {
        let n in 1 .. T::MaxLen::get().saturating_sub(prepare_input(0).len() as u32);

        // Construct a user doing everything.
        let user = funded_user::<T>();

        // Construct our contract.
        let input = prepare_input(n);
        let def = chain_extension_module_def(0x_00_00_00_00u32 as i32, 0, input.len() as i32);
        let wasm = WasmModule::<T>::from(ModuleDefinition {
            memory: Some(ImportedMemory::max::<T>()),
            data_segments: vec![DataSegment { offset: 0, value: input }],
            ..def
        });

        // Instantiate the contract.
        let callee = instantiate::<T>(&user, wasm, salt());
    }: call(user.origin(), callee.clone(), 0, Weight::MAX, None, vec![])

    chain_extension_full {
        let n in 1 .. T::MaxLen::get() as u32;

        // Construct a user doing everything.
        let user = funded_user::<T>();

        // Construct our contract.
        let input = vec![b'A'; n as usize].encode();
        let def = chain_extension_module_def(0x_00_01_11_00, 0, input.len() as i32);
        let wasm = WasmModule::<T>::from(ModuleDefinition {
            memory: Some(ImportedMemory::max::<T>()),
            data_segments: vec![DataSegment { offset: 0, value: input }],
            ..def
        });

        // Instantiate the contract.
        let callee = instantiate::<T>(&user, wasm, salt());
    }: call(user.origin(), callee.clone(), 0, Weight::MAX, None, vec![])

    chain_extension_early_exit {
        // Construct a user doing everything.
        let user = funded_user::<T>();

        // Construct our contract.
        let wasm = WasmModule::<T>::from(chain_extension_module_def(0, 0, 0));

        // Instantiate the contract.
        let callee = instantiate::<T>(&user, wasm, salt());
    }: call(user.origin(), callee.clone(), 0, Weight::MAX, None, vec![])

    basic_runtime_call {
        let n in 1 .. T::MaxLen::get() as u32;

        let user = funded_user::<T>();
        let custom_type = vec![b'A'; n as usize];
        let origin = user.origin().into();
    }: {
        Asset::<T>::register_custom_asset_type(origin, custom_type).unwrap();
    }

    call {
        // Construct a user doing everything.
        let user = funded_user::<T>();

        // Instantiate a dummy contract.
        let callee = instantiate::<T>(&user, WasmModule::<T>::dummy(), salt());
        let before = free_balance::<T>(&callee);

        // Arguments to pass to contract call.
        let data = vec![42u8; 1024];
    }: _(user.origin(), callee.clone(), ENDOWMENT, Weight::MAX, None, data)
    verify {
        // Contract should have received the value.
        assert_eq!(free_balance::<T>(&callee), before + ENDOWMENT);
    }

    // Use a dummy contract constructor to measure the overhead.
    // `s`: Size of the salt in kilobytes.
    instantiate_with_hash {
        let s in 0 .. max_pages::<T>() * 64 * 1024;
        let other_salt = vec![42u8; s as usize];

        // Construct a user doing everything.
        let user = funded_user::<T>();

        // Have the user instantiate a dummy contract.
        let wasm = WasmModule::<T>::dummy();
        let hash = wasm.hash.clone();
        let addr = FrameContracts::<T>::contract_address(&user.account(), &hash, &other_salt);

        // Pre-instantiate a contract so that one with the hash exists.
        let _ = instantiate::<T>(&user, wasm, salt());
    }: instantiate(user.origin(), ENDOWMENT, Weight::MAX, None, hash, vec![], other_salt)
    verify {
        // Ensure contract has the full value.
        assert_eq!(free_balance::<T>(&addr), ENDOWMENT);
    }

    // This constructs a contract that is maximal expensive to instrument.
    // It creates a maximum number of metering blocks per byte.
    // The size of the salt influences the runtime because is is hashed in order to
    // determine the contract address. All code is generated to the `call` function so that
    // we don't benchmark the actual execution of this code but merely what it takes to load
    // a code of that size into the sandbox.
    //
    // `c`: Size of the code in kilobytes.
    // `s`: Size of the salt in kilobytes.
    //
    // # Note
    //
    // We cannot let `c` grow to the maximum code size because the code is not allowed
    // to be larger than the maximum size **after instrumentation**.
    instantiate_with_code {
        let c in 0 .. Perbill::from_percent(50).mul_ceil(T::Schedule::get().limits.code_len);
        let s in 0 .. max_pages::<T>() * 64 * 1024;
        let salt = vec![42u8; s as usize];

        // Construct a user doing everything.
        let user = funded_user::<T>();

        // Construct the contract code + get addr.
        let wasm = WasmModule::<T>::sized(c, Location::Deploy);
        let addr = FrameContracts::<T>::contract_address(&user.account(), &wasm.hash, &salt);
    }: _(user.origin(), ENDOWMENT, Weight::MAX, None, wasm.code, vec![], salt)
    verify {
        // Ensure contract has the full value.
        assert_eq!(free_balance::<T>(&addr), ENDOWMENT);
    }

    upload_code {
        let c in 0 .. Perbill::from_percent(50).mul_ceil(T::Schedule::get().limits.code_len);
        // Construct a user doing everything.
        let user = funded_user::<T>();

        let WasmModule { code, hash, .. } = WasmModule::<T>::sized(c, Location::Call);
    }: _(user.origin(), code, None)
    verify {
        // Uploading code reserves some balance in the callers account.
        assert!(reserved_balance::<T>(&user.account) > 0u32.into());
        //assert!(<FrameContracts<T>>::code_exists(&hash));
    }

    remove_code {
        // Construct a user doing everything.
        let user = funded_user::<T>();

        // Upload a dummy contract.
        let WasmModule { code, hash, .. } = WasmModule::<T>::dummy();
        let uploaded = <FrameContracts<T>>::bare_upload_code(user.account(), code, None)?;
        assert_eq!(uploaded.code_hash, hash);
        assert_eq!(uploaded.deposit, reserved_balance::<T>(&user.account()));
        //assert!(<FrameContracts<T>>::code_exists(&hash));
    }: _(user.origin(), hash)
    verify {
        // Removing the code should have unreserved the deposit.
        assert_eq!(reserved_balance::<T>(&user.account), 0u32.into());
        //assert!(<FrameContracts<T>>::code_removed(&hash));
    }
}
