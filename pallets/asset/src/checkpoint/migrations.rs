use sp_runtime::runtime_logger::RuntimeLogger;
use sp_std::collections::btree_map::BTreeMap;

use super::*;

mod v1 {
    use super::*;
    use polymesh_primitives::Ticker;

    decl_storage! {
        trait Store for Module<T: Config> as Checkpoint {
            // This storage changed the Ticker key to AssetID.
            pub TotalSupply get(fn total_supply_at):
                double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) CheckpointId => polymesh_primitives::Balance;

            // This storage changed the Ticker key to AssetID.
            pub Balance get(fn balance_at_checkpoint):
                double_map hasher(blake2_128_concat) (Ticker, CheckpointId), hasher(twox_64_concat) IdentityId => polymesh_primitives::Balance;

            // This storage changed the Ticker key to AssetID.
            pub CheckpointIdSequence get(fn checkpoint_id_sequence):
                map hasher(blake2_128_concat) Ticker => CheckpointId;

            // This storage changed the Ticker key to AssetID.
            pub BalanceUpdates get(fn balance_updates):
                double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) IdentityId => Vec<CheckpointId>;

            // This storage changed the Ticker key to AssetID.
            pub Timestamps get(fn timestamps):
                double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) CheckpointId => Moment;

            // This storage changed the Ticker key to AssetID.
            pub ScheduleIdSequence get(fn schedule_id_sequence):
                map hasher(blake2_128_concat) Ticker => ScheduleId;

            // This storage changed the Ticker key to AssetID.
            pub CachedNextCheckpoints get(fn cached_next_checkpoints):
                map hasher(blake2_128_concat) Ticker => Option<NextCheckpoints>;

            // This storage changed the Ticker key to AssetID.
            pub ScheduledCheckpoints get(fn scheduled_checkpoints):
                double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) ScheduleId => Option<ScheduleCheckpoints>;

            // This storage changed the Ticker key to AssetID.
            pub ScheduleRefCount get(fn schedule_ref_count):
                double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) ScheduleId => u32;

            // This storage changed the Ticker key to AssetID.
            pub SchedulePoints get(fn schedule_points):
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

    log::info!("Updating types for the TotalSupply storage");
    v1::TotalSupply::drain().for_each(|(ticker, checkpoint_id, balance)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        TotalSupply::insert(asset_id, checkpoint_id, balance);
    });

    log::info!("Updating types for the Balance storage");
    v1::Balance::drain().for_each(|((ticker, checkpoint_id), did, balance)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        Balance::insert((asset_id, checkpoint_id), did, balance);
    });

    log::info!("Updating types for the CheckpointIdSequence storage");
    v1::CheckpointIdSequence::drain().for_each(|(ticker, checkpoint_id)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        CheckpointIdSequence::insert(asset_id, checkpoint_id);
    });

    log::info!("Updating types for the BalanceUpdates storage");
    v1::BalanceUpdates::drain().for_each(|(ticker, did, checkpoint_id)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        BalanceUpdates::insert(asset_id, did, checkpoint_id);
    });

    log::info!("Updating types for the Timestamps storage");
    v1::Timestamps::drain().for_each(|(ticker, checkpoint_id, when)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        Timestamps::insert(asset_id, checkpoint_id, when);
    });

    log::info!("Updating types for the ScheduleIdSequence storage");
    v1::ScheduleIdSequence::drain().for_each(|(ticker, schedule_id)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        ScheduleIdSequence::insert(asset_id, schedule_id);
    });

    log::info!("Updating types for the CachedNextCheckpoints storage");
    v1::CachedNextCheckpoints::drain().for_each(|(ticker, next_checkpoint)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        CachedNextCheckpoints::insert(asset_id, next_checkpoint);
    });

    log::info!("Updating types for the ScheduledCheckpoints storage");
    v1::ScheduledCheckpoints::drain().for_each(|(ticker, schedule_id, next_checkpoint)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        ScheduledCheckpoints::insert(asset_id, schedule_id, next_checkpoint);
    });

    log::info!("Updating types for the ScheduleRefCount storage");
    v1::ScheduleRefCount::drain().for_each(|(ticker, schedule_id, count)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        ScheduleRefCount::insert(asset_id, schedule_id, count);
    });

    log::info!("Updating types for the SchedulePoints storage");
    v1::SchedulePoints::drain().for_each(|(ticker, schedule_id, checkpoint_id)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        SchedulePoints::insert(asset_id, schedule_id, checkpoint_id);
    });
}
