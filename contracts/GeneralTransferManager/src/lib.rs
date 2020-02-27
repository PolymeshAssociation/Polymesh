#![cfg_attr(not(any(test, feature = "std")), no_std)]

use ink_core::{memory::format, memory::vec::Vec, storage};

use ink_lang::contract;

contract! {
    #![env = ink_core::env::DefaultSrmlTypes]

    struct GeneralTransferManager {
        // (transfer type, restriction type) => enabl. True signifies that the restricition is in force.
        // Transfer types are general(0), issuance(1) and redemption(2).
        // Transfer restricitons are fromValidCDD(0), toValidCDD(1), fromRestricted(2), toRestricted(3).
        // Not using enums as ink is not able to serialize and deserialze them. As a result,
        // enums can not be used in function parameters.
        transfer_restrictions: storage::HashMap<(u64, u64), bool>,
        owner: storage::Value<AccountId>,
        issuance_address: storage::Value<AccountId>,
        redemption_address: storage::Value<AccountId>,
        default_can_send_after: storage::Value<u64>,
        default_can_receive_after: storage::Value<u64>,
    }


    impl Deploy for GeneralTransferManager {
        fn deploy(&mut self) {
            for i in 0..3 {
                for j in 0..4 {
                    self.transfer_restrictions.insert((i, j), false);
                }
            }
            self.owner.set(env.caller());
            // TODO Replace with proper issuance and redemption addresses
            self.issuance_address.set(env.caller());
            self.redemption_address.set(env.caller());
            self.default_can_send_after.set(0);
            self.default_can_receive_after.set(0);
        }
    }

    impl GeneralTransferManager {
        /// Sets restriction status
        pub(external) fn set_restriction_status(&mut self, transfer_type: u64, restriciton_type: u64, status: bool) {
            if env.caller() != *self.owner {
                return;
            }
            self.transfer_restrictions.insert((transfer_type, restriciton_type), status);
        }

        /// Returns restriction status
        pub(external) fn get_restriction_status(&self, transfer_type: u64, restriciton_type: u64) -> bool {
            env.println(&format!("Restriction enabled: {:?}", *self.transfer_restrictions.get(&(transfer_type, restriciton_type)).unwrap_or(&false)));
            *self.transfer_restrictions.get(&(transfer_type, restriciton_type)).unwrap_or(&false)
        }

        /// Sets issuance address
        pub(external) fn set_issuance_address(&mut self, issuance_address: AccountId) {
            if env.caller() != *self.owner {
                return;
            }
            self.issuance_address.set(issuance_address);
        }

        /// Sets default_can_send_after
        pub(external) fn set_default_can_send_after(&mut self, default_can_send_after: u64) {
            if env.caller() != *self.owner {
                return;
            }
            self.default_can_send_after.set(default_can_send_after);
        }

        /// Sets default_can_receive_after
        pub(external) fn set_default_can_receive_after(&mut self, default_can_receive_after: u64) {
            if env.caller() != *self.owner {
                return;
            }
            self.default_can_receive_after.set(default_can_receive_after);
        }

        /// Sets redemption address
        pub(external) fn set_redemption_address(&mut self, redemption_address: AccountId) {
            if env.caller() != *self.owner {
                return;
            }
            self.redemption_address.set(redemption_address);
        }

        /// Gets issuance address
        pub(external) fn get_issuance_address(&self) -> Option<AccountId> {
            Some(*self.issuance_address.get())
        }

        /// Gets redemption address
        pub(external) fn get_redemption_address(&self) -> Option<AccountId> {
            Some(*self.redemption_address.get())
        }

        /// Gets default_can_send_after
        pub(external) fn get_default_can_send_after(&self) -> u64 {
            *self.default_can_send_after.get()
        }

        /// Gets default_can_receive_after
        pub(external) fn get_default_can_receive_after(&self) -> u64 {
            *self.default_can_receive_after.get()
        }

        /// Runtime will call this function to fetch the claims needed by this module
        pub(external) fn get_claims(&self) -> Option<Vec<Vec<u8>>> {
            // #TODO Finalize encoding/decoding mechanism for claims.
            // Currently, it is a vector of vector of u8. Every u8 represents one ASCII character. Vec<u8> represents a string.
            // Vec<Vec<u8>> represents a collection of strings. Every string represents one claim.
            // The string is <0|1>;<claim_identifier>;<datatype>. 0 represents sender, 1 represents receiver. example: 0;can_send_after;u64
            let mut claim1 = Vec::new();
            claim1.push('0' as u8);
            claim1.push(';' as u8);
            claim1.push('f' as u8);
            claim1.push('e' as u8);
            claim1.push(';' as u8);
            claim1.push('u' as u8);
            claim1.push('6' as u8);
            claim1.push('4' as u8);

            let mut claim2 = Vec::new();
            claim2.push('1' as u8);
            claim2.push(';' as u8);
            claim2.push('t' as u8);
            claim2.push('e' as u8);
            claim2.push(';' as u8);
            claim2.push('u' as u8);
            claim2.push('6' as u8);
            claim2.push('4' as u8);

            let mut claim3 = Vec::new();
            claim3.push('0' as u8);
            claim3.push(';' as u8);
            claim3.push('c' as u8);
            claim3.push('s' as u8);
            claim3.push('a' as u8);
            claim3.push(';' as u8);
            claim3.push('u' as u8);
            claim3.push('6' as u8);
            claim3.push('4' as u8);

            let mut claim4 = Vec::new();
            claim4.push('1' as u8);
            claim4.push(';' as u8);
            claim4.push('c' as u8);
            claim4.push('r' as u8);
            claim4.push('a' as u8);
            claim4.push(';' as u8);
            claim4.push('u' as u8);
            claim4.push('6' as u8);
            claim4.push('4' as u8);

            let mut claims = Vec::new();
            claims.push(claim1);
            claims.push(claim2);
            claims.push(claim3);
            claims.push(claim4);
            Some(claims)
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
            number_of_investors: u64,
            from_expiry: u64,
            to_expiry: u64,
            can_send_after: u64,
            can_receive_after: u64
        ) -> u16 { //u8 doesn't work. enums don't work. Hence, using u16.

            let mut tx_type:u64 = 0; //General (Default)
            if sender == *self.issuance_address {
                tx_type = 1; //Issuance
            } else if receiver == *self.redemption_address {
                tx_type = 2; //Redemption
            }

            if (*self.transfer_restrictions.get(&(tx_type, 0)).unwrap_or(&false) && from_expiry >= env.now()) ||
                (*self.transfer_restrictions.get(&(tx_type, 1)).unwrap_or(&false) && to_expiry >= env.now())
            {
                return 1; //NA
            }

            if (*self.transfer_restrictions.get(&(tx_type, 2)).unwrap_or(&false) &&
                if can_send_after == 0 { *self.default_can_send_after.get() } else { can_send_after } >= env.now()) || (
                *self.transfer_restrictions.get(&(tx_type, 3)).unwrap_or(&false) &&
                if can_receive_after == 0 { *self.default_can_receive_after.get() } else { can_receive_after } >= env.now())
            {
                return 1; //NA
            }

            return 2; //Valid
        }
        // pub(external) fn test_now(&self) -> u64 {
        //     env.now()
        // }
    }
}

#[cfg(all(test, feature = "test-env"))]
mod tests {
    use super::*;
    use ink_core::env;
    type Types = ink_core::env::DefaultSrmlTypes;

    #[test]
    fn owner_can_change_restriction_status() {
        let alice = AccountId::from([0x0; 32]);
        env::test::set_caller::<Types>(alice);
        let mut contract = GeneralTransferManager::deploy_mock();
        assert_eq!(contract.get_restriction_status(0, 0), false);
        contract.set_restriction_status(0, 0, true);
        assert_eq!(contract.get_restriction_status(0, 0), true);
    }
    #[test]
    fn not_owner_can_not_change_restriction_status() {
        let alice = AccountId::from([0x0; 32]);
        let bob = AccountId::from([0x1; 32]);
        env::test::set_caller::<Types>(alice);
        let mut contract = GeneralTransferManager::deploy_mock();
        env::test::set_caller::<Types>(bob);
        assert_eq!(contract.get_restriction_status(0, 0), false);
        contract.set_restriction_status(0, 0, true);
        assert_eq!(contract.get_restriction_status(0, 0), false);
    }
    // #[test]
    // fn test_now() {
    //     let time = Moment::from(10u64);
    //     env::test::set_now::<Types>(time);
    //     let mut contract = GeneralTransferManager::deploy_mock();
    //     assert_eq!(contract.test_now(), 10);
    // }
}
