#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::storage::Mapping;
use scale::{Encode, Decode};

use polymesh_api::ink::basic_types::IdentityId;
use polymesh_api::ink::extension::PolymeshEnvironment;
use polymesh_api::ink::Error as PolymeshChainError;
use polymesh_api::polymesh::types::polymesh_primitives::asset_metadata::AssetMetadataKey;
use polymesh_api::polymesh::types::polymesh_primitives::asset_metadata::AssetMetadataName;
use polymesh_api::polymesh::types::polymesh_primitives::asset_metadata::AssetMetadataValue;
use polymesh_api::polymesh::types::polymesh_primitives::identity_id::PortfolioId;
use polymesh_api::polymesh::types::polymesh_primitives::nft::{NFTId, NFTs};
use polymesh_api::polymesh::types::polymesh_primitives::settlement::{Leg, VenueId};
use polymesh_api::polymesh::types::polymesh_primitives::ticker::Ticker;
use polymesh_api::Api;

use polymesh_api::polymesh::types::sp_arithmetic::per_things::Percent;

#[cfg(test)]
mod tests;

#[ink::contract(env = PolymeshEnvironment)]
mod nft_royalty {
    use super::*;
    use crate::nft_royalty::types::{NFTOffer, NFTTransferDetails};

    /// The [`AssetMetadataName`] for the key that holds the mandatory NFT collection metadata.
    const NFT_METADATA_NAME: AssetMetadataName = AssetMetadataName(Vec::new());

    /// The contract result type.
    pub type Result<T> = core::result::Result<T, Error>;

    /// Contract Errors.
    #[derive(Debug)]
    pub enum Error {
        /// Polymesh runtime error.
        PolymeshRuntimeError(PolymeshChainError),
        /// Royalty metadata value not found.
        RoyaltyMetadataValueNotFound(Ticker),
        /// Royalty metadata key not found.
        RoyaltyMetadataKeyNotFound(Ticker),
        /// Trying to decode [``] from [`AssetMetadataValue`] failed.
        FailedToDecodeMetadataValue,
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
        /// The identity of the contract.
        contract_identity: IdentityId,
        /// The portfolios that will receive the royalty value for each ticker.
        royalty_portfolios: Mapping<Ticker, PortfolioId>,
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

        /// Returns the [`AssetMetadataKey`] for the given `ticker`.
        fn asset_metadata_key(&mut self, ticker: Ticker) -> Result<AssetMetadataKey> {
            // TODO: Check if the value is in storage
            let api = Api::new();

            let local_metadata_key = api
                .query()
                .asset()
                .asset_metadata_local_name_to_key(ticker, NFT_METADATA_NAME)
                .map_err(|e| Into::<Error>::into(e))?
                .ok_or(Error::RoyaltyMetadataKeyNotFound(ticker))?;
            let metadata_key = AssetMetadataKey::Local(local_metadata_key);
            // TODO: keep metadata key in storage
            Ok(metadata_key)
        }

        /// Returns the [`AssetMetadataValue`] for the given `ticker`.
        fn asset_metadata_value(&mut self, ticker: Ticker) -> Result<AssetMetadataValue> {
            let api = Api::new();

            let metadata_key = self.asset_metadata_key(ticker)?;

            api.query()
                .asset()
                .asset_metadata_values(ticker, metadata_key)
                .map_err(|e| Into::<Error>::into(e))?
                .ok_or(Error::RoyaltyMetadataValueNotFound(ticker))
        }

        /// Returns the decoded metadata value ([``]) for the given [`Ticker`].
        fn decoded_asset_metadata_value(&mut self, ticker: Ticker) -> Result<()> {
            let _asset_metadata_value = self.asset_metadata_value(ticker)?;
            // TODO: decode here
            unimplemented!()
        }

        /// Returns [`Balance`] representing the royalty amount that the artist will receive for a NFT transfer of `transfer_price`
        /// for the given `collection_ticker`.
        pub fn get_royalty_amount(
            &mut self,
            collection_ticker: Ticker,
            transfer_price: Balance,
        ) -> Result<Balance> {
            let royalty_percentage = self.royalty_percentage(collection_ticker)?;
            // TODO: verify this
            Ok((royalty_percentage.0 as Balance * transfer_price) / 100)
        }

        /// Returns the [`Percent`] that corresponds to percentage amount that the artist receives as royalty for each NFT transfer.
        fn royalty_percentage(&mut self, ticker: Ticker) -> Result<Percent> {
            let _decoded_metadata_value = self.decoded_asset_metadata_value(ticker)?;
            // TODO: return percent
            unimplemented!()
        }

        /// Adds a settlement instruction.
        ///
        /// The instruction will have three legs. One [`Leg`] where [`NFTTransferDetails::nft_owner_portfolio`] is transferring
        /// [`NFTTransferDetails::nft_id`] to [`NFTTransferDetails::nft_receiver_portfolio`], another leg where
        /// [`NFTOffer::payer_portfolio`] sends [`NFTOffer::transfer_price`] to [`NFTOffer::receiver_portfolio`], and one leg
        /// where the payer is transferring the royalty to the artist.
        ///
        /// Note: Call `get_royalty_amount` to figure out the amount for the royalty.
        pub fn create_transfer(
            &mut self,
            _venue_id: VenueId,
            nft_transfer_details: NFTTransferDetails,
            nft_offer: NFTOffer,
        ) -> Result<()> {
            let _decoded_metadata_value =
                self.decoded_asset_metadata_value(nft_transfer_details.collection_ticker)?;

            Self::ensure_valid_transfer_values()?;

            let _legs: Vec<Leg> = self.setup_legs(nft_transfer_details, nft_offer)?;
            unimplemented!()
        }

        /// Ensures the metadata rules are being respected in the transfer
        fn ensure_valid_transfer_values() -> Result<()> {
            unimplemented!();
        }

        /// Returns a [`Vec<Leg>`] for an instruction transfering an NFT.
        fn setup_legs(
            &mut self,
            nft_transfer_details: NFTTransferDetails,
            nft_offer: NFTOffer,
        ) -> Result<Vec<Leg>> {
            // The first leg transfers the NFT to the buyer
            let nfts = NFTs {
                ticker: nft_transfer_details.collection_ticker,
                ids: vec![nft_transfer_details.nft_id],
            };
            let nft_leg = Leg::NonFungible {
                sender: nft_transfer_details.nft_owner_portfolio,
                receiver: nft_transfer_details.nft_receiver_portfolio,
                nfts,
            };
            // The second leg transfers the payment to the seller
            let nft_payment_leg = Leg::Fungible {
                sender: nft_offer.payer_portfolio,
                receiver: nft_offer.receiver_portfolio,
                ticker: nft_offer.purchase_ticker,
                amount: nft_offer.transfer_price,
            };
            // The third leg transfers the royalty to the artist
            let royalty_portfolio = Self::royalty_portfolio(&nft_offer.purchase_ticker)?;
            let royalty_amount = self.get_royalty_amount(
                nft_transfer_details.collection_ticker,
                nft_offer.transfer_price,
            )?;
            let royalty_leg = Leg::Fungible {
                sender: nft_offer.payer_portfolio,
                receiver: royalty_portfolio,
                ticker: nft_offer.purchase_ticker,
                amount: royalty_amount,
            };
            Ok(vec![nft_leg, nft_payment_leg, royalty_leg])
        }

        /// Returns the [`PortfolioId`] that will receive the royalty.
        fn royalty_portfolio(_royalty_ticker: &Ticker) -> Result<PortfolioId> {
            unimplemented!()
        }
    }

    mod types {
        use super::*;

        /// The details of an NFT transfer.
        pub struct NFTTransferDetails {
            /// The [`Ticker`] of the NFT collection.
            pub collection_ticker: Ticker,
            /// The [`NFTId`] of the non-fungible token being transferred.
            pub nft_id: NFTId,
            /// The [`PortfolioId`] that contains the NFT being sold.
            pub nft_owner_portfolio: PortfolioId,
            /// The [`PortfolioId`] that will receive the NFT.
            pub nft_receiver_portfolio: PortfolioId,
        }

        impl NFTTransferDetails {
            /// Creates an instance of [`NFTTransferDetails`].
            pub fn new(
                collection_ticker: Ticker,
                nft_id: NFTId,
                nft_owner_portfolio: PortfolioId,
                nft_receiver_portfolio: PortfolioId,
            ) -> Self {
                Self {
                    collection_ticker,
                    nft_id,
                    nft_owner_portfolio,
                    nft_receiver_portfolio,
                }
            }
        }

        /// The details of the proposed offer in exchange for the NFT.
        pub struct NFTOffer {
            /// The [`Ticker`] of the asset being used for buying the NFT.
            pub purchase_ticker: Ticker,
            /// The price the buyer is paying for the NFT.
            pub transfer_price: Balance,
            /// The [`PortfolioId`] that is paying for the NFT.
            pub payer_portfolio: PortfolioId,
            /// The [`PortfolioId`] that is receiving the payment for the NFT.
            pub receiver_portfolio: PortfolioId,
        }

        impl NFTOffer {
            /// Creates an instance of [`NFTOffer`].
            pub fn new(
                purchase_ticker: Ticker,
                transfer_price: Balance,
                payer_portfolio: PortfolioId,
                receiver_portfolio: PortfolioId,
            ) -> Self {
                Self {
                    purchase_ticker,
                    transfer_price,
                    payer_portfolio,
                    receiver_portfolio,
                }
            }
        }
    }
}
