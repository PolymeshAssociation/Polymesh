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

    log::info!("{:?} mappings will be removed.", remove_mappings.len());
    for (asset_id, ticker) in remove_mappings {
        AssetIdTicker::remove(asset_id);
        TickerAssetId::remove(ticker);

        let new_expiry = TickerConfig::<T>::get()
            .registration_length
            .map(|x| <pallet_timestamp::Pallet<T>>::get() + x);
        UniqueTickerRegistration::<T>::mutate(ticker, |registration| {
            if let Some(registration) = registration {
                registration.expiry = new_expiry;
            }
        });
    }

    log::info!("Migration has finished running.");
}
