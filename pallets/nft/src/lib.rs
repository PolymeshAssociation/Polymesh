#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::DispatchResult;
use frame_support::traits::Get;
use frame_support::{decl_error, decl_module, decl_storage, ensure, require_transactional};

use pallet_asset::Frozen;
use pallet_base::try_next_pre;
use pallet_portfolio::PortfolioNFT;
use polymesh_common_utilities::compliance_manager::ComplianceFnConfig;
pub use polymesh_common_utilities::traits::nft::{Config, Event, NFTTrait, WeightInfo};
use polymesh_primitives::asset::{AssetName, AssetType, NonFungibleType};
use polymesh_primitives::asset_metadata::{AssetMetadataKey, AssetMetadataValue};
use polymesh_primitives::nft::{
    NFTCollection, NFTCollectionId, NFTCollectionKeys, NFTCount, NFTId, NFTMetadataAttribute, NFTs,
};
use polymesh_primitives::{IdentityId, PortfolioId, PortfolioKind, Ticker, WeightMeter};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::{vec, vec::Vec};

type Asset<T> = pallet_asset::Module<T>;
type ExternalAgents<T> = pallet_external_agents::Module<T>;
type Identity<T> = pallet_identity::Module<T>;
type Portfolio<T> = pallet_portfolio::Module<T>;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

decl_storage!(
    trait Store for Module<T: Config> as NFT {
        /// The total number of NFTs per identity.
        pub NumberOfNFTs get(fn balance_of): double_map hasher(blake2_128_concat) Ticker, hasher(identity) IdentityId => NFTCount;

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
    pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin {

        type Error = Error<T>;

        const MaxNumberOfCollectionKeys: u8 = T::MaxNumberOfCollectionKeys::get();
        const MaxNumberOfNFTsCount: u32 = T::MaxNumberOfNFTsCount::get();

        /// Initializes the default event for this module.
        fn deposit_event() = default;

        /// Cretes a new `NFTCollection`.
        ///
        /// # Arguments
        /// * `origin` - contains the secondary key of the caller (i.e. who signed the transaction to execute this function).
        /// * `ticker` - the ticker associated to the new collection.
        /// * `nft_type` - in case the asset hasn't been created yet, one will be created with the given type.
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
        pub fn create_nft_collection(origin, ticker: Ticker, nft_type: Option<NonFungibleType>, collection_keys: NFTCollectionKeys) -> DispatchResult {
            Self::base_create_nft_collection(origin, ticker, nft_type, collection_keys)
        }

        /// Issues an NFT to the caller.
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
        #[weight = <T as Config>::WeightInfo::issue_nft(nft_metadata_attributes.len() as u32)]
        pub fn issue_nft(origin, ticker: Ticker, nft_metadata_attributes: Vec<NFTMetadataAttribute>, portfolio_kind: PortfolioKind) -> DispatchResult {
            Self::base_issue_nft(origin, ticker, nft_metadata_attributes, portfolio_kind)
        }

        /// Redeems the given NFT from the caller's portfolio.
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
        #[weight = <T as Config>::WeightInfo::redeem_nft(T::MaxNumberOfCollectionKeys::get() as u32)]
        pub fn redeem_nft(origin, ticker: Ticker, nft_id: NFTId, portfolio_kind: PortfolioKind) -> DispatchResult {
            Self::base_redeem_nft(origin, ticker, nft_id, portfolio_kind)
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
        /// Duplicate ids are not allowed.
        DuplicatedNFTId,
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
        /// Failed to transfer an NFT - identity count would overflow.
        InvalidNFTTransferCountOverflow,
        /// Failed to transfer an NFT - compliance failed.
        InvalidNFTTransferComplianceFailure,
        /// Failed to transfer an NFT - asset is frozen.
        InvalidNFTTransferFrozenAsset,
        /// Failed to transfer an NFT - the number of nfts in the identity is insufficient.
        InvalidNFTTransferInsufficientCount,
        /// The maximum number of metadata keys was exceeded.
        MaxNumberOfKeysExceeded,
        /// The maximum number of nfts being transferred in one leg was exceeded.
        MaxNumberOfNFTsPerLegExceeded,
        /// The NFT does not exist.
        NFTNotFound,
        /// At least one of the metadata keys has not been registered.
        UnregisteredMetadataKey,
        /// It is not possible to transferr zero nft.
        ZeroCount,
    }
}

impl<T: Config> Module<T> {
    fn base_create_nft_collection(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        nft_type: Option<NonFungibleType>,
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
            let nft_type = nft_type.ok_or(Error::<T>::InvalidAssetType)?;
            Asset::<T>::create_asset(
                origin,
                AssetName(ticker.as_slice().to_vec()),
                ticker.clone(),
                false,
                AssetType::NonFungible(nft_type),
                Vec::new(),
                None,
                true,
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

    fn base_issue_nft(
        origin: T::RuntimeOrigin,
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
        let new_balance = NumberOfNFTs::get(&ticker, &caller_portfolio.did)
            .checked_add(1)
            .ok_or(Error::<T>::BalanceOverflow)?;
        let nft_id = NextNFTId::try_mutate(&collection_id, try_next_pre::<T, _>)?;
        NumberOfNFTs::insert(&ticker, &caller_portfolio.did, new_balance);
        for (metadata_key, metadata_value) in nft_attributes.into_iter() {
            MetadataValue::insert((&collection_id, &nft_id), metadata_key, metadata_value);
        }
        PortfolioNFT::insert(caller_portfolio, (ticker, nft_id), true);

        Self::deposit_event(Event::IssuedNFT(
            caller_portfolio.did,
            collection_id,
            nft_id,
        ));
        Ok(())
    }

    fn base_redeem_nft(
        origin: T::RuntimeOrigin,
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
        let new_balance = NumberOfNFTs::get(&ticker, &caller_portfolio.did)
            .checked_sub(1)
            .ok_or(Error::<T>::BalanceUnderflow)?;
        NumberOfNFTs::insert(&ticker, &caller_portfolio.did, new_balance);
        PortfolioNFT::remove(&caller_portfolio, (&ticker, &nft_id));
        #[allow(deprecated)]
        MetadataValue::remove_prefix((&collection_id, &nft_id), None);

        Self::deposit_event(Event::RedeemedNFT(caller_portfolio.did, ticker, nft_id));
        Ok(())
    }

    /// Tranfer ownership of all NFTs.
    #[require_transactional]
    pub fn base_nft_transfer(
        sender_portfolio: &PortfolioId,
        receiver_portfolio: &PortfolioId,
        nfts: &NFTs,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        // Verifies if there is a collection associated to the NFTs
        CollectionTicker::try_get(nfts.ticker())
            .map_err(|_| Error::<T>::InvalidNFTTransferCollectionNotFound)?;
        // Verifies if all rules for transfering the NFTs are being respected
        Self::validate_nft_transfer(sender_portfolio, receiver_portfolio, &nfts, weight_meter)?;

        // Transfer ownership of the NFT
        // Update the balance of the sender and the receiver
        let transferred_amount = nfts.len() as u64;
        NumberOfNFTs::mutate(nfts.ticker(), sender_portfolio.did, |balance| {
            *balance -= transferred_amount
        });
        NumberOfNFTs::mutate(nfts.ticker(), receiver_portfolio.did, |balance| {
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
    pub fn validate_nft_transfer(
        sender_portfolio: &PortfolioId,
        receiver_portfolio: &PortfolioId,
        nfts: &NFTs,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        let nfts_transferred = nfts.len() as u64;
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
        // Verifies that the sender has the required nft count
        ensure!(
            NumberOfNFTs::get(nfts.ticker(), sender_portfolio.did) >= nfts_transferred,
            Error::<T>::InvalidNFTTransferInsufficientCount
        );
        // Verifies that the number of nfts being transferred are within the allowed limits
        Self::ensure_within_nfts_transfer_limits(nfts)?;
        // Verifies that all ids are unique
        Self::ensure_no_duplicate_nfts(nfts)?;
        // Verfies that the sender owns the nfts
        for nft_id in nfts.ids() {
            ensure!(
                PortfolioNFT::contains_key(sender_portfolio, (nfts.ticker(), nft_id)),
                Error::<T>::InvalidNFTTransferNFTNotOwned
            );
        }
        // Verfies that the receiver will not overflow
        NumberOfNFTs::get(nfts.ticker(), receiver_portfolio.did)
            .checked_add(nfts_transferred)
            .ok_or(Error::<T>::InvalidNFTTransferCountOverflow)?;

        // Verifies that all compliance rules are being respected
        if !T::Compliance::is_compliant(
            nfts.ticker(),
            sender_portfolio.did,
            receiver_portfolio.did,
            weight_meter,
        )? {
            return Err(Error::<T>::InvalidNFTTransferComplianceFailure.into());
        }

        Ok(())
    }

    /// Verifies that the number of NFTs being transferred is greater than zero and less or equal to `MaxNumberOfNFTsPerLeg`.
    pub fn ensure_within_nfts_transfer_limits(nfts: &NFTs) -> DispatchResult {
        ensure!(nfts.len() > 0, Error::<T>::ZeroCount);
        ensure!(
            nfts.len() <= (T::MaxNumberOfNFTsCount::get() as usize),
            Error::<T>::MaxNumberOfNFTsPerLegExceeded
        );
        Ok(())
    }

    /// Verifies that there are no duplicate ids in the `NFTs` struct.
    pub fn ensure_no_duplicate_nfts(nfts: &NFTs) -> DispatchResult {
        let unique_nfts: BTreeSet<&NFTId> = nfts.ids().iter().collect();
        ensure!(unique_nfts.len() == nfts.len(), Error::<T>::DuplicatedNFTId);
        Ok(())
    }
}

impl<T: Config> NFTTrait<T::RuntimeOrigin> for Module<T> {
    fn is_collection_key(ticker: &Ticker, metadata_key: &AssetMetadataKey) -> bool {
        match CollectionTicker::try_get(ticker) {
            Ok(collection_id) => {
                let key_set = CollectionKeys::get(&collection_id);
                key_set.contains(metadata_key)
            }
            Err(_) => false,
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn create_nft_collection(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        nft_type: Option<NonFungibleType>,
        collection_keys: NFTCollectionKeys,
    ) -> DispatchResult {
        Module::<T>::create_nft_collection(origin, ticker, nft_type, collection_keys)
    }
}
