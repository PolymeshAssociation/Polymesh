use frame_support::pallet_prelude::*;

use crate::{Config, Pallet};

pub mod v6 {
    use frame_support::pallet_prelude::*;

    use polymesh_primitives::asset::AssetId;
    use polymesh_primitives::nft::NFTId;
    use polymesh_primitives::PortfolioId;

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
}

pub fn migrate_to_v7<T: Config>() -> Weight {
    let result = frame_support::migration::clear_storage_prefix(
        Pallet::<T>::name().as_bytes(),
        b"NFTOwner",
        b"",
        None,
        None,
    );

    if let Some(cursor) = result.maybe_cursor {
        log::info!("nft::migrations second clear call");
        let _ = frame_support::migration::clear_storage_prefix(
            Pallet::<T>::name().as_bytes(),
            b"NFTOwner",
            b"",
            None,
            Some(cursor.as_ref()),
        );
    }

    Weight::zero()
}
