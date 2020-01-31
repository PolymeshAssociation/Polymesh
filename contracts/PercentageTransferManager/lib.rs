#![feature(proc_macro_hygiene)]
#![cfg_attr(not(feature = "std"), no_std)]

use ink_prelude::{
    format
};

use ink_core::storage;
use ink_lang2 as ink;

#[ink::contract(version = "0.1.0")]
mod PercentageTransferManager {
    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    struct PercentageTransferManagerStorage {
        /// Maximum allowed percentage of the tokens hold by an investor
        /// %age is based on the total supply of the asset.
        max_allowed_percentage: storage::Value<u128>,
    }

    /// Copy of the custom type defined in `/runtime/src/template.rs`.
    ///
    /// # Requirements
    /// In order to decode a value of that type from the runtime storage:
    ///   - The type must match exactly the custom type defined in the runtime
    ///   - It must implement `Decode`, usually by deriving it as below
    ///   - It should implement `Metadata` for use with `generate-metadata` (required for the UI).
    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
    pub enum RestrictionResult {
        VALID,
        INVALID,
        NA,
        FORCE_VALID
    }

    impl PercentageTransferManagerStorage {
        /// Constructor that initializes the `u128` value to the given `max_allowed_percentage`.
        #[ink(constructor)]
        fn new(&mut self, max_percentage: u128) {
            self.max_allowed_percentage.set(max_percentage);
        }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message)]
        fn verify_transfer(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
            balance_from: Balance,
            balance_to: Balance,
            total_supply: Balance
        ) -> RestrictionResult {
            
            if (balance_to + value * 100) / total_supply > *self.max_allowed_percentage.get() {
                return RestrictionResult::NA;
            } else {
                return RestrictionResult::INVALID;
            }
        }

        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        fn get_max_allowed_percentage(&self) -> u128 {
            self.env().println(&format!("number of max holders: {:?}", *self.max_allowed_percentage.get()));
            *self.max_allowed_percentage.get()
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        #[test]
        fn default_works() {
            // Note that even though we defined our `#[ink(constructor)]`
            // above as `&mut self` functions that return nothing we can call
            // them in test code as if they were normal Rust constructors
            // that take no `self` argument but return `Self`.
            let PercentageTransferManager = PercentageTransferManager::default();
            assert_eq!(PercentageTransferManager.get(), false);
        }

        /// We test a simple use case of our contract.
        #[test]
        fn it_works() {
            let mut PercentageTransferManager = PercentageTransferManager::new(false);
            assert_eq!(PercentageTransferManager.get(), false);
            PercentageTransferManager.flip();
            assert_eq!(PercentageTransferManager.get(), true);
        }
    }
}
