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
    let mut cursor = None;
    let mut count = 0;

    loop {
        let result = frame_support::migration::clear_storage_prefix(
            Pallet::<T>::name().as_bytes(),
            b"NFTOwners",
            b"",
            None,
            cursor.as_deref(),
        );

        if result.maybe_cursor.is_none() {
            break;
        };

        cursor = result.maybe_cursor.clone();
        count += result.unique;
    }

    T::DbWeight::get().writes(count.into())
}
