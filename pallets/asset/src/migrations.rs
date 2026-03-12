use frame_support::pallet_prelude::*;
use sp_runtime::runtime_logger::RuntimeLogger;
use sp_std::collections::btree_map::BTreeMap;

use super::*;

pub(crate) fn migrate_to_v6<T: Config>() {
    RuntimeLogger::init();

    log::info!("Running Migration for removing asset_id/tickers mappings.");
    let mut remove_mappings = BTreeMap::new();

    for (asset_id, ticker) in AssetIdTicker::<T>::iter() {
        if !Assets::<T>::contains_key(asset_id) {
            remove_mappings.insert(asset_id, ticker);
        }
    }

    let new_expiry = TickerConfig::<T>::get()
        .registration_length
        .map(|x| <pallet_timestamp::Pallet<T>>::get() + x);
    log::info!("{:?} mappings will be removed.", remove_mappings.len());
    for (asset_id, ticker) in remove_mappings {
        log::info!(
            "AssetId: {:?} Ticker: {:?} have been unlinked",
            asset_id,
            ticker
        );

        AssetIdTicker::<T>::remove(asset_id);
        TickerAssetId::<T>::remove(ticker);

        UniqueTickerRegistration::<T>::mutate(ticker, |registration| {
            if let Some(registration) = registration {
                if registration.expiry.is_some() {
                    registration.expiry = new_expiry;

                    Pallet::<T>::deposit_event(Event::TickerRegistered(
                        registration.owner,
                        ticker,
                        new_expiry,
                    ));
                }
            }
        });
    }

    log::info!("Migration has finished running.");
}

/// Pre-fills `MandatoryReceiverAffirmation` with `true` for all existing identities.
/// New identities created after this migration will default to `false`.
pub(crate) fn migrate_to_v7<T: Config>() -> Weight {
    RuntimeLogger::init();

    log::info!(
        "Running migration to pre-fill MandatoryReceiverAffirmation for existing identities."
    );

    let mut count: u64 = 0;
    for (did, _) in pallet_identity::DidRecords::<T>::iter() {
        MandatoryReceiverAffirmation::<T>::insert(did, true);
        count += 1;
    }

    log::info!("MandatoryReceiverAffirmation set for {count} existing identities.");

    T::DbWeight::get().reads_writes(count, count)
}
