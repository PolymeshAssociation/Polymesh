#![cfg_attr(not(feature = "std"), no_std, no_main)]

use frame_support::pallet_prelude::Get;
use frame_support::BoundedBTreeSet;
use ink::storage::Mapping;
use scale::{Decode, Encode};
use scale_info::TypeInfo;
use sp_arithmetic::per_things::Perbill;

pub use nft_royalty::types::{NFTArtistRules, NFTOffer, NFTTransferDetails};
use polymesh_api::ink::basic_types::IdentityId;
use polymesh_api::ink::extension::{
    PolymeshEnvironment, PolymeshRuntimeErr as PolymeshChainExtError,
};
use polymesh_api::ink::Error as PolymeshChainError;
use polymesh_api::polymesh::types::polymesh_primitives::asset_metadata::{
    AssetMetadataKey, AssetMetadataLocalKey, AssetMetadataName, AssetMetadataValue,
};
use polymesh_api::polymesh::types::polymesh_primitives::identity_id::{
    PortfolioId, PortfolioKind, PortfolioName,
};
use polymesh_api::polymesh::types::polymesh_primitives::nft::{NFTId, NFTs};
use polymesh_api::polymesh::types::polymesh_primitives::settlement::{
    Leg, SettlementType, VenueId,
};
use polymesh_api::polymesh::types::polymesh_primitives::ticker::Ticker;
use polymesh_api::Api;
use polymesh_ink::{PolymeshError, PolymeshInk};

#[cfg(test)]
mod tests;

#[ink::contract(env = PolymeshEnvironment)]
mod nft_royalty {
    use super::*;

    /// The [`AssetMetadataName`] for the key that holds the mandatory NFT collection metadata.
    const NFT_METADATA_NAME: AssetMetadataName = AssetMetadataName(Vec::new());

    /// The contract's result type.
    pub type Result<T> = core::result::Result<T, Error>;

    /// Contract Errors.
    #[derive(Debug, Decode, Encode, TypeInfo)]
    pub enum Error {
        /// Polymesh ink error.
        PolymeshInk(PolymeshError),
        /// Polymesh runtime error.
        PolymeshRuntime(PolymeshChainError),
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
    }

    impl From<PolymeshChainError> for Error {
        fn from(error: PolymeshChainError) -> Self {
            Self::PolymeshRuntime(error)
        }
    }

    impl From<PolymeshChainExtError> for Error {
        fn from(err: PolymeshChainExtError) -> Self {
            Self::PolymeshRuntime(err.into())
        }
    }

    impl From<PolymeshError> for Error {
        fn from(err: PolymeshError) -> Self {
            Self::PolymeshInk(err)
        }
    }

    /// A contract that manages non-fungible token transfers.
    #[ink(storage)]
    #[derive(Default)]
    pub struct NftRoyalty {
        /// Upgradable Polymesh Ink API.
        api: PolymeshInk,
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
                api: PolymeshInk::new()?,
                contract_identity: Self::get_identity(Self::env().account_id())?,
                royalty_portfolios: Mapping::default(),
                metadata_keys: Mapping::default(),
            })
        }

        /// Inititializes the [`NftRoyalty`] storage with an address of the contract.
        #[ink(constructor)]
        pub fn new_with_hash(hash: Hash) -> Result<Self> {
            Ok(Self {
                api: PolymeshInk::new_with_hash(hash),
                contract_identity: Self::get_identity(Self::env().account_id())?,
                royalty_portfolios: Mapping::default(),
                metadata_keys: Mapping::default(),
            })
        }

        /// Update the `polymesh-ink` API using the tracker.
        ///
        /// Anyone can pay the gas fees to do the update using the tracker.
        #[ink(message)]
        pub fn update_polymesh_ink(&mut self) -> Result<()> {
            self.api.check_for_upgrade()?;
            Ok(())
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
        /// [`NFTTransferDetails::nft_id`] to [`NFTTransferDetails::nft_receiver_portfolio`], another leg where
        /// [`NFTOffer::payer_portfolio`] sends [`NFTOffer::transfer_price`] to [`NFTOffer::receiver_portfolio`], and one leg
        /// where the payer is transferring the royalty to the artist.
        ///
        /// Note: Call `royalty_percentage` to figure out the royalty percentage.
        #[ink(message)]
        pub fn create_transfer(
            &mut self,
            venue_id: VenueId,
            nft_transfer_details: NFTTransferDetails,
            nft_offer: NFTOffer,
        ) -> Result<()> {
            let nft_artist_rules =
                self.decoded_asset_metadata_value(nft_transfer_details.collection_ticker)?;

            Self::ensure_valid_transfer_values(&nft_artist_rules, &nft_offer)?;

            let royalty_portfolio = self.royalty_portfolio(Self::get_callers_identity()?)?;
            let legs: Vec<Leg> =
                self.setup_legs(nft_transfer_details, nft_offer, royalty_portfolio)?;

            self.api
                .add_and_affirm_instruction(venue_id, legs, vec![royalty_portfolio])?;
            Ok(())
        }

        /// Creates a portoflio owned by the contract's caller and transfer its custody to the smart contract.
        #[ink(message)]
        pub fn create_custody_portfolio(&mut self, portfolio_name: PortfolioName) -> Result<()> {
            let callers_identity = Self::get_callers_identity()?;

            if self.royalty_portfolios.contains(callers_identity) {
                return Err(Error::RoyaltyPortfolioAlreadyExists);
            }

            let api = Api::new();

            let portfolio_number = api
                .query()
                .portfolio()
                .next_portfolio_number(callers_identity)
                .map_err(Into::<Error>::into)?;

            self.api
                .create_custody_portfolio(callers_identity, portfolio_name)?;

            self.royalty_portfolios.insert(
                callers_identity,
                &PortfolioId {
                    did: callers_identity,
                    kind: PortfolioKind::User(portfolio_number),
                },
            );
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
            royalty_portfolio: PortfolioId,
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

        /// Returns [`Balance`] representing the royalty amount that the artist will receive for an NFT transfer of `transfer_price`
        /// for the given `collection_ticker`.
        fn get_royalty_amount(
            &mut self,
            collection_ticker: Ticker,
            transfer_price: Balance,
        ) -> Result<Balance> {
            let royalty_percentage = self.royalty_percentage(collection_ticker)?;
            Ok(royalty_percentage * transfer_price)
        }

        /// Returns the [`AssetMetadataKey`] for the given `ticker`.
        fn asset_metadata_key(&mut self, ticker: Ticker) -> Result<AssetMetadataKey> {
            // Checks if the key is already in storage.
            if let Some(key_id) = self.metadata_keys.get(ticker) {
                return Ok(AssetMetadataKey::Local(AssetMetadataLocalKey(key_id)));
            }

            let api = Api::new();
            let local_metadata_key = api
                .query()
                .asset()
                .asset_metadata_local_name_to_key(ticker, NFT_METADATA_NAME)
                .map_err(Into::<Error>::into)?
                .ok_or(Error::RoyaltyMetadataKeyNotFound(ticker))?;

            // Add the key to the storage
            self.metadata_keys.insert(ticker, &local_metadata_key.0);
            Ok(AssetMetadataKey::Local(local_metadata_key))
        }

        /// Returns the [`AssetMetadataValue`] for the given `ticker`.
        fn asset_metadata_value(&mut self, ticker: Ticker) -> Result<AssetMetadataValue> {
            let metadata_key = self.asset_metadata_key(ticker)?;

            let api = Api::new();
            api.query()
                .asset()
                .asset_metadata_values(ticker, metadata_key)
                .map_err(Into::<Error>::into)?
                .ok_or(Error::RoyaltyMetadataValueNotFound(ticker))
        }

        /// Returns the [`IdentityId`] for the given `account_id`.
        fn get_identity(account_id: AccountId) -> Result<IdentityId> {
            Self::env()
                .extension()
                .get_key_did(account_id)?
                .ok_or(Error::IdentityNotFound(account_id))
        }

        /// Returns the [`IdentityId`] of whoever called the contract.
        fn get_callers_identity() -> Result<IdentityId> {
            Self::get_identity(Self::env().caller())
        }

        /// Returns the [`PortfolioId`] that will receive the royalty for `caller_identity`.
        fn royalty_portfolio(&self, caller_identity: IdentityId) -> Result<PortfolioId> {
            self.royalty_portfolios
                .get(caller_identity)
                .ok_or(Error::RoyaltyPortfolioNotFound(caller_identity))
        }
    }

    pub mod types {
        use super::*;

        /// The details of an NFT transfer.
        #[derive(Decode, Encode, TypeInfo)]
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
        #[derive(Decode, Encode, TypeInfo)]
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

        /// Type used for definig an upper bound to the maximum number of tickers allowed for paying royalty.
        pub struct MaxNumberOfTickers(u32);
        /// The maximum number of tickers allowed for paying royalty.
        pub const MAX_TICKERS_ALLOWED: u32 = 8;

        impl Get<u32> for MaxNumberOfTickers {
            fn get() -> u32 {
                MAX_TICKERS_ALLOWED
            }
        }

        /// All mandatoty information NFT artists must set as metadata.
        #[derive(Decode, Encode, TypeInfo)]
        pub struct NFTArtistRules {
            /// All [`Ticker`] the artist is willing to receive as royalty.
            pub allowed_purchase_tickers: BoundedBTreeSet<Ticker, MaxNumberOfTickers>,
            /// The royalty percentage the artist will receive for each transfer.
            pub royalty_percentage: Perbill,
        }

        impl NFTArtistRules {
            /// Creates an instance of [`NFTArtistRules`].
            pub fn new(
                allowed_purchase_tickers: BoundedBTreeSet<Ticker, MaxNumberOfTickers>,
                royalty_percentage: Perbill,
            ) -> Self {
                Self {
                    allowed_purchase_tickers,
                    royalty_percentage,
                }
            }

            /// Returns `true` if `ticker` is in the [`NFTArtistRules::allowed_purchase_tickers`] set. Otherwise, returns `false`.
            pub fn is_ticker_allowed(&self, ticker: &Ticker) -> bool {
                self.allowed_purchase_tickers.contains(ticker)
            }
        }
    }
}
