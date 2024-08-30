use frame_support::storage::migration::move_prefix;
use sp_runtime::runtime_logger::RuntimeLogger;
use sp_std::collections::btree_map::BTreeMap;

use super::*;

mod v1 {
    use super::*;
    use polymesh_primitives::Ticker;

    decl_storage! {
        trait Store for Module<T: Config> as Checkpoint {
            // This storage changed the Ticker key to AssetID.
            pub OldTotalSupply get(fn total_supply_at):
                double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) CheckpointId => polymesh_primitives::Balance;

            // This storage changed the Ticker key to AssetID.
            pub OldBalance get(fn balance_at_checkpoint):
                double_map hasher(blake2_128_concat) (Ticker, CheckpointId), hasher(twox_64_concat) IdentityId => polymesh_primitives::Balance;

            // This storage changed the Ticker key to AssetID.
            pub OldCheckpointIdSequence get(fn checkpoint_id_sequence):
                map hasher(blake2_128_concat) Ticker => CheckpointId;

            // This storage changed the Ticker key to AssetID.
            pub OldBalanceUpdates get(fn balance_updates):
                double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) IdentityId => Vec<CheckpointId>;

            // This storage changed the Ticker key to AssetID.
            pub OldTimestamps get(fn timestamps):
                double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) CheckpointId => Moment;

            // This storage changed the Ticker key to AssetID.
            pub OldScheduleIdSequence get(fn schedule_id_sequence):
                map hasher(blake2_128_concat) Ticker => ScheduleId;

            // This storage changed the Ticker key to AssetID.
            pub OldCachedNextCheckpoints get(fn cached_next_checkpoints):
                map hasher(blake2_128_concat) Ticker => Option<NextCheckpoints>;

            // This storage changed the Ticker key to AssetID.
            pub OldScheduledCheckpoints get(fn scheduled_checkpoints):
                double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) ScheduleId => Option<ScheduleCheckpoints>;

            // This storage changed the Ticker key to AssetID.
            pub OldScheduleRefCount get(fn schedule_ref_count):
                double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) ScheduleId => u32;

            // This storage changed the Ticker key to AssetID.
            pub OldSchedulePoints get(fn schedule_points):
                double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) ScheduleId => Vec<CheckpointId>;
        }
    }

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin { }
    }
}

pub(crate) fn migrate_to_v2<T: Config>() {
    RuntimeLogger::init();
    let mut ticker_to_asset_id = BTreeMap::new();

    // Removes all elements in the old storage and inserts it in the new storage

    let mut count = 0;
    log::info!("Updating types for the TotalSupply storage");
    move_prefix(
        &TotalSupply::final_prefix(),
        &v1::OldTotalSupply::final_prefix(),
    );
    v1::OldTotalSupply::drain().for_each(|(ticker, checkpoint_id, balance)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        TotalSupply::insert(asset_id, checkpoint_id, balance);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the Balance storage");
    move_prefix(&Balance::final_prefix(), &v1::OldBalance::final_prefix());
    v1::OldBalance::drain().for_each(|((ticker, checkpoint_id), did, balance)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        Balance::insert((asset_id, checkpoint_id), did, balance);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the CheckpointIdSequence storage");
    move_prefix(
        &CheckpointIdSequence::final_prefix(),
        &v1::OldCheckpointIdSequence::final_prefix(),
    );
    v1::OldCheckpointIdSequence::drain().for_each(|(ticker, checkpoint_id)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        CheckpointIdSequence::insert(asset_id, checkpoint_id);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the BalanceUpdates storage");
    move_prefix(
        &BalanceUpdates::final_prefix(),
        &v1::OldBalanceUpdates::final_prefix(),
    );
    v1::OldBalanceUpdates::drain().for_each(|(ticker, did, checkpoint_id)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        BalanceUpdates::insert(asset_id, did, checkpoint_id);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the Timestamps storage");
    move_prefix(
        &Timestamps::final_prefix(),
        &v1::OldTimestamps::final_prefix(),
    );
    v1::OldTimestamps::drain().for_each(|(ticker, checkpoint_id, when)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        Timestamps::insert(asset_id, checkpoint_id, when);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the ScheduleIdSequence storage");
    move_prefix(
        &ScheduleIdSequence::final_prefix(),
        &v1::OldScheduleIdSequence::final_prefix(),
    );
    v1::OldScheduleIdSequence::drain().for_each(|(ticker, schedule_id)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        ScheduleIdSequence::insert(asset_id, schedule_id);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the CachedNextCheckpoints storage");
    move_prefix(
        &CachedNextCheckpoints::final_prefix(),
        &v1::OldCachedNextCheckpoints::final_prefix(),
    );
    v1::OldCachedNextCheckpoints::drain().for_each(|(ticker, next_checkpoint)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        CachedNextCheckpoints::insert(asset_id, next_checkpoint);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the ScheduledCheckpoints storage");
    move_prefix(
        &ScheduledCheckpoints::final_prefix(),
        &v1::OldScheduledCheckpoints::final_prefix(),
    );
    v1::OldScheduledCheckpoints::drain().for_each(|(ticker, schedule_id, next_checkpoint)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        ScheduledCheckpoints::insert(asset_id, schedule_id, next_checkpoint);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the ScheduleRefCount storage");
    move_prefix(
        &ScheduleRefCount::final_prefix(),
        &v1::OldScheduleRefCount::final_prefix(),
    );
    v1::OldScheduleRefCount::drain().for_each(|(ticker, schedule_id, ref_count)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        ScheduleRefCount::insert(asset_id, schedule_id, ref_count);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the SchedulePoints storage");
    move_prefix(
        &SchedulePoints::final_prefix(),
        &v1::OldSchedulePoints::final_prefix(),
    );
    v1::OldSchedulePoints::drain().for_each(|(ticker, schedule_id, checkpoint_id)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        SchedulePoints::insert(asset_id, schedule_id, checkpoint_id);
    });
    log::info!("{:?} items migrated", count);
}
