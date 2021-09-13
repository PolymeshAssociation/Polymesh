use crate::{BridgeTx, Config, Module, StorageVersion, Version};

use frame_support::{
    migration::StorageKeyIterator, storage::StorageValue, weights::Weight, Twox64Concat,
};
use polymesh_primitives::storage_migrate_on;
use sp_runtime::traits::One;
use sp_std::prelude::*;

pub(crate) fn on_runtime_upgrade<T: Config>() -> Weight {
    let storage_ver = Module::<T>::storage_version();

    storage_migrate_on!(storage_ver, 1, {
        let now = frame_system::Module::<T>::block_number();

        // Migrate timelocked transactions.
        StorageKeyIterator::<T::BlockNumber, Vec<BridgeTx<T::AccountId>>, Twox64Concat>::new(
            b"Bridge",
            b"TimelockedTxs",
        )
        .drain()
        .for_each(|(block_number, txs)| {
            // Schedule only for future blocks.
            let block_number = T::BlockNumber::max(block_number, now + One::one());
            txs.into_iter().for_each(|tx| {
                if let Err(e) = Module::<T>::schedule_call(block_number, tx) {
                    pallet_base::emit_unexpected_error::<T>(Some(e));
                }
            });
        });
    });

    // No need to calculate correct weight for testnet
    0
}
