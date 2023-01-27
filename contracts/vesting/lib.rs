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
        address: AccountId,
        start: u64,
        duration: u64,
    }

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    /// Event emitted when Polyx is released.
    #[ink(event)]
    pub struct PolyxReleased {
        value: Balance,
    }

    /// Event emitted when an approval occurs that `spender` is allowed to withdraw
    /// up to the amount of `value` tokens from `owner`.
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }

    /// The ERC-20 error types.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Returned if not enough balance to fulfill a request is available.
        InsufficientBalance,
        /// Returned if not enough allowance to fulfill a request is available.
        InsufficientAllowance,
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
            self.address = beneficiaryAddress;
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
        pub fn address(&self) -> Balance {
            self.address
        }

        /// Returns the amount of POLYX already released.
        #[ink(message)]
        pub fn released(&self) -> Balance {
            self.released
        }

        /// Returns the amount of releasable POLYX.
        #[ink(message)]
        pub fn releasable(&self) -> Balance {
            vestedAmount().saturating_sub(released(self))
        }

        /// Release the native token (POLYX) that have already vested.
        #[ink(message)]
        pub fn release(&self) -> Balance {
            let amount = releasable(self);
            let address = address(self);
            self.released += amount;
            Self::env().emit_event(PolyxReleased { value: amount });
            transfer(&self, address, amount)
        }

        fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let from = self.env().caller();
            self.transfer_from_to(&from, &to, value)
        }

        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the caller's account balance.
        fn transfer_from_to(
            &mut self,
            from: &AccountId,
            to: &AccountId,
            value: Balance,
        ) -> Result<()> {
            let from_balance = self.balance_of_impl(from);
            if from_balance < value {
                return Err(Error::InsufficientBalance);
            }

            let new_from_balance = from_balance - value;
            if new_from_balance > 0 {
                self.balances.insert(from, &new_from_balance);
            } else {
                // Cleanup storage, don't save zeros.  Refunds storage fee to caller.
                self.balances.remove(from);
            }
            let to_balance = self.balance_of_impl(to);
            self.balances.insert(to, &(to_balance + value));
            self.env().emit_event(Transfer {
                from: Some(*from),
                to: Some(*to),
                value,
            });
            Ok(())
        }

        /// Calculates the amount of tokens that has already vested.
        fn vestedAmount(timestamp: u64) -> Balance {
            vestingSchedule(address(this).balance + released(), timestamp);
        }

        /// This returns the amount vested.
        fn vestingSchedule(totalAllocation: Balance, timestamp: u64) -> Balance {
            if (timestamp < start()) {
                return 0;
            } else if (timestamp > start() + duration()) {
                return totalAllocation;
            } else {
                return (totalAllocation * (timestamp - start())) / duration();
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
