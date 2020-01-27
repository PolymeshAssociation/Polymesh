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
//!   - When code is instantiated enforce a POLY fee to the DID owning the code (i.e. that executed put_code)

use crate::identity;
use primitives::{IdentityId, Key, Signer};

use codec::Encode;
use frame_support::traits::Currency;
use frame_support::{decl_module, decl_storage, dispatch::DispatchResult, ensure};
use frame_system::ensure_signed;
use pallet_contracts::{CodeHash, Gas, Schedule};
use sp_runtime::traits::StaticLookup;
use sp_std::{convert::TryFrom, prelude::*};

// pub type CodeHash<T> = <T as frame_system::Trait>::Hash;

pub type BalanceOf<T> = <<T as pallet_contracts::Trait>::Currency as Currency<
    <T as frame_system::Trait>::AccountId,
>>::Balance;

pub trait Trait: pallet_contracts::Trait + identity::Trait {}

decl_storage! {
    trait Store for Module<T: Trait> as ContractsWrapper {
        pub CodeHashDid: map CodeHash<T> => Option<IdentityId>;
    }
}

decl_module! {
    // Wrap dispatchable functions for contracts so that we can add additional gating logic
    // TODO: Figure out how to remove dispatchable calls from the underlying contracts module
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        // Simply forwards to the `update_schedule` function in the Contract module.
        pub fn update_schedule(origin, schedule: Schedule) -> DispatchResult {
            <pallet_contracts::Module<T>>::update_schedule(origin, schedule)
        }

        // Simply forwards to the `put_code` function in the Contract module.
        pub fn put_code(
            origin,
            did: IdentityId,
            #[compact] gas_limit: Gas,
            code: Vec<u8>
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let signer = Signer::Key( Key::try_from(sender.encode())?);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &signer), "sender must be a signing key for DID");

            // Call underlying function
            let new_origin = frame_system::RawOrigin::Signed(sender).into();
            <pallet_contracts::Module<T>>::put_code(new_origin, gas_limit, code)
        }

        // Simply forwards to the `call` function in the Contract module.
        pub fn call(
            origin,
            dest: <T::Lookup as StaticLookup>::Source,
            #[compact] value: BalanceOf<T>,
            #[compact] gas_limit: Gas,
            data: Vec<u8>
        ) -> DispatchResult {
            <pallet_contracts::Module<T>>::call(origin, dest, value, gas_limit, data)
        }

        // Simply forwards to the `instantiate` function in the Contract module.
        pub fn instantiate(
            origin,
            #[compact] endowment: BalanceOf<T>,
            #[compact] gas_limit: Gas,
            code_hash: CodeHash<T>,
            data: Vec<u8>
        ) -> DispatchResult {
            <pallet_contracts::Module<T>>::instantiate(origin, endowment, gas_limit, code_hash, data)
        }
    }
}

impl<T: Trait> Module<T> {}
