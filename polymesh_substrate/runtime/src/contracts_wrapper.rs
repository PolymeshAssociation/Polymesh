use rstd::prelude::*;

use contracts::{CodeHash, Schedule, Gas};
use codec::Encode;
use sr_primitives::traits::StaticLookup;
use srml_support::traits::Currency;
use srml_support::{decl_module, decl_storage, dispatch::Result, ensure};
use system::ensure_signed;

pub type BalanceOf<T> =
    <<T as contracts::Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

pub trait Trait: contracts::Trait {

}

decl_storage! {
    trait Store for Module<T: Trait> as ContractsWrapper {
    }
}

decl_module! {
    // Wrap dispatchable functions for contracts so that we can add additional gating logic
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        // Simply forwards to the `update_schedule` function in the Contract module.
        pub fn update_schedule(origin, schedule: Schedule) -> Result {
            <contracts::Module<T>>::update_schedule(origin, schedule)
        }

        // Simply forwards to the `put_code` function in the Contract module.
        pub fn put_code(
            origin,
            #[compact] gas_limit: Gas,
            code: Vec<u8>
        ) -> Result {
            <contracts::Module<T>>::put_code(origin, gas_limit, code)
        }

        // Simply forwards to the `call` function in the Contract module.
        pub fn call(
            origin,
            dest: <T::Lookup as StaticLookup>::Source,
            #[compact] value: BalanceOf<T>,
            #[compact] gas_limit: Gas,
            data: Vec<u8>
        ) -> Result {
            <contracts::Module<T>>::call(origin, dest, value, gas_limit, data)
        }

        // Simply forwards to the `instantiate` function in the Contract module.
		pub fn instantiate(
			origin,
			#[compact] endowment: BalanceOf<T>,
			#[compact] gas_limit: Gas,
			code_hash: CodeHash<T>,
			data: Vec<u8>
		) -> Result {
            <contracts::Module<T>>::instantiate(origin, endowment, gas_limit, code_hash, data)
		}        
    }
}

impl<T: Trait> Module<T> {
}