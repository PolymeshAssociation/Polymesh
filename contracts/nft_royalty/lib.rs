// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymesh Association

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

//! # NFT royalty handler smart contract.
//!
//! The goal of this contract is to handle the distribution of royalty for secondary NFT sales. This contract enforces royalty by
//! adding one [`Leg`] transfering [`NFTArtistRules::royalty_percentage`] of [`NFTOffer::transfer_price`] to the artist portfolio.
//!
//! ### Public Functions
//!
//! - `initialize_contract`: Initializes the contract creating a venue that will be controlled by the contract.
//! - `venue_id`: Returns the [`VenueId`] if the contract is initialized. Otherwise, returns an error.
//! - `contracts_identity`: Returns the [`IdentityId`] of the contract.
//! - `royalty_portfolio_identity`: Returns the [`PortfolioId`] of the contract's caller.
//! - `royalty_percentage`: Returns the [`Perbill`] that corresponds to the percentage amount that the artist receives as royalty for each NFT transfer.
//! - `decoded_asset_metadata_value`: Returns the decoded metadata value ([`NFTArtistRules`]) for the given [`Ticker`].
//! - `create_transfer`: Adds a settlement instruction. The instruction will have three legs. One [`Leg`] where [`NFTTransferDetails::nft_owner_portfolio`] is transferring [`NFTTransferDetails::nfts`] to [`NFTTransferDetails::nft_receiver_portfolio`], another leg where [`NFTOffer::payer_portfolio`] sends [`NFTOffer::transfer_price`] to [`NFTOffer::receiver_portfolio`], and one leg where the payer is transferring the royalty to the artist.
//! - `create_custody_portfolio`: Creates a portoflio owned by the contract's caller and transfer its custody to the smart contract.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use ink::storage::Mapping;
use scale::{Decode, Encode};
use sp_arithmetic::per_things::Perbill;

use polymesh_ink::{
    AssetMetadataKey, AssetMetadataLocalKey, AssetMetadataName, AssetMetadataValue, IdentityId,
    Leg, NFTId, NFTs, PolymeshEnvironment, PolymeshError, PolymeshInk, PortfolioId, PortfolioName,
    Ticker, VenueDetails, VenueId, VenueType,
};

pub use crate::nft_royalty::types::{NFTArtistRules, NFTOffer, NFTTransferDetails};

#[ink::contract(env = PolymeshEnvironment)]
mod nft_royalty {
    use super::*;
    use alloc::vec;

    /// The asset metadata name for the key that holds the mandatory NFT collection metadata.
    const NFT_METADATA_NAME: &[u8] = "v0_nft_madantory_metadata".as_bytes();
    /// The details of the venue controlled by the smart contract.
    const VENUE_DETAILS: &[u8] = "sc-controlled-venue".as_bytes();

    /// The contract's result type.
    pub type Result<T> = core::result::Result<T, Error>;

    /// Contract Errors.
    #[derive(Debug, Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Polymesh ink error.
        PolymeshInk(PolymeshError),
        /// [`IdentityId`] not found for the given [`AccountId`].
        IdentityNotFound(AccountId),
        /// Royalty metadata value not found.
        RoyaltyMetadataValueNotFound(Ticker),
        /// Royalty metadata key not found.
        RoyaltyMetadataKeyNotFound(Ticker),
        /// Trying to decode [`NFTArtistRules`] from [`AssetMetadataValue`] failed.
        FailedToDecodeMetadataValue(String),
        /// [`NFTOffer::purchase_ticker`] can't be used as royalty payment.
        TickerNotAllowedForRoyalty(Ticker),
        /// No [`PortfolioId`] for the given [`IdentityId`].
        RoyaltyPortfolioNotFound(IdentityId),
        /// The contract caller has already set up a custodial portfolio.
        RoyaltyPortfolioAlreadyExists,
        /// Contract hasn't been initialized.
        MissingContractInitialization,
        /// Contract has already been initialized.
        ContractIsAlreadyInitialized,
    }

    impl From<PolymeshError> for Error {
        fn from(err: PolymeshError) -> Self {
            Self::PolymeshInk(err)
        }
    }

    /// A contract that manages non-fungible token transfers.
    #[ink(storage)]
    pub struct NftRoyalty {
        /// Flag indicating whether the contract has been initialized.
        initialized: bool,
        /// The [`AccountId`] that called the contract's constructor.
        admin: AccountId,
        /// Venue owned by the contract.
        venue_id: Option<VenueId>,
        /// The identity of the contract.
        contract_identity: IdentityId,
        /// The portfolios that will receive the royalty value for each identity.
        royalty_portfolios: Mapping<IdentityId, PortfolioId>,
        /// The asset metadata key for each ticker.
        metadata_keys: Mapping<Ticker, u64>,
    }

    impl NftRoyalty {
        /// Inititializes the [`NftRoyalty`] storage.
        #[ink(constructor)]
        pub fn new() -> Result<Self> {
            Ok(Self {
                initialized: false,
                admin: Self::env().caller(),
                venue_id: None,
                contract_identity: PolymeshInk::get_our_did()?,
                royalty_portfolios: Mapping::default(),
                metadata_keys: Mapping::default(),
            })
        }

        /// Initializes the contract creating a venue that will be controlled by the contract.
        #[ink(message)]
        pub fn initialize_contract(&mut self) -> Result<VenueId> {
            if self.initialized {
                return Err(Error::ContractIsAlreadyInitialized);
            }

            self.contract_identity = PolymeshInk::get_our_did()?;
            let api = PolymeshInk::new()?;
            let venue_id =
                api.create_venue(VenueDetails(VENUE_DETAILS.to_vec()), VenueType::Other)?;
            self.venue_id = Some(venue_id);
            self.initialized = true;

            Ok(venue_id)
        }

        /// Returns the [`VenueId`] if the contract is initialized. Otherwise, returns an error.
        #[ink(message)]
        pub fn venue_id(&self) -> Result<VenueId> {
            self.venue_id.ok_or(Error::MissingContractInitialization)
        }

        /// Returns the [`IdentityId`] of the contract.
        #[ink(message)]
        pub fn contracts_identity(&self) -> IdentityId {
            self.contract_identity
        }

        /// Returns the [`PortfolioId`] of the contract's caller.
        #[ink(message)]
        pub fn royalty_portfolio_identity(&self) -> Result<PortfolioId> {
            let callers_identity = Self::get_callers_identity()?;
            self.royalty_portfolio(callers_identity)
        }

        /// Returns the [`Perbill`] that corresponds to the percentage amount that the artist receives as royalty for each NFT transfer.
        #[ink(message)]
        pub fn royalty_percentage(&mut self, ticker: Ticker) -> Result<Perbill> {
            let nft_artist_rules = self.decoded_asset_metadata_value(ticker)?;
            Ok(nft_artist_rules.royalty_percentage)
        }

        /// Returns the decoded metadata value ([`NFTArtistRules`]) for the given [`Ticker`].
        #[ink(message)]
        pub fn decoded_asset_metadata_value(&mut self, ticker: Ticker) -> Result<NFTArtistRules> {
            let asset_metadata_value = self.asset_metadata_value(ticker)?;
            NFTArtistRules::decode::<&[u8]>(&mut asset_metadata_value.0.as_ref())
                .map_err(|e| Error::FailedToDecodeMetadataValue(e.to_string()))
        }

        /// Adds a settlement instruction.
        ///
        /// The instruction will have three legs. One [`Leg`] where [`NFTTransferDetails::nft_owner_portfolio`] is transferring
        /// [`NFTTransferDetails::nfts`] to [`NFTTransferDetails::nft_receiver_portfolio`], another leg where
        /// [`NFTOffer::payer_portfolio`] sends [`NFTOffer::transfer_price`] to [`NFTOffer::receiver_portfolio`], and one leg
        /// where the payer is transferring the royalty to the artist.
        ///
        /// Note: Call `royalty_percentage` to figure out the royalty percentage.
        #[ink(message)]
        pub fn create_transfer(
            &mut self,
            nft_transfer_details: NFTTransferDetails,
            nft_offer: NFTOffer,
        ) -> Result<Vec<Leg>> {
            let venue_id = self.venue_id()?;
            let nft_artist_rules =
                self.decoded_asset_metadata_value(nft_transfer_details.collection_ticker)?;

            let royalty_portfolio = self.royalty_portfolio(nft_artist_rules.artist_identity)?;
            Self::ensure_valid_transfer_values(&nft_artist_rules, &nft_offer)?;

            let api = PolymeshInk::new()?;
            let legs: Vec<Leg> = self.setup_legs(
                nft_transfer_details,
                nft_offer,
                nft_artist_rules.royalty_percentage,
                royalty_portfolio,
            );
            api.add_and_affirm_instruction(venue_id, legs.clone(), vec![royalty_portfolio])?;
            Ok(legs)
        }

        /// Creates a portoflio owned by the contract's caller and transfer its custody to the smart contract.
        #[ink(message)]
        pub fn create_custody_portfolio(&mut self, portfolio_name: PortfolioName) -> Result<()> {
            if !self.initialized {
                return Err(Error::MissingContractInitialization);
            }

            let callers_identity = Self::get_callers_identity()?;
            if self.royalty_portfolios.contains(callers_identity) {
                return Err(Error::RoyaltyPortfolioAlreadyExists);
            }

            let api = PolymeshInk::new()?;
            let portfolio_id = api.create_custody_portfolio(callers_identity, portfolio_name)?;

            self.royalty_portfolios
                .insert(callers_identity, &portfolio_id);
            Ok(())
        }

        /// Ensures the metadata rules are being respected in the transfer
        fn ensure_valid_transfer_values(
            nft_artist_rules: &NFTArtistRules,
            nft_offer: &NFTOffer,
        ) -> Result<()> {
            if !nft_artist_rules.is_ticker_allowed(&nft_offer.purchase_ticker) {
                return Err(Error::TickerNotAllowedForRoyalty(nft_offer.purchase_ticker));
            }
            Ok(())
        }

        /// Returns a [`Vec<Leg>`] for an instruction transfering an NFT.
        fn setup_legs(
            &mut self,
            nft_transfer_details: NFTTransferDetails,
            nft_offer: NFTOffer,
            royalty_percentage: Perbill,
            royalty_portfolio: PortfolioId,
        ) -> Vec<Leg> {
            // The first leg transfers the NFT to the buyer
            let nfts = NFTs {
                ticker: nft_transfer_details.collection_ticker,
                ids: nft_transfer_details.nfts,
            };
            let nft_leg = Leg::NonFungible {
                sender: nft_transfer_details.nft_owner_portfolio,
                receiver: nft_transfer_details.nft_receiver_portfolio,
                nfts,
            };
            // Calculate the royalty_amount
            let royalty_amount = royalty_percentage * nft_offer.transfer_price;
            // The second leg transfers the payment to the seller
            let nft_payment_leg = Leg::Fungible {
                sender: nft_offer.payer_portfolio,
                receiver: nft_offer.receiver_portfolio,
                ticker: nft_offer.purchase_ticker,
                amount: nft_offer.transfer_price - royalty_amount,
            };
            // The third leg transfers the royalty to the artist
            let royalty_amount = royalty_percentage * nft_offer.transfer_price;
            let royalty_leg = Leg::Fungible {
                sender: nft_offer.payer_portfolio,
                receiver: royalty_portfolio,
                ticker: nft_offer.purchase_ticker,
                amount: royalty_amount,
            };
            vec![nft_leg, nft_payment_leg, royalty_leg]
        }

        /// Returns the [`AssetMetadataKey`] for the given `ticker`.
        fn asset_metadata_key(
            &mut self,
            ticker: Ticker,
            api: &PolymeshInk,
        ) -> Result<AssetMetadataKey> {
            // Checks if the key is already in storage.
            if let Some(key_id) = self.metadata_keys.get(ticker) {
                return Ok(AssetMetadataKey::Local(AssetMetadataLocalKey(key_id)));
            }

            let local_metadata_key = api
                .asset_metadata_local_name_to_key(
                    ticker,
                    AssetMetadataName(NFT_METADATA_NAME.to_vec()),
                )?
                .ok_or(Error::RoyaltyMetadataKeyNotFound(ticker))?;

            // Add the key to the storage
            self.metadata_keys.insert(ticker, &local_metadata_key.0);
            Ok(AssetMetadataKey::Local(local_metadata_key))
        }

        /// Returns the [`AssetMetadataValue`] for the given `ticker`.
        fn asset_metadata_value(&mut self, ticker: Ticker) -> Result<AssetMetadataValue> {
            let api = PolymeshInk::new()?;
            let metadata_key = self.asset_metadata_key(ticker, &api)?;

            api.asset_metadata_value(ticker, metadata_key)?
                .ok_or(Error::RoyaltyMetadataValueNotFound(ticker))
        }

        /// Returns the [`IdentityId`] of whoever called the contract.
        fn get_callers_identity() -> Result<IdentityId> {
            Ok(PolymeshInk::get_caller_did()?)
        }

        /// Returns the [`PortfolioId`] that will receive the royalty for `artist_identity`.
        fn royalty_portfolio(&self, artist_identity: IdentityId) -> Result<PortfolioId> {
            self.royalty_portfolios
                .get(artist_identity)
                .ok_or(Error::RoyaltyPortfolioNotFound(artist_identity))
        }
    }

    pub mod types {
        use super::*;

        /// The details of an NFT transfer.
        #[derive(Decode, Encode)]
        #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
        pub struct NFTTransferDetails {
            /// The [`Ticker`] of the NFT collection.
            pub collection_ticker: Ticker,
            /// All NFTs being transferred.
            pub nfts: Vec<NFTId>,
            /// The [`PortfolioId`] that contains the NFT being sold.
            pub nft_owner_portfolio: PortfolioId,
            /// The [`PortfolioId`] that will receive the NFT.
            pub nft_receiver_portfolio: PortfolioId,
        }

        impl NFTTransferDetails {
            /// Creates an instance of [`NFTTransferDetails`].
            pub fn new(
                collection_ticker: Ticker,
                nfts: Vec<NFTId>,
                nft_owner_portfolio: PortfolioId,
                nft_receiver_portfolio: PortfolioId,
            ) -> Self {
                Self {
                    collection_ticker,
                    nfts,
                    nft_owner_portfolio,
                    nft_receiver_portfolio,
                }
            }
        }

        /// The details of the proposed offer in exchange for the NFT.
        #[derive(Decode, Encode)]
        #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

        /// All mandatoty information NFT artists must set as metadata.
        #[derive(Decode, Encode)]
        #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
        pub struct NFTArtistRules {
            /// All [`Ticker`] the artist is willing to receive as royalty.
            pub allowed_purchase_tickers: BTreeSet<Ticker>,
            /// The royalty percentage the artist will receive for each transfer.
            pub royalty_percentage: Perbill,
            /// The identity that will receive royalty payments.
            pub artist_identity: IdentityId,
        }

        impl NFTArtistRules {
            /// Creates an instance of [`NFTArtistRules`].
            pub fn new(
                allowed_purchase_tickers: BTreeSet<Ticker>,
                royalty_percentage: Perbill,
                artist_identity: IdentityId,
            ) -> Self {
                Self {
                    allowed_purchase_tickers,
                    royalty_percentage,
                    artist_identity,
                }
            }

            /// Returns `true` if `ticker` is in the [`NFTArtistRules::allowed_purchase_tickers`] set. Otherwise, returns `false`.
            pub fn is_ticker_allowed(&self, ticker: &Ticker) -> bool {
                self.allowed_purchase_tickers.contains(ticker)
            }
        }
    }
}
