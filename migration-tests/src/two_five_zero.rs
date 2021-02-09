use crate::test_migration;
use frame_support::storage::{StorageDoubleMap, StorageMap};
use std::convert::TryFrom;

use pallet_asset::checkpoint;
use polymesh_primitives::{calendar::CheckpointId, Ticker};
use polymesh_primitives_old::{calendar::CheckpointId as CheckpointIdOld, Ticker as TickerOld};
use polymesh_runtime::Runtime;
use polymesh_runtime_old::Runtime as RuntimeOld;

type CheckpointOldModule = pallet_asset_old::checkpoint::Module<RuntimeOld>;

#[test]
fn checkpoints_upgrade() {
    test_migration(
        pre_migration_checkpoint_tests,
        post_migration_checkpoint_tests,
    );
}

fn pre_migration_checkpoint_tests() {
    // Ensure that the cached data is valid
    let ticker_name = b"SBL";
    let ticker = TickerOld::try_from(&ticker_name[..]).unwrap();
    assert!(
        CheckpointOldModule::total_supply_at((ticker, CheckpointIdOld(1))) == 10_000_000_000u128
    );
    assert!(CheckpointOldModule::checkpoint_id_sequence(ticker) == CheckpointIdOld(1));
}

fn post_migration_checkpoint_tests() {
    // Ensure that the storage is nuked after the upgrade
    let ticker_name = b"SBL";
    let ticker = Ticker::try_from(&ticker_name[..]).unwrap();
    assert!(!checkpoint::TotalSupply::<Runtime>::contains_key(
        ticker,
        CheckpointId(1)
    ));
    assert!(!checkpoint::CheckpointIdSequence::contains_key(ticker));
    assert!(!checkpoint::Timestamps::contains_key(
        ticker,
        CheckpointId(1)
    ));
}
