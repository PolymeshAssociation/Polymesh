#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

mod custom_types {
    use scale::{Encode, Decode};
    use ink_prelude::{vec, vec::Vec};
    #[cfg(feature = "std")]
    use serde::{Serialize, Deserialize};
    use ink_core::storage::Flush;

    const TICKER_LEN: usize = 12;

    #[derive(Decode, Encode, PartialEq, Ord, Eq, PartialOrd, Copy, Hash, Clone, Default)]
    #[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
    pub struct IdentityId([u8; 32]);

    impl Flush for IdentityId {}

    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    #[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
    #[derive(
        Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord,
    )]
    pub struct JurisdictionName(pub Vec<u8>);

    /// Scope: Almost all claim needs a valid scope identity.
    pub type Scope = IdentityId;

    /// All possible claims in polymesh
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
    #[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
    #[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Claim {
        /// User is Accredited
        Accredited(Scope),
        /// User is Accredited
        Affiliate(Scope),
        /// User has an active BuyLockup (end date defined in claim expiry)
        BuyLockup(Scope),
        /// User has an active SellLockup (date defined in claim expiry)
        SellLockup(Scope),
        /// User has passed CDD
        CustomerDueDiligence,
        /// User is KYC'd
        KnowYourCustomer(Scope),
        /// This claim contains a string that represents the jurisdiction of the user
        Jurisdiction(JurisdictionName, Scope),
        /// User is exempted
        Exempted(Scope),
        /// User is Blocked
        Blocked(Scope),
        /// Empty claim
        NoData,
    }

    #[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
    #[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
    #[derive(Encode, Decode, Clone, PartialEq, Eq)]
    /// It defines the type of rule supported, and the filter information we will use to evaluate as a
    /// predicate.
    pub enum RuleType {
        /// Rule to ensure that claim filter produces one claim.
        IsPresent(Claim),
        /// Rule to ensure that claim filter produces an empty list.
        IsAbsent(Claim),
        /// Rule to ensure that at least one claim is fetched when filter is applied.
        IsAnyOf(Vec<Claim>),
        /// Rule to ensure that at none of claims is fetched when filter is applied.
        IsNoneOf(Vec<Claim>),
    }

    /// Type of claim requirements that a rule can have
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
    #[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
    #[derive(Encode, Decode, Clone, PartialEq, Eq)]
    pub struct Rule {
        /// Type of rule.
        pub rule_type: RuleType,
        /// Trusted issuers.
        pub issuers: Vec<IdentityId>,
    }

    #[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
    #[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
    #[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
    pub struct AssetTransferRule {
        pub sender_rules: Vec<Rule>,
        pub receiver_rules: Vec<Rule>,
        /// Unique identifier of the asset rule
        pub rule_id: u32,
    }

    /// List of rules associated to an asset.
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
    #[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
    #[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
    pub struct AssetTransferRules {
        /// This flag indicates if asset transfer rules are active or paused.
        pub is_paused: bool,
        /// List of rules.
        pub rules: Vec<AssetTransferRule>,
    }

    /// Ticker symbol.
    ///
    /// This type stores fixed-length case-sensitive byte strings. Any value of this type that is
    /// received by a Substrate module call method has to be converted to canonical uppercase
    /// representation using [`Ticker::canonize`].
    #[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
    #[derive(Encode, Decode, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Ticker([u8; TICKER_LEN]);

    impl Default for Ticker {
        fn default() -> Self {
            Ticker([0u8; TICKER_LEN])
        }
    }

}

#[ink::contract(version = "0.1.0")]
mod RuntimeInteraction {
    use scale::{Encode, Decode};
    use super::custom_types::{AssetTransferRules, Ticker};
    use ink_core::{
        env,
        hash::Blake2x128,
        storage
    };
    use ink_prelude::{
        format,
        vec,
        vec::Vec,
    };

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    struct RuntimeInteractionStorage {
        /// Stores a single `bool` value on the storage.
        value: storage::Value<bool>,
    }

    impl RuntimeInteractionStorage {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        fn new(&mut self, init_value: bool) {
            self.value.set(init_value);
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        fn default(&mut self) {
            self.new(false)
        }

        #[ink(message)]
        fn read_compliance_manager_storage(&mut self, ticker: Ticker) -> AssetTransferRules {
            // Read the storage of compliance transfer manager
            // Read the map storage
            // Twox128(module_prefix) ++ Twox128(storage_prefix) ++ Hasher(encode(key))

            let mut key = vec![
                // Precomputed: Twox128("ComplianceManager")
                255,219,199,116,193,144,129,130,221,228,247,53,193,10,8,220,
                // Precomputed: Twox128("AssetRulesMap")
                43,166,32,27,86,56,171,63,215,7,88,63,149,251,213,120,
            ];

            let encoded_ticker = &ticker.encode();

            let mut blake2_128 = Blake2x128::from(Vec::new());
            let hashed_ticker = blake2_128.hash_raw(&encoded_ticker);

            // The hasher is `Blake2_128Concat` which appends the unhashed account to the hashed account
            key.extend_from_slice(&hashed_ticker);
            key.extend_from_slice(&encoded_ticker);

            // fetch from runtime storage
            let result = self.env().get_runtime_storage::<AssetTransferRules>(&key[..]);
            match result {
                Some(Ok(asset_rules)) => asset_rules,
                Some(Err(err)) => {
                    env::println(&format!("Error reading AssetTransferRules {:?}", err));
                    AssetTransferRules::default()
                }
                None => {
                    env::println(&format!("No data at key {:?}", key));
                    AssetTransferRules::default()
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

        /// We test if the default constructor does its job.
        #[test]
        fn default_works() {
            // Note that even though we defined our `#[ink(constructor)]`
            // above as `&mut self` functions that return nothing we can call
            // them in test code as if they were normal Rust constructors
            // that take no `self` argument but return `Self`.
            let RuntimeInteraction = RuntimeInteraction::default();
            assert_eq!(RuntimeInteraction.get(), false);
        }

        /// We test a simple use case of our contract.
        #[test]
        fn it_works() {
            let mut RuntimeInteraction = RuntimeInteraction::new(false);
            assert_eq!(RuntimeInteraction.get(), false);
            RuntimeInteraction.flip();
            assert_eq!(RuntimeInteraction.get(), true);
        }
    }
}
