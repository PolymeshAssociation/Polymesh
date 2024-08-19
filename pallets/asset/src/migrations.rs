use sp_runtime::runtime_logger::RuntimeLogger;
use sp_std::collections::btree_map::BTreeMap;

use super::*;

mod v4 {
    use super::*;

    decl_storage! {
        trait Store for Module<T: Config> as Asset {
            // This storage was renamed to UniqueTickerRegistration.
            pub Tickers get(fn ticker_registration):
                map hasher(blake2_128_concat) Ticker => Option<TickerRegistration<T::Moment>>;

            // This storage was renamed to Assets and changed the Ticker key to AssetID.
            pub Tokens get(fn tokens): map hasher(blake2_128_concat) Ticker => Option<AssetDetails>;

            // This storage changed the Ticker key to AssetID.
            pub AssetNames get(fn asset_names):
                map hasher(blake2_128_concat) Ticker => Option<AssetName>;

            // This storage changed the Ticker key to AssetID.
            pub BalanceOf get(fn balance_of):
                double_map hasher(blake2_128_concat) Ticker, hasher(identity) IdentityId => Balance;

            // This storage was renamed to AssetIdentifiers and changed the Ticker key to AssetID.
            pub Identifiers get(fn identifiers):
                map hasher(blake2_128_concat) Ticker => Vec<AssetIdentifier>;

            // This storage changed the Ticker key to AssetID.
            pub FundingRound get(fn funding_round):
                map hasher(blake2_128_concat) Ticker => FundingRoundName;

            // This storage changed the Ticker key to AssetID.
            pub IssuedInFundingRound get(fn issued_in_funding_round):
                map hasher(blake2_128_concat) (Ticker, FundingRoundName) => Balance;

            // This storage changed the Ticker key to AssetID.
            pub Frozen get(fn frozen): map hasher(blake2_128_concat) Ticker => bool;

            // This storage was split into TickersOwnedByUser and SecurityTokensOwnedByUser.
            pub AssetOwnershipRelations get(fn asset_ownership_relation):
                double_map hasher(identity) IdentityId, hasher(blake2_128_concat) Ticker => AssetOwnershipRelation;

            // This storage changed the Ticker key to AssetID.
            pub AssetDocuments get(fn asset_documents):
                double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) DocumentId => Option<Document>;

            // This storage changed the Ticker key to AssetID.
            pub AssetDocumentsIdSequence get(fn asset_documents_id_sequence):
                map hasher(blake2_128_concat) Ticker => DocumentId;

            // This storage changed the Ticker key to AssetID.
            pub AssetMetadataValues get(fn asset_metadata_values):
                double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) AssetMetadataKey => Option<AssetMetadataValue>;

            // This storage changed the Ticker key to AssetID.
            pub AssetMetadataValueDetails get(fn asset_metadata_value_details):
                double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) AssetMetadataKey => Option<AssetMetadataValueDetail<T::Moment>>;

            // This storage changed the Ticker key to AssetID.
            pub AssetMetadataLocalNameToKey get(fn asset_metadata_local_name_to_key):
                double_map hasher(blake2_128_concat) Ticker, hasher(blake2_128_concat) AssetMetadataName => Option<AssetMetadataLocalKey>;

            // This storage changed the Ticker key to AssetID.
            pub AssetMetadataLocalKeyToName get(fn asset_metadata_local_key_to_name):
                double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) AssetMetadataLocalKey => Option<AssetMetadataName>;

            // This storage changed the Ticker key to AssetID.
            pub AssetMetadataLocalSpecs get(fn asset_metadata_local_specs):
                double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) AssetMetadataLocalKey => Option<AssetMetadataSpec>;

            // This storage has been removed.
            pub AssetMetadataNextLocalKey get(fn asset_metadata_next_local_key):
                map hasher(blake2_128_concat) Ticker => AssetMetadataLocalKey;

            // This storage was renamed to AssetsExemptFromAffirmation and changed the Ticker key to AssetID.
            pub TickersExemptFromAffirmation get(fn tickers_exempt_from_affirmation):
                map hasher(blake2_128_concat) Ticker => bool;

            // This storage was renamed to PreApprovedAsset and changed the Ticker key to AssetID.
            pub PreApprovedTicker get(fn pre_approved_tickers):
                double_map hasher(identity) IdentityId, hasher(blake2_128_concat) Ticker => bool;

            // This storage changed the Ticker key to AssetID.
            pub MandatoryMediators get(fn mandatory_mediators):
                map hasher(blake2_128_concat) Ticker => BoundedBTreeSet<IdentityId, T::MaxAssetMediators>;

            // This storage changed the Ticker key to AssetID.
            pub CurrentAssetMetadataLocalKey get(fn current_asset_metadata_local_key):
                map hasher(blake2_128_concat) Ticker => Option<AssetMetadataLocalKey>;

            // This storage has been removed.
            pub AssetMetadataNextGlobalKey get(fn asset_metadata_next_global_key): AssetMetadataGlobalKey;
        }
    }

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin { }
    }
}

pub(crate) fn migrate_to_v5<T: Config>() {
    RuntimeLogger::init();
    let mut ticker_to_asset_id = BTreeMap::new();

    // Removes all elements in the old storage and inserts it in the new storage

    let mut count = 0;
    log::info!("Moving items from Tickers to UniqueTickerRegistration");
    v4::Tickers::<T>::drain().for_each(|(ticker, ticker_registration)| {
        count += 1;
        let asset_id = AssetID::from(ticker);
        ticker_to_asset_id.insert(ticker, asset_id);
        UniqueTickerRegistration::<T>::insert(ticker, ticker_registration);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Moving items from Tokens to Assets");
    v4::Tokens::drain().for_each(|(ticker, asset_details)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        Assets::insert(asset_id, asset_details);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the AssetNames storage");
    v4::AssetNames::drain().for_each(|(ticker, asset_name)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        AssetNames::insert(asset_id, asset_name);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the BalanceOf storage");
    v4::BalanceOf::drain().for_each(|(ticker, identity, balance)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        BalanceOf::insert(asset_id, identity, balance);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Moving items from Identifiers to AssetIdentifiers");
    v4::Identifiers::drain().for_each(|(ticker, identifiers)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        AssetIdentifiers::insert(asset_id, identifiers);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the FundingRound storage");
    v4::FundingRound::drain().for_each(|(ticker, name)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        FundingRound::insert(asset_id, name);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the IssuedInFundingRound storage");
    v4::IssuedInFundingRound::drain().for_each(|((ticker, name), balance)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        IssuedInFundingRound::insert((asset_id, name), balance);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the Frozen storage");
    v4::Frozen::drain().for_each(|(ticker, frozen)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        Frozen::insert(asset_id, frozen);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Moving items from AssetOwnershipRelations to TickersOwnedByUser and SecurityTokensOwnedByUser");
    v4::AssetOwnershipRelations::drain().for_each(|(owner_did, ticker, ownership_detail)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));

        match ownership_detail {
            AssetOwnershipRelation::TickerOwned => {
                TickersOwnedByUser::insert(owner_did, ticker, true);
            }
            AssetOwnershipRelation::AssetOwned => {
                TickersOwnedByUser::insert(owner_did, ticker, true);
                SecurityTokensOwnedByUser::insert(owner_did, asset_id, true);
            }
            AssetOwnershipRelation::NotOwned => {}
        }
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the AssetDocuments storage");
    v4::AssetDocuments::drain().for_each(|(ticker, doc_id, doc)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));

        AssetDocuments::insert(asset_id, doc_id, doc);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the AssetDocumentsIdSequence storage");
    v4::AssetDocumentsIdSequence::drain().for_each(|(ticker, seq)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));

        AssetDocumentsIdSequence::insert(asset_id, seq);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the AssetMetadataValues storage");
    v4::AssetMetadataValues::drain().for_each(|(ticker, key, value)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));

        AssetMetadataValues::insert(asset_id, key, value);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the AssetMetadataValueDetails storage");
    v4::AssetMetadataValueDetails::<T>::drain().for_each(|(ticker, key, value)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));

        AssetMetadataValueDetails::<T>::insert(asset_id, key, value);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the AssetMetadataLocalNameToKey storage");
    v4::AssetMetadataLocalNameToKey::drain().for_each(|(ticker, name, local_key)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));

        AssetMetadataLocalNameToKey::insert(asset_id, name, local_key);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the AssetMetadataLocalKeyToName storage");
    v4::AssetMetadataLocalKeyToName::drain().for_each(|(ticker, local_key, name)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));

        AssetMetadataLocalKeyToName::insert(asset_id, local_key, name);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the AssetMetadataLocalSpecs storage");
    v4::AssetMetadataLocalSpecs::drain().for_each(|(ticker, local_key, spec)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));

        AssetMetadataLocalSpecs::insert(asset_id, local_key, spec);
    });
    log::info!("{:?} items migrated", count);

    log::info!("Removing old AssetMetadataNextLocalKey storage");
    let res = v4::AssetMetadataNextLocalKey::clear(u32::max_value(), None);
    log::info!("{:?} items have been cleared", res.unique);

    let mut count = 0;
    log::info!("Moving items from TickersExemptFromAffirmation to AssetsExemptFromAffirmation");
    v4::TickersExemptFromAffirmation::drain().for_each(|(ticker, exempt)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));

        AssetsExemptFromAffirmation::insert(asset_id, exempt);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Moving items from PreApprovedTicker to PreApprovedAsset");
    v4::PreApprovedTicker::drain().for_each(|(did, ticker, approved)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));

        PreApprovedAsset::insert(did, asset_id, approved);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the MandatoryMediators storage");
    v4::MandatoryMediators::<T>::drain().for_each(|(ticker, mediators)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));

        MandatoryMediators::<T>::insert(asset_id, mediators);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the CurrentAssetMetadataLocalKey storage");
    v4::CurrentAssetMetadataLocalKey::drain().for_each(|(ticker, current_key)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));

        CurrentAssetMetadataLocalKey::insert(asset_id, current_key);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Adding link from legacy tickers to an asset_id");
    for (ticker, asset_id) in ticker_to_asset_id.into_iter() {
        count += 1;
        AssetIDTicker::insert(asset_id, ticker);
        TickerAssetID::insert(ticker, asset_id);
    }
    log::info!("{:?} items migrated", count);

    log::info!("AssetMetadataNextGlobalKey has been cleared");
    v4::AssetMetadataNextGlobalKey::kill();
}
