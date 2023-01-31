#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod vesting {
    use ink_storage::{traits::SpreadAllocate, Mapping};

    /// Defines the storage of your contract.
    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct Vesting {
        released: Balance,
        beneficiary: AccountId,
        start: u64,
        duration: u64,
    }

    /// Event emitted when Polyx is released.
    #[ink(event)]
    pub struct PolyxReleased {
        value: Balance,
    }

    /// The error types.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Returned if not enough balance to fulfill a request is available.
        InsufficientBalance,
    }

    impl Vesting {
        /// Constructor
        #[ink(constructor)]
        pub fn new(
            beneficiaryAddress: AccountId,
            startTimestamp: u64,
            durationSeconds: u64,
        ) -> Self {
            ink_lang::utils::initialize_contract(|contract| {
                Self::new_init(
                    contract,
                    beneficiaryAddress,
                    startTimestamp,
                    durationSeconds,
                )
            })
        }

        fn new_init(
            &mut self,
            beneficiaryAddress: AccountId,
            startTimestamp: u64,
            durationSeconds: u64,
        ) {
            self.beneficiary = beneficiaryAddress;
            self.start = startTimestamp;
            self.duration = durationSeconds;
        }

        // Getters
        /// Returns the vesting duration.
        #[ink(message)]
        pub fn duration(&self) -> Balance {
            self.duration
        }

        /// Returns the start timestamp.
        #[ink(message)]
        pub fn start(&self) -> Balance {
            self.start
        }

        /// Returns the beneficiary address.
        #[ink(message)]
        pub fn beneficiary(&self) -> Balance {
            self.beneficiary
        }

        /// Returns the amount of POLYX already released.
        #[ink(message)]
        pub fn released(&self) -> Balance {
            self.released
        }

        /// Returns the amount of releasable POLYX.
        #[ink(message)]
        pub fn releasable(&self) -> Balance {
            vestedAmount(self, self.env.block_timestamp()).saturating_sub(released(self))
        }

        /// Release the native token (POLYX) that have already vested.
        #[ink(message)]
        pub fn release(&mut self) {
            let amount = releasable(self);
            self.released += amount;
            Self::env().emit_event(PolyxReleased { value: amount });
            if self.env().transfer(self.env().caller(), amount).is_err() {
                Err(Error::InsufficientBalance)
            }
        }

        /// Calculates the amount of tokens that has already vested.
        #[ink(message)]
        fn vestedAmount(&self, timestamp: u64) -> Balance {
            vestingSchedule(self, self.env().balance() + self.released, timestamp);
        }

        /// This returns the amount vested.
        fn vestingSchedule(&self, totalAllocation: Balance, timestamp: u64) -> Balance {
            if (timestamp < self.start) {
                return 0;
            } else if (timestamp > self.start + self.duration) {
                return totalAllocation;
            } else {
                return (totalAllocation * (timestamp - self.start)) / self.duration;
            }
        }
    }
}

/// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
/// module and test functions are marked with a `#[test]` attribute.
/// The below code is technically just normal Rust code.
#[cfg(test)]
mod tests {
    /// Imports all the definitions from the outer scope so we can use them here.
    use super::*;

    /// Imports `ink_lang` so we can use `#[ink::test]`.
    use ink_lang as ink;

    /// We test if the default constructor does its job.
    #[ink::test]
    fn default_works() {
        let vesting = Vesting::default();
        assert_eq!(vesting.get(), false);
    }

    /// We test a simple use case of our contract.
    #[ink::test]
    fn it_works() {
        let mut vesting = Vesting::new(false);
        assert_eq!(vesting.get(), false);
        vesting.flip();
        assert_eq!(vesting.get(), true);
    }
}
