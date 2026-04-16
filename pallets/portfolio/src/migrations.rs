use frame_support::pallet_prelude::*;
use frame_support::storage::migration::move_prefix;
use frame_support::StoragePrefixedMap;

use crate::Config;

pub mod v3 {
    use frame_support::pallet_prelude::*;

    use polymesh_primitives::asset::AssetId;
    use polymesh_primitives::nft::NFTId;
    use polymesh_primitives::PortfolioId;

    use crate::{Config, Pallet};

    /// The nft associated to the portfolio.
    #[frame_support::storage_alias]
    pub type OldPortfolioNFT<T: Config> = StorageDoubleMap<
        Pallet<T>,
        Twox64Concat,
        PortfolioId,
        Blake2_128Concat,
        (AssetId, NFTId),
        bool,
        ValueQuery,
    >;
}

pub fn migrate_to_v4<T: Config>() -> Weight {
    let mut writes: u64 = 0;
    let mut reads: u64 = 0;

    move_prefix(
        &crate::PortfolioNFT::<T>::final_prefix(),
        &v3::OldPortfolioNFT::<T>::final_prefix(),
    );

    for (portfolio_id, (asset_id, nft_id), value) in v3::OldPortfolioNFT::<T>::drain() {
        crate::PortfolioNFT::<T>::insert((portfolio_id, asset_id, nft_id), value);
        writes += 2;
        reads += 1;
    }

    log::info!("PortfolioNFT storage migrated: {} itens", reads);

    T::DbWeight::get().reads_writes(reads, writes)
}
