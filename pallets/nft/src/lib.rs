#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::DispatchResult;
use frame_support::traits::Get;
use frame_support::{decl_error, decl_module, decl_storage};
use frame_support::{ensure, require_transactional};
use pallet_asset::Frozen;
use pallet_base::try_next_pre;
use pallet_portfolio::PortfolioNFT;
use polymesh_common_utilities::compliance_manager::Config as ComplianceManagerConfig;
use polymesh_common_utilities::constants::currency::ONE_UNIT;
use polymesh_common_utilities::constants::ERC1400_TRANSFER_SUCCESS;
pub use polymesh_common_utilities::traits::nft::{Config, Event, WeightInfo};
use polymesh_primitives::asset::{AssetName, AssetType};
use polymesh_primitives::asset_metadata::{AssetMetadataKey, AssetMetadataValue};
use polymesh_primitives::nft::{
    NFTCollection, NFTCollectionId, NFTCollectionKeys, NFTId, NFTMetadataAttribute, NFTs,
};
use polymesh_primitives::{Balance, IdentityId, PortfolioId, PortfolioKind, Ticker};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::vec::Vec;

type Asset<T> = pallet_asset::Module<T>;
type ExternalAgents<T> = pallet_external_agents::Module<T>;
type Identity<T> = pallet_identity::Module<T>;
type Portfolio<T> = pallet_portfolio::Module<T>;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

decl_storage!(
    trait Store for Module<T: Config> as NFT {
        /// The total number of NFTs per identity.
        pub BalanceOf get(fn balance_of): double_map hasher(blake2_128_concat) Ticker, hasher(identity) IdentityId => Balance;

        /// The collection id corresponding to each ticker.
        pub CollectionTicker get(fn collection_ticker): map hasher(blake2_128_concat) Ticker => NFTCollectionId;

        /// All collection details for a given collection id.
        pub Collection get(fn nft_collection): map hasher(blake2_128_concat) NFTCollectionId => NFTCollection;

        /// All mandatory metadata keys for a given collection.
        pub CollectionKeys get(fn collection_keys): map hasher(blake2_128_concat) NFTCollectionId => BTreeSet<AssetMetadataKey>;

        /// The metadata value of an nft given its collection id, token id and metadata key.
        pub MetadataValue get(fn metadata_value): double_map hasher(blake2_128_concat) (NFTCollectionId, NFTId), hasher(blake2_128_concat) AssetMetadataKey => AssetMetadataValue;

        /// The next available id for an NFT collection.
        pub NextCollectionId get(fn collection_id): NFTCollectionId;

        /// The next available id for an NFT within a collection.
        pub NextNFTId get(fn nft_id): map hasher(blake2_128_concat) NFTCollectionId => NFTId;
    }
);

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {

        type Error = Error<T>;

        const MaxNumberOfCollectionKeys: u8 = T::MaxNumberOfCollectionKeys::get();

        /// Initializes the default event for this module.
        fn deposit_event() = default;

        /// Cretes a new `NFTCollection`.
        ///
        /// # Arguments
        /// * `origin` - contains the secondary key of the caller (i.e. who signed the transaction to execute this function).
        /// * `ticker` - the ticker associated to the new collection.
        /// * `asset_type` - in case the asset hasn't been created yet, one will be created with the given type.
        /// * `collection_keys` - all mandatory metadata keys that the tokens in the collection must have.
        ///
        /// ## Errors
        /// - `CollectionAlredyRegistered` - if the ticker is already associated to an NFT collection.
        /// - `InvalidAssetType` - if the associated asset is not of type NFT.
        /// - `MaxNumberOfKeysExceeded` - if the number of metadata keys for the collection is greater than the maximum allowed.
        /// - `UnregisteredMetadataKey` - if any of the metadata keys needed for the collection has not been registered.
        /// - `DuplicateMetadataKey` - if a duplicate metadata keys has been passed as input.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::create_nft_collection(collection_keys.len() as u32)]
        pub fn create_nft_collection(origin, ticker: Ticker, asset_type: Option<AssetType>, collection_keys: NFTCollectionKeys) -> DispatchResult {
            Self::base_create_nft_collection(origin, ticker, asset_type, collection_keys)
        }

        /// Mints an NFT to the caller.
        ///
        /// # Arguments
        /// * `origin` - is a signer that has permissions to act as an agent of `ticker`.
        /// * `ticker` - the ticker of the NFT collection.
        /// * `nft_metadata_attributes` - all mandatory metadata keys and values for the NFT.
        /// - `portfolio_kind` - the portfolio that will receive the minted nft.
        ///
        /// ## Errors
        /// - `CollectionNotFound` - if the collection associated to the given ticker has not been created.
        /// - `InvalidMetadataAttribute` - if the number of attributes is not equal to the number set in the collection or attempting to set a value for a key not definied in the collection.
        /// - `DuplicateMetadataKey` - if a duplicate metadata keys has been passed as input.
        ///
        ///
        /// # Permissions
        /// * Asset
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::mint_nft(nft_metadata_attributes.len() as u32)]
        pub fn mint_nft(origin, ticker: Ticker, nft_metadata_attributes: Vec<NFTMetadataAttribute>, portfolio_kind: PortfolioKind) -> DispatchResult {
            Self::base_mint_nft(origin, ticker, nft_metadata_attributes, portfolio_kind)
        }

        /// Burns the given NFT from the caller's portfolio.
        ///
        /// # Arguments
        /// * `origin` - is a signer that has permissions to act as an agent of `ticker`.
        /// * `ticker` - the ticker of the NFT collection.
        /// * `nft_id` - the id of the NFT to be burned.
        /// * `portfolio_kind` - the portfolio that contains the nft.
        ///
        /// ## Errors
        /// - `CollectionNotFound` - if the collection associated to the given ticker has not been created.
        /// - `NFTNotFound` - if the given NFT does not exist in the portfolio.
        ///
        /// # Permissions
        /// * Asset
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::burn_nft(T::MaxNumberOfCollectionKeys::get() as u32)]
        pub fn burn_nft(origin, ticker: Ticker, nft_id: NFTId, portfolio_kind: PortfolioKind) -> DispatchResult {
            Self::base_burn_nft(origin, ticker, nft_id, portfolio_kind)
        }
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        /// An overflow while calculating the balance.
        BalanceOverflow,
        /// An underflow while calculating the balance.
        BalanceUnderflow,
        /// The ticker is already associated to an NFT collection.
        CollectionAlredyRegistered,
        /// The NFT collection does not exist.
        CollectionNotFound,
        /// A duplicate metadata key has been passed as parameter.
        DuplicateMetadataKey,
        /// The asset must be of type non-fungible.
        InvalidAssetType,
        /// Either the number of keys or the key identifier does not match the keys defined for the collection.
        InvalidMetadataAttribute,
        /// Failed to transfer an NFT - NFT collection not found.
        InvalidNFTTransferCollectionNotFound,
        /// Failed to transfer an NFT - attempt to move to the same portfolio.
        InvalidNFTTransferSamePortfolio,
        /// Failed to transfer an NFT - NFT not found in portfolio.
        InvalidNFTTransferNFTNotOwned,
        /// Failed to transfer an NFT - balance would overflow.
        InvalidNFTTransferBalanceOverflow,
        /// Failed to transfer an NFT - not enough balance.
        InvalidNFTTransferNoBalance,
        /// Failed to transfer an NFT - compliance failed.
        InvalidNFTTransferComplianceFailure,
        /// Failed to transfer an NFT - asset is frozen.
        InvalidNFTTransferFrozenAsset,
        /// The maximum number of metadata keys was exceeded.
        MaxNumberOfKeysExceeded,
        /// The NFT does not exist.
        NFTNotFound,
        /// At least one of the metadata keys has not been registered.
        UnregisteredMetadataKey,
    }
}

impl<T: Config> Module<T> {
    fn base_create_nft_collection(
        origin: T::Origin,
        ticker: Ticker,
        asset_type: Option<AssetType>,
        collection_keys: NFTCollectionKeys,
    ) -> DispatchResult {
        // Verifies if the asset has already been created and the caller's permission to create the collection
        let (create_asset, caller_did) = {
            match Asset::<T>::nft_asset(&ticker) {
                Some(is_nft_asset) => {
                    ensure!(is_nft_asset, Error::<T>::InvalidAssetType);
                    let caller_did =
                        <ExternalAgents<T>>::ensure_agent_asset_perms(origin.clone(), ticker)?
                            .primary_did;
                    (false, caller_did)
                }
                None => {
                    let caller_did = Identity::<T>::ensure_perms(origin.clone())?;
                    (true, caller_did)
                }
            }
        };

        // Verifies if the ticker is already associated to an NFT collection
        ensure!(
            !CollectionTicker::contains_key(&ticker),
            Error::<T>::CollectionAlredyRegistered
        );

        // Verifies if the maximum number of keys is respected
        ensure!(
            collection_keys.len() <= (T::MaxNumberOfCollectionKeys::get() as usize),
            Error::<T>::MaxNumberOfKeysExceeded
        );

        // Verifies that there are no duplicated keys
        let n_keys = collection_keys.len();
        let collection_keys: BTreeSet<AssetMetadataKey> = collection_keys.into_iter().collect();
        ensure!(
            n_keys == collection_keys.len(),
            Error::<T>::DuplicateMetadataKey
        );

        // Verifies that all keys have been registered
        for key in &collection_keys {
            ensure!(
                Asset::<T>::check_asset_metadata_key_exists(&ticker, key),
                Error::<T>::UnregisteredMetadataKey
            )
        }

        // Creates an nft asset if it hasn't been created yet
        if create_asset {
            let asset_type = asset_type.ok_or(Error::<T>::InvalidAssetType)?;
            ensure!(asset_type.is_non_fungible(), Error::<T>::InvalidAssetType);
            Asset::<T>::create_asset(
                origin,
                AssetName(ticker.as_slice().to_vec()),
                ticker.clone(),
                false,
                asset_type,
                Vec::new(),
                None,
                false,
            )?;
        }

        // Creates the nft collection
        let collection_id = NextCollectionId::try_mutate(try_next_pre::<T, _>)?;
        let nft_collection = NFTCollection::new(collection_id, ticker.clone());
        Collection::insert(&collection_id, nft_collection);
        CollectionKeys::insert(&collection_id, collection_keys);
        CollectionTicker::insert(&ticker, &collection_id);

        Self::deposit_event(Event::NftCollectionCreated(
            caller_did,
            ticker,
            collection_id,
        ));
        Ok(())
    }

    fn base_mint_nft(
        origin: T::Origin,
        ticker: Ticker,
        metadata_attributes: Vec<NFTMetadataAttribute>,
        portfolio_kind: PortfolioKind,
    ) -> DispatchResult {
        // Verifies if the collection exists
        let collection_id =
            CollectionTicker::try_get(&ticker).map_err(|_| Error::<T>::CollectionNotFound)?;

        // Verifies if the caller has the right permissions (regarding asset and portfolio)
        let caller_portfolio = Asset::<T>::ensure_agent_with_custody_and_perms(
            origin,
            ticker.clone(),
            portfolio_kind,
        )?;

        Portfolio::<T>::ensure_portfolio_validity(&caller_portfolio)?;

        // Verifies that all mandatory keys are being set and that there are no duplicated keys
        let mandatory_keys: BTreeSet<AssetMetadataKey> = Self::collection_keys(&collection_id);
        ensure!(
            mandatory_keys.len() == metadata_attributes.len(),
            Error::<T>::InvalidMetadataAttribute
        );

        let n_keys = metadata_attributes.len();
        let nft_attributes: BTreeMap<_, _> = metadata_attributes
            .into_iter()
            .map(|a| (a.key, a.value))
            .collect();
        ensure!(
            n_keys == nft_attributes.len(),
            Error::<T>::DuplicateMetadataKey
        );

        for metadata_key in nft_attributes.keys() {
            ensure!(
                mandatory_keys.contains(metadata_key),
                Error::<T>::InvalidMetadataAttribute
            );
        }

        // Mints the NFT and adds it to the caller's portfolio
        let new_balance = BalanceOf::get(&ticker, &caller_portfolio.did)
            .checked_add(ONE_UNIT)
            .ok_or(Error::<T>::BalanceOverflow)?;
        let nft_id = NextNFTId::try_mutate(&collection_id, try_next_pre::<T, _>)?;
        BalanceOf::insert(&ticker, &caller_portfolio.did, new_balance);
        for (metadata_key, metadata_value) in nft_attributes.into_iter() {
            MetadataValue::insert((&collection_id, &nft_id), metadata_key, metadata_value);
        }
        PortfolioNFT::insert(caller_portfolio, (ticker, nft_id), true);

        Self::deposit_event(Event::MintedNft(
            caller_portfolio.did,
            collection_id,
            nft_id,
        ));
        Ok(())
    }

    fn base_burn_nft(
        origin: T::Origin,
        ticker: Ticker,
        nft_id: NFTId,
        portfolio_kind: PortfolioKind,
    ) -> DispatchResult {
        // Verifies if the collection exists
        let collection_id =
            CollectionTicker::try_get(&ticker).map_err(|_| Error::<T>::CollectionNotFound)?;

        // Ensure origin is agent with custody and permissions for default portfolio.
        let caller_portfolio =
            Asset::<T>::ensure_agent_with_custody_and_perms(origin, ticker, portfolio_kind)?;

        // Verifies if the NFT exists
        ensure!(
            PortfolioNFT::contains_key(&caller_portfolio, (&ticker, &nft_id)),
            Error::<T>::NFTNotFound
        );

        // Burns the NFT
        let new_balance = BalanceOf::get(&ticker, &caller_portfolio.did)
            .checked_sub(ONE_UNIT)
            .ok_or(Error::<T>::BalanceUnderflow)?;
        BalanceOf::insert(&ticker, &caller_portfolio.did, new_balance);
        PortfolioNFT::remove(&caller_portfolio, (&ticker, &nft_id));
        MetadataValue::remove_prefix((&collection_id, &nft_id), None);

        Self::deposit_event(Event::BurnedNFT(caller_portfolio.did, ticker, nft_id));
        Ok(())
    }

    /// Tranfer ownership of all NFTs.
    #[require_transactional]
    pub fn base_nft_transfer(
        sender_portfolio: &PortfolioId,
        receiver_portfolio: &PortfolioId,
        nfts: &NFTs,
    ) -> DispatchResult {
        // Verifies if there is a collection associated to the NFTs
        CollectionTicker::try_get(nfts.ticker())
            .map_err(|_| Error::<T>::InvalidNFTTransferCollectionNotFound)?;
        // Verifies if all rules for transfering the NFTs are being respected
        Self::validate_nft_transfer(sender_portfolio, receiver_portfolio, &nfts)?;

        // Transfer ownership of the NFT
        // Update the balance of the sender and the receiver
        let transferred_amount = nfts.amount();
        BalanceOf::mutate(nfts.ticker(), sender_portfolio.did, |balance| {
            *balance -= transferred_amount
        });
        BalanceOf::mutate(nfts.ticker(), receiver_portfolio.did, |balance| {
            *balance += transferred_amount
        });
        // Update the portfolio of the sender and the receiver
        for nft_id in nfts.ids() {
            PortfolioNFT::remove(sender_portfolio, (nfts.ticker(), nft_id));
            PortfolioNFT::insert(receiver_portfolio, (nfts.ticker(), nft_id), true);
        }
        Ok(())
    }

    /// Verifies if and the sender and receiver are not the same, if both have valid balances,
    /// if the sender owns the nft, and if all compliance rules are being respected.
    fn validate_nft_transfer(
        sender_portfolio: &PortfolioId,
        receiver_portfolio: &PortfolioId,
        nfts: &NFTs,
    ) -> DispatchResult {
        let transferred_amount = nfts.amount();
        // Verifies that the sender and receiver are not the same
        ensure!(
            sender_portfolio != receiver_portfolio,
            Error::<T>::InvalidNFTTransferSamePortfolio
        );
        // Verifies that the asset is not frozen
        ensure!(
            !Frozen::get(nfts.ticker()),
            Error::<T>::InvalidNFTTransferFrozenAsset
        );
        // Verifies that the sender has the required balance
        ensure!(
            BalanceOf::get(nfts.ticker(), sender_portfolio.did) >= transferred_amount,
            Error::<T>::InvalidNFTTransferNoBalance
        );
        // Verfies that the sender owns the nfts
        for nft_id in nfts.ids() {
            ensure!(
                PortfolioNFT::contains_key(sender_portfolio, (nfts.ticker(), nft_id)),
                Error::<T>::InvalidNFTTransferNFTNotOwned
            );
        }
        // Verfies that the receiver will not overflow
        BalanceOf::get(nfts.ticker(), receiver_portfolio.did)
            .checked_add(transferred_amount)
            .ok_or(Error::<T>::InvalidNFTTransferBalanceOverflow)?;
        // Verifies that all compliance rules are being respected
        let code = T::Compliance::verify_restriction(
            nfts.ticker(),
            Some(sender_portfolio.did),
            Some(receiver_portfolio.did),
            transferred_amount,
        )?;
        if code != ERC1400_TRANSFER_SUCCESS {
            return Err(Error::<T>::InvalidNFTTransferComplianceFailure.into());
        }

        Ok(())
    }
}
