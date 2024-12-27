use sp_runtime::runtime_logger::RuntimeLogger;
use sp_std::collections::btree_map::BTreeMap;

use super::*;

pub(crate) fn migrate_to_v6<T: Config>() {
    RuntimeLogger::init();

    log::info!("Running Migration for removing asset_id/tickers mappings.");
    let mut remove_mappings = BTreeMap::new();

    for (asset_id, ticker) in AssetIdTicker::iter() {
        if !Assets::contains_key(asset_id) {
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

        AssetIdTicker::remove(asset_id);
        TickerAssetId::remove(ticker);

        UniqueTickerRegistration::<T>::mutate(ticker, |registration| {
            if let Some(registration) = registration {
                if registration.expiry.is_some() {
                    registration.expiry = new_expiry;

                    Module::<T>::deposit_event(RawEvent::TickerRegistered(
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
