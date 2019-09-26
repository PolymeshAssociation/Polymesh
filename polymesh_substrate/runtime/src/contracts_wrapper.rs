use codec::Encode;
use rstd::prelude::*;
use srml_support::traits::Currency;
use srml_support::{decl_event, decl_module, decl_storage, dispatch::Result, ensure, StorageValue};
use sr_primitives::traits::StaticLookup;
use system::{self, ensure_signed};

use contracts::{CodeHash, Schedule, Gas};

pub type BalanceOf<T> =
    <<T as contracts::Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

/// The module's configuration trait.
pub trait Trait: contracts::Trait {

}

decl_storage! {
    trait Store for Module<T: Trait> as ContractsWrapper {

    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        // this is needed only if you are using events in your module
        //fn deposit_event() = default;

        /// Simply forwards to the `update_schedule` function in the Contract module.
        pub fn update_schedule(origin, schedule: Schedule) -> Result {
            <contracts::Module<T>>::update_schedule(origin, schedule)
        }


        /// Checks that sender is the Sudo `key` before forwarding to `put_code` in the Contract module.
        pub fn put_code(
            origin,
            #[compact] gas_limit: Gas,
            code: Vec<u8>
        ) -> Result {
            <contracts::Module<T>>::put_code(origin, gas_limit, code)
        }


        /// Simply forwards to the `call` function in the Contract module.
        pub fn call(
            origin,
            dest: <T::Lookup as StaticLookup>::Source,
            #[compact] value: BalanceOf<T>,
            #[compact] gas_limit: Gas,
            data: Vec<u8>
        ) -> Result {
            <contracts::Module<T>>::call(origin, dest, value, gas_limit, data)
        }

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

/// tests for this module
#[cfg(test)]
mod tests {
    /*
     *    use super::*;
     *
     *    use substrate_primitives::{Blake2Hasher, H256};
     *    use sr_io::with_externalities;
     *    use sr_primitives::{
     *        testing::{Digest, DigestItem, Header},
     *        traits::{BlakeTwo256, IdentityLookup},
     *        BuildStorage,
     *    };
     *    use srml_support::{assert_ok, impl_outer_origin};
     *
     *    impl_outer_origin! {
     *        pub enum Origin for Test {}
     *    }
     *
     *    // For testing the module, we construct most of a mock runtime. This means
     *    // first constructing a configuration type (`Test`) which `impl`s each of the
     *    // configuration traits of modules we want to use.
     *    #[derive(Clone, Eq, PartialEq)]
     *    pub struct Test;
     *    impl system::Trait for Test {
     *        type Origin = Origin;
     *        type Index = u64;
     *        type BlockNumber = u64;
     *        type Hash = H256;
     *        type Hashing = BlakeTwo256;
     *        type Digest = H256;
     *        type AccountId = u64;
     *        type Lookup = IdentityLookup<Self::AccountId>;
     *        type Header = Header;
     *        type Event = ();
     *        type Log = DigestItem;
     *    }
     *    impl Trait for Test {
     *        type Event = ();
     *    }
     *    type TransferValidationModule = Module<Test>;
     *
     *    // This function basically just builds a genesis storage key/value store according to
     *    // our desired mockup.
     *    fn new_test_ext() -> sr_io::TestExternalities<Blake2Hasher> {
     *        system::GenesisConfig::default()
     *            .build_storage()
     *            .unwrap()
     *            .0
     *            .into()
     *    }
     */
}
