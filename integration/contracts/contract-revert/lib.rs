#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;

#[ink::contract]
mod revert_tester {
    use crate::*;

    /// A simple contract.
    #[ink(storage)]
    pub struct RevertTester {}

    /// The contract error types.
    #[derive(Debug, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Revert contract.
        Revert,
    }

    /// The contract result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl RevertTester {
        /// Creates a new contract.
        #[ink(constructor)]
        pub fn new(return_error: bool) -> Result<Self> {
            if return_error {
                Err(Error::Revert)
            } else {
                Ok(Self {})
            }
        }

        #[ink(message)]
        pub fn test(&mut self, return_error: bool) -> Result<()> {
            if return_error {
                Err(Error::Revert)
            } else {
                Ok(())
            }
        }
    }
}
