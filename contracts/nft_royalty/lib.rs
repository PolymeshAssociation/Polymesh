#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::storage::Mapping;
use scale::Decode;

use polymesh_api::ink::basic_types::IdentityId;
use polymesh_api::ink::extension::PolymeshEnvironment;
use polymesh_api::ink::Error as PolymeshChainError;
use polymesh_api::polymesh::types::polymesh_primitives::asset_metadata::AssetMetadataGlobalKey;
use polymesh_api::polymesh::types::polymesh_primitives::asset_metadata::AssetMetadataKey;
use polymesh_api::polymesh::types::polymesh_primitives::asset_metadata::AssetMetadataValue;
use polymesh_api::polymesh::types::polymesh_primitives::identity_id::PortfolioId;
use polymesh_api::polymesh::types::polymesh_primitives::nft::NFTId;
use polymesh_api::polymesh::types::polymesh_primitives::settlement::VenueId;
use polymesh_api::polymesh::types::polymesh_primitives::ticker::Ticker;
use polymesh_api::Api;

use polymesh_api::polymesh::types::sp_arithmetic::per_things::Percent;

#[ink::contract(env = PolymeshEnvironment)]
mod nft_royalty {

    use super::*;

    // TODO: Replace this value.
    const NFT_MANDATORY_METADATA: AssetMetadataKey =
        AssetMetadataKey::Global(AssetMetadataGlobalKey(1_000));

    /// The contract result type.
    pub type Result<T> = core::result::Result<T, Error>;

    /// Contract Errors.
    #[derive(Debug)]
    pub enum Error {
        /// Polymesh runtime error.
        PolymeshRuntimeError(PolymeshChainError),
        /// Royalty metadata value not found.
        RoyaltyMetadataValueNotFound(Ticker),
        /// Trying to decode [`Percentage`] from [`AssetMetadataValue`] failed.
        FailedToDecodeRoyaltyPercentage,
    }

    impl From<PolymeshChainError> for Error {
        fn from(error: PolymeshChainError) -> Self {
            Self::PolymeshRuntimeError(error)
        }
    }

    /// A contract that manages non-fungible token transfers.
    #[ink(storage)]
    #[derive(Default)]
    pub struct NftRoyalty {
        /// Returns `true` if the contract has already been initialized.
        initialized: bool,
        // The identity of the contract.
        contract_identity: IdentityId,
        /// The portfolios that will receive the royalty value for each ticker.
        royalty_portfolios: Mapping<Ticker, PortfolioId>,
    }

    /// The details of an NFT transfer.
    pub struct NftTransferDetails {
        /// The [`VenueId`] of the transfer.
        pub venue_id: VenueId,
        /// The [`Ticker`] of the NFT collection.
        pub collection_ticker: Ticker,
        /// The [`NFTId`] of the non-fungible token being transferred.
        pub nft_id: NFTId,
        /// The [`PortfolioId`] of the NFT buyer.
        pub buyer_portfolio: PortfolioId,
        /// The [`PortfolioId`] of the NFT seller.
        pub seller_portfolio: PortfolioId,
        /// The price the buyer is paying for the NFT.
        pub transfer_price: Balance,
    }

    impl NftTransferDetails {
        /// Creates an instance of [`NftTransferDetails`].
        pub fn new(
            venue_id: VenueId,
            collection_ticker: Ticker,
            nft_id: NFTId,
            buyer_portfolio: PortfolioId,
            seller_portfolio: PortfolioId,
            transfer_price: Balance,
        ) -> Self {
            Self {
                venue_id,
                collection_ticker,
                nft_id,
                buyer_portfolio,
                seller_portfolio,
                transfer_price,
            }
        }
    }

    impl NftRoyalty {
        /// Inititializes the [`NftRoyalty`] storage.
        #[ink(constructor)]
        pub fn new() -> Self {
            let contract_identity = Self::env()
                .extension()
                .get_key_did(Self::env().account_id())
                .unwrap()
                .map(|did| did.into())
                .unwrap();
            Self {
                initialized: true,
                contract_identity,
                royalty_portfolios: Mapping::default(),
            }
        }

        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        pub fn get(&self) {
            unimplemented!()
        }

        /// Adds a settlement instruction containing [`NftTransferDetails`].
        ///
        /// The instruction will have three legs. One [`Leg`] where [`NftTransferDetails::seller_portfolio`] is transferring [`NFTId`]
        /// to [`NftTransferDetails::buyer_portfolio`], another leg where the buyer is transferring [`NftTransferDetails::transfer_price`]
        /// to the seller, and one leg where the buyer is transferring the royalty to the artist.
        pub fn create_transfer(_nft_transfer_details: NftTransferDetails) -> Result<()> {
            unimplemented!()
        }

        /// Returns [`Balance`] representing the royalty amount that the artist will receive for a NFT transfer of `transfer_price`
        /// for the given `collection_ticker`.
        pub fn get_royalty_amount(
            collection_ticker: Ticker,
            transfer_price: Balance,
        ) -> Result<Balance> {
            let royalty_percentage = Self::royalty_percentage(collection_ticker)?;
            // TODO: verify this
            Ok((royalty_percentage.0 as Balance * transfer_price) / 100)
        }

        /// Returns the [`AssetMetadataValue`] for the given `ticker` and `asset_metadata_key`.
        fn asset_metadata_value(
            ticker: Ticker,
            asset_metadata_key: AssetMetadataKey,
        ) -> Result<Option<AssetMetadataValue>> {
            let api = Api::new();

            api.query()
                .asset()
                .asset_metadata_values(ticker, asset_metadata_key)
                .map_err(|e| e.into())
        }

        /// Returns the [`Percent`] that corresponds to percentage amount that the artist receives as royalty for each NFT transfer.
        fn royalty_percentage(ticker: Ticker) -> Result<Percent> {
            let metadata_value: Option<AssetMetadataValue> =
                Self::asset_metadata_value(ticker, NFT_MANDATORY_METADATA)?;
            let metadata_value =
                metadata_value.ok_or(Error::RoyaltyMetadataValueNotFound(ticker))?;
            // TODO: Since we still have to define what the value is, decoding here is only a placeholder.
            Percent::decode::<&[u8]>(&mut metadata_value.0.as_ref())
                .map_err(|_| Error::FailedToDecodeRoyaltyPercentage)
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
            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            Ok(())
        }
    }
}
