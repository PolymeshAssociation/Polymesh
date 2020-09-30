#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

mod custom_types {
    use scale::{Decode, Encode};

    #[derive(Decode, Encode, PartialEq, Ord, Eq, PartialOrd, Copy, Hash, Clone, Default)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo, Debug))]
    pub struct IdentityId([u8; 32]);

    impl From<u128> for IdentityId {
        fn from(id: u128) -> Self {
            let mut encoded_id = id.encode();
            encoded_id.resize(32, 0);
            let mut did = [0; 32];
            did.copy_from_slice(&encoded_id);
            IdentityId(did)
        }
    }

    #[derive(Decode, Encode, PartialEq, Ord, Eq, PartialOrd)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo, Debug))]
    pub enum RestrictionResult {
        Valid,
        Invalid,
        ForceValid,
    }
}

#[ink::contract]
mod count_transfer_manager {
    use crate::custom_types::{IdentityId, RestrictionResult};
    pub type Counter = u64;

    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_core::storage::lazy::Lazy;

    /// Event emitted when maximum holders set
    #[ink(event)]
    pub struct SetMaximumHolders {
        #[ink(topic)]
        new_holder_count: Counter,
        #[ink(topic)]
        old_holder_count: Counter,
    }

    /// Struct that contains the storage items of this smart contract.
    #[ink(storage)]
    #[derive(Default)]
    pub struct CountTransferManagerStorage {
        /// No. of maximum holders a ticker can have.
        max_holders: Counter,
        /// Owner of the smart contract. It has the special privileges over other callers
        owner: Lazy<AccountId>,
    }

    impl CountTransferManagerStorage {
        /// Constructor use to set the no. of maximum holder count a
        /// ticker can have.
        #[ink(constructor)]
        pub fn new(max_holders: Counter) -> Self {
            Self {
                max_holders,
                owner: Lazy::new(Self::env().caller())
            }
        }

        /// Sets number of max holders
        /// # Arguments
        /// * max_holders No. of maximum holders
        #[ink(message)]
        pub fn set_max_holders(&mut self, max_holders: Counter) {
            self.ensure_owner(self.env().caller());
            self.env().emit_event(SetMaximumHolders {
                new_holder_count: self.max_holders,
                old_holder_count: max_holders,
            });
            self.max_holders = max_holders;
        }

        /// Returns number of max holders
        #[ink(message)]
        pub fn get_max_holders(&self) -> Counter {
            self.max_holders
        }

        /// This function is used to verify transfers initiated by the
        /// runtime assets
        ///
        /// It will be a valid transfer even when value > from balance as we are not checking the overflow / underflow
        /// of the sender balances. Assuming these will be checked in the blockchain itself.
        ///
        /// # Arguments
        /// * `from` - Identity Id of the sender.
        /// * `to` - Identity Id of the receiver.
        /// * `value` - Asset amount need to transfer to the receiver.
        /// * `balance_from` - Balance of sender at the time of transaction.
        /// * `balance_to` - Balance of receiver at the time of transaction.
        /// * `total_supply` - Total supply of the asset
        /// * `current_holder_count - Total no. of investors of a ticker.
        #[ink(message)]
        pub fn verify_transfer(
            &self,
            _from: Option<IdentityId>,
            _to: Option<IdentityId>,
            value: Balance,
            balance_from: Balance,
            balance_to: Balance,
            _total_supply: Balance,
            current_holder_count: Counter,
        ) -> RestrictionResult {
            // Strict checking only the cases where no. of holders get increases.
            if self.max_holders == current_holder_count
                && balance_to == 0
                && balance_from > value
            {
                return RestrictionResult::Invalid; // INVALID
            }
            RestrictionResult::Valid // VALID
        }

        /// Simply returns the current value of `owner`.
        #[ink(message)]
        pub fn owner(&self) -> AccountId {
            *Lazy::get(&self.owner)
        }

        fn ensure_owner(&self, owner: AccountId) {
            assert!(owner == self.owner(), "Not Authorized");
        }
    }

    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
        use ink_core::env::{ call, test };
        type Accounts = test::DefaultAccounts<EnvTypes>;
        const CALLEE: [u8; 32] = [7; 32];

        fn set_sender(sender: AccountId) {
            test::push_execution_context::<EnvTypes>(
                sender,
                CALLEE.into(),
                1000000,
                1000000,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );
        }

        fn set_from_owner() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
        }

        fn default_accounts() -> Accounts {
            test::default_accounts()
                .expect("Test environment is expected to be initialized.")
        }

        /// We test if the default constructor does its job.
        #[test]
        fn constructor_initialization_check() {
            let default_accounts = default_accounts();
            set_from_owner();
            let count_transfer_manager = CountTransferManagerStorage::new(5000u64);
            assert_eq!(count_transfer_manager.get_max_holders(), 5000u64);
            assert_eq!(count_transfer_manager.owner(), default_accounts.alice);
        }

        #[test]
        fn verify_transfer_check() {
            let alice_did = IdentityId::from(1);
            let bob_did = IdentityId::from(2);
            set_from_owner();
            let mut count_transfer_manager = CountTransferManagerStorage::new(5u64);
            assert_eq!(count_transfer_manager.get_max_holders(), 5u64);

            // Check for simple transfer case
            assert_eq!(
                count_transfer_manager.verify_transfer(
                    Some(alice_did),
                    Some(bob_did),
                    100,
                    200,
                    10,
                    500,
                    5
                ),
                RestrictionResult::Valid
            );

            assert_eq!(
                count_transfer_manager.verify_transfer(
                    Some(alice_did),
                    Some(bob_did),
                    100,
                    200,
                    0,
                    500,
                    5
                ),
                RestrictionResult::Invalid
            );

            // allowing transfer when holder counts get change
            assert_eq!(count_transfer_manager.set_max_holders(10u64), ());
            assert_eq!(count_transfer_manager.get_max_holders(), 10u64);

            assert_eq!(
                count_transfer_manager.verify_transfer(
                    Some(alice_did),
                    Some(bob_did),
                    100,
                    200,
                    0,
                    500,
                    5
                ),
                RestrictionResult::Valid
            );

            // It will be a valid transfer as we are not checking the overflow / underflow
            // of the sender balances. Assuming these will be checked in the blockchain itself
            assert_eq!(
                count_transfer_manager.verify_transfer(
                    Some(alice_did),
                    Some(bob_did),
                    100,
                    50,
                    0,
                    500,
                    5
                ),
                RestrictionResult::Valid
            );
        }
    }
}
