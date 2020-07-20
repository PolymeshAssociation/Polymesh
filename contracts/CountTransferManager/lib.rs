#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

mod custom_types {

    use ink_core::storage::Flush;
    use scale::{ Encode, Decode };

    #[derive(
        Decode,
        Encode,
        PartialEq,
        Ord,
        Eq,
        PartialOrd,
        Copy,
        Hash,
        Clone,
        Debug,
        Default,
    )]
    #[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
    pub struct IdentityId([u8; 32]);

    impl Flush for IdentityId {}

    impl From<u128> for IdentityId {
        fn from(id: u128) -> Self {
            let mut encoded_id = id.encode();
            encoded_id.resize(32, 0);
            let mut did = [0; 32];
            did.copy_from_slice(&encoded_id);
            IdentityId(did)
        }
    }

    /// Custom type
    #[derive(Decode, Encode, Debug, PartialEq, Ord, Eq, PartialOrd)]
    #[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
    pub enum RestrictionResult {
        Valid,
        Invalid,
        ForceValid,
    }
}

#[ink::contract(version = "0.1.0")]
mod count_transfer_manager {
    use crate::custom_types::{ IdentityId, RestrictionResult };
    use ink_core::storage;
    type HolderCount = u64;

    /// Event emitted when maximum holders set
    #[ink(event)]
    struct SetMaximumHolders {
        #[ink(topic)]
        new_holder_count: HolderCount,
        #[ink(topic)]
        old_holder_count: HolderCount,
    }

    /// Struct that contains the storage items of this smart contract.
    #[ink(storage)]
    struct CountTransferManagerStorage {
        /// No. of maximum holders a ticker can have.
        max_holders: storage::Value<HolderCount>,
        /// Owner of the smart contract. It has the special privileges over other callers
        owner: storage::Value<AccountId>,
    }

    impl CountTransferManagerStorage {

        /// Constructor use to set the no. of maximum holder count a
        /// ticker can have.
        #[ink(constructor)]
        fn new(&mut self, max_holders: HolderCount) {
            self.owner.set(self.env().caller());
            self.max_holders.set(max_holders);
        }

        /// Sets number of max holders
        /// # Arguments
        /// * max_holders No. of maximum holders
        #[ink(message)]
        fn set_max_holders(&mut self, max_holders: HolderCount) {
            self._ensure_owner(self.env().caller());
            self.env().emit_event(SetMaximumHolders {
                new_holder_count: *self.max_holders.get(),
                old_holder_count: max_holders
            });
            self.max_holders.set(max_holders);
        }

        /// Returns number of max holders
        #[ink(message)]
        fn get_max_holders(&self) -> HolderCount {
            *self.max_holders.get()
        }

        /// This function is used to verify transfers initiated by the
        /// runtime assets
        ///
        /// # Arguments
        /// * `from` - Identity Id of the sender.
        /// * `to` - Identity Id of the receiver.
        /// * `value` - Asset amount need to transfer to the receiver.
        /// * `balance_from` - Balance of sender at the time of transaction.
        /// * `balance_to` - Balance of receiver at the time of transaction.
        /// * `total_supply` - Total supply of the asset
        /// * `number_of_investors - Total no. of investors of a ticker.
        #[ink(message)]
        fn verify_transfer(
            &self,
            from: Option<IdentityId>,
            to: Option<IdentityId>,
            value: Balance,
            balance_from: Balance,
            balance_to: Balance,
            total_supply: Balance,
            number_of_investors: HolderCount
        ) -> RestrictionResult {
            if *self.max_holders.get() < number_of_investors &&
                balance_to == 0 &&
                balance_from > value
            {
                return RestrictionResult::Invalid; // INVALID
            }
            return RestrictionResult::Valid; // VALID
        }

        /// Simply returns the current value of `owner`.
        #[ink(message)]
        fn owner(&self) -> AccountId {
            *self.owner.get()
        }

        fn _ensure_owner(&self, owner: AccountId) {
            assert!(owner == *self.owner.get(), "Not Authorized");
        }

    }

    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
        use ink_core::env::test::*;
        type EnvTypes = ink_core::env::DefaultEnvTypes;

        /// We test if the default constructor does its job.
        #[test]
        fn constructor_initialization_check() {
            let default_accounts = default_accounts::<EnvTypes>().unwrap();
            let count_transfer_manager =
                CountTransferManagerStorage::new(5000u64);
            assert_eq!(
                count_transfer_manager.get_max_holders(),
                5000u64
            );
            assert_eq!(count_transfer_manager.owner(), default_accounts.alice);
        }

        #[test]
        fn verify_transfer_check() {
            let default_accounts = default_accounts::<EnvTypes>().unwrap();
            let alice_did = IdentityId::from(1);
            let bob_did = IdentityId::from(2);
            let count_transfer_manager =
                CountTransferManagerStorage::new(5u64);

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
        }
    }
}