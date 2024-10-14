use frame_support::storage::migration::move_prefix;
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

            // This storage was renamed to Assets and changed the Ticker key to AssetId.
            pub Tokens get(fn tokens): map hasher(blake2_128_concat) Ticker => Option<AssetDetails>;

            // This storage changed the Ticker key to AssetId.
            pub OldAssetNames get(fn asset_names):
                map hasher(blake2_128_concat) Ticker => Option<AssetName>;

            // This storage changed the Ticker key to AssetId.
            pub OldBalanceOf get(fn balance_of):
                double_map hasher(blake2_128_concat) Ticker, hasher(identity) IdentityId => Balance;

            // This storage was renamed to AssetIdentifiers and changed the Ticker key to AssetId.
            pub Identifiers get(fn identifiers):
                map hasher(blake2_128_concat) Ticker => Vec<AssetIdentifier>;

            // This storage changed the Ticker key to AssetId.
            pub OldFundingRound get(fn funding_round):
                map hasher(blake2_128_concat) Ticker => FundingRoundName;

            // This storage changed the Ticker key to AssetId.
            pub OldIssuedInFundingRound get(fn issued_in_funding_round):
                map hasher(blake2_128_concat) (Ticker, FundingRoundName) => Balance;

            // This storage changed the Ticker key to AssetId.
            pub OldFrozen get(fn frozen): map hasher(blake2_128_concat) Ticker => bool;

            // This storage was split into TickersOwnedByUser and SecurityTokensOwnedByUser.
            pub AssetOwnershipRelations get(fn asset_ownership_relation):
                double_map hasher(identity) IdentityId, hasher(blake2_128_concat) Ticker => AssetOwnershipRelation;

            // This storage changed the Ticker key to AssetId.
            pub OldAssetDocuments get(fn asset_documents):
                double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) DocumentId => Option<Document>;

            // This storage changed the Ticker key to AssetId.
            pub OldAssetDocumentsIdSequence get(fn asset_documents_id_sequence):
                map hasher(blake2_128_concat) Ticker => DocumentId;

            // This storage changed the Ticker key to AssetId.
            pub OldAssetMetadataValues get(fn asset_metadata_values):
                double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) AssetMetadataKey => Option<AssetMetadataValue>;

            // This storage changed the Ticker key to AssetId.
            pub OldAssetMetadataValueDetails get(fn asset_metadata_value_details):
                double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) AssetMetadataKey => Option<AssetMetadataValueDetail<T::Moment>>;

            // This storage changed the Ticker key to AssetId.
            pub OldAssetMetadataLocalNameToKey get(fn asset_metadata_local_name_to_key):
                double_map hasher(blake2_128_concat) Ticker, hasher(blake2_128_concat) AssetMetadataName => Option<AssetMetadataLocalKey>;

            // This storage changed the Ticker key to AssetId.
            pub OldAssetMetadataLocalKeyToName get(fn asset_metadata_local_key_to_name):
                double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) AssetMetadataLocalKey => Option<AssetMetadataName>;

            // This storage changed the Ticker key to AssetId.
            pub OldAssetMetadataLocalSpecs get(fn asset_metadata_local_specs):
                double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) AssetMetadataLocalKey => Option<AssetMetadataSpec>;

            // This storage has been removed.
            pub AssetMetadataNextLocalKey get(fn asset_metadata_next_local_key):
                map hasher(blake2_128_concat) Ticker => AssetMetadataLocalKey;

            // This storage was renamed to AssetsExemptFromAffirmation and changed the Ticker key to AssetId.
            pub TickersExemptFromAffirmation get(fn tickers_exempt_from_affirmation):
                map hasher(blake2_128_concat) Ticker => bool;

            // This storage was renamed to PreApprovedAsset and changed the Ticker key to AssetId.
            pub PreApprovedTicker get(fn pre_approved_tickers):
                double_map hasher(identity) IdentityId, hasher(blake2_128_concat) Ticker => bool;

            // This storage changed the Ticker key to AssetId.
            pub OldMandatoryMediators get(fn mandatory_mediators):
                map hasher(blake2_128_concat) Ticker => BoundedBTreeSet<IdentityId, T::MaxAssetMediators>;

            // This storage changed the Ticker key to AssetId.
            pub OldCurrentAssetMetadataLocalKey get(fn current_asset_metadata_local_key):
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
        let asset_id = AssetId::from(ticker);
        ticker_to_asset_id.insert(ticker, asset_id);
        UniqueTickerRegistration::<T>::insert(ticker, ticker_registration);
    });
    log::info!("Migrated {:?} Asset.Tickers entries.", count);

    let mut count = 0;
    log::info!("Moving items from Tokens to Assets");
    v4::Tokens::drain().for_each(|(ticker, asset_details)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetId::from(ticker));
        Assets::insert(asset_id, asset_details);
    });
    log::info!("Migrated {:?} Asset.Tokens entries.", count);

    let mut count = 0;
    log::info!("Updating types for the AssetNames storage");
    move_prefix(
        &AssetNames::final_prefix(),
        &v4::OldAssetNames::final_prefix(),
    );
    v4::OldAssetNames::drain().for_each(|(ticker, asset_name)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetId::from(ticker));
        AssetNames::insert(asset_id, asset_name);
    });
    log::info!("Migrated {:?} Asset.AssetNames entries.", count);

    let mut count = 0;
    log::info!("Updating types for the BalanceOf storage");
    move_prefix(
        &BalanceOf::final_prefix(),
        &v4::OldBalanceOf::final_prefix(),
    );
    v4::OldBalanceOf::drain().for_each(|(ticker, identity, balance)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetId::from(ticker));
        BalanceOf::insert(asset_id, identity, balance);
    });
    log::info!("Migrated {:?} Asset.BalanceOf entries.", count);

    let mut count = 0;
    log::info!("Moving items from Identifiers to AssetIdentifiers");
    v4::Identifiers::drain().for_each(|(ticker, identifiers)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetId::from(ticker));
        AssetIdentifiers::insert(asset_id, identifiers);
    });
    log::info!("Migrated {:?} Asset.Identifiers entries.", count);

    let mut count = 0;
    log::info!("Updating types for the FundingRound storage");
    move_prefix(
        &FundingRound::final_prefix(),
        &v4::OldFundingRound::final_prefix(),
    );
    v4::OldFundingRound::drain().for_each(|(ticker, name)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetId::from(ticker));
        FundingRound::insert(asset_id, name);
    });
    log::info!("Migrated {:?} Asset.FundingRound entries.", count);

    let mut count = 0;
    log::info!("Updating types for the IssuedInFundingRound storage");
    move_prefix(
        &IssuedInFundingRound::final_prefix(),
        &v4::OldIssuedInFundingRound::final_prefix(),
    );
    v4::OldIssuedInFundingRound::drain().for_each(|((ticker, name), balance)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetId::from(ticker));
        IssuedInFundingRound::insert((asset_id, name), balance);
    });
    log::info!("Migrated {:?} Asset.IssuedInFundingRound entries.", count);

    let mut count = 0;
    log::info!("Updating types for the Frozen storage");
    move_prefix(&Frozen::final_prefix(), &v4::OldFrozen::final_prefix());
    v4::OldFrozen::drain().for_each(|(ticker, frozen)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetId::from(ticker));
        Frozen::insert(asset_id, frozen);
    });
    log::info!("Migrated {:?} Asset.Frozen entries.", count);

    let mut count = 0;
    log::info!("Moving items from AssetOwnershipRelations to TickersOwnedByUser and SecurityTokensOwnedByUser");
    v4::AssetOwnershipRelations::drain().for_each(|(owner_did, ticker, ownership_detail)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetId::from(ticker));

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
    log::info!("Migrated {:?} Asset.AssetOwnershipRelation entries.", count);

    let mut count = 0;
    log::info!("Updating types for the AssetDocuments storage");
    move_prefix(
        &AssetDocuments::final_prefix(),
        &v4::OldAssetDocuments::final_prefix(),
    );
    v4::OldAssetDocuments::drain().for_each(|(ticker, doc_id, doc)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetId::from(ticker));

        AssetDocuments::insert(asset_id, doc_id, doc);
    });
    log::info!("Migrated {:?} Asset.AssetDocuments entries.", count);

    let mut count = 0;
    log::info!("Updating types for the AssetDocumentsIdSequence storage");
    move_prefix(
        &AssetDocumentsIdSequence::final_prefix(),
        &v4::OldAssetDocumentsIdSequence::final_prefix(),
    );
    v4::OldAssetDocumentsIdSequence::drain().for_each(|(ticker, seq)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetId::from(ticker));

        AssetDocumentsIdSequence::insert(asset_id, seq);
    });
    log::info!(
        "Migrated {:?} Asset.AssetDocumentsIdSequence entries.",
        count
    );

    let mut count = 0;
    log::info!("Updating types for the AssetMetadataValues storage");
    move_prefix(
        &AssetMetadataValues::final_prefix(),
        &v4::OldAssetMetadataValues::final_prefix(),
    );
    v4::OldAssetMetadataValues::drain().for_each(|(ticker, key, value)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetId::from(ticker));

        AssetMetadataValues::insert(asset_id, key, value);
    });
    log::info!("Migrated {:?} Asset.AssetMetadataValues entries.", count);

    let mut count = 0;
    log::info!("Updating types for the AssetMetadataValueDetails storage");
    move_prefix(
        &AssetMetadataValueDetails::<T>::final_prefix(),
        &v4::OldAssetMetadataValueDetails::<T>::final_prefix(),
    );
    v4::OldAssetMetadataValueDetails::<T>::drain().for_each(|(ticker, key, value)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetId::from(ticker));

        AssetMetadataValueDetails::<T>::insert(asset_id, key, value);
    });
    log::info!(
        "Migrated {:?} Asset.AssetMetadataValueDetails entries.",
        count
    );

    let mut count = 0;
    log::info!("Updating types for the AssetMetadataLocalNameToKey storage");
    move_prefix(
        &AssetMetadataLocalNameToKey::final_prefix(),
        &v4::OldAssetMetadataLocalNameToKey::final_prefix(),
    );
    v4::OldAssetMetadataLocalNameToKey::drain().for_each(|(ticker, name, local_key)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetId::from(ticker));

        AssetMetadataLocalNameToKey::insert(asset_id, name, local_key);
    });
    log::info!(
        "Migrated {:?} Asset.AssetMetadataLocalNameToKey entries.",
        count
    );

    let mut count = 0;
    log::info!("Updating types for the AssetMetadataLocalKeyToName storage");
    move_prefix(
        &AssetMetadataLocalKeyToName::final_prefix(),
        &v4::OldAssetMetadataLocalKeyToName::final_prefix(),
    );
    v4::OldAssetMetadataLocalKeyToName::drain().for_each(|(ticker, local_key, name)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetId::from(ticker));

        AssetMetadataLocalKeyToName::insert(asset_id, local_key, name);
    });
    log::info!(
        "Migrated {:?} Asset.AssetMetadataLocalKeyToName entries.",
        count
    );

    let mut count = 0;
    log::info!("Updating types for the AssetMetadataLocalSpecs storage");
    move_prefix(
        &AssetMetadataLocalSpecs::final_prefix(),
        &v4::OldAssetMetadataLocalSpecs::final_prefix(),
    );
    v4::OldAssetMetadataLocalSpecs::drain().for_each(|(ticker, local_key, spec)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetId::from(ticker));

        AssetMetadataLocalSpecs::insert(asset_id, local_key, spec);
    });
    log::info!(
        "Migrated {:?} Asset.AssetMetadataLocalSpecs entries.",
        count
    );

    log::info!("Removing old AssetMetadataNextLocalKey storage");
    let res = v4::AssetMetadataNextLocalKey::clear(u32::max_value(), None);
    log::info!(
        "Cleared {:?} items from Asset.AssetMetadataNextLocalKey",
        res.unique
    );

    let mut count = 0;
    log::info!("Moving items from TickersExemptFromAffirmation to AssetsExemptFromAffirmation");
    v4::TickersExemptFromAffirmation::drain().for_each(|(ticker, exempt)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetId::from(ticker));

        AssetsExemptFromAffirmation::insert(asset_id, exempt);
    });
    log::info!(
        "Migrated {:?} Asset.TickersExemptFromAffirmation entries.",
        count
    );

    let mut count = 0;
    log::info!("Moving items from PreApprovedTicker to PreApprovedAsset");
    v4::PreApprovedTicker::drain().for_each(|(did, ticker, approved)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetId::from(ticker));

        PreApprovedAsset::insert(did, asset_id, approved);
    });
    log::info!("Migrated {:?} Asset.PreApprovedTicker entries.", count);

    let mut count = 0;
    log::info!("Updating types for the MandatoryMediators storage");
    move_prefix(
        &MandatoryMediators::<T>::final_prefix(),
        &v4::OldMandatoryMediators::<T>::final_prefix(),
    );
    v4::OldMandatoryMediators::<T>::drain().for_each(|(ticker, mediators)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetId::from(ticker));

        MandatoryMediators::<T>::insert(asset_id, mediators);
    });
    log::info!("Migrated {:?} Asset.MandatoryMediators entries.", count);

    let mut count = 0;
    log::info!("Updating types for the CurrentAssetMetadataLocalKey storage");
    move_prefix(
        &CurrentAssetMetadataLocalKey::final_prefix(),
        &v4::OldCurrentAssetMetadataLocalKey::final_prefix(),
    );
    v4::OldCurrentAssetMetadataLocalKey::drain().for_each(|(ticker, current_key)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetId::from(ticker));

        CurrentAssetMetadataLocalKey::insert(asset_id, current_key);
    });
    log::info!(
        "Migrated {:?} Asset.CurrentAssetMetadataLocalKey entries.",
        count
    );

    let mut count = 0;
    log::info!("Adding link from legacy tickers to an asset_id");
    for (ticker, asset_id) in ticker_to_asset_id.into_iter() {
        count += 1;
        AssetIdTicker::insert(asset_id, ticker);
        TickerAssetId::insert(ticker, asset_id);
    }
    log::info!(
        "Added {:?} Asset.TickerAssetId/AssetIdTicker entries",
        count
    );

    log::info!("AssetMetadataNextGlobalKey has been cleared");
    v4::AssetMetadataNextGlobalKey::kill();
}
