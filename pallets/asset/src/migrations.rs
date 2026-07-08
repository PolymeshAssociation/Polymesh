use codec::Encode;
use sp_runtime::runtime_logger::RuntimeLogger;

use super::*;

pub(crate) fn migrate_to_v7<T: Config + pallet_identity::Config>() -> Weight {
    RuntimeLogger::init();

    const SECURITY_TOKEN_PREFIX: &[u8; 15] = b"SECURITY_TOKEN:";

    let mut removed = 0u64;
    let mut ticker_count = 0u64;
    for ticker in UniqueTickerRegistration::<T>::iter_keys() {
        ticker_count += 1;
        let hash = (SECURITY_TOKEN_PREFIX, ticker).using_encoded(sp_io::hashing::blake2_256);
        let did = polymesh_primitives::IdentityId::try_from(&hash[..])
            .expect("BlakeTwo256 output is 32 bytes = IdentityId size");

        if pallet_identity::DidRecords::<T>::contains_key(&did) {
            pallet_identity::DidRecords::<T>::remove(&did);
            removed += 1;
        }
    }

    log::info!(
        "asset.migrate_to_v7: Removed {} asset DidRecords from {} tickers.",
        removed,
        ticker_count,
    );

    let erc20_weight = initialize_erc20_index::<T>();

    // Reads: 1 (version check) + ticker_count (iter_keys) + ticker_count (contains_key).
    // Writes: 1 (version put) + removed (DidRecords removals).
    T::DbWeight::get()
        .reads_writes(1 + (2 * ticker_count), 1 + removed)
        .saturating_add(erc20_weight)
}

fn initialize_erc20_index<T: Config>() -> Weight {
    log::info!("Initializing ERC20 index for existing assets.");

    let mut reads = 1;
    let mut writes = 0;

    let mut next_index = NextAssetIndex::<T>::get();
    for asset_id in Assets::<T>::iter_keys() {
        Erc20IndexToAssetId::<T>::insert(next_index, asset_id);
        Erc20AssetIdToIndex::<T>::insert(asset_id, next_index);
        next_index = next_index.saturating_add(1);
        writes += 2;
        reads += 1;
    }
    writes += 1;
    NextAssetIndex::<T>::put(next_index);

    T::DbWeight::get().reads_writes(reads, writes)
}
