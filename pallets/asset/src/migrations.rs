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

            // This storage was renamed to SecurityTokens and changed the Ticker key to AssetID.
            pub Tokens get(fn tokens): map hasher(blake2_128_concat) Ticker => Option<SecurityToken>;

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

            // This storage changed the Ticker key to AssetID.
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

    log::info!("Moving items from Tickers to UniqueTickerRegistration");
    v4::Tickers::<T>::drain().for_each(|(ticker, ticker_registration)| {
        let asset_id = AssetID::from(ticker);
        ticker_to_asset_id.insert(ticker, asset_id);
        UniqueTickerRegistration::<T>::insert(ticker, ticker_registration);
    });

    log::info!("Moving items from Tokens to SecurityTokens");
    v4::Tokens::drain().for_each(|(ticker, security_token)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        SecurityTokens::insert(asset_id, security_token);
    });

    log::info!("Updating types for the AssetNames storage");
    v4::AssetNames::drain().for_each(|(ticker, asset_name)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        AssetNames::insert(asset_id, asset_name);
    });

    log::info!("Updating types for the BalanceOf storage");
    v4::BalanceOf::drain().for_each(|(ticker, identity, balance)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        BalanceOf::insert(asset_id, identity, balance);
    });

    log::info!("Moving items from Identifiers to AssetIdentifiers");
    v4::Identifiers::drain().for_each(|(ticker, identifiers)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        AssetIdentifiers::insert(asset_id, identifiers);
    });

    log::info!("Updating types for the FundingRound storage");
    v4::FundingRound::drain().for_each(|(ticker, name)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        FundingRound::insert(asset_id, name);
    });

    log::info!("Updating types for the IssuedInFundingRound storage");
    v4::IssuedInFundingRound::drain().for_each(|((ticker, name), balance)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        IssuedInFundingRound::insert((asset_id, name), balance);
    });

    log::info!("Updating types for the Frozen storage");
    v4::Frozen::drain().for_each(|(ticker, frozen)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        Frozen::insert(asset_id, frozen);
    });

    log::info!("Moving items from AssetOwnershipRelations to TickersOwnedByUser and SecurityTokensOwnedByUser");
    v4::AssetOwnershipRelations::drain().for_each(|(owner_did, ticker, ownership_detail)| {
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

    log::info!("Updating types for the AssetDocuments storage");
    v4::AssetDocuments::drain().for_each(|(ticker, doc_id, doc)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));

        AssetDocuments::insert(asset_id, doc_id, doc);
    });

    log::info!("Updating types for the AssetDocumentsIdSequence storage");
    v4::AssetDocumentsIdSequence::drain().for_each(|(ticker, seq)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));

        AssetDocumentsIdSequence::insert(asset_id, seq);
    });

    log::info!("Updating types for the AssetMetadataValues storage");
    v4::AssetMetadataValues::drain().for_each(|(ticker, key, value)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));

        AssetMetadataValues::insert(asset_id, key, value);
    });

    log::info!("Updating types for the AssetMetadataValueDetails storage");
    v4::AssetMetadataValueDetails::<T>::drain().for_each(|(ticker, key, value)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));

        AssetMetadataValueDetails::<T>::insert(asset_id, key, value);
    });

    log::info!("Updating types for the AssetMetadataLocalNameToKey storage");
    v4::AssetMetadataLocalNameToKey::drain().for_each(|(ticker, name, local_key)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));

        AssetMetadataLocalNameToKey::insert(asset_id, name, local_key);
    });

    log::info!("Updating types for the AssetMetadataLocalKeyToName storage");
    v4::AssetMetadataLocalKeyToName::drain().for_each(|(ticker, local_key, name)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));

        AssetMetadataLocalKeyToName::insert(asset_id, local_key, name);
    });

    log::info!("Updating types for the AssetMetadataLocalSpecs storage");
    v4::AssetMetadataLocalSpecs::drain().for_each(|(ticker, local_key, spec)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));

        AssetMetadataLocalSpecs::insert(asset_id, local_key, spec);
    });

    log::info!("Updating types for the AssetMetadataNextLocalKey storage");
    v4::AssetMetadataNextLocalKey::drain().for_each(|(ticker, next)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));

        AssetMetadataNextLocalKey::insert(asset_id, next);
    });

    log::info!("Moving items from TickersExemptFromAffirmation to AssetsExemptFromAffirmation");
    v4::TickersExemptFromAffirmation::drain().for_each(|(ticker, exempt)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));

        AssetsExemptFromAffirmation::insert(asset_id, exempt);
    });

    log::info!("Moving items from PreApprovedTicker to PreApprovedAsset");
    v4::PreApprovedTicker::drain().for_each(|(did, ticker, approved)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));

        PreApprovedAsset::insert(did, asset_id, approved);
    });

    log::info!("Updating types for the MandatoryMediators storage");
    v4::MandatoryMediators::<T>::drain().for_each(|(ticker, mediators)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));

        MandatoryMediators::<T>::insert(asset_id, mediators);
    });

    log::info!("Updating types for the CurrentAssetMetadataLocalKey storage");
    v4::CurrentAssetMetadataLocalKey::drain().for_each(|(ticker, current_key)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));

        CurrentAssetMetadataLocalKey::insert(asset_id, current_key);
    });

    log::info!("Adding link from legacy tickers to an asset_id");
    for (ticker, asset_id) in ticker_to_asset_id.into_iter() {
        AssetIDTicker::insert(asset_id, ticker);
        TickerAssetID::insert(ticker, asset_id);
    }
}
