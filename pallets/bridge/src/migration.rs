use crate::{BridgeTx, Module, StorageVersion, Trait, Version};

use frame_support::{
    migration::StorageKeyIterator, storage::StorageValue, weights::Weight, Twox64Concat,
};
use polymesh_primitives::storage_migrate_on;
use sp_runtime::traits::One;
use sp_std::prelude::*;

pub(crate) fn on_runtime_upgrade<T: Trait>() -> Weight {
    let storage_ver = Module::<T>::storage_version();

    storage_migrate_on!(storage_ver, 1, {
        let now = frame_system::Module::<T>::block_number();

        // Migrate timelocked transactions.
        StorageKeyIterator::<T::BlockNumber, Vec::<BridgeTx<T::AccountId, T::Balance>>, Twox64Concat>::new(b"Bridge", b"TimelockedTxs")
            .drain()
            .for_each(|(block_number, txs)| {
                // Schedule only for future blocks.
                let block_number = T::BlockNumber::max(block_number, now + One::one());
                txs.into_iter().for_each(|tx| Module::<T>::schedule_call(block_number, tx));
            });
    });

    // No need to calculate correct weight for testnet
    0
}
