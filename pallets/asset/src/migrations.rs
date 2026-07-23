use super::*;

pub(crate) fn migrate_to_v8<T: Config>() -> Weight {
    log::info!("Initializing ERC20 index for existing assets.");

    let mut reads = 1;
    let mut writes = 0;

    let mut next_index = NextAssetIndex::<T>::get();
    for asset_id in Assets::<T>::iter_keys() {
        IndexToAssetId::<T>::insert(next_index, asset_id);
        AssetIdToIndex::<T>::insert(asset_id, next_index);
        next_index = next_index.saturating_add(1);
        writes += 2;
        reads += 1;
    }
    writes += 1;
    NextAssetIndex::<T>::put(next_index);

    log::info!(
        "ERC20 index initialized for {} assets. Next asset index: {}",
        writes / 2,
        next_index
    );

    T::DbWeight::get().reads_writes(reads, writes)
}
