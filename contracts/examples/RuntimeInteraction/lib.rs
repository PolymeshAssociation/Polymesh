#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract(version = "0.1.0", env = PolymeshRuntimeTypes)]
#[allow(non_snake_case)]
mod RuntimeInteraction {
    use custom_ink_env_types::{
        calls as runtime_calls, AssetTransferRules, PolymeshRuntimeTypes, Ticker,
    };
    use ink_core::{env, hash::Blake2x128, storage};
    use ink_prelude::{format, vec, vec::Vec};
    use scale::Encode;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    struct RuntimeInteractionStorage {}

    impl RuntimeInteractionStorage {
        /// Constructor.
        #[ink(constructor)]
        fn new(&mut self) {}

        #[ink(message)]
        fn read_compliance_manager_storage(&self, ticker: Ticker) -> AssetTransferRules {
            // Read the storage of compliance transfer manager
            // Read the map storage
            // Twox128(module_prefix) ++ Twox128(storage_prefix) ++ Hasher(encode(key))

            let mut key = vec![
                // Precomputed: Twox128("ComplianceManager")
                255, 219, 199, 116, 193, 144, 129, 130, 221, 228, 247, 53, 193, 10, 8, 220,
                // Precomputed: Twox128("AssetRulesMap")
                43, 166, 32, 27, 86, 56, 171, 63, 215, 7, 88, 63, 149, 251, 213, 120,
            ];

            let encoded_ticker = &ticker.encode();

            let mut blake2_128 = Blake2x128::from(Vec::new());
            let hashed_ticker = blake2_128.hash_raw(&encoded_ticker);

            // The hasher is `Blake2_128Concat` which appends the unhashed account to the hashed account
            key.extend_from_slice(&hashed_ticker);
            key.extend_from_slice(&encoded_ticker);

            // fetch from runtime storage
            let result = self.env().get_runtime_storage::<AssetCompliances>(&key[..]);
            env::println(&format!("PRINT THE KEY {:?}", key));
            match result {
                Some(Ok(asset_compliance)) => {
                    env::println(&format!("AssetCompliances {:?}", asset_compliance));
                    asset_compliance
                }
                Some(Err(err)) => {
                    env::println(&format!("Error reading AssetCompliances {:?}", err));
                    AssetCompliances::default()
                }
                None => {
                    env::println(&format!("No data at key {:?}", key));
                    AssetCompliances::default()
                }
            }
        }

        #[ink(message)]
        fn call_runtime_dispatch(&self, ticker: Ticker, id: u32) {
            // Creating the instance of the runtime call
            let remove_requirement_call =
                runtime_calls::cm_remove_compliance_requirement(ticker, id);
            // dispatch the call to the runtime
            let result = self.env().invoke_runtime(&remove_requirement_call);
            // Print the result if the async call
            env::println(&format!(
                "Remove active call invoke_runtime result {:?}",
                result
            ));
        }
    }
}
