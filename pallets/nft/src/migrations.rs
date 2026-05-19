use frame_support::pallet_prelude::*;
use frame_support::storage::migration::move_prefix;
use frame_support::StoragePrefixedMap;

use crate::{Config, Pallet};

pub mod v6 {
    use frame_support::pallet_prelude::*;

    use polymesh_primitives::asset::AssetId;
    use polymesh_primitives::nft::{NFTId, NFTOwnerStatus};
    use polymesh_primitives::{AccountId as AccountId32, PortfolioId};

    use crate::{Config, Pallet};

    #[frame_support::storage_alias]
    pub type NFTOwner<T: Config> = StorageDoubleMap<
        Pallet<T>,
        Blake2_128Concat,
        AssetId,
        Blake2_128Concat,
        NFTId,
        PortfolioId,
        OptionQuery,
    >;

    #[frame_support::storage_alias]
    pub type OldNFTHolder<T: Config> = StorageDoubleMap<
        Pallet<T>,
        Twox64Concat,
        AccountId32,
        Blake2_128Concat,
        (AssetId, NFTId),
        NFTOwnerStatus,
        ValueQuery,
    >;
}

pub fn migrate_to_v7<T: Config>() -> Weight {
    let mut cursor = None;
    let mut writes = 0u64;
    let mut reads = 0u64;

    loop {
        let result = frame_support::migration::clear_storage_prefix(
            Pallet::<T>::name().as_bytes(),
            b"NFTOwner",
            b"",
            None,
            cursor.as_deref(),
        );

        writes += result.unique as u64;

        if result.maybe_cursor.is_none() {
            break;
        };

        cursor = result.maybe_cursor.clone();
    }

    log::info!("NFTOwner storage deleted");

    move_prefix(
        &crate::NFTHolder::<T>::final_prefix(),
        &v6::OldNFTHolder::<T>::final_prefix(),
    );

    for (account, (asset_id, nft_id), status) in v6::OldNFTHolder::<T>::drain() {
        crate::NFTHolder::<T>::insert((account, asset_id, nft_id), status);
        writes += 2;
        reads += 1;
    }

    log::info!("NFTHolder storage migrated: {} itens", reads);

    T::DbWeight::get().reads_writes(reads, writes)
}
