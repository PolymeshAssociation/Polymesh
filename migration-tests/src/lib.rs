use futures::executor::block_on;
use polymesh_runtime::{DryRunRuntimeUpgrade, Runtime};
use remote_externalities::{Builder, CacheConfig, Mode, OfflineConfig, OnlineConfig};
use remote_externalities_old::{
    Builder as BuilderOld, Mode as ModeOld, OfflineConfig as OfflineConfigOld,
};
use sp_core::storage::{StorageData, StorageKey};
use sp_state_machine_old::backend::Backend;
use std::{sync::Mutex, time::Instant};

#[macro_use]
extern crate lazy_static;

#[cfg(test)]
mod two_five_zero;

struct TestState;

impl TestState {
    fn init() {
        if !std::path::Path::new(".").join("CACHE").exists() {
            block_on(
                Builder::new()
                    .mode(Mode::Online(OnlineConfig {
                        uri: "http://159.69.94.51:9933".into(),
                        cache: Some(CacheConfig::default()),
                        ..Default::default()
                    }))
                    .build(),
            );
        }
    }
}

lazy_static! {
    static ref TEST_STATE: Mutex<()> = Mutex::new(TestState::init());
}

/// Main helper function for writing migration tests.
///
/// Takes two closures, executing the first on the current blockchain state,
/// then the storage migrations, and finally the second closure.
pub fn test_migration(pre_tests: impl FnOnce(), post_tests: impl FnOnce()) {
    lazy_static::initialize(&TEST_STATE);

    let mut state = block_on(
        BuilderOld::new()
            .mode(ModeOld::Offline(OfflineConfigOld::default()))
            .build(),
    );

    state.execute_with(pre_tests);

    let pairs = state
        .commit_all()
        .pairs()
        .iter()
        .map(|(key, data)| (StorageKey(key.clone()), StorageData(data.clone())))
        .collect::<Vec<_>>();

    let mut new_state = block_on(
        Builder::new()
            .inject(&pairs[..])
            .mode(Mode::Offline(OfflineConfig::default()))
            .build(),
    );

    let now = Instant::now();
    new_state.execute_with(<Runtime as DryRunRuntimeUpgrade>::dry_run_runtime_upgrade);
    let elapsed = now.elapsed();
    println!("Storage Migrations took: {:#?}", elapsed);

    new_state.execute_with(post_tests);
}
