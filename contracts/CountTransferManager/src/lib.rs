#![cfg_attr(not(any(test, feature = "std")), no_std)]

use ink_core::{memory::format, memory::vec::Vec, storage};

use ink_lang::contract;

contract! {
    #![env = ink_core::env::DefaultSrmlTypes]

    struct CountTransferManager {
        max_holders: storage::Value<u64>,
        owner: storage::Value<AccountId>,
    }


    impl Deploy for CountTransferManager {
        fn deploy(&mut self) {
            self.owner.set(env.caller());
            self.max_holders.set(0);
        }
    }

    impl CountTransferManager {
        /// Sets number of max holders
        pub(external) fn set_max_holders(&mut self, max_holders: u64) {
            if env.caller() != *self.owner {
                return;
            }
            self.max_holders.set(max_holders);
        }

        /// Returns number of max holders
        pub(external) fn get_max_holders(&self) -> u64 {
            env.println(&format!("number of max holders: {:?}", *self.max_holders.get()));
            *self.max_holders.get()
        }

        /// Runtime will call this function to fetch the claims needed by this module
        pub(external) fn get_claims(&self) -> Option<Vec<Vec<u8>>> {
            // This module needs no extra claims data
            None
        }

        /// Verify transfer function with standard interface.
        pub(external) fn verify_tranfer(
            &self,
            sender: AccountId,
            receiver: AccountId,
            amount: Balance,
            sender_balance: Balance,
            receiver_balance: Balance,
            total_supply: Balance,
            number_of_investors: u64
        ) -> u16 { //u8 doesn't work. enums don't work. Hence, using u16.

            if *self.max_holders.get() < number_of_investors &&
                receiver_balance == 0 &&
                sender_balance > amount
            {
                return 0; // INVALID
            }

            return 1; //NA
        }
    }
}

#[cfg(all(test, feature = "test-env"))]
mod tests {
    use super::*;
    use ink_core::env;
    type Types = ink_core::env::DefaultSrmlTypes;

    #[test]
    fn owner_can_change_max_holders() {
        let alice = AccountId::from([0x0; 32]);
        env::test::set_caller::<Types>(alice);
        let mut contract = CountTransferManager::deploy_mock();
        assert_eq!(contract.get_max_holders(), 0);
        contract.set_max_holders(10);
        assert_eq!(contract.get_max_holders(), 10);
    }
    #[test]
    fn not_owner_can_not_change_max_holders() {
        let alice = AccountId::from([0x0; 32]);
        let bob = AccountId::from([0x1; 32]);
        env::test::set_caller::<Types>(alice);
        let mut contract = CountTransferManager::deploy_mock();
        env::test::set_caller::<Types>(bob);
        assert_eq!(contract.get_max_holders(), 0);
        contract.set_max_holders(10);
        assert_eq!(contract.get_max_holders(), 0);
    }
    #[test]
    fn get_claims_returns_none() {
        let alice = AccountId::from([0x0; 32]);
        env::test::set_caller::<Types>(alice);
        let contract = CountTransferManager::deploy_mock();
        assert_eq!(contract.get_claims(), None);
    }
    #[test]
    fn verify_tranfer_returns_na() {
        let alice = AccountId::from([0x0; 32]);
        let bob = AccountId::from([0x1; 32]);
        env::test::set_caller::<Types>(alice);
        let mut contract = CountTransferManager::deploy_mock();
        contract.set_max_holders(10);
        assert_eq!(contract.verify_tranfer(alice, bob, 10, 20, 0, 20, 2), 1);
    }
    #[test]
    fn verify_tranfer_returns_invalid() {
        let alice = AccountId::from([0x0; 32]);
        let bob = AccountId::from([0x1; 32]);
        env::test::set_caller::<Types>(alice);
        let mut contract = CountTransferManager::deploy_mock();
        contract.set_max_holders(1);
        assert_eq!(contract.verify_tranfer(alice, bob, 10, 20, 0, 20, 2), 0);
    }
}
