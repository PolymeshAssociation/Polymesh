#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod nft_royalty {

    pub enum Error {}

    /// A contract that manages non-fungible token transfers.
    #[ink(storage)]
    pub struct NftRoyalty {}

    impl NftRoyalty {
        /// Inititializes the [`NftRoyalty`] storage.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Inititializes the [`NftRoyalty`] storage to the default values.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new()
        }

        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        pub fn get(&self) -> bool {
            unimplemented!()
        }


    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            unimplemented!()
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            unimplemented!()
        }
    }

    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;

        use ink_e2e::build_message;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let constructor = NftRoyaltyRef::default();

            let contract_account_id = client
                .instantiate("nft_royalty", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let get = build_message::<NftRoyaltyRef>(contract_account_id.clone())
                .call(|nft_royalty| nft_royalty.get());
            let get_result = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let constructor = NftRoyaltyRef::new();
            let contract_account_id = client
                .instantiate("nft_royalty", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let get = build_message::<NftRoyaltyRef>(contract_account_id.clone())
                .call(|nft_royalty| nft_royalty.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            let flip = build_message::<NftRoyaltyRef>(contract_account_id.clone())
                .call(|nft_royalty| nft_royalty.flip());
            let _flip_result = client
                .call(&ink_e2e::bob(), flip, 0, None)
                .await
                .expect("flip failed");

            let get = build_message::<NftRoyaltyRef>(contract_account_id.clone())
                .call(|nft_royalty| nft_royalty.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), true));

            Ok(())
        }
    }
}
