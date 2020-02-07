#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod percentage_transfer_manager {
    use ink_core::storage;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    struct PercentageTransferManagerStorage {
        /// Owner of the smart extension
        pub owner: storage::Value<AccountId>,
        /// Maximum allowed percentage of the tokens hold by an investor
        /// %age is based on the total supply of the asset.
        pub max_allowed_percentage: storage::Value<u128>,
        /// By toggling the primary issuance variable it will bypass
        /// all the restrictions imposed by this smart extension
        pub allow_primary_issuance: storage::Value<bool>,
        // /// Exemption list that contains the list of investor's identities
        // /// which are not affected by this module restrictions
        // pub exemption_list: storage::HashMap<IdentityId, bool>  
    }

    #[ink(event)]
    struct ChangeAllowedPercentage {
        #[ink(topic)]
        old_percentage: u128,
        #[ink(topic)]
        new_percentage: u128
    }

    #[ink(event)]
    struct ChangePrimaryIssuance {
        #[ink(topic)]
        allow_primary_issuance: bool
    }

    // #[ink(event)]
    // struct ModifyExemptionList {
    //     #[ink(topic)]
    //     identity: IdentityId,
    //     #[ink(topic)]
    //     exempted: bool
    // }

    #[ink(event)]
    struct TransferOwnership {
        #[ink(topic)]
        new_owner: AccountId,
        #[ink(topic)]
        old_owner: AccountId
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
        Valid,
        Invalid,
        ForceValid
    }

    impl PercentageTransferManagerStorage {
        /// Constructor that initializes the `u128` value to the given `max_allowed_percentage`
        /// & boolean value for the `allow_primary_issuance`.
        #[ink(constructor)]
        fn new(&mut self, max_percentage: u128, primary_issuance: bool) {
            self.owner.set(self.env().caller());
            self.max_allowed_percentage.set(max_percentage);
            self.allow_primary_issuance.set(primary_issuance);
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
        #[ink(message)]
        fn verify_transfer(
            &self,
            // from: IdentityId,
            // to: IdentityId,
            value: Balance,
            balance_from: Balance,
            balance_to: Balance,
            total_supply: Balance
        ) -> RestrictionResult {
            if /*from == None &&*/ *self.allow_primary_issuance.get() {
                return RestrictionResult::Valid;
            }
            // if *self.exemption_list.get(to) {
            //     return RestrictionResult::Valid;
            // }
            if ((balance_to + value) * 10u128.pow(6)) / total_supply > *self.max_allowed_percentage.get() {
                return RestrictionResult::Invalid;
            } else {
                return RestrictionResult::Valid;
            }
        }

        /// Change the value of allowed percentage
        ///
        /// # Arguments
        /// * `new_percentage` - New value of Max percentage of assets hold by an investor
        #[ink(message)]
        fn change_allowed_percentage(&mut self, new_percentage: u128) {
            // ensure!(self.env().caller() == *self.owner.get(), "Incorrect owner");
            // ensure!(*self.max_allowed_percentage.get() != new_percentage, "Must change setting");
            self.env().emit_event(ChangeAllowedPercentage{
                old_percentage: *self.max_allowed_percentage.get(),
                new_percentage: new_percentage,
            });
            self.max_allowed_percentage.set(new_percentage);
        }

        /// Sets whether or not to consider primary issuance transfers
        ///
        /// # Arguments
        /// * `primary_issuance` - whether to allow all primary issuance transfers
        #[ink(message)]
        fn change_primary_issuance(&mut self, primary_issuance: bool) {
            // ensure!(self.env().caller() == *self.owner.get(), "Incorrect owner");
            // ensure!(*self.allow_primary_issuance.get() != primary_issuance, "Must change setting");
            self.allow_primary_issuance.set(primary_issuance);
            self.env().emit_event(ChangePrimaryIssuance{
                allow_primary_issuance: primary_issuance,
            });
        }

        // /// To exempt the given Identity from the restriction
        // ///
        // /// # Arguments
        // /// * `identity` - Identity of the token holder whose exemption status needs to change
        // /// * `is_exempted` - New exemption status of the identity
        // #[ink(message)]
        // fn modify_exemption_list(&mut self, identity: IdentityId, is_exempted: bool) {
        //     ensure!(self.env().caller() == *self.owner.get(), "Incorrect owner");
        //     ensure!(*self.exemption_list.get(&identity) != is_exempted, "Must change setting");
        //     self.exemption_list.insert(&identity, is_exempted);
        //     self.env().emit_event(ModifyExemptionList {
        //         identity: identity,
        //         exempted: is_exempted
        //     });
        // }

        /// Transfer ownership of the smart extension
        ///
        /// # Arguments
        /// * `new_owner` - AccountId of the new owner
        #[ink(message)]
        fn transfer_ownership(&mut self, new_owner: AccountId) {
            //ensure!(self.env().caller() == *self.owner.get(), "Incorrect owner");
            self.env().emit_event(TransferOwnership{
                old_owner: self.env().caller(),
                new_owner: new_owner,
            });
            self.owner.set(new_owner);
        }

        /// Simply returns the current value of `max_allowed_percentage`.
        #[ink(message)]
        fn get_max_allowed_percentage(&self) -> u128 {
            *self.max_allowed_percentage.get()
        }

        /// Simply returns the current value of `allow_primary_issuance`.
        #[ink(message)]
        fn is_primary_issuance_allowed(&self) -> bool {
            *self.allow_primary_issuance.get()
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
        fn constructor_initialization_check() {
            let PercentageTransferManager = PercentageTransferManagerStorage::new(200000, false);
            assert_eq!(PercentageTransferManager.max_allowed_percentage.get(), 200000);
            assert_eq!(PercentageTransferManager.allow_primary_issuance.get(), false);
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
