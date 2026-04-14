use frame_support::pallet_prelude::*;

use crate::Config;

pub mod v3 {
    use frame_support::pallet_prelude::*;

    use polymesh_primitives::asset::AssetId;
    use polymesh_primitives::nft::NFTId;
    use polymesh_primitives::PortfolioId;

    use crate::{Config, Pallet};

    /// The nft associated to the portfolio.
    #[frame_support::storage_alias]
    pub type PortfolioNFT<T: Config> = StorageDoubleMap<
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

    let old_entries: sp_std::vec::Vec<_> =
        crate::migrations::v3::PortfolioNFT::<T>::drain().collect();

    reads += old_entries.len() as u64;
    writes += old_entries.len() as u64;

    for (portfolio_id, (asset_id, nft_id), value) in old_entries {
        crate::PortfolioNFT::<T>::insert((portfolio_id, asset_id, nft_id), value);
        writes += 1;
    }

    log::info!("PortfolioNFT storage migrated: {} itens", reads);

    T::DbWeight::get().reads_writes(reads, writes)
}
