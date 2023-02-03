#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod polyx_vesting {
    use ink_storage::traits::SpreadAllocate;

    /// Defines the storage of your contract.
    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct PolyxVesting {
        released: Balance,
        beneficiary: AccountId,
        start: u128,
        duration: u128,
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
        /// Funds are not released as yet.
        FundsNotReleased,
    }

    /// The contract result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl PolyxVesting {
        /// Constructor
        #[ink(constructor)]
        pub fn new(
            beneficiary_address: AccountId,
            start_timestamp: u128,
            duration_seconds: u128,
        ) -> Self {
            ink_lang::utils::initialize_contract(|contract| {
                Self::new_init(
                    contract,
                    beneficiary_address,
                    start_timestamp,
                    duration_seconds,
                )
            })
        }

        fn new_init(
            &mut self,
            beneficiary_address: AccountId,
            start_timestamp: u128,
            duration_seconds: u128,
        ) {
            self.beneficiary = beneficiary_address;
            self.start = start_timestamp;
            self.duration = duration_seconds;
        }

        // Getters
        /// Returns the vesting duration.
        #[ink(message)]
        pub fn duration(&self) -> u128 {
            self.duration
        }

        /// Returns the start timestamp.
        #[ink(message)]
        pub fn start(&self) -> u128 {
            self.start
        }

        /// Returns the beneficiary address.
        #[ink(message)]
        pub fn beneficiary(&self) -> AccountId {
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
            self.vested_amount(self.env().block_timestamp().into())
                .saturating_sub(self.released())
        }

        /// Release the native token (POLYX) that have already vested.
        #[ink(message)]
        pub fn release(&mut self) -> Result<()> {
            let amount = self.releasable();
            if amount == 0 {
                return Err(Error::FundsNotReleased);
            }
            self.released += amount;
            Self::env().emit_event(PolyxReleased { value: amount });
            if self.env().transfer(self.beneficiary, amount).is_err() {
                Err(Error::InsufficientBalance)
            } else {
                Ok(())
            }
        }

        /// Calculates the amount of tokens that has already vested.
        #[ink(message)]
        pub fn vested_amount(&self, timestamp: u128) -> Balance {
            self.vesting_schedule(self.env().balance() + self.released, timestamp)
        }

        /// This returns the amount vested.
        fn vesting_schedule(&self, total_allocation: u128, timestamp: u128) -> Balance {
            if timestamp < self.start {
                return 0;
            } else if timestamp > self.start + self.duration {
                return total_allocation;
            } else {
                return (total_allocation * (timestamp - self.start)) / self.duration;
            }
        }
    }

    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

        fn next_x_block(x: u8) {
            for _i in 0..x {
                ink_env::test::advance_block::<ink_env::DefaultEnvironment>();
            }
        }

        fn beneficiary_balance(beneficiary_address: AccountId) -> Balance {
            ink_env::test::get_account_balance::<ink_env::DefaultEnvironment>(beneficiary_address)
                .expect("failed to get account balance")
        }

        /// We test if the new constructor does its job.
        #[ink::test]
        fn new_works() {
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
            // Constructor works.
            let polyx_vesting = PolyxVesting::new(accounts.alice, 24, 80);
            // Ensure the values are stored correctly
            assert_eq!(polyx_vesting.beneficiary(), accounts.alice);
            assert_eq!(polyx_vesting.start(), 24u128);
            assert_eq!(polyx_vesting.duration(), 80u128);
            assert_eq!(polyx_vesting.released(), 0u128);
        }

        /// We test if vesting does its job.
        #[ink::test]
        fn vesting_works() {
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
            // Constructor works.
            let mut polyx_vesting = PolyxVesting::new(accounts.alice, 24, 80);

            // Ensure error when calling relase before start of vesting period
            assert_eq!(polyx_vesting.release(), Err(Error::FundsNotReleased));

            // Check beneficiary current balance
            let old_balance = beneficiary_balance(accounts.alice);

            next_x_block(5);

            // Release Polyx
            assert_eq!(polyx_vesting.release(), Ok(()));

            // Check beneficiary updated balance
            let new_balance = beneficiary_balance(accounts.alice);

            assert_eq!(polyx_vesting.released(), new_balance - old_balance);

            next_x_block(6);

            assert_eq!(polyx_vesting.release(), Ok(()));
        }
    }
}
